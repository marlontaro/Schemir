//! Lectura del esquema de una base SQL Server usando el driver `tiberius`.
//!
//! Se leen los catálogos del sistema (`sys.*`): tablas con sus columnas, y los
//! objetos con cuerpo SQL (vistas, procedimientos y funciones). No se leen datos
//! de las filas, solo metadatos — por eso es rápido.

use std::collections::BTreeMap;

use schemir_core::{CheckConstraint, Column, ForeignKey, Identity, Index, Routine, RoutineKind, Schema, Table, Trigger};
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::row::{Row, Value};
use super::{ConnectionConfig, DbResult};

type SqlClient = Client<Compat<TcpStream>>;

/// Autenticación integrada: entra como el usuario de Windows que tiene la
/// sesión abierta, por SSPI, sin contraseña de por medio. `AuthMethod::Integrated`
/// solo existe en Windows (y en Unix con GSSAPI, que no habilitamos), así que en
/// los demás sistemas se devuelve un error legible en vez de no compilar.
#[cfg(windows)]
fn integrated_auth() -> DbResult<AuthMethod> {
    Ok(AuthMethod::Integrated)
}

#[cfg(not(windows))]
fn integrated_auth() -> DbResult<AuthMethod> {
    Err("La autenticación de Windows solo está disponible cuando Schemir corre en Windows; usa usuario y contraseña.".into())
}

/// Abre una conexión a SQL Server, con autenticación de Windows o de SQL Server.
async fn connect(cfg: &ConnectionConfig) -> DbResult<SqlClient> {
    let mut config = Config::new();
    config.host(&cfg.host);
    config.port(cfg.port);
    config.database(&cfg.database);
    config.authentication(if cfg.windows_auth {
        integrated_auth()?
    } else {
        AuthMethod::sql_server(&cfg.user, &cfg.password)
    });
    // En desarrollo confiamos en el certificado del servidor (TLS autofirmado).
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let client = Client::connect(config, tcp.compat_write()).await?;
    Ok(client)
}

/// Prueba la conexión y devuelve la primera línea de `@@VERSION`, diciendo
/// además por qué transporte entró: no es lo mismo haber llegado por ODBC
/// (memoria compartida, canalizaciones o TCP, como SSMS) que por TCP a secas,
/// y si algo falla después, saberlo ahorra media hora.
pub async fn test_connection(cfg: &ConnectionConfig) -> DbResult<String> {
    #[cfg(windows)]
    if odbc::available() {
        let cfg = cfg.clone();
        return tokio::task::spawn_blocking(move || odbc::test(&cfg))
            .await
            .map_err(|e| -> super::DbError { format!("fallo en el hilo de ODBC: {e}").into() })?;
    }
    let mut client = connect(cfg).await?;
    let row = client
        .query("SELECT @@VERSION AS version, DB_NAME() AS base", &[])
        .await?
        .into_row()
        .await?;
    let version = row
        .as_ref()
        .and_then(|r| r.get::<&str, _>("version"))
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| "desconocida".to_string());
    let base = row
        .as_ref()
        .and_then(|r| r.get::<&str, _>("base"))
        .unwrap_or("")
        .to_string();
    Ok(format!("Conexión OK · {} · {base} · {version}", transport_label()))
}

/// Lee el esquema completo: tablas+columnas y objetos con cuerpo SQL.
pub async fn read_schema(cfg: &ConnectionConfig) -> DbResult<Schema> {
    let raw = fetch(cfg).await?;
    let tables = parse_tables(&raw.tables, &raw.pk_names);
    let (views, procedures, functions) = parse_routines(&raw.routines, &raw.routine_deps);
    let foreign_keys = parse_foreign_keys(&raw.foreign_keys);
    let indexes = parse_indexes(&raw.indexes);
    let checks = parse_checks(&raw.checks);
    let triggers = parse_triggers(&raw.triggers);
    Ok(Schema {
        engine_label: format!("SQL Server · {}", cfg.database),
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

/// Ejecuta el script contra la base, lote por lote (separados por GO) y dentro
/// de UNA transacción: si cualquier lote falla, se revierte todo y la base queda
/// como estaba.
///
/// Los lotes viajan por la misma conexión, así que la transacción que abre el
/// primero sigue viva en los siguientes; SQL Server revierte también el DDL
/// (crear o modificar tablas, índices, vistas…), no solo los datos.
///
/// `SET XACT_ABORT ON` hace que un error de ejecución anule la transacción en el
/// acto, pero aun así revertimos a mano: `RAISERROR` —que es lo que usa el
/// guardia de posible pérdida de datos— no la anula por sí solo.
///
/// Ojo: SQL Server no admite algunas sentencias dentro de una transacción
/// (`CREATE`/`ALTER DATABASE`, `ALTER FULLTEXT`…). El script de sincronización
/// no las genera; si alguna llegara, el error se reporta y no se aplica nada.
pub async fn apply_script(cfg: &ConnectionConfig, script: &str) -> DbResult<String> {
    #[cfg(windows)]
    if odbc::available() {
        let (cfg, script) = (cfg.clone(), script.to_string());
        return tokio::task::spawn_blocking(move || odbc::apply(&cfg, &script))
            .await
            .map_err(|e| -> super::DbError { format!("fallo en el hilo de ODBC: {e}").into() })?;
    }
    let mut client = connect(cfg).await?;
    client
        .simple_query("SET XACT_ABORT ON; BEGIN TRANSACTION;")
        .await?
        .into_results()
        .await?;

    for batch in split_go_batches(script) {
        if batch.trim().is_empty() {
            continue;
        }
        // Resolvemos el lote a un Result propio para soltar el préstamo del
        // cliente antes de pedirle el ROLLBACK.
        let res = match client.simple_query(batch.as_str()).await {
            Ok(stream) => stream.into_results().await.map(|_| ()),
            Err(e) => Err(e),
        };
        if let Err(e) = res {
            // Hay que consumir también la respuesta: soltar el stream sin
            // leerlo deja la petición a medias y no garantiza que el servidor
            // haya terminado el ROLLBACK antes de que se cierre la conexión, y
            // este es justo el camino donde más importa que se revierta.
            // Los `let _` son para no tapar el error original, que es el que le
            // interesa a quien llama: si el ROLLBACK también falla, el servidor
            // revierte igualmente al cerrarse la sesión.
            if let Ok(stream) = client
                .simple_query("IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION;")
                .await
            {
                let _ = stream.into_results().await;
            }
            return Err(e.into());
        }
    }

    client
        .simple_query("IF @@TRANCOUNT > 0 COMMIT TRANSACTION;")
        .await?
        .into_results()
        .await?;
    Ok("Cambios aplicados a SQL Server (en una transacción).".to_string())
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
    CASE WHEN pk.column_id IS NOT NULL THEN 1 ELSE 0 END AS is_pk,
    c.is_identity AS is_identity,
    CAST(idc.seed_value      AS bigint) AS identity_seed,
    CAST(idc.increment_value AS bigint) AS identity_increment
FROM sys.tables t
JOIN sys.schemas s ON t.schema_id = s.schema_id
JOIN sys.columns c ON c.object_id = t.object_id
JOIN sys.types  ty ON c.user_type_id = ty.user_type_id
LEFT JOIN sys.identity_columns idc
       ON idc.object_id = c.object_id AND idc.column_id = c.column_id
LEFT JOIN (
    SELECT ic.object_id, ic.column_id
    FROM sys.indexes i
    JOIN sys.index_columns ic ON i.object_id = ic.object_id AND i.index_id = ic.index_id
    WHERE i.is_primary_key = 1
) pk ON pk.object_id = c.object_id AND pk.column_id = c.column_id
ORDER BY s.name, t.name, c.column_id";

/// Arma las tablas con sus columnas a partir de las filas de `TABLES_SQL`, y
/// les pega el nombre de su restriccion de clave primaria desde `PK_NAME_SQL`.
fn parse_tables(rows: &[Row], pk_rows: &[Row]) -> Vec<Table> {

    // Agrupamos columnas por tabla, preservando el orden de aparición.
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Table> = BTreeMap::new();

    for row in rows {
        let schema = row.text("schema_name");
        let table = row.text("table_name");
        let col_name = row.text("column_name");
        let data_type = row.text("data_type");
        let max_length = row.int("max_length", 0) as i16;
        let precision = row.int("precision", 0) as u8;
        let scale = row.int("scale", 0) as u8;
        let is_nullable = row.bool("is_nullable", true);
        let column_id = row.int("column_id", 0) as i32;
        let default_def = row.opt_text("default_definition");
        let is_pk = row.int("is_pk", 0) as i32;
        // El `IDENTITY` de la columna. Aquí SÍ tenemos el inicio y el
        // incremento: SQL Server es el único de los cuatro motores que los
        // expone columna por columna, en `sys.identity_columns`.
        let is_identity = row.bool("is_identity", false);
        let identity = if is_identity {
            let seed = row.int("identity_seed", 1) as i64;
            let increment = row.int("identity_increment", 1) as i64;
            Some(Identity { seed, increment, always: false })
        } else {
            None
        };

        let key = format!("{schema}.{table}");
        map.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            Table { schema: schema.to_string(), name: table.to_string(), columns: Vec::new(), pk_name: None }
        });

        if let Some(t) = map.get_mut(&key) {
            t.columns.push(Column {
                name: col_name.to_string(),
                data_type: format_type(data_type, max_length, precision, scale),
                is_nullable,
                default_value: default_def.map(|d| d.trim().to_string()),
                ordinal: column_id,
                is_primary_key: is_pk != 0,
                identity,
            });
        }
    }

    // Nombre de la restricción de PK de cada tabla (hace falta para poder
    // emitir el DROP CONSTRAINT cuando la clave primaria cambia).
    for row in pk_rows {
        let s = row.text("schema_name");
        let t = row.text("table_name");
        let n = row.text("pk_name");
        if n.is_empty() {
            continue;
        }
        if let Some(tbl) = map.get_mut(&format!("{s}.{t}")) {
            tbl.pk_name = Some(n.to_string());
        }
    }

    order.into_iter().filter_map(|k| map.remove(&k)).collect()
}

/// Las bases de datos que este login puede abrir de verdad.
///
/// Se saltan las del sistema (`database_id <= 4`: master, tempdb, model y msdb),
/// que no son lo que nadie quiere comparar, y `HAS_DBACCESS` deja fuera las que
/// salen en el catálogo pero no se pueden abrir, para que la lista no ofrezca
/// algo que luego falla.
const DATABASES_SQL: &str = "\
SELECT name
FROM sys.databases
WHERE database_id > 4 AND state_desc = 'ONLINE' AND HAS_DBACCESS(name) = 1
ORDER BY name";

/// Lista las bases del servidor. Conecta a NIVEL DE SERVIDOR: lo que haya en
/// `cfg.database` se ignora a propósito, porque esto es justo lo que se llama
/// ANTES de saber qué base elegir.
pub async fn list_databases(cfg: &ConnectionConfig) -> DbResult<Vec<String>> {
    let sin_base = ConnectionConfig { database: String::new(), ..cfg.clone() };
    #[cfg(windows)]
    if odbc::available() {
        return tokio::task::spawn_blocking(move || odbc::list(&sin_base))
            .await
            .map_err(|e| -> super::DbError { format!("fallo en el hilo de ODBC: {e}").into() })?;
    }
    let mut c = connect(&sin_base).await?;
    let filas = c.query(DATABASES_SQL, &[]).await?.into_first_result().await?;
    Ok(filas
        .into_iter()
        .filter_map(|r| r.get::<&str, _>("name").map(str::to_string))
        .collect())
}

const PK_NAME_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, kc.name AS pk_name
FROM sys.key_constraints kc
JOIN sys.tables t  ON t.object_id = kc.parent_object_id
JOIN sys.schemas s ON s.schema_id = t.schema_id
WHERE kc.type = 'PK'";

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

/// Dependencias de cada módulo: de qué tablas lee y a qué otros módulos invoca.
///
/// `referenced_id` se resuelve con `OBJECT_NAME` en vez de usar
/// `referenced_entity_name`, que trae el nombre TAL COMO lo escribió quien
/// programó la vista (sin calificar, o con otras mayúsculas). Las referencias
/// que el motor no pudo resolver (`referenced_id IS NULL`: una tabla que ya no
/// existe, o de otra base) se descartan, porque no hay nada que refrescar.
const ROUTINE_DEPS_SQL: &str = "\
SELECT DISTINCT
    OBJECT_SCHEMA_NAME(d.referencing_id) AS modulo_schema,
    OBJECT_NAME(d.referencing_id)        AS modulo_name,
    OBJECT_SCHEMA_NAME(d.referenced_id)  AS ref_schema,
    OBJECT_NAME(d.referenced_id)         AS ref_name
FROM sys.sql_expression_dependencies d
JOIN sys.objects o ON o.object_id = d.referencing_id
WHERE o.type IN ('P','FN','IF','TF','V')
  AND d.referenced_id IS NOT NULL
  AND d.referenced_class = 1
ORDER BY 1, 2, 3, 4";

/// Lee las dependencias y las devuelve agrupadas por módulo ("schema.nombre").
fn parse_routine_deps(rows: &[Row]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in rows {
        let ms = row.text("modulo_schema");
        let mn = row.text("modulo_name");
        let rs = row.text("ref_schema");
        let rn = row.text("ref_name");
        if mn.is_empty() || rn.is_empty() {
            continue;
        }
        out.entry(format!("{ms}.{mn}")).or_default().push(format!("{rs}.{rn}"));
    }
    out
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
fn parse_routines(
    rows: &[Row],
    dep_rows: &[Row],
) -> (Vec<Routine>, Vec<Routine>, Vec<Routine>) {
    let deps = parse_routine_deps(dep_rows);

    let mut views = Vec::new();
    let mut procedures = Vec::new();
    let mut functions = Vec::new();

    for row in rows {
        let schema = row.text("schema_name");
        let name = row.text("object_name");
        let otype = row.text("object_type");
        let definition = row.text("definition");

        let make = |kind: RoutineKind| Routine {
            schema: schema.to_string(),
            name: name.to_string(),
            kind,
            definition: definition.to_string(),
            references: deps.get(&format!("{schema}.{name}")).cloned().unwrap_or_default(),
        };

        match otype {
            "V" => views.push(make(RoutineKind::View)),
            "P" => procedures.push(make(RoutineKind::Procedure)),
            "FN" => functions.push(make(RoutineKind::ScalarFunction)),
            "IF" | "TF" => functions.push(make(RoutineKind::TableFunction)),
            _ => {}
        }
    }

    (views, procedures, functions)
}

const FK_SQL: &str = "\
SELECT
    fk.name AS fk_name,
    fk.is_system_named AS is_system_named,
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
fn parse_foreign_keys(rows: &[Row]) -> Vec<ForeignKey> {
    use std::collections::BTreeMap;

    let mut by_name: BTreeMap<String, ForeignKey> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();

    for row in rows {
        let fk_name = row.text("fk_name");
        let ps = row.text("parent_schema");
        let pt = row.text("parent_table");
        let pc = row.text("parent_col");
        let rs = row.text("ref_schema");
        let rt = row.text("ref_table");
        let rc = row.text("ref_col");

        let key = fk_name.to_string();
        if !by_name.contains_key(&key) {
            order.push(key.clone());
            by_name.insert(
                key.clone(),
                ForeignKey {
                    name: fk_name.to_string(),
                    system_named: row.bool("is_system_named", false),
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

    order.into_iter().filter_map(|n| by_name.remove(&n)).collect()
}

const INDEX_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, i.name AS index_name,
       i.is_unique AS is_unique, c.name AS col_name,
       CAST(COALESCE(kc.is_system_named, 0) AS bit) AS is_system_named
FROM sys.indexes i
JOIN sys.tables  t ON t.object_id = i.object_id
JOIN sys.schemas s ON s.schema_id = t.schema_id
JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id
JOIN sys.columns c ON c.object_id = ic.object_id AND c.column_id = ic.column_id
-- Un índice solo puede estar bautizado por el motor si respalda una
-- restricción (UNIQUE declarada sin nombre); un CREATE INDEX siempre lo nombra
-- quien lo escribe. Por eso la marca sale de sys.key_constraints.
LEFT JOIN sys.key_constraints kc ON kc.parent_object_id = i.object_id AND kc.unique_index_id = i.index_id
WHERE i.is_primary_key = 0 AND i.type > 0 AND i.name IS NOT NULL
ORDER BY s.name, t.name, i.name, ic.key_ordinal";

/// Lee los índices (sin contar el de la clave primaria).
fn parse_indexes(rows: &[Row]) -> Vec<Index> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, Index> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for row in rows {
        let s = row.text("schema_name");
        let t = row.text("table_name");
        let iname = row.text("index_name");
        let uniq = row.bool("is_unique", false);
        let col = row.text("col_name");
        let key = format!("{s}.{t}.{iname}");
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(
                key.clone(),
                Index {
                    name: iname.to_string(),
                    system_named: row.bool("is_system_named", false),
                    table: format!("{s}.{t}"),
                    columns: Vec::new(),
                    is_unique: uniq,
                },
            );
        }
        if let Some(ix) = map.get_mut(&key) {
            ix.columns.push(col.to_string());
        }
    }
    order.into_iter().filter_map(|k| map.remove(&k)).collect()
}

const CHECK_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, cc.name AS check_name, cc.definition AS definition,
       cc.is_system_named AS is_system_named
FROM sys.check_constraints cc
JOIN sys.tables  t ON t.object_id = cc.parent_object_id
JOIN sys.schemas s ON s.schema_id = t.schema_id
ORDER BY s.name, t.name, cc.name";

/// Lee las restricciones CHECK.
fn parse_checks(rows: &[Row]) -> Vec<CheckConstraint> {
    let mut out = Vec::new();
    for row in rows {
        let s = row.text("schema_name");
        let t = row.text("table_name");
        let n = row.text("check_name");
        let d = row.text("definition");
        out.push(CheckConstraint {
            name: n.to_string(),
            system_named: row.bool("is_system_named", false),
            table: format!("{s}.{t}"),
            definition: d.to_string(),
        });
    }
    out
}

const TRIGGER_SQL: &str = "\
SELECT s.name AS schema_name, t.name AS table_name, tr.name AS trigger_name, m.definition AS definition
FROM sys.triggers tr
JOIN sys.tables   t ON t.object_id = tr.parent_id
JOIN sys.schemas  s ON s.schema_id = t.schema_id
LEFT JOIN sys.sql_modules m ON m.object_id = tr.object_id
WHERE tr.is_ms_shipped = 0
ORDER BY s.name, t.name, tr.name";

/// Lee los disparadores (triggers) DML sobre tablas.
fn parse_triggers(rows: &[Row]) -> Vec<Trigger> {
    let mut out = Vec::new();
    for row in rows {
        let s = row.text("schema_name");
        let t = row.text("table_name");
        let n = row.text("trigger_name");
        let d = row.text("definition");
        out.push(Trigger { schema: s.to_string(), name: n.to_string(), table: format!("{s}.{t}"), definition: d.to_string() });
    }
    out
}

// ===========================================================================
// Transportes
// ===========================================================================
//
// A SQL Server se puede llegar por dos caminos, y los dos devuelven las MISMAS
// filas para las mismas consultas:
//
//   · ODBC, que es como se conecta SSMS. El controlador de Microsoft habla
//     memoria compartida, canalizaciones con nombre Y TCP, y resuelve él mismo
//     la autenticación de Windows. Es el único que sirve contra una instancia
//     que no tenga TCP/IP habilitado, que es la configuración por defecto de
//     muchas instalaciones locales.
//   · tiberius sobre TCP, en Rust puro. No necesita que haya nada instalado en
//     la máquina, así que es el que queda fuera de Windows.
//
// En Windows se prefiere ODBC cuando hay un controlador instalado, y si no lo
// hay se cae a tiberius. `test_connection` dice por cuál de los dos entró, para
// que no sea magia.

/// Las filas en crudo de las consultas del esquema, ya traducidas al `Row`
/// común. De aquí en adelante el parseo es uno solo, venga de donde venga.
#[derive(Default)]
struct RawSchema {
    tables: Vec<Row>,
    pk_names: Vec<Row>,
    routines: Vec<Row>,
    routine_deps: Vec<Row>,
    foreign_keys: Vec<Row>,
    indexes: Vec<Row>,
    checks: Vec<Row>,
    triggers: Vec<Row>,
}

/// Lee todas las filas del esquema por el transporte que corresponda.
async fn fetch(cfg: &ConnectionConfig) -> DbResult<RawSchema> {
    #[cfg(windows)]
    if odbc::available() {
        let cfg = cfg.clone();
        return tokio::task::spawn_blocking(move || odbc::fetch(&cfg))
            .await
            .map_err(|e| -> super::DbError { format!("fallo en el hilo de ODBC: {e}").into() })?;
    }
    tds::fetch(cfg).await
}

/// Nombre del transporte que se usaría, para decírselo a quien conecta.
fn transport_label() -> &'static str {
    #[cfg(windows)]
    if odbc::available() {
        return "ODBC";
    }
    "TCP"
}

// ───────────────────────────── tiberius (TCP) ───────────────────────────────
mod tds {
    use super::*;

    pub(super) async fn fetch(cfg: &ConnectionConfig) -> DbResult<RawSchema> {
        let mut c = connect(cfg).await?;
        Ok(RawSchema {
            tables: rows(&mut c, TABLES_SQL).await?,
            pk_names: rows(&mut c, PK_NAME_SQL).await?,
            routines: rows(&mut c, ROUTINES_SQL).await?,
            routine_deps: rows(&mut c, ROUTINE_DEPS_SQL).await?,
            foreign_keys: rows(&mut c, FK_SQL).await?,
            indexes: rows(&mut c, INDEX_SQL).await?,
            checks: rows(&mut c, CHECK_SQL).await?,
            triggers: rows(&mut c, TRIGGER_SQL).await?,
        })
    }

    async fn rows(client: &mut SqlClient, sql: &str) -> DbResult<Vec<Row>> {
        let got = client.query(sql, &[]).await?.into_first_result().await?;
        Ok(got.into_iter().map(convert).collect())
    }

    /// Una fila de tiberius al `Row` común. Los nombres se copian antes de
    /// consumir la fila, porque `columns()` la presta y `into_iter()` la mueve.
    fn convert(r: tiberius::Row) -> Row {
        let names: Vec<String> = r.columns().iter().map(|c| c.name().to_string()).collect();
        let mut out = Row::new();
        for (name, data) in names.iter().zip(r.into_iter()) {
            out.put(name, value(data));
        }
        out
    }

    fn value(d: tiberius::ColumnData<'static>) -> Value {
        use tiberius::ColumnData as C;
        match d {
            C::U8(Some(n)) => Value::Int(i64::from(n)),
            C::I16(Some(n)) => Value::Int(i64::from(n)),
            C::I32(Some(n)) => Value::Int(i64::from(n)),
            C::I64(Some(n)) => Value::Int(n),
            C::Bit(Some(b)) => Value::Bool(b),
            C::String(Some(s)) => Value::Text(s.into_owned()),
            // Lo demás no aparece en las consultas del esquema; si algún día
            // apareciera, se lee como NULL y el accesor da su valor por defecto.
            _ => Value::Null,
        }
    }
}

// ─────────────────────────────────── ODBC ────────────────────────────────────
#[cfg(windows)]
mod odbc {
    use super::*;
    use odbc_api::{Connection, ConnectionOptions, Cursor, Environment, ResultSetMetadata};
    use std::sync::OnceLock;

    /// Controladores que sabemos leer, del mejor al peor. Se elige el primero
    /// que esté instalado de verdad, en vez de dar por hecho uno.
    const PREFERIDOS: [&str; 4] = [
        "ODBC Driver 18 for SQL Server",
        "ODBC Driver 17 for SQL Server",
        "SQL Server Native Client 11.0",
        "SQL Server",
    ];

    /// El entorno ODBC se crea una sola vez por proceso, como manda la API.
    fn environment() -> DbResult<&'static Environment> {
        static ENV: OnceLock<Environment> = OnceLock::new();
        if let Some(e) = ENV.get() {
            return Ok(e);
        }
        let creado = Environment::new()?;
        Ok(ENV.get_or_init(|| creado))
    }

    /// El controlador instalado que usaremos, si hay alguno.
    fn driver() -> Option<&'static str> {
        static DRIVER: OnceLock<Option<String>> = OnceLock::new();
        DRIVER
            .get_or_init(|| {
                let env = environment().ok()?;
                let instalados: Vec<String> = env
                    .drivers()
                    .ok()?
                    .into_iter()
                    .map(|d| d.description)
                    .collect();
                PREFERIDOS
                    .iter()
                    .find(|p| instalados.iter().any(|i| i == *p))
                    .map(|p| p.to_string())
            })
            .as_deref()
    }

    pub(super) fn available() -> bool {
        driver().is_some()
    }

    /// Un valor de cadena de conexión va entre llaves si trae algo que la
    /// rompería; las llaves de cierre se duplican, que es como se escapan.
    fn escapar(v: &str) -> String {
        if v.contains(|c| c == ';' || c == '{' || c == '}' || c == '=') {
            format!("{{{}}}", v.replace('}', "}}"))
        } else {
            v.to_string()
        }
    }

    /// Arma la cadena de conexión al estilo de SSMS.
    ///
    /// El puerto solo se escribe si NO es el 1433. Con el puerto por defecto se
    /// manda el nombre del servidor a secas, y así el controlador elige el
    /// transporte igual que SSMS: memoria compartida si es local, y si no,
    /// canalizaciones o TCP. Escribirlo siempre forzaría TCP, que es justo lo
    /// que queremos evitar.
    pub(super) fn connection_string(cfg: &ConnectionConfig, driver: &str) -> String {
        let servidor = if cfg.port == 0 || cfg.port == 1433 {
            cfg.host.clone()
        } else {
            format!("{},{}", cfg.host, cfg.port)
        };
        let mut s = format!("Driver={{{driver}}};Server={};", escapar(&servidor));
        if !cfg.database.is_empty() {
            s.push_str(&format!("Database={};", escapar(&cfg.database)));
        }
        if cfg.windows_auth {
            s.push_str("Trusted_Connection=yes;");
        } else {
            s.push_str(&format!("UID={};PWD={};", escapar(&cfg.user), escapar(&cfg.password)));
        }
        s.push_str("TrustServerCertificate=yes;");
        s
    }

    fn conectar(cfg: &ConnectionConfig) -> DbResult<Connection<'static>> {
        let d = driver().ok_or_else(|| -> super::super::DbError {
            "No hay ningún controlador ODBC de SQL Server instalado.".into()
        })?;
        let cs = connection_string(cfg, d);
        Ok(environment()?.connect_with_connection_string(&cs, ConnectionOptions::default())?)
    }

    pub(super) fn fetch(cfg: &ConnectionConfig) -> DbResult<RawSchema> {
        let c = conectar(cfg)?;
        Ok(RawSchema {
            tables: rows(&c, TABLES_SQL)?,
            pk_names: rows(&c, PK_NAME_SQL)?,
            routines: rows(&c, ROUTINES_SQL)?,
            routine_deps: rows(&c, ROUTINE_DEPS_SQL)?,
            foreign_keys: rows(&c, FK_SQL)?,
            indexes: rows(&c, INDEX_SQL)?,
            checks: rows(&c, CHECK_SQL)?,
            triggers: rows(&c, TRIGGER_SQL)?,
        })
    }

    /// Todo se pide como texto: el `Row` común sabe convertir a número o a
    /// booleano, y así no hay que mapear cada tipo de SQL Server a mano.
    fn rows(conn: &Connection<'_>, sql: &str) -> DbResult<Vec<Row>> {
        let Some(mut cursor) = conn.execute(sql, (), None)? else {
            return Ok(Vec::new());
        };
        let n = cursor.num_result_cols()? as u16;
        let mut nombres = Vec::with_capacity(n as usize);
        for i in 1..=n {
            nombres.push(cursor.col_name(i)?);
        }
        let mut out = Vec::new();
        // El buffer se reserva grande y se reutiliza: `get_text` pide el valor
        // en trozos del tamaño que le quepa, y un cuerpo de vista o de
        // procedimiento puede ser de decenas de miles de caracteres. Con un
        // buffer pequeño eso son cientos de viajes al controlador por celda.
        let mut buf = Vec::with_capacity(64 * 1024);
        while let Some(mut fila) = cursor.next_row()? {
            let mut r = Row::new();
            for (i, nombre) in nombres.iter().enumerate() {
                let col = (i + 1) as u16;
                let v = if fila.get_text(col, &mut buf)? {
                    Value::Text(String::from_utf8_lossy(&buf).to_string())
                } else {
                    Value::Null
                };
                r.put(nombre, v);
            }
            out.push(r);
        }
        Ok(out)
    }

    /// Primera línea de `@@VERSION` y el transporte por el que se entró, que es
    /// el dato que de verdad importa aquí.
    pub(super) fn test(cfg: &ConnectionConfig) -> DbResult<String> {
        let c = conectar(cfg)?;
        let filas = rows(
            &c,
            "SELECT @@VERSION AS version, DB_NAME() AS base, \
             (SELECT net_transport FROM sys.dm_exec_connections WHERE session_id = @@SPID) AS transporte",
        )?;
        let (v, base, t) = match filas.first() {
            Some(r) => (
                r.text("version").lines().next().unwrap_or("").trim().to_string(),
                r.text("base").to_string(),
                r.text("transporte").to_string(),
            ),
            None => ("desconocida".to_string(), String::new(), String::new()),
        };
        // La base va en el mensaje a propósito: es la forma de ver de un vistazo
        // que se entró a la que se pidió y no a la que el servidor tenía por defecto.
        Ok(format!("Conexión OK · ODBC ({t}) · {base} · {v}"))
    }

    pub(super) fn list(cfg: &ConnectionConfig) -> DbResult<Vec<String>> {
        let c = conectar(cfg)?;
        Ok(rows(&c, DATABASES_SQL)?
            .iter()
            .map(|r| r.text("name").to_string())
            .filter(|n| !n.is_empty())
            .collect())
    }

    /// Aplica el script lote por lote dentro de una transacción, igual que el
    /// camino de tiberius: si uno falla se revierte todo.
    pub(super) fn apply(cfg: &ConnectionConfig, script: &str) -> DbResult<String> {
        let c = conectar(cfg)?;
        c.set_autocommit(false)?;
        for lote in super::split_go_batches(script) {
            if lote.trim().is_empty() {
                continue;
            }
            if let Err(e) = c.execute(&lote, (), None) {
                let _ = c.rollback();
                return Err(e.into());
            }
        }
        c.commit()?;
        Ok("Cambios aplicados a SQL Server por ODBC (en una transacción).".to_string())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::db::EngineKind;

    fn cfg(host: &str, port: u16, windows_auth: bool) -> ConnectionConfig {
        ConnectionConfig {
            engine: EngineKind::SqlServer,
            host: host.into(),
            port,
            database: "ventas".into(),
            user: "sa".into(),
            password: "clave".into(),
            windows_auth,
            path: String::new(),
        }
    }

    #[test]
    fn con_el_puerto_por_defecto_no_se_escribe_el_puerto() {
        // Es lo que deja al controlador elegir memoria compartida igual que
        // SSMS; escribir el puerto forzaría TCP.
        let cs = odbc::connection_string(&cfg("localhost", 1433, true), "D");
        assert!(cs.contains("Server=localhost;"), "{cs}");
        assert!(!cs.contains("1433"), "{cs}");
    }

    #[test]
    fn un_puerto_distinto_si_se_escribe() {
        let cs = odbc::connection_string(&cfg("localhost", 1444, true), "D");
        assert!(cs.contains("Server=localhost,1444;"), "{cs}");
    }

    #[test]
    fn la_autenticacion_de_windows_no_lleva_usuario_ni_clave() {
        let cs = odbc::connection_string(&cfg("srv", 1433, true), "D");
        assert!(cs.contains("Trusted_Connection=yes;"), "{cs}");
        assert!(!cs.contains("UID="), "{cs}");
        assert!(!cs.contains("clave"), "{cs}");
    }

    #[test]
    fn la_autenticacion_sql_lleva_usuario_y_clave() {
        let cs = odbc::connection_string(&cfg("srv", 1433, false), "D");
        assert!(cs.contains("UID=sa;"), "{cs}");
        assert!(cs.contains("PWD=clave;"), "{cs}");
        assert!(!cs.contains("Trusted_Connection"), "{cs}");
    }

    #[test]
    fn una_clave_con_punto_y_coma_no_parte_la_cadena() {
        let mut c = cfg("srv", 1433, false);
        c.password = "a;b=c}d".into();
        let cs = odbc::connection_string(&c, "D");
        // Va entre llaves, y la llave de cierre se duplica.
        assert!(cs.contains("PWD={a;b=c}}d};"), "{cs}");
    }

    #[test]
    fn el_nombre_de_instancia_viaja_entero() {
        let cs = odbc::connection_string(&cfg("EQUIPO\\SQLEXPRESS", 1433, true), "D");
        assert!(cs.contains("Server=EQUIPO\\SQLEXPRESS;"), "{cs}");
    }
}
