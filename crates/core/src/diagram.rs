//! Modelo estructurado para dibujar el diagrama entidad-relación con un
//! renderizador propio (la interfaz lo posiciona con dagre y lo dibuja en HTML).
//!
//! A diferencia de Mermaid, esto nos da control total del formato: encabezados
//! «Campo · Tipo de dato · Clave», orden de columnas y colores.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Column, ForeignKey, Schema, Table};

/// Una columna tal como se muestra en el diagrama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramColumn {
    pub name: String,
    pub data_type: String,
    /// "PK", "FK", "PK·FK" o "" (vacío).
    pub key: String,
}

/// Una tabla (entidad) del diagrama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramTable {
    /// Id saneado para usar como clave de nodo.
    pub id: String,
    /// Nombre calificado para mostrar, p.ej. "dbo.Clientes".
    pub name: String,
    /// Estado en el diagrama de diferencias: "", "only_in_left", "only_in_right",
    /// "different" o "equal". Vacío en el diagrama de un solo esquema.
    #[serde(default)]
    pub status: String,
    pub columns: Vec<DiagramColumn>,
}

/// Una relación de clave foránea (child -> parent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramRelation {
    pub from: String, // tabla con la FID (lado "muchos")
    pub to: String,   // tabla referenciada (lado "uno")
    pub label: String,
}

/// El diagrama completo de un esquema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagramModel {
    pub tables: Vec<DiagramTable>,
    pub relations: Vec<DiagramRelation>,
}

fn did(qualified: &str) -> String {
    qualified
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Marca PK/FK y arma las columnas de una tabla para el diagrama.
fn build_diagram_columns(table: &Table, fks: &[ForeignKey]) -> Vec<DiagramColumn> {
    let qn = table.qualified_name();
    let fk_cols: BTreeSet<&str> = fks
        .iter()
        .filter(|f| f.from_table == qn)
        .flat_map(|f| f.from_columns.iter().map(|s| s.as_str()))
        .collect();
    let mut cols: Vec<_> = table.columns.iter().collect();
    cols.sort_by_key(|c| c.ordinal);
    cols.iter()
        .map(|c| {
            let mut key = String::new();
            if c.is_primary_key {
                key.push_str("PK");
            }
            if fk_cols.contains(c.name.as_str()) {
                if !key.is_empty() {
                    key.push('·');
                }
                key.push_str("FK");
            }
            DiagramColumn { name: c.name.clone(), data_type: c.data_type.clone(), key }
        })
        .collect()
}

/// Agrega las relaciones de FK cuyos dos extremos estén en `ids`, sin duplicar.
fn build_relations(ids: &BTreeSet<String>, fks: &[ForeignKey], seen: &mut BTreeSet<(String, String, String)>, out: &mut Vec<DiagramRelation>) {
    for fk in fks {
        let (from, to) = (did(&fk.from_table), did(&fk.to_table));
        if !ids.contains(&from) || !ids.contains(&to) {
            continue;
        }
        let label = if fk.from_columns.is_empty() { fk.name.clone() } else { fk.from_columns.join(", ") };
        if seen.insert((from.clone(), to.clone(), label.clone())) {
            out.push(DiagramRelation { from, to, label });
        }
    }
}

/// Construye el modelo de diagrama a partir de un esquema.
pub fn to_diagram_model(schema: &Schema) -> DiagramModel {
    let tables: Vec<DiagramTable> = schema
        .tables
        .iter()
        .map(|t| DiagramTable {
            id: did(&t.qualified_name()),
            name: t.qualified_name(),
            status: String::new(),
            columns: build_diagram_columns(t, &schema.foreign_keys),
        })
        .collect();

    let ids: BTreeSet<String> = schema.tables.iter().map(|t| did(&t.qualified_name())).collect();
    let mut seen = BTreeSet::new();
    let mut relations = Vec::new();
    build_relations(&ids, &schema.foreign_keys, &mut seen, &mut relations);

    DiagramModel { tables, relations }
}

/// ¿Difieren las columnas de dos tablas? (tipo, nulabilidad o valor por defecto).
fn columns_differ(a: &Table, b: &Table) -> bool {
    let am: BTreeMap<&str, &Column> = a.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let bm: BTreeMap<&str, &Column> = b.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    if am.len() != bm.len() {
        return true;
    }
    for (name, ca) in &am {
        match bm.get(name) {
            Some(cb) => {
                if ca.data_type != cb.data_type || ca.is_nullable != cb.is_nullable || ca.default_value != cb.default_value {
                    return true;
                }
            }
            None => return true,
        }
    }
    false
}

/// Diagrama de DIFERENCIAS: la unión de tablas de A y B, cada una marcada con su
/// estado (only_in_left / only_in_right / different / equal).
pub fn to_diff_diagram(source: &Schema, target: &Schema) -> DiagramModel {
    let src: BTreeMap<String, &Table> = source.tables.iter().map(|t| (t.qualified_name(), t)).collect();
    let tgt: BTreeMap<String, &Table> = target.tables.iter().map(|t| (t.qualified_name(), t)).collect();
    let mut names: Vec<String> = src.keys().chain(tgt.keys()).cloned().collect();
    names.sort();
    names.dedup();

    let mut tables = Vec::new();
    for qn in &names {
        let (status, table, fks) = match (src.get(qn), tgt.get(qn)) {
            (Some(&t), None) => ("only_in_left", t, source.foreign_keys.as_slice()),
            (None, Some(&t)) => ("only_in_right", t, target.foreign_keys.as_slice()),
            (Some(&a), Some(&b)) => {
                let st = if columns_differ(a, b) { "different" } else { "equal" };
                (st, a, source.foreign_keys.as_slice())
            }
            (None, None) => unreachable!(),
        };
        tables.push(DiagramTable {
            id: did(qn),
            name: qn.clone(),
            status: status.to_string(),
            columns: build_diagram_columns(table, fks),
        });
    }

    let ids: BTreeSet<String> = names.iter().map(|qn| did(qn)).collect();
    let mut seen = BTreeSet::new();
    let mut relations = Vec::new();
    build_relations(&ids, &source.foreign_keys, &mut seen, &mut relations);
    build_relations(&ids, &target.foreign_keys, &mut seen, &mut relations);

    DiagramModel { tables, relations }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Column, ForeignKey, Schema, Table};

    fn col(name: &str, ty: &str, pk: bool, ord: i32) -> Column {
        Column { name: name.into(), data_type: ty.into(), is_nullable: true, default_value: None, ordinal: ord, is_primary_key: pk }
    }

    #[test]
    fn modelo_con_pk_fk_y_relacion() {
        let clientes = Table { schema: "dbo".into(), name: "Clientes".into(), columns: vec![col("id", "int", true, 1)] };
        let pedidos = Table {
            schema: "dbo".into(),
            name: "Pedidos".into(),
            columns: vec![col("id", "int", true, 1), col("cliente_id", "int", false, 2)],
        };
        let fk = ForeignKey {
            name: "fk".into(),
            from_table: "dbo.Pedidos".into(),
            from_columns: vec!["cliente_id".into()],
            to_table: "dbo.Clientes".into(),
            to_columns: vec!["id".into()],
        };
        let s = Schema { tables: vec![clientes, pedidos], foreign_keys: vec![fk], ..Default::default() };
        let d = to_diagram_model(&s);

        assert_eq!(d.tables.len(), 2);
        let ped = d.tables.iter().find(|t| t.name == "dbo.Pedidos").unwrap();
        assert_eq!(ped.columns.iter().find(|c| c.name == "id").unwrap().key, "PK");
        assert_eq!(ped.columns.iter().find(|c| c.name == "cliente_id").unwrap().key, "FK");
        assert_eq!(d.relations.len(), 1);
        assert_eq!(d.relations[0].from, "dbo_Pedidos");
        assert_eq!(d.relations[0].to, "dbo_Clientes");
    }

    #[test]
    fn diff_diagram_marca_estados() {
        let a = Schema {
            tables: vec![
                Table { schema: "dbo".into(), name: "Clientes".into(), columns: vec![col("id", "int", true, 1)] },
                Table { schema: "dbo".into(), name: "SoloA".into(), columns: vec![] },
            ],
            ..Default::default()
        };
        let b = Schema {
            tables: vec![
                Table { schema: "dbo".into(), name: "Clientes".into(), columns: vec![col("id", "bigint", true, 1)] },
                Table { schema: "dbo".into(), name: "SoloB".into(), columns: vec![] },
            ],
            ..Default::default()
        };
        let d = to_diff_diagram(&a, &b);
        let st = |n: &str| d.tables.iter().find(|t| t.name == n).unwrap().status.clone();
        assert_eq!(st("dbo.Clientes"), "different");
        assert_eq!(st("dbo.SoloA"), "only_in_left");
        assert_eq!(st("dbo.SoloB"), "only_in_right");
    }
}
