//! Lectura del esquema de una base SQL Server usando el driver `tiberius`.
//!
//! Se leen los catálogos del sistema (`sys.*`): tablas con sus columnas, y los
//! objetos con cuerpo SQL (vistas, procedimientos y funciones). No se leen datos
//! de las filas, solo metadatos — por eso es rápido.

use std::collections::BTreeMap;

use datadiff_core::{CheckConstraint, Column, ForeignKey, Index, Routine, RoutineKind, Schema, Table};
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::{ConnectionConfig, DbResult};

type SqlClient = Client<Compat<TcpStream>>;

/// Abre una conexión a SQL Server con autenticación de usuario/contraseña.
async fn connect(cfg: &ConnectionConfig) -> DbResult<SqlClient> {
    let mut config = Config::new();
    config.host(&cfg.host);
    config.port(cfg.port);
    config.database(&cfg.database);
    config.authentication(AuthMethod::sql_server(&cfg.user, &cfg.password));
    // En desarrollo confiamos en el certificado del servidor (TLS autofirmado).
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let client = Client::connect(config, tcp.compat_write()).await?;
    Ok(client)
}

/// Prueba la conexión y devuelve la primera línea de `@@VERSION`.
pub async fn test_connection(cfg: &ConnectionConfig) -> DbResult<String> {
    let mut client = connect(cfg).await?;
    let row = client
        .query("SELECT @@VERSION AS version", &[])
        .await?
        .into_row()
        .await?;
    let version = row
        .as_ref()
        .and_then(|r| r.get::<&str, _>("version"))
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "desconocida".to_string());
    Ok(format!("Conexión OK · {version}"))
}

/// Lee el esquema completo: tablas+columnas y objetos con cuerpo SQL.
pub async fn read_schema(cfg: &ConnectionConfig) -> DbResult<Schema> {
    let mut client = connect(cfg).await?;
    let tables = read_tables(&mut client).await?;
    let (views, procedures, functions) = read_routines(&mut client).await?;
    let foreign_keys = read_foreign_keys(&mut client).await?;
    let indexes = read_indexes(&mut client).await?;
    let checks = read_checks(&mut client).await?;
    Ok(Schema {
        engine_label: format!("SQL Server · {}", cfg.database),
        tables,
        views,
        procedures,
        functions,
        foreign_keys,
        indexes,
        checks,
    })
}

/// Separa el script en lotes por las líneas que son solo "GO".
fn split_go_batches(script: &str) -> Vec<String> {
    let mut batches = Vec::new();
    let mut cur = String::new();
    for line in script.lines() {
        if line.trim().eq_ignore_ascii_case("GO") {
            batches.push(std::mem::take(&mut cur));
        } else {
            cur.push_str(line);
            cur.push('\n');
        }
    }
    if !cur.trim().is_empty() {
        batches.push(cur);
    }
    batches
}

/// Ejecuta el script contra la base, lote por lote (separados por GO).
pub async fn apply_script(cfg: &ConnectionConfig, script: &str) -> DbResult<String> {
    let mut client = connect(cfg).await?;
    for batch in split_go_batches(script) {
        if batch.trim().is_empty() {
            continue;
        }
        client.simple_query(batch.as_str()).await?.into_results().await?;
    }
    Ok("Cambios aplicados a SQL Server.".to_string())
}

const TABLES_SQL: &str = "\
SELECT
    s.name  AS schema_name,
    t.name  AS table_name,
    c.name  AS column_name,
    ty.name AS data_type,
    c.max_length AS max_length,
    c.precision  AS precision,
    c.scale      AS scale,
    c.is_nullable AS is_nullable,
    c.column_id   AS column_id,
    OBJECT_DEFINITION(c.default_object_id) AS default_definition,
    CASE WHEN pk.column_id IS NOT NULL THEN 1 ELSE 0 END AS is_pk
FROM sys.tables t
JOIN sys.schemas s ON t.schema_id = s.schema_id
JOIN sys.columns c ON c.object_id = t.object_id
JOIN sys.types  ty ON c.user_type_id = ty.user_type_id
LEFT JOIN (
    SELECT ic.object_id, ic.column_id
    FROM sys.indexes i
    JOIN sys.index_columns ic ON i.object_id = ic.object_id AND i.index_id = ic.index_id
    WHERE i.is_primary_key = 1
) pk ON pk.object_id = c.object_id AND pk.column_id = c.column_id
ORDER BY s.name, t.name, c.column_id";

async fn read_tables(client: &mut SqlClient) -> DbResult<Vec<Table>> {
    let rows = client.query(TABLES_SQL, &[]).await?.into_first_result().await?;

    // Agrupamos columnas por tabla, preservando el orden de aparición.
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Table> = BTreeMap::new();

    for row in rows {
        let schema: &str = row.get("schema_name").unwrap_or("");
        let table: &str = row.get("table_name").unwrap_or("");
        let col_name: &str = row.get("column_name").unwrap_or("");
        let data_type: &str = row.get("data_type").unwrap_or("");
        let max_length: i16 = row.get("max_length").unwrap_or(0);
        let precision: u8 = row.get("precision").unwrap_or(0);
        let scale: u8 = row.get("scale").unwrap_or(0);
        let is_nullable: bool = row.get("is_nullable").unwrap_or(true);
        let column_id: i32 = row.get("column_id").unwrap_or(0);
        let default_def: Option<&str> = row.get("default_definition");
        let is_pk: i32 = row.get("is_pk").unwrap_or(0);

        let key = format!("{schema}.{table}");
        map.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            Table { schema: schema.to_string(), name: table.to_string(), columns: Vec::new() }
        });

        if let Some(t) = map.get_mut(&key) {
            t.columns.push(Column {
                name: col_name.to_string(),
                data_type: format_type(data_type, max_length, precision, scale),
                is_nullable,
                default_value: default_def.map(|d| d.trim().to_string()),
                ordinal: column_id,
                is_primary_key: is_pk != 0,
            });
        }
    }

    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

/// Arma el tipo con longitud/precisión al estilo SQL, p.ej. "varchar(50)".
fn format_type(name: &str, max_length: i16, precision: u8, scale: u8) -> String {
    match name {
        "varchar" | "char" | "varbinary" | "binary" => {
            if max_length == -1 {
                format!("{name}(max)")
            } else {
                format!("{name}({max_length})")
            }
        }
        // nchar/nvarchar guardan 2 bytes por carácter: max_length es en bytes.
        "nvarchar" | "nchar" => {
            if max_length == -1 {
                format!("{name}(max)")
            } else {
                format!("{name}({})", max_length / 2)
            }
        }
        "decimal" | "numeric" => format!("{name}({precision},{scale})"),
        other => other.to_string(),
    }
}

const ROUTINES_SQL: &str = "\
SELECT
    s.name        AS schema_name,
    o.name        AS object_name,
    RTRIM(o.type) AS object_type,
    m.definition  AS definition
FROM sys.sql_modules m
JOIN sys.objects o ON m.object_id = o.object_id
JOIN sys.schemas s ON o.schema_id = s.schema_id
WHERE o.type IN ('P','FN','IF','TF','V')
ORDER BY s.name, o.name";

/// Devuelve (vistas, procedimientos, funciones).
async fn read_routines(
    client: &mut SqlClient,
) -> DbResult<(Vec<Routine>, Vec<Routine>, Vec<Routine>)> {
    let rows = client.query(ROUTINES_SQL, &[]).await?.into_first_result().await?;

    let mut views = Vec::new();
    let mut procedures = Vec::new();
    let mut functions = Vec::new();

    for row in rows {
        let schema: &str = row.get("schema_name").unwrap_or("");
        let name: &str = row.get("object_name").unwrap_or("");
        let otype: &str = row.get("object_type").unwrap_or("");
        let definition: &str = row.get("definition").unwrap_or("");

        let make = |kind: RoutineKind| Routine {
            schema: schema.to_string(),
            name: name.to_string(),
            kind,
            definition: definition.to_string(),
        };

        match otype {
            "V" => views.push(make(RoutineKind::View)),
            "P" => procedures.push(make(RoutineKind::Procedure)),
            "FN" => functions.push(make(RoutineKind::ScalarFunction)),
            "IF" | "TF" => functions.push(make(RoutineKind::TableFunction)),
            _ => {}
        }
    }

    Ok((views, procedures, functions))
}

const FK_SQL: &str = "\
SELECT
    fk.name AS fk_name,
    sp.name AS parent_schema,
    tp.name AS parent_table,
    cp.name AS parent_col,
    sr.name AS ref_schema,
    tr.name AS ref_table,
    cr.name AS ref_col
FROM sys.foreign_keys fk
JOIN sys.foreign_key_columns fkc ON fkc.constraint_object_id = fk.object_id
JOIN sys.tables  tp ON tp.object_id = fkc.parent_object_id
JOIN sys.schemas sp ON sp.schema_id = tp.schema_id
JOIN sys.columns cp ON cp.object_id = fkc.parent_object_id AND cp.column_id = fkc.parent_column_id
JOIN sys.tables  tr ON tr.object_id = fkc.referenced_object_id
JOIN sys.schemas sr ON sr.schema_id = tr.schema_id
JOIN sys.columns cr ON cr.object_id = fkc.referenced_object_id AND cr.column_id = fkc.referenced_column_id
ORDER BY fk.name, fkc.constraint_column_id";

/// Lee las relaciones de clave foránea del esquema.
async fn read_foreign_keys(client: &mut SqlClient) -> DbResult<Vec<ForeignKey>> {
    use std::collections::BTreeMap;
    let rows = client.query(FK_SQL, &[]).await?.into_first_result().await?;

    let mut by_name: BTreeMap<String, ForeignKey> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();

    for row in rows {
        let fk_name: &str = row.get("fk_name").unwrap_or("");
        let ps: &str = row.get("parent_schema").unwrap_or("");
        let pt: &str = row.get("parent_table").unwrap_or("");
        let pc: &str = row.get("parent_col").unwrap_or("");
        let rs: &str = row.get("ref_schema").unwrap_or("");
        let rt: &str = row.get("ref_table").unwrap_or("");
        let rc: &str = row.get("ref_col").unwrap_or("");

        let key = fk_name.to_string();
        if !by_name.contains_key(&key) {
            order.push(key.clone());
            by_name.insert(
                key.clone(),
                ForeignKey {
                    name: fk_name.to_string(),
                    from_table: format!("{ps}.{pt}"),
                    from_columns: Vec::new(),
                    to_table: format!("{rs}.{rt}"),
                    to_columns: Vec::new(),
                },
            );
        }
        if let Some(fk) = by_name.get_mut(&key) {
            fk.from_columns.push(pc.to_string());
            fk.to_columns.push(rc.to_string());
        }
    }

    Ok(order.into_iter().filter_map(|n| by_name.remove(&n)).collect())
}

const INDEX_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, i.name AS index_name,
       i.is_unique AS is_unique, c.name AS col_name
FROM sys.indexes i
JOIN sys.tables  t ON t.object_id = i.object_id
JOIN sys.schemas s ON s.schema_id = t.schema_id
JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id
JOIN sys.columns c ON c.object_id = ic.object_id AND c.column_id = ic.column_id
WHERE i.is_primary_key = 0 AND i.type > 0 AND i.name IS NOT NULL
ORDER BY s.name, t.name, i.name, ic.key_ordinal";

/// Lee los índices (sin contar el de la clave primaria).
async fn read_indexes(client: &mut SqlClient) -> DbResult<Vec<Index>> {
    use std::collections::BTreeMap;
    let rows = client.query(INDEX_SQL, &[]).await?.into_first_result().await?;
    let mut map: BTreeMap<String, Index> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for row in rows {
        let s: &str = row.get("schema_name").unwrap_or("");
        let t: &str = row.get("table_name").unwrap_or("");
        let iname: &str = row.get("index_name").unwrap_or("");
        let uniq: bool = row.get("is_unique").unwrap_or(false);
        let col: &str = row.get("col_name").unwrap_or("");
        let key = format!("{s}.{t}.{iname}");
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(
                key.clone(),
                Index { name: iname.to_string(), table: format!("{s}.{t}"), columns: Vec::new(), is_unique: uniq },
            );
        }
        if let Some(ix) = map.get_mut(&key) {
            ix.columns.push(col.to_string());
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

const CHECK_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, cc.name AS check_name, cc.definition AS definition
FROM sys.check_constraints cc
JOIN sys.tables  t ON t.object_id = cc.parent_object_id
JOIN sys.schemas s ON s.schema_id = t.schema_id
ORDER BY s.name, t.name, cc.name";

/// Lee las restricciones CHECK.
async fn read_checks(client: &mut SqlClient) -> DbResult<Vec<CheckConstraint>> {
    let rows = client.query(CHECK_SQL, &[]).await?.into_first_result().await?;
    let mut out = Vec::new();
    for row in rows {
        let s: &str = row.get("schema_name").unwrap_or("");
        let t: &str = row.get("table_name").unwrap_or("");
        let n: &str = row.get("check_name").unwrap_or("");
        let d: &str = row.get("definition").unwrap_or("");
        out.push(CheckConstraint { name: n.to_string(), table: format!("{s}.{t}"), definition: d.to_string() });
    }
    Ok(out)
}
