//! Comparación de DATOS de filas entre dos tablas (al estilo SQL Data Compare).
//!
//! Esta capa es independiente del motor: recibe las filas ya leídas (cada celda
//! como texto canónico, o `None` para NULL) y una clasificación por columna para
//! saber cómo entrecomillar cada valor en el DML. Empareja las filas por clave
//! primaria y genera el script para dejar los DATOS de B iguales que los de A:
//! `INSERT` (filas solo en A), `UPDATE` (filas distintas) y `DELETE` (solo en B).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::script::Dialect;

/// Cómo se debe emitir el valor de una columna en el DML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnKind {
    /// Número: se emite tal cual (sin comillas).
    Numeric,
    /// Texto: se emite entre comillas simples.
    Text,
    /// Booleano: `TRUE`/`FALSE` en PostgreSQL, `1`/`0` en los demás.
    Bool,
    /// Binario: el valor ya viene en hexadecimal; se emite como literal del motor.
    Blob,
}

/// Una columna de datos: nombre + cómo entrecomillarla.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataColumn {
    pub name: String,
    pub kind: ColumnKind,
}

/// Las filas de una tabla, con sus columnas y su clave primaria.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataTable {
    /// Nombre calificado de la tabla ("schema.tabla").
    pub table: String,
    pub columns: Vec<DataColumn>,
    /// Nombres de las columnas que forman la clave primaria (en orden).
    pub pk: Vec<String>,
    /// Filas (en el mismo orden que `columns`); cada celda es texto o `None` (NULL).
    pub rows: Vec<Vec<Option<String>>>,
    /// La tabla superó el límite y no se leyó entera (no se compara).
    #[serde(default)]
    pub truncated: bool,
}

/// Una fila de la muestra de diferencias (para mostrarla en una cuadrícula).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRow {
    /// "insert" (solo en A), "update" (distinta) o "delete" (solo en B).
    pub op: String,
    /// Valores de la fila en el orden de `columns` (texto o `None` = NULL).
    pub cells: Vec<Option<String>>,
}

/// Cuántas filas de diferencia se incluyen en la muestra (para no cargar miles).
const SAMPLE_CAP: usize = 200;

/// Resultado de comparar los datos de una tabla entre A y B.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataComparison {
    pub table: String,
    pub pk: Vec<String>,
    /// Nombres de columna (de A), para los encabezados de la cuadrícula.
    #[serde(default)]
    pub columns: Vec<String>,
    /// Filas que están en A pero no en B (se insertarían en B).
    pub only_in_left: usize,
    /// Filas que están en B pero no en A (se borrarían de B).
    pub only_in_right: usize,
    /// Filas con la misma PK pero valores distintos (se actualizaría B).
    pub different: usize,
    /// Filas idénticas en ambos.
    pub equal: usize,
    /// Muestra de hasta `SAMPLE_CAP` filas con diferencias (para la cuadrícula).
    #[serde(default)]
    pub sample: Vec<SampleRow>,
    /// Script DML para dejar los datos de B iguales que los de A.
    pub dml: String,
    #[serde(default)]
    pub truncated: bool,
    /// Mensaje informativo cuando no se puede comparar (sin PK, tabla enorme…).
    #[serde(default)]
    pub note: String,
}

fn split_qualified(qn: &str) -> (&str, &str) {
    qn.split_once('.').unwrap_or(("", qn))
}

/// Literal SQL de un valor según su clasificación y el dialecto destino.
fn literal(d: Dialect, kind: ColumnKind, value: &Option<String>) -> String {
    let s = match value {
        None => return "NULL".to_string(),
        Some(s) => s,
    };
    match kind {
        ColumnKind::Numeric => {
            if s.is_empty() {
                "NULL".to_string()
            } else {
                s.clone()
            }
        }
        ColumnKind::Bool => {
            let truthy = matches!(s.to_ascii_lowercase().as_str(), "1" | "true" | "t" | "yes");
            match d {
                Dialect::Postgres => if truthy { "TRUE".into() } else { "FALSE".into() },
                _ => if truthy { "1".into() } else { "0".into() },
            }
        }
        ColumnKind::Blob => match d {
            Dialect::SqlServer => format!("0x{s}"),
            Dialect::Postgres => format!("'\\x{s}'"),
            Dialect::Sqlite => format!("X'{s}'"),
        },
        ColumnKind::Text => format!("'{}'", s.replace('\'', "''")),
    }
}

fn qname(d: Dialect, table: &str) -> String {
    let (schema, name) = split_qualified(table);
    if schema.is_empty() {
        d.quote(name)
    } else {
        d.qname(schema, name)
    }
}

/// Clave de emparejamiento de una fila a partir de las columnas de la PK.
fn row_key(row: &[Option<String>], pk_idx: &[usize]) -> String {
    pk_idx
        .iter()
        .map(|&i| match row.get(i) {
            Some(Some(v)) => format!("v:{v}"),
            _ => "NULL".to_string(),
        })
        .collect::<Vec<_>>()
        .join("\u{1}")
}

fn insert_stmt(d: Dialect, dt: &DataTable, row: &[Option<String>]) -> String {
    let cols = dt.columns.iter().map(|c| d.quote(&c.name)).collect::<Vec<_>>().join(", ");
    let vals = dt
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| literal(d, c.kind, row.get(i).unwrap_or(&None)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("INSERT INTO {} ({}) VALUES ({});", qname(d, &dt.table), cols, vals)
}

fn where_pk(d: Dialect, dt: &DataTable, row: &[Option<String>], pk_idx: &[usize]) -> String {
    dt.pk
        .iter()
        .zip(pk_idx)
        .map(|(name, &i)| {
            let kind = dt.columns.get(i).map(|c| c.kind).unwrap_or(ColumnKind::Text);
            let v = row.get(i).unwrap_or(&None);
            match v {
                None => format!("{} IS NULL", d.quote(name)),
                some => format!("{} = {}", d.quote(name), literal(d, kind, some)),
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn update_stmt(d: Dialect, dt: &DataTable, row: &[Option<String>], pk_idx: &[usize]) -> String {
    let sets = dt
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| !dt.pk.contains(&c.name))
        .map(|(i, c)| format!("{} = {}", d.quote(&c.name), literal(d, c.kind, row.get(i).unwrap_or(&None))))
        .collect::<Vec<_>>()
        .join(", ");
    format!("UPDATE {} SET {} WHERE {};", qname(d, &dt.table), sets, where_pk(d, dt, row, pk_idx))
}

/// Índices de las columnas de la PK dentro de una lista de columnas.
fn pk_indices(columns: &[DataColumn], pk: &[String]) -> Option<Vec<usize>> {
    let idx: BTreeMap<&str, usize> = columns.iter().enumerate().map(|(i, c)| (c.name.as_str(), i)).collect();
    pk.iter().map(|p| idx.get(p.as_str()).copied()).collect()
}

/// Compara los datos de una tabla y arma el DML para dejar B igual que A.
pub fn compare_table_data(left: &DataTable, right: &DataTable, d: Dialect) -> DataComparison {
    let mut out = DataComparison { table: left.table.clone(), pk: left.pk.clone(), ..Default::default() };

    if left.truncated || right.truncated {
        out.truncated = true;
        out.note = "La tabla supera el límite para comparar datos.".into();
        return out;
    }
    if left.pk.is_empty() {
        out.note = "La tabla no tiene clave primaria; no se pueden comparar datos por fila.".into();
        return out;
    }

    let lpk = match pk_indices(&left.columns, &left.pk) {
        Some(v) => v,
        None => {
            out.note = "No se encontró la clave primaria en las columnas de A.".into();
            return out;
        }
    };
    let rpk = match pk_indices(&right.columns, &left.pk) {
        Some(v) => v,
        None => {
            out.note = "La clave primaria de A no existe en B.".into();
            return out;
        }
    };

    // Índice de columnas de B por nombre, para comparar por nombre (no por posición).
    let ridx: BTreeMap<&str, usize> = right.columns.iter().enumerate().map(|(i, c)| (c.name.as_str(), i)).collect();

    out.columns = left.columns.iter().map(|c| c.name.clone()).collect();

    let rmap: BTreeMap<String, &Vec<Option<String>>> = right.rows.iter().map(|r| (row_key(r, &rpk), r)).collect();

    let mut left_keys: BTreeSet<String> = BTreeSet::new();
    let mut inserts = String::new();
    let mut updates = String::new();

    for lrow in &left.rows {
        let k = row_key(lrow, &lpk);
        left_keys.insert(k.clone());
        match rmap.get(&k) {
            None => {
                out.only_in_left += 1;
                let _ = writeln!(inserts, "{}", insert_stmt(d, left, lrow));
                if out.sample.len() < SAMPLE_CAP {
                    out.sample.push(SampleRow { op: "insert".into(), cells: lrow.clone() });
                }
            }
            Some(rrow) => {
                if row_values_differ(left, right, lrow, rrow, &ridx) {
                    out.different += 1;
                    let _ = writeln!(updates, "{}", update_stmt(d, left, lrow, &lpk));
                    if out.sample.len() < SAMPLE_CAP {
                        out.sample.push(SampleRow { op: "update".into(), cells: lrow.clone() });
                    }
                } else {
                    out.equal += 1;
                }
            }
        }
    }

    let mut deletes = String::new();
    for rrow in &right.rows {
        let k = row_key(rrow, &rpk);
        if !left_keys.contains(&k) {
            out.only_in_right += 1;
            let _ = writeln!(deletes, "{}", delete_stmt_right(d, left, rrow, &rpk));
            if out.sample.len() < SAMPLE_CAP {
                // Reordenamos la fila de B al orden de columnas de A para la cuadrícula.
                let cells = left
                    .columns
                    .iter()
                    .map(|c| ridx.get(c.name.as_str()).and_then(|&j| rrow.get(j).cloned().flatten()))
                    .collect();
                out.sample.push(SampleRow { op: "delete".into(), cells });
            }
        }
    }

    out.dml = assemble_dml(d, left, &inserts, &updates, &deletes, &out);
    out
}

/// DELETE usando los valores de PK de una fila de B (puede tener otras columnas).
fn delete_stmt_right(d: Dialect, left: &DataTable, rrow: &[Option<String>], rpk: &[usize]) -> String {
    // Construimos el WHERE con los nombres de PK (de A) y los valores de la fila de B.
    let clause = left
        .pk
        .iter()
        .zip(rpk)
        .map(|(name, &i)| {
            // La clasificación la tomamos de A (misma columna, misma tabla).
            let kind = left
                .columns
                .iter()
                .find(|c| &c.name == name)
                .map(|c| c.kind)
                .unwrap_or(ColumnKind::Text);
            let v = rrow.get(i).unwrap_or(&None);
            match v {
                None => format!("{} IS NULL", d.quote(name)),
                some => format!("{} = {}", d.quote(name), literal(d, kind, some)),
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    format!("DELETE FROM {} WHERE {};", qname(d, &left.table), clause)
}

/// ¿Difieren los valores (no-PK) de dos filas emparejadas? Compara por nombre de
/// columna; si B no tiene una columna de A, se considera distinta.
fn row_values_differ(left: &DataTable, right: &DataTable, lrow: &[Option<String>], rrow: &[Option<String>], ridx: &BTreeMap<&str, usize>) -> bool {
    for (i, c) in left.columns.iter().enumerate() {
        if left.pk.contains(&c.name) {
            continue;
        }
        match ridx.get(c.name.as_str()) {
            Some(&j) => {
                if lrow.get(i).unwrap_or(&None) != rrow.get(j).unwrap_or(&None) {
                    return true;
                }
            }
            None => return true,
        }
    }
    let _ = right; // sólo se usa right.columns vía ridx
    false
}

fn assemble_dml(d: Dialect, dt: &DataTable, inserts: &str, updates: &str, deletes: &str, cmp: &DataComparison) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "-- ============================================================");
    let _ = writeln!(out, "-- Sincronización de DATOS generada por DataDiff");
    let _ = writeln!(out, "-- Tabla: {}  ·  Motor destino: {}", dt.table, d.label());
    let _ = writeln!(out, "-- Deja los datos del DESTINO (B) iguales que los del ORIGEN (A)");
    let _ = writeln!(out, "-- (!) Revísalo y respalda el destino antes de ejecutarlo.");
    let _ = writeln!(out, "-- ============================================================\n");

    if cmp.only_in_left == 0 && cmp.only_in_right == 0 && cmp.different == 0 {
        let _ = writeln!(out, "-- Sin diferencias de datos: las filas ya coinciden.");
        return out;
    }
    if !inserts.is_empty() {
        let _ = write!(out, "-- ─── Filas nuevas (solo en A) ───\n{inserts}\n");
    }
    if !updates.is_empty() {
        let _ = write!(out, "-- ─── Filas distintas ───\n{updates}\n");
    }
    if !deletes.is_empty() {
        let _ = write!(out, "-- ─── Filas sobrantes (solo en B) ───\n{deletes}\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: &str) -> Option<String> {
        Some(n.to_string())
    }
    fn txt(s: &str) -> Option<String> {
        Some(s.to_string())
    }
    fn cols() -> Vec<DataColumn> {
        vec![
            DataColumn { name: "id".into(), kind: ColumnKind::Numeric },
            DataColumn { name: "nombre".into(), kind: ColumnKind::Text },
        ]
    }

    #[test]
    fn insert_update_delete() {
        // A: (1,"Ana"),(2,"Beto")   B: (2,"Beto cambiado"),(3,"Caro")
        let a = DataTable {
            table: "main.cli".into(),
            columns: cols(),
            pk: vec!["id".into()],
            rows: vec![vec![num("1"), txt("Ana")], vec![num("2"), txt("Beto")]],
            truncated: false,
        };
        let b = DataTable {
            table: "main.cli".into(),
            columns: cols(),
            pk: vec!["id".into()],
            rows: vec![vec![num("2"), txt("Beto cambiado")], vec![num("3"), txt("Caro")]],
            truncated: false,
        };
        let c = compare_table_data(&a, &b, Dialect::Sqlite);
        assert_eq!(c.only_in_left, 1); // id=1 → INSERT
        assert_eq!(c.only_in_right, 1); // id=3 → DELETE
        assert_eq!(c.different, 1); // id=2 → UPDATE
        assert_eq!(c.equal, 0);
        assert!(c.dml.contains("INSERT INTO \"main\".\"cli\" (\"id\", \"nombre\") VALUES (1, 'Ana');"), "{}", c.dml);
        assert!(c.dml.contains("UPDATE \"main\".\"cli\" SET \"nombre\" = 'Beto' WHERE \"id\" = 2;"), "{}", c.dml);
        assert!(c.dml.contains("DELETE FROM \"main\".\"cli\" WHERE \"id\" = 3;"), "{}", c.dml);
        // Muestra para la cuadrícula: 1 insert + 1 update + 1 delete, con encabezados.
        assert_eq!(c.columns, vec!["id".to_string(), "nombre".to_string()]);
        assert_eq!(c.sample.len(), 3);
        let ins = c.sample.iter().find(|s| s.op == "insert").unwrap();
        assert_eq!(ins.cells, vec![Some("1".to_string()), Some("Ana".to_string())]);
        // La fila de borrado viene de B (id=3, Caro), reordenada al orden de A.
        let del = c.sample.iter().find(|s| s.op == "delete").unwrap();
        assert_eq!(del.cells, vec![Some("3".to_string()), Some("Caro".to_string())]);
    }

    #[test]
    fn sin_pk_lo_dice() {
        let a = DataTable { table: "main.t".into(), columns: cols(), pk: vec![], rows: vec![], truncated: false };
        let b = a.clone();
        let c = compare_table_data(&a, &b, Dialect::Sqlite);
        assert!(c.note.contains("clave primaria"), "{}", c.note);
        assert!(c.dml.is_empty());
    }

    #[test]
    fn iguales_sin_dml() {
        let rows = vec![vec![num("1"), txt("Ana")]];
        let a = DataTable { table: "main.t".into(), columns: cols(), pk: vec!["id".into()], rows: rows.clone(), truncated: false };
        let b = DataTable { table: "main.t".into(), columns: cols(), pk: vec!["id".into()], rows, truncated: false };
        let c = compare_table_data(&a, &b, Dialect::Sqlite);
        assert_eq!(c.equal, 1);
        assert_eq!(c.only_in_left + c.only_in_right + c.different, 0);
        assert!(c.dml.contains("Sin diferencias de datos"), "{}", c.dml);
    }

    #[test]
    fn texto_con_comilla_se_escapa() {
        let a = DataTable {
            table: "main.t".into(),
            columns: cols(),
            pk: vec!["id".into()],
            rows: vec![vec![num("1"), txt("O'Brien")]],
            truncated: false,
        };
        let b = DataTable { table: "main.t".into(), columns: cols(), pk: vec!["id".into()], rows: vec![], truncated: false };
        let c = compare_table_data(&a, &b, Dialect::Sqlite);
        assert!(c.dml.contains("'O''Brien'"), "{}", c.dml);
    }
}
