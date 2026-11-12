//! Lectura del esquema de una base SQLite (un archivo local).
//!
//! Nota: SQLite no tiene procedimientos ni funciones almacenadas en el esquema,
//! así que esas listas quedan vacías. Sí tiene tablas y vistas.
//! Abrimos el archivo en modo SOLO LECTURA: nunca creamos ni modificamos nada.

use datadiff_core::{Column, ColumnKind, DataColumn, DataTable, ForeignKey, Index, Routine, RoutineKind, Schema, Table, Trigger};
use rusqlite::{Connection, OpenFlags};

use super::DbResult;

/// Abre el archivo SQLite en modo solo-lectura, validando antes la ruta.
/// Así evitamos el caso confuso en que una ruta vacía o inexistente "funciona"
/// creando una base nueva y vacía.
fn open_ro(path: &str) -> DbResult<Connection> {
    if path.trim().is_empty() {
        return Err("Indica la ruta del archivo SQLite.".into());
    }
    if !std::path::Path::new(path).exists() {
        return Err(format!("No existe el archivo: {path}").into());
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(conn)
}

/// Prueba la conexión abriendo el archivo y devolviendo la versión de SQLite.
pub fn test_connection(path: &str) -> DbResult<String> {
    let conn = open_ro(path)?;
    let version: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let tables: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    Ok(format!("Conexión OK · SQLite {version} · {tables} tablas"))
}

/// Lee tablas (con columnas) y vistas de la base.
pub fn read_schema(path: &str) -> DbResult<Schema> {
    let conn = open_ro(path)?;
    let tables = read_tables(&conn)?;
    let foreign_keys = read_foreign_keys(&conn, &tables)?;
    let indexes = read_indexes(&conn, &tables)?;
    Ok(Schema {
        engine_label: format!("SQLite · {}", file_label(path)),
        views: read_views(&conn)?,
        procedures: Vec::new(),
        functions: Vec::new(),
        foreign_keys,
        indexes,
        checks: Vec::new(), // SQLite guarda los CHECK dentro del DDL de la tabla
        triggers: read_triggers(&conn)?,
        tables,
    })
}

fn file_label(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Ejecuta un script SQL contra la base en modo escritura, dentro de una
/// transacción: si algo falla, se revierte todo (no deja la base a medias).
pub fn apply_script(path: &str, script: &str) -> DbResult<String> {
    if path.trim().is_empty() {
        return Err("Indica la ruta del archivo SQLite.".into());
    }
    if !std::path::Path::new(path).exists() {
        return Err(format!("No existe el archivo: {path}").into());
    }
    let mut conn = Connection::open(path)?; // lectura-escritura
    let tx = conn.transaction()?;
    tx.execute_batch(script)?;
    tx.commit()?;
    Ok("Cambios aplicados a la base SQLite.".to_string())
}

fn read_tables(conn: &Connection) -> DbResult<Vec<Table>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master \
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;

    let mut tables = Vec::with_capacity(names.len());
    for name in names {
        let columns = read_columns(conn, &name)?;
        tables.push(Table { schema: "main".to_string(), name, columns });
    }
    Ok(tables)
}

fn read_columns(conn: &Connection, table: &str) -> DbResult<Vec<Column>> {
    // PRAGMA no admite parámetros; el nombre viene de sqlite_master, aun así
    // escapamos las comillas dobles por prolijidad.
    let sql = format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\""));
    let mut stmt = conn.prepare(&sql)?;
    // Columnas de table_info: cid, name, type, notnull, dflt_value, pk
    let cols = stmt
        .query_map([], |row| {
            let cid: i32 = row.get("cid")?;
            let name: String = row.get("name")?;
            let decl_type: String = row.get::<_, Option<String>>("type")?.unwrap_or_default();
            let notnull: i32 = row.get("notnull")?;
            let default: Option<String> = row.get("dflt_value")?;
            let pk: i32 = row.get("pk")?;
            Ok(Column {
                name,
                data_type: if decl_type.is_empty() {
                    "BLOB".to_string()
                } else {
                    decl_type.to_lowercase()
                },
                is_nullable: notnull == 0,
                default_value: default,
                ordinal: cid,
                is_primary_key: pk > 0,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(cols)
}

fn read_foreign_keys(conn: &Connection, tables: &[Table]) -> DbResult<Vec<ForeignKey>> {
    use std::collections::BTreeMap;
    let mut fks = Vec::new();
    for t in tables {
        let sql = format!("PRAGMA foreign_key_list(\"{}\")", t.name.replace('"', "\"\""));
        let mut stmt = conn.prepare(&sql)?;
        // Columnas: id, seq, table (referenciada), from (col local), to (col ref), ...
        let rows: Vec<(i64, String, String, Option<String>)> = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>("id")?,
                    r.get::<_, String>("table")?,
                    r.get::<_, String>("from")?,
                    r.get::<_, Option<String>>("to")?,
                ))
            })?
            .collect::<Result<_, _>>()?;

        let mut by_id: BTreeMap<i64, ForeignKey> = BTreeMap::new();
        for (id, ref_table, from_col, to_col) in rows {
            let fk = by_id.entry(id).or_insert_with(|| ForeignKey {
                name: format!("fk_{}_{id}", t.name),
                from_table: format!("main.{}", t.name),
                from_columns: Vec::new(),
                to_table: format!("main.{ref_table}"),
                to_columns: Vec::new(),
            });
            fk.from_columns.push(from_col);
            if let Some(tc) = to_col {
                fk.to_columns.push(tc);
            }
        }
        fks.extend(by_id.into_values());
    }
    Ok(fks)
}

fn read_indexes(conn: &Connection, tables: &[Table]) -> DbResult<Vec<Index>> {
    let mut out = Vec::new();
    for t in tables {
        let list_sql = format!("PRAGMA index_list(\"{}\")", t.name.replace('"', "\"\""));
        let mut stmt = conn.prepare(&list_sql)?;
        // Columnas: seq, name, unique, origin, partial
        let idxs: Vec<(String, i64, String)> = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>("name")?,
                    r.get::<_, i64>("unique")?,
                    r.get::<_, String>("origin")?,
                ))
            })?
            .collect::<Result<_, _>>()?;

        for (iname, uniq, origin) in idxs {
            if origin == "pk" {
                continue; // el índice de la PK se compara como clave primaria
            }
            let info_sql = format!("PRAGMA index_info(\"{}\")", iname.replace('"', "\"\""));
            let mut s2 = conn.prepare(&info_sql)?;
            // Columnas: seqno, cid, name
            let cols: Vec<String> = s2
                .query_map([], |r| r.get::<_, Option<String>>("name"))?
                .collect::<Result<Vec<Option<String>>, _>>()?
                .into_iter()
                .flatten()
                .collect();
            out.push(Index {
                name: iname,
                table: format!("main.{}", t.name),
                columns: cols,
                is_unique: uniq != 0,
            });
        }
    }
    Ok(out)
}

fn read_views(conn: &Connection) -> DbResult<Vec<Routine>> {
    let mut stmt = conn.prepare(
        "SELECT name, COALESCE(sql, '') AS definition FROM sqlite_master \
         WHERE type = 'view' ORDER BY name",
    )?;
    let views = stmt
        .query_map([], |row| {
            Ok(Routine {
                schema: "main".to_string(),
                name: row.get("name")?,
                kind: RoutineKind::View,
                definition: row.get("definition")?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(views)
}

fn read_triggers(conn: &Connection) -> DbResult<Vec<Trigger>> {
    let mut stmt = conn.prepare(
        "SELECT name, tbl_name, COALESCE(sql, '') AS definition FROM sqlite_master \
         WHERE type = 'trigger' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let triggers = stmt
        .query_map([], |row| {
            let name: String = row.get("name")?;
            let tbl: String = row.get("tbl_name")?;
            Ok(Trigger {
                schema: "main".to_string(),
                name,
                table: format!("main.{tbl}"),
                definition: row.get("definition")?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(triggers)
}

/// Límite de filas por tabla para comparar datos (evita leer tablas enormes).
const DATA_ROW_CAP: i64 = 100_000;

/// Clasifica el tipo declarado de SQLite para saber cómo entrecomillar en el DML.
fn sqlite_kind(ty: &str) -> ColumnKind {
    let t = ty.to_lowercase();
    if t.contains("int") || t.contains("real") || t.contains("floa") || t.contains("doub") || t.contains("num") || t.contains("dec") {
        ColumnKind::Numeric
    } else if t.contains("bool") {
        ColumnKind::Bool
    } else if t.contains("blob") {
        ColumnKind::Blob
    } else {
        ColumnKind::Text
    }
}

/// Convierte el valor de una celda a texto canónico (o `None` para NULL).
fn cell_to_text(v: rusqlite::types::ValueRef<'_>) -> Option<String> {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => None,
        ValueRef::Integer(n) => Some(n.to_string()),
        ValueRef::Real(f) => Some(f.to_string()),
        ValueRef::Text(t) => Some(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => Some(b.iter().map(|x| format!("{x:02X}")).collect()),
    }
}

/// Lee los datos de una tabla para comparar filas (columnas, PK y filas).
pub fn read_table_data(path: &str, qualified: &str) -> DbResult<DataTable> {
    // "main.clientes" -> "clientes"
    let table = qualified.rsplit('.').next().unwrap_or(qualified).to_string();
    let quoted = format!("\"{}\"", table.replace('"', "\"\""));
    let conn = open_ro(path)?;

    // Columnas, tipos y clave primaria.
    let info_sql = format!("PRAGMA table_info({quoted})");
    let mut stmt = conn.prepare(&info_sql)?;
    let info: Vec<(String, String, i32)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>("name")?,
                r.get::<_, Option<String>>("type")?.unwrap_or_default(),
                r.get::<_, i32>("pk")?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    if info.is_empty() {
        return Err(format!("La tabla «{table}» no existe o no tiene columnas.").into());
    }

    let mut columns = Vec::with_capacity(info.len());
    let mut pk_order: Vec<(i32, String)> = Vec::new();
    for (name, ty, pk) in &info {
        columns.push(DataColumn { name: name.clone(), kind: sqlite_kind(ty) });
        if *pk > 0 {
            pk_order.push((*pk, name.clone()));
        }
    }
    pk_order.sort();
    let pk: Vec<String> = pk_order.into_iter().map(|(_, n)| n).collect();

    // ¿Demasiado grande? No la comparamos (sería parcial y peligroso).
    let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {quoted}"), [], |r| r.get(0))?;
    if count > DATA_ROW_CAP {
        return Ok(DataTable { table: qualified.to_string(), columns, pk, rows: Vec::new(), truncated: true });
    }

    // Filas.
    let collist = columns
        .iter()
        .map(|c| format!("\"{}\"", c.name.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(", ");
    let sel = format!("SELECT {collist} FROM {quoted}");
    let mut s2 = conn.prepare(&sel)?;
    let n = columns.len();
    let rows: Vec<Vec<Option<String>>> = s2
        .query_map([], |row| {
            let mut cells = Vec::with_capacity(n);
            for i in 0..n {
                cells.push(cell_to_text(row.get_ref(i)?));
            }
            Ok(cells)
        })?
        .collect::<Result<_, _>>()?;

    Ok(DataTable { table: qualified.to_string(), columns, pk, rows, truncated: false })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_disparadores_de_sqlite() {
        // Crea una base temporal con un trigger y confirma que read_schema lo lee.
        let path = std::env::temp_dir().join(format!("datadiff_trg_{}.db", std::process::id()));
        let p = path.to_string_lossy().into_owned();
        let _ = std::fs::remove_file(&p);
        {
            let conn = Connection::open(&p).unwrap();
            conn.execute_batch(
                "CREATE TABLE t(id INTEGER PRIMARY KEY, n TEXT);\
                 CREATE TABLE log(msg TEXT);\
                 CREATE TRIGGER trg_ins AFTER INSERT ON t BEGIN INSERT INTO log(msg) VALUES('x'); END;",
            )
            .unwrap();
        }
        let schema = read_schema(&p).unwrap();
        let _ = std::fs::remove_file(&p);
        assert_eq!(schema.triggers.len(), 1, "debe leer 1 disparador");
        let tr = &schema.triggers[0];
        assert_eq!(tr.name, "trg_ins");
        assert_eq!(tr.table, "main.t");
        assert!(tr.definition.to_uppercase().contains("CREATE TRIGGER"), "def: {}", tr.definition);
    }

    #[test]
    fn compara_datos_sqlite_end_to_end() {
        use datadiff_core::{compare_table_data, Dialect};
        let path = std::env::temp_dir().join(format!("datadiff_data_{}.db", std::process::id()));
        let p = path.to_string_lossy().into_owned();
        let _ = std::fs::remove_file(&p);
        {
            let conn = Connection::open(&p).unwrap();
            conn.execute_batch(
                "CREATE TABLE cli(id INTEGER PRIMARY KEY, nombre TEXT, saldo REAL);\
                 INSERT INTO cli VALUES (1,'Ana',10.5),(2,'Beto',0);",
            )
            .unwrap();
        }
        let dt = read_table_data(&p, "main.cli").unwrap();
        let _ = std::fs::remove_file(&p);
        assert_eq!(dt.pk, vec!["id".to_string()]);
        assert_eq!(dt.columns.len(), 3);
        assert_eq!(dt.rows.len(), 2);
        // Comparar contra una tabla vacía → 2 INSERT, con tipos bien entrecomillados.
        let empty = DataTable { table: "main.cli".into(), columns: dt.columns.clone(), pk: dt.pk.clone(), rows: vec![], truncated: false };
        let c = compare_table_data(&dt, &empty, Dialect::Sqlite);
        assert_eq!(c.only_in_left, 2);
        assert!(c.dml.contains("INSERT INTO \"main\".\"cli\""), "{}", c.dml);
        assert!(c.dml.contains("'Ana'"), "{}", c.dml); // texto entrecomillado
        assert!(c.dml.contains("10.5"), "{}", c.dml); // número sin comillas
    }
}
