//! Lectura del esquema de una base PostgreSQL usando `tokio-postgres`.
//!
//! Lee los catálogos del sistema (`pg_catalog`): tablas con columnas (PK incluida),
//! vistas, funciones/procedimientos, claves foráneas, índices y restricciones CHECK.
//! Para el MVP conecta sin TLS (`NoTls`), suficiente para desarrollo local.

use std::collections::BTreeMap;

use schemir_core::{CheckConstraint, Column, ForeignKey, Index, Routine, RoutineKind, Schema, Table, Trigger};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use tokio_postgres::{Client, Config};

use super::{ConnectionConfig, DbResult};

/// Abre una conexión y lanza la tarea que maneja el protocolo.
async fn connect(cfg: &ConnectionConfig) -> DbResult<Client> {
    let mut config = Config::new();
    config
        .host(&cfg.host)
        .port(cfg.port)
        .dbname(&cfg.database)
        .user(&cfg.user)
        .password(&cfg.password);
    // TLS que confía en el certificado del servidor (como trust_cert en SQL Server).
    // Con sslmode=prefer (por defecto) usa TLS si el servidor lo soporta y texto plano si no,
    // así funciona con Postgres local y con la nube (RDS, Supabase…) que exige SSL.
    let tls = TlsConnector::builder().danger_accept_invalid_certs(true).build()?;
    let (client, connection) = config.connect(MakeTlsConnector::new(tls)).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(client)
}

/// Prueba la conexión y devuelve la versión del servidor.
pub async fn test_connection(cfg: &ConnectionConfig) -> DbResult<String> {
    let client = connect(cfg).await?;
    let version: String = client.query_one("SELECT version()", &[]).await?.get(0);
    let tables: i64 = client
        .query_one(
            "SELECT count(*) FROM information_schema.tables \
             WHERE table_type = 'BASE TABLE' AND table_schema NOT IN ('pg_catalog','information_schema')",
            &[],
        )
        .await?
        .get(0);
    let short = version.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
    Ok(format!("Conexión OK · {short} · {tables} tablas"))
}

/// Lee el esquema completo.
pub async fn read_schema(cfg: &ConnectionConfig) -> DbResult<Schema> {
    let client = connect(cfg).await?;
    let tables = read_tables(&client).await?;
    let (views, procedures, functions) = read_routines(&client).await?;
    let foreign_keys = read_foreign_keys(&client).await?;
    let indexes = read_indexes(&client).await?;
    let checks = read_checks(&client).await?;
    let triggers = read_triggers(&client).await?;
    Ok(Schema {
        engine_label: format!("PostgreSQL · {}", cfg.database),
        tables,
        views,
        procedures,
        functions,
        foreign_keys,
        indexes,
        checks,
        triggers,
    })
}

/// Ejecuta el script contra la base. Postgres corre un lote multi-sentencia como
/// una sola transacción implícita, así que un error revierte todo.
pub async fn apply_script(cfg: &ConnectionConfig, script: &str) -> DbResult<String> {
    let client = connect(cfg).await?;
    client.batch_execute(script).await?;
    Ok("Cambios aplicados a PostgreSQL.".to_string())
}

const TABLES_SQL: &str = "\
SELECT
    n.nspname AS schema_name,
    c.relname AS table_name,
    a.attname AS column_name,
    format_type(a.atttypid, a.atttypmod) AS data_type,
    (NOT a.attnotnull) AS is_nullable,
    pg_get_expr(d.adbin, d.adrelid) AS default_value,
    a.attnum::int AS ordinal,
    COALESCE(pk.is_pk, false) AS is_primary_key
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
JOIN pg_attribute a ON a.attrelid = c.oid
LEFT JOIN pg_attrdef d ON d.adrelid = c.oid AND d.adnum = a.attnum
LEFT JOIN (
    SELECT i.indrelid, a2.attnum, true AS is_pk
    FROM pg_index i
    JOIN pg_attribute a2 ON a2.attrelid = i.indrelid AND a2.attnum = ANY (i.indkey)
    WHERE i.indisprimary
) pk ON pk.indrelid = c.oid AND pk.attnum = a.attnum
WHERE c.relkind = 'r' AND a.attnum > 0 AND NOT a.attisdropped
  AND n.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY n.nspname, c.relname, a.attnum";

async fn read_tables(client: &Client) -> DbResult<Vec<Table>> {
    let rows = client.query(TABLES_SQL, &[]).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Table> = BTreeMap::new();

    for row in &rows {
        let schema: String = row.get("schema_name");
        let table: String = row.get("table_name");
        let key = format!("{schema}.{table}");
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(key.clone(), Table { schema: schema.clone(), name: table.clone(), columns: Vec::new() });
        }
        if let Some(t) = map.get_mut(&key) {
            t.columns.push(Column {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                default_value: row.get("default_value"),
                ordinal: row.get("ordinal"),
                is_primary_key: row.get("is_primary_key"),
            });
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

const VIEWS_SQL: &str = "\
SELECT schemaname AS schema_name, viewname AS view_name, definition
FROM pg_views
WHERE schemaname NOT IN ('pg_catalog','information_schema')
ORDER BY schemaname, viewname";

const ROUTINES_SQL: &str = "\
SELECT n.nspname AS schema_name, p.proname AS name, p.prokind::text AS kind,
       pg_get_functiondef(p.oid) AS definition
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
WHERE n.nspname NOT IN ('pg_catalog','information_schema')
  AND p.prokind IN ('f','p')
ORDER BY n.nspname, p.proname";

async fn read_routines(client: &Client) -> DbResult<(Vec<Routine>, Vec<Routine>, Vec<Routine>)> {
    let mut views = Vec::new();
    for row in &client.query(VIEWS_SQL, &[]).await? {
        let schema: String = row.get("schema_name");
        let name: String = row.get("view_name");
        let def: String = row.get("definition");
        let definition = format!("CREATE OR REPLACE VIEW \"{schema}\".\"{name}\" AS {def}");
        views.push(Routine { schema, name, kind: RoutineKind::View, definition });
    }

    let mut procedures = Vec::new();
    let mut functions = Vec::new();
    for row in &client.query(ROUTINES_SQL, &[]).await? {
        let schema: String = row.get("schema_name");
        let name: String = row.get("name");
        let kind: String = row.get("kind");
        let definition: String = row.get("definition");
        if kind == "p" {
            procedures.push(Routine { schema, name, kind: RoutineKind::Procedure, definition });
        } else {
            functions.push(Routine { schema, name, kind: RoutineKind::ScalarFunction, definition });
        }
    }

    Ok((views, procedures, functions))
}

const FK_SQL: &str = "\
SELECT con.conname AS fk_name,
       ns.nspname  AS from_schema, cl.relname  AS from_table, att.attname  AS from_col,
       fns.nspname AS to_schema,  fcl.relname AS to_table,  fatt.attname AS to_col
FROM pg_constraint con
JOIN pg_class cl ON cl.oid = con.conrelid
JOIN pg_namespace ns ON ns.oid = cl.relnamespace
JOIN pg_class fcl ON fcl.oid = con.confrelid
JOIN pg_namespace fns ON fns.oid = fcl.relnamespace
JOIN LATERAL unnest(con.conkey) WITH ORDINALITY AS k(attnum, ord) ON true
JOIN pg_attribute att  ON att.attrelid = con.conrelid  AND att.attnum = k.attnum
JOIN pg_attribute fatt ON fatt.attrelid = con.confrelid AND fatt.attnum = con.confkey[k.ord]
WHERE con.contype = 'f' AND ns.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY con.conname, k.ord";

async fn read_foreign_keys(client: &Client) -> DbResult<Vec<ForeignKey>> {
    let rows = client.query(FK_SQL, &[]).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, ForeignKey> = BTreeMap::new();
    for row in &rows {
        let name: String = row.get("fk_name");
        let from_table = format!("{}.{}", row.get::<_, String>("from_schema"), row.get::<_, String>("from_table"));
        let to_table = format!("{}.{}", row.get::<_, String>("to_schema"), row.get::<_, String>("to_table"));
        if !map.contains_key(&name) {
            order.push(name.clone());
            map.insert(name.clone(), ForeignKey { name: name.clone(), from_table, from_columns: Vec::new(), to_table, to_columns: Vec::new() });
        }
        if let Some(fk) = map.get_mut(&name) {
            fk.from_columns.push(row.get("from_col"));
            fk.to_columns.push(row.get("to_col"));
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

const INDEX_SQL: &str = "\
SELECT ns.nspname AS schema_name, t.relname AS table_name, i.relname AS index_name,
       ix.indisunique AS is_unique, a.attname AS col_name
FROM pg_index ix
JOIN pg_class i ON i.oid = ix.indexrelid
JOIN pg_class t ON t.oid = ix.indrelid
JOIN pg_namespace ns ON ns.oid = t.relnamespace
JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS k(attnum, ord) ON true
JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = k.attnum
WHERE NOT ix.indisprimary AND k.attnum > 0
  AND ns.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY ns.nspname, t.relname, i.relname, k.ord";

async fn read_indexes(client: &Client) -> DbResult<Vec<Index>> {
    let rows = client.query(INDEX_SQL, &[]).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Index> = BTreeMap::new();
    for row in &rows {
        let schema: String = row.get("schema_name");
        let table: String = row.get("table_name");
        let iname: String = row.get("index_name");
        let key = format!("{schema}.{table}.{iname}");
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(key.clone(), Index { name: iname.clone(), table: format!("{schema}.{table}"), columns: Vec::new(), is_unique: row.get("is_unique") });
        }
        if let Some(ix) = map.get_mut(&key) {
            ix.columns.push(row.get("col_name"));
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

const CHECK_SQL: &str = "\
SELECT ns.nspname AS schema_name, cl.relname AS table_name, con.conname AS check_name,
       pg_get_constraintdef(con.oid) AS definition
FROM pg_constraint con
JOIN pg_class cl ON cl.oid = con.conrelid
JOIN pg_namespace ns ON ns.oid = cl.relnamespace
WHERE con.contype = 'c' AND ns.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY ns.nspname, cl.relname, con.conname";

async fn read_checks(client: &Client) -> DbResult<Vec<CheckConstraint>> {
    let rows = client.query(CHECK_SQL, &[]).await?;
    let mut out = Vec::new();
    for row in &rows {
        let schema: String = row.get("schema_name");
        let table: String = row.get("table_name");
        out.push(CheckConstraint {
            name: row.get("check_name"),
            table: format!("{schema}.{table}"),
            definition: row.get("definition"),
        });
    }
    Ok(out)
}

const TRIGGER_SQL: &str = "\
SELECT ns.nspname AS schema_name, cl.relname AS table_name, tg.tgname AS trigger_name,
       pg_get_triggerdef(tg.oid) AS definition
FROM pg_trigger tg
JOIN pg_class cl ON cl.oid = tg.tgrelid
JOIN pg_namespace ns ON ns.oid = cl.relnamespace
WHERE NOT tg.tgisinternal AND ns.nspname NOT IN ('pg_catalog','information_schema')
ORDER BY ns.nspname, cl.relname, tg.tgname";

async fn read_triggers(client: &Client) -> DbResult<Vec<Trigger>> {
    let rows = client.query(TRIGGER_SQL, &[]).await?;
    let mut out = Vec::new();
    for row in &rows {
        let schema: String = row.get("schema_name");
        let table: String = row.get("table_name");
        out.push(Trigger {
            schema: schema.clone(),
            name: row.get("trigger_name"),
            table: format!("{schema}.{table}"),
            definition: row.get("definition"),
        });
    }
    Ok(out)
}
