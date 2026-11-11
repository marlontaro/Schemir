//! Lectura del esquema de una base SQLite (un archivo local).
//!
//! Nota: SQLite no tiene procedimientos ni funciones almacenadas en el esquema,
//! así que esas listas quedan vacías. Sí tiene tablas y vistas.
//! Abrimos el archivo en modo SOLO LECTURA: nunca creamos ni modificamos nada.

use datadiff_core::{Column, ForeignKey, Index, Routine, RoutineKind, Schema, Table};
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
