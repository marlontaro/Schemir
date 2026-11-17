//! Lectura del esquema de una base MySQL o MariaDB usando `mysql_async`.
//!
//! Los dos motores hablan el mismo protocolo y exponen los mismos catálogos, así
//! que un solo lector cubre ambos. Todo sale de `information_schema`: tablas con
//! columnas (PK incluida), vistas, procedimientos y funciones, claves foráneas,
//! índices, restricciones CHECK y disparadores.
//!
//! Dos detalles propios de MySQL que conviene tener presentes:
//!
//! * El «esquema» ES la base de datos. Lo que en SQL Server es `dbo.Clientes`
//!   aquí es `tienda.clientes`, y `Schema.engine_label` queda como
//!   `MySQL · tienda`.
//! * Siempre se filtran las bases del sistema (`information_schema`, `mysql`,
//!   `performance_schema`, `sys`): si no, cada comparación arrastraría cientos
//!   de objetos internos.

use std::collections::BTreeMap;

use mysql_async::prelude::Queryable;
use mysql_async::{Conn, Opts, OptsBuilder, SslOpts};
use schemir_core::{CheckConstraint, Column, ForeignKey, Index, Routine, RoutineKind, Schema, Table, Trigger};

use super::{ConnectionConfig, DbResult};

/// Bases internas del motor: nunca entran en la comparación.
const SYSTEM_SCHEMAS: &str = "'information_schema','mysql','performance_schema','sys'";

/// Entrecomilla un identificador al estilo MySQL (acento grave, duplicado para
/// escaparlo). Es el mismo criterio que `Dialect::MySql` usa en el generador de
/// script, repetido aquí porque ese método es privado del núcleo.
fn q(ident: &str) -> String {
    format!("`{}`", ident.replace('`', "``"))
}

fn qname(schema: &str, name: &str) -> String {
    format!("{}.{}", q(schema), q(name))
}

/// Abre una conexión. Intenta primero con TLS (confiando en el certificado del
/// servidor, igual que hace el lector de PostgreSQL) y, si el servidor no lo
/// ofrece, reintenta en claro. Así funciona tanto con un MySQL local sin
/// certificados como con uno en la nube que exige cifrado.
async fn connect(cfg: &ConnectionConfig) -> DbResult<Conn> {
    let base = || {
        let db = (!cfg.database.is_empty()).then(|| cfg.database.clone());
        OptsBuilder::default()
            .ip_or_hostname(cfg.host.clone())
            .tcp_port(cfg.port)
            .user(Some(cfg.user.clone()))
            .pass(Some(cfg.password.clone()))
            .db_name(db)
    };
    let tls = base().ssl_opts(
        SslOpts::default()
            .with_danger_accept_invalid_certs(true)
            .with_danger_skip_domain_validation(true),
    );
    match Conn::new(Opts::from(tls)).await {
        Ok(conn) => Ok(conn),
        // Si falló por algo que no es el TLS (contraseña, host, permisos), el
        // segundo intento falla igual y es ESE error el que se reporta.
        Err(_) => Ok(Conn::new(Opts::from(base())).await?),
    }
}

/// Prueba la conexión y devuelve la versión del servidor.
pub async fn test_connection(cfg: &ConnectionConfig) -> DbResult<String> {
    let mut conn = connect(cfg).await?;
    let version: String = conn
        .query_first("SELECT VERSION()")
        .await?
        .unwrap_or_else(|| "(versión desconocida)".to_string());
    let sql = format!(
        "SELECT COUNT(*) FROM information_schema.TABLES \
         WHERE TABLE_TYPE = 'BASE TABLE' AND TABLE_SCHEMA NOT IN ({SYSTEM_SCHEMAS})"
    );
    let tables: i64 = conn.query_first(sql).await?.unwrap_or(0);
    conn.disconnect().await?;
    // La cadena de versión de MariaDB trae el sufijo "-MariaDB": lo dejamos tal
    // cual, porque es justo lo que distingue un motor del otro.
    Ok(format!("Conexión OK · MySQL {version} · {tables} tablas"))
}

/// Lee el esquema completo.
pub async fn read_schema(cfg: &ConnectionConfig) -> DbResult<Schema> {
    let mut conn = connect(cfg).await?;
    let tables = read_tables(&mut conn).await?;
    let views = read_views(&mut conn).await?;
    let (procedures, functions) = read_routines(&mut conn).await?;
    let foreign_keys = read_foreign_keys(&mut conn).await?;
    let indexes = read_indexes(&mut conn).await?;
    let checks = read_checks(&mut conn).await?;
    let triggers = read_triggers(&mut conn).await?;
    conn.disconnect().await?;
    Ok(Schema {
        engine_label: format!("MySQL · {}", cfg.database),
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

/// Ejecuta el script contra la base, sentencia a sentencia.
///
/// (!) A diferencia de PostgreSQL, MySQL hace un *commit* implícito en cada DDL:
/// si una sentencia falla, las anteriores YA se aplicaron y no se revierten. Por
/// eso no se abre una transacción, que daría una falsa sensación de seguridad.
pub async fn apply_script(cfg: &ConnectionConfig, script: &str) -> DbResult<String> {
    let mut conn = connect(cfg).await?;
    let mut n = 0usize;
    for stmt in split_statements(script) {
        conn.query_drop(stmt.as_str()).await?;
        n += 1;
    }
    conn.disconnect().await?;
    Ok(format!("Cambios aplicados a MySQL ({n} sentencias)."))
}

/// Trocea el script en sentencias respetando las directivas `DELIMITER` que
/// MySQL necesita para los cuerpos con `;` dentro (procedimientos, funciones y
/// disparadores), que son directivas del cliente y no SQL que el servidor
/// entienda.
///
/// Corta donde el delimitador CIERRA un renglón, que es como lo escribe el
/// generador de Schemir y como lo escriben los clientes de MySQL. No analiza el
/// SQL, así que un `;` dentro de una cadena de texto no lo confunde mientras no
/// quede al final de la línea.
fn split_statements(script: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut delim = String::from(";");
    let mut buf = String::new();

    for line in script.lines() {
        let trimmed = line.trim();
        // ── Directiva DELIMITER: cambia el separador y no se envía al servidor ──
        let is_delim_directive = trimmed.get(..9).is_some_and(|h| h.eq_ignore_ascii_case("DELIMITER"))
            && trimmed[9..].starts_with(char::is_whitespace);
        if is_delim_directive {
            flush(&mut out, &mut buf);
            let rest = trimmed[9..].trim();
            if !rest.is_empty() {
                delim = rest.to_string();
            }
            continue;
        }

        match line.trim_end().strip_suffix(delim.as_str()) {
            Some(head) => {
                buf.push_str(head);
                flush(&mut out, &mut buf);
            }
            None => {
                buf.push_str(line);
                buf.push('\n');
            }
        }
    }
    flush(&mut out, &mut buf);
    out
}

/// Cierra la sentencia acumulada y la guarda si tiene SQL de verdad (los
/// bloques que son solo comentarios del script se descartan).
fn flush(out: &mut Vec<String>, buf: &mut String) {
    let stmt = buf.trim().to_string();
    buf.clear();
    let has_sql = stmt
        .lines()
        .any(|l| !l.trim().is_empty() && !l.trim_start().starts_with("--"));
    if has_sql {
        out.push(stmt);
    }
}

// ───────────────────────────── Tablas y columnas ─────────────────────────────

/// `COLUMN_TYPE` (y no `DATA_TYPE`) porque es el que trae el tipo completo:
/// `varchar(50)`, `decimal(10,2)`, `int unsigned`…
/// `EXTRA` hace falta para saber si el valor por defecto es una expresión.
const TABLES_SQL: &str = "\
SELECT c.TABLE_SCHEMA, c.TABLE_NAME, c.COLUMN_NAME, c.COLUMN_TYPE,
       c.IS_NULLABLE, c.COLUMN_DEFAULT, c.ORDINAL_POSITION,
       c.COLUMN_KEY, c.EXTRA
FROM information_schema.COLUMNS c
JOIN information_schema.TABLES t
  ON t.TABLE_SCHEMA = c.TABLE_SCHEMA AND t.TABLE_NAME = c.TABLE_NAME
WHERE t.TABLE_TYPE = 'BASE TABLE'
  AND c.TABLE_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY c.TABLE_SCHEMA, c.TABLE_NAME, c.ORDINAL_POSITION";

type ColumnRow = (String, String, String, String, String, Option<String>, i64, String, String);

async fn read_tables(conn: &mut Conn) -> DbResult<Vec<Table>> {
    let rows: Vec<ColumnRow> = conn.query(expand(TABLES_SQL)).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Table> = BTreeMap::new();

    for (schema, table, col, col_type, nullable, default, ordinal, key, extra) in rows {
        let qn = format!("{schema}.{table}");
        if !map.contains_key(&qn) {
            order.push(qn.clone());
            map.insert(
                qn.clone(),
                Table {
                    schema: schema.clone(),
                    name: table.clone(),
                    columns: Vec::new(),
                    // En MySQL la clave primaria se llama SIEMPRE «PRIMARY»: el
                    // nombre no distingue nada, y el generador de script usa la
                    // forma anónima para este motor. Dejarlo en None evita que
                    // una comparación contra otro motor marque una diferencia
                    // que no existe.
                    pk_name: None,
                },
            );
        }
        if let Some(t) = map.get_mut(&qn) {
            t.columns.push(Column {
                name: col,
                data_type: col_type.clone(),
                is_nullable: nullable.eq_ignore_ascii_case("YES"),
                default_value: default_literal(&default, &extra),
                ordinal: ordinal as i32,
                is_primary_key: key.eq_ignore_ascii_case("PRI"),
            });
        }
    }

    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

/// MySQL guarda el valor por defecto SIN comillas (`COLUMN_DEFAULT` de una
/// columna de texto con default `hola` vale literalmente `hola`), mientras que
/// el modelo espera una EXPRESIÓN que se pueda pegar tras `DEFAULT`. Así que hay
/// que devolverle las comillas, salvo cuando ya es una expresión:
///
/// * `EXTRA` contiene `DEFAULT_GENERATED` (MySQL 8 marca así los defaults por
///   expresión);
/// * es un número;
/// * es `NULL` o una función de fecha (`CURRENT_TIMESTAMP`, `now()`…), que es
///   como lo reportan MySQL 5.7 y MariaDB, sin marca en `EXTRA`.
fn default_literal(default: &Option<String>, extra: &str) -> Option<String> {
    let raw = default.as_deref()?;
    if extra.to_ascii_uppercase().contains("DEFAULT_GENERATED") {
        return Some(raw.to_string());
    }
    let t = raw.trim();
    if t.parse::<f64>().is_ok() {
        return Some(t.to_string());
    }
    let upper = t.to_ascii_uppercase();
    let is_expr = upper == "NULL"
        || upper.starts_with("CURRENT_TIMESTAMP")
        || upper.starts_with("CURRENT_DATE")
        || upper.starts_with("CURRENT_TIME")
        || upper.starts_with("NOW(")
        || upper.starts_with("UUID(");
    if is_expr {
        return Some(t.to_string());
    }
    Some(format!("'{}'", raw.replace('\'', "''")))
}

// ──────────────────────── Vistas, procedimientos, funciones ───────────────────

const VIEWS_SQL: &str = "\
SELECT TABLE_SCHEMA, TABLE_NAME, VIEW_DEFINITION
FROM information_schema.VIEWS
WHERE TABLE_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY TABLE_SCHEMA, TABLE_NAME";

async fn read_views(conn: &mut Conn) -> DbResult<Vec<Routine>> {
    let rows: Vec<(String, String, Option<String>)> = conn.query(expand(VIEWS_SQL)).await?;
    Ok(rows
        .into_iter()
        .map(|(schema, name, def)| {
            let body = def.unwrap_or_default();
            let definition = format!("CREATE OR REPLACE VIEW {} AS {body}", qname(&schema, &name));
            Routine { schema, name, kind: RoutineKind::View, definition }
        })
        .collect())
}

const ROUTINES_SQL: &str = "\
SELECT ROUTINE_SCHEMA, ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER,
       IS_DETERMINISTIC, SQL_DATA_ACCESS, SECURITY_TYPE, ROUTINE_DEFINITION
FROM information_schema.ROUTINES
WHERE ROUTINE_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY ROUTINE_SCHEMA, ROUTINE_NAME";

/// Los parámetros van en su propia vista. `ORDINAL_POSITION = 0` es el valor de
/// retorno de una función, no un parámetro.
const PARAMS_SQL: &str = "\
SELECT SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION, PARAMETER_MODE,
       PARAMETER_NAME, DTD_IDENTIFIER
FROM information_schema.PARAMETERS
WHERE SPECIFIC_SCHEMA NOT IN (SYSTEM_SCHEMAS) AND ORDINAL_POSITION > 0
ORDER BY SPECIFIC_SCHEMA, SPECIFIC_NAME, ORDINAL_POSITION";

type RoutineRow = (String, String, String, Option<String>, String, String, String, Option<String>);
type ParamRow = (String, String, i64, Option<String>, Option<String>, String);

/// Devuelve `(procedimientos, funciones)`.
///
/// `ROUTINE_DEFINITION` trae SOLO el cuerpo (sin el `CREATE` ni la lista de
/// parámetros), así que el encabezado se arma con `PARAMETERS` y con las
/// características que publica `ROUTINES`. Se hace así, y no con
/// `SHOW CREATE PROCEDURE`, para leer todo con dos consultas a
/// `information_schema` en vez de una por rutina.
async fn read_routines(conn: &mut Conn) -> DbResult<(Vec<Routine>, Vec<Routine>)> {
    let params: Vec<ParamRow> = conn.query(expand(PARAMS_SQL)).await?;
    let mut by_routine: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (schema, name, _ord, mode, pname, dtd) in params {
        let mode = match mode.as_deref() {
            // Las funciones no llevan modo; los procedimientos sí (IN/OUT/INOUT).
            Some(m) if !m.is_empty() => format!("{m} "),
            _ => String::new(),
        };
        let pname = pname.unwrap_or_default();
        by_routine
            .entry(format!("{schema}.{name}"))
            .or_default()
            .push(format!("{mode}{} {dtd}", q(&pname)));
    }

    let rows: Vec<RoutineRow> = conn.query(expand(ROUTINES_SQL)).await?;
    let mut procedures = Vec::new();
    let mut functions = Vec::new();
    for (schema, name, rtype, returns, deterministic, data_access, security, body) in rows {
        let is_function = rtype.eq_ignore_ascii_case("FUNCTION");
        let args = by_routine.get(&format!("{schema}.{name}")).cloned().unwrap_or_default().join(", ");
        let kw = if is_function { "FUNCTION" } else { "PROCEDURE" };

        let mut def = format!("CREATE {kw} {}({args})", qname(&schema, &name));
        if is_function {
            def.push_str(&format!("\nRETURNS {}", returns.unwrap_or_else(|| "varchar(255)".into())));
        }
        if !deterministic.eq_ignore_ascii_case("YES") {
            def.push_str("\nNOT DETERMINISTIC");
        } else {
            def.push_str("\nDETERMINISTIC");
        }
        if !data_access.is_empty() {
            def.push_str(&format!("\n{data_access}"));
        }
        if !security.is_empty() {
            def.push_str(&format!("\nSQL SECURITY {security}"));
        }
        def.push('\n');
        def.push_str(body.unwrap_or_default().trim());

        if is_function {
            functions.push(Routine { schema, name, kind: RoutineKind::ScalarFunction, definition: def });
        } else {
            procedures.push(Routine { schema, name, kind: RoutineKind::Procedure, definition: def });
        }
    }
    Ok((procedures, functions))
}

// ───────────────────────────── Claves foráneas ───────────────────────────────

const FK_SQL: &str = "\
SELECT k.CONSTRAINT_NAME, k.TABLE_SCHEMA, k.TABLE_NAME, k.COLUMN_NAME,
       k.REFERENCED_TABLE_SCHEMA, k.REFERENCED_TABLE_NAME, k.REFERENCED_COLUMN_NAME
FROM information_schema.KEY_COLUMN_USAGE k
JOIN information_schema.REFERENTIAL_CONSTRAINTS r
  ON r.CONSTRAINT_SCHEMA = k.CONSTRAINT_SCHEMA
 AND r.CONSTRAINT_NAME   = k.CONSTRAINT_NAME
 AND r.TABLE_NAME        = k.TABLE_NAME
WHERE k.REFERENCED_TABLE_NAME IS NOT NULL
  AND k.TABLE_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY k.CONSTRAINT_SCHEMA, k.CONSTRAINT_NAME, k.ORDINAL_POSITION";

type FkRow = (String, String, String, String, String, String, String);

async fn read_foreign_keys(conn: &mut Conn) -> DbResult<Vec<ForeignKey>> {
    let rows: Vec<FkRow> = conn.query(expand(FK_SQL)).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, ForeignKey> = BTreeMap::new();
    for (name, from_schema, from_table, from_col, to_schema, to_table, to_col) in rows {
        if !map.contains_key(&name) {
            order.push(name.clone());
            map.insert(
                name.clone(),
                ForeignKey {
                    name: name.clone(),
                    from_table: format!("{from_schema}.{from_table}"),
                    from_columns: Vec::new(),
                    to_table: format!("{to_schema}.{to_table}"),
                    to_columns: Vec::new(),
                },
            );
        }
        if let Some(fk) = map.get_mut(&name) {
            fk.from_columns.push(from_col);
            fk.to_columns.push(to_col);
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

// ────────────────────────────────── Índices ──────────────────────────────────

/// `NON_UNIQUE` es 0 en los índices únicos (ojo, está al revés de `is_unique`).
/// Se excluye `PRIMARY`, que es la clave primaria y ya viaja en las columnas.
/// `SEQ_IN_INDEX` da el orden de las columnas dentro del índice.
///
/// `COLUMN_NAME` viene en NULL en los índices por expresión (MySQL 8.0.13+).
/// Se deja una marca visible en vez de saltarse la columna, para que el índice
/// no parezca igual que otro cuando no lo es.
const INDEX_SQL: &str = "\
SELECT TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, NON_UNIQUE,
       COALESCE(COLUMN_NAME, CONCAT('(expresión ', SEQ_IN_INDEX, ')'))
FROM information_schema.STATISTICS
WHERE INDEX_NAME <> 'PRIMARY'
  AND TABLE_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY TABLE_SCHEMA, TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX";

type IndexRow = (String, String, String, i64, String);

async fn read_indexes(conn: &mut Conn) -> DbResult<Vec<Index>> {
    let rows: Vec<IndexRow> = conn.query(expand(INDEX_SQL)).await?;
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Index> = BTreeMap::new();
    for (schema, table, iname, non_unique, col) in rows {
        let key = format!("{schema}.{table}.{iname}");
        if !map.contains_key(&key) {
            order.push(key.clone());
            map.insert(
                key.clone(),
                Index {
                    name: iname.clone(),
                    table: format!("{schema}.{table}"),
                    columns: Vec::new(),
                    is_unique: non_unique == 0,
                },
            );
        }
        if let Some(ix) = map.get_mut(&key) {
            ix.columns.push(col);
        }
    }
    Ok(order.into_iter().filter_map(|k| map.remove(&k)).collect())
}

// ───────────────────────────── Restricciones CHECK ───────────────────────────

/// `CHECK_CONSTRAINTS` existe desde MySQL 8.0.16 y MariaDB 10.2. En MySQL no
/// trae la tabla, así que hay que cruzarla con `TABLE_CONSTRAINTS`.
///
/// `CHECK_CLAUSE` es SOLO la expresión (sin la palabra `CHECK`), igual que en
/// SQL Server: el generador de script la antepone al emitir el `ALTER`.
const CHECK_SQL: &str = "\
SELECT tc.CONSTRAINT_SCHEMA, tc.TABLE_NAME, cc.CONSTRAINT_NAME, cc.CHECK_CLAUSE
FROM information_schema.CHECK_CONSTRAINTS cc
JOIN information_schema.TABLE_CONSTRAINTS tc
  ON tc.CONSTRAINT_SCHEMA = cc.CONSTRAINT_SCHEMA
 AND tc.CONSTRAINT_NAME   = cc.CONSTRAINT_NAME
WHERE tc.CONSTRAINT_TYPE = 'CHECK'
  AND tc.CONSTRAINT_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY tc.CONSTRAINT_SCHEMA, tc.TABLE_NAME, cc.CONSTRAINT_NAME";

/// En un servidor anterior a MySQL 8.0.16 / MariaDB 10.2 la vista no existe y la
/// consulta falla: en ese caso se devuelve la lista vacía en vez de romper toda
/// la comparación (ese motor tampoco tiene restricciones CHECK que leer).
async fn read_checks(conn: &mut Conn) -> DbResult<Vec<CheckConstraint>> {
    let rows: Vec<(String, String, String, String)> = match conn.query(expand(CHECK_SQL)).await {
        Ok(rows) => rows,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(rows
        .into_iter()
        .map(|(schema, table, name, definition)| CheckConstraint {
            name,
            table: format!("{schema}.{table}"),
            definition,
        })
        .collect())
}

// ──────────────────────────────── Disparadores ───────────────────────────────

const TRIGGER_SQL: &str = "\
SELECT TRIGGER_SCHEMA, TRIGGER_NAME, EVENT_OBJECT_SCHEMA, EVENT_OBJECT_TABLE,
       ACTION_TIMING, EVENT_MANIPULATION, ACTION_STATEMENT
FROM information_schema.TRIGGERS
WHERE TRIGGER_SCHEMA NOT IN (SYSTEM_SCHEMAS)
ORDER BY TRIGGER_SCHEMA, EVENT_OBJECT_TABLE, TRIGGER_NAME";

type TriggerRow = (String, String, String, String, String, String, String);

/// `information_schema.TRIGGERS` no guarda el `CREATE` completo, solo sus
/// piezas: se vuelve a armar aquí para que el generador de script lo pueda
/// emitir tal cual (envuelto en `DELIMITER`, porque el cuerpo lleva `;`).
async fn read_triggers(conn: &mut Conn) -> DbResult<Vec<Trigger>> {
    let rows: Vec<TriggerRow> = conn.query(expand(TRIGGER_SQL)).await?;
    Ok(rows
        .into_iter()
        .map(|(schema, name, tbl_schema, table, timing, event, body)| {
            let definition = format!(
                "CREATE TRIGGER {} {timing} {event} ON {} FOR EACH ROW\n{}",
                qname(&schema, &name),
                qname(&tbl_schema, &table),
                body.trim()
            );
            Trigger { schema, name, table: format!("{tbl_schema}.{table}"), definition }
        })
        .collect())
}

/// Sustituye el marcador `SYSTEM_SCHEMAS` por la lista real. Las consultas se
/// escriben con el marcador para que se lea de un vistazo qué se está filtrando.
fn expand(sql: &str) -> String {
    sql.replace("SYSTEM_SCHEMAS", SYSTEM_SCHEMAS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrecomilla_y_escapa_el_acento_grave() {
        assert_eq!(q("clientes"), "`clientes`");
        assert_eq!(q("ra`ro"), "`ra``ro`");
        assert_eq!(qname("tienda", "clientes"), "`tienda`.`clientes`");
    }

    #[test]
    fn el_marcador_de_esquemas_del_sistema_se_expande() {
        let sql = expand("WHERE TABLE_SCHEMA NOT IN (SYSTEM_SCHEMAS)");
        assert!(sql.contains("'mysql'"), "{sql}");
        assert!(sql.contains("'performance_schema'"), "{sql}");
        assert!(!sql.contains("SYSTEM_SCHEMAS"), "{sql}");
    }

    #[test]
    fn el_default_de_texto_recupera_sus_comillas() {
        assert_eq!(default_literal(&Some("hola".into()), ""), Some("'hola'".into()));
        assert_eq!(default_literal(&Some("d'Arc".into()), ""), Some("'d''Arc'".into()));
    }

    #[test]
    fn el_default_numerico_o_de_expresion_va_tal_cual() {
        assert_eq!(default_literal(&Some("0".into()), ""), Some("0".into()));
        assert_eq!(default_literal(&Some("-1.5".into()), ""), Some("-1.5".into()));
        assert_eq!(
            default_literal(&Some("CURRENT_TIMESTAMP".into()), ""),
            Some("CURRENT_TIMESTAMP".into())
        );
        // MySQL 8 marca así los defaults por expresión.
        assert_eq!(
            default_literal(&Some("(1 + 1)".into()), "DEFAULT_GENERATED"),
            Some("(1 + 1)".into())
        );
    }

    #[test]
    fn sin_default_no_hay_literal() {
        assert_eq!(default_literal(&None, ""), None);
    }

    #[test]
    fn trocea_por_punto_y_coma() {
        let s = split_statements("CREATE TABLE a (id int);\nDROP TABLE b;\n");
        assert_eq!(s, vec!["CREATE TABLE a (id int)", "DROP TABLE b"]);
    }

    #[test]
    fn respeta_la_directiva_delimiter() {
        let script = "\
DELIMITER $$
CREATE PROCEDURE sp() BEGIN
  SELECT 1;
END$$
DELIMITER ;
DROP TABLE t;
";
        let s = split_statements(script);
        assert_eq!(s.len(), 2, "{s:?}");
        assert!(s[0].starts_with("CREATE PROCEDURE sp() BEGIN"), "{s:?}");
        // El `;` interno del cuerpo NO parte la sentencia.
        assert!(s[0].contains("SELECT 1;"), "{s:?}");
        assert_eq!(s[1], "DROP TABLE t");
    }

    #[test]
    fn descarta_los_bloques_de_solo_comentarios() {
        // Un script sin SQL (solo el encabezado de Schemir, o el comentario que
        // deja cuando no hay nada que sincronizar) no manda nada al servidor.
        let solo_comentarios = "-- Script generado por Schemir\n-- Sin diferencias.\n";
        assert!(split_statements(solo_comentarios).is_empty());
    }

    #[test]
    fn los_comentarios_viajan_con_su_sentencia() {
        // Los comentarios que preceden a una sentencia quedan dentro de ella;
        // MySQL los acepta y así el que lea el log ve de qué sección venía.
        let script = "-- ─── Tablas eliminadas ───\nDROP TABLE t;\n";
        let s = split_statements(script);
        assert_eq!(s.len(), 1, "{s:?}");
        assert!(s[0].ends_with("DROP TABLE t"), "{s:?}");
    }
}
