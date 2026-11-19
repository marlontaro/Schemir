//! Generación del "script de sincronización": el SQL que deja el DESTINO (B)
//! igual que el ORIGEN (A). Es la función estrella de SSDT Schema Compare.
//!
//! El script es un BORRADOR para revisar: Schemir nunca ejecuta ni modifica las
//! bases, solo genera el texto para que tú lo revises y lo corras.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::diff::{
    check_shape, columns_equal, fk_sig, index_name_key, index_shape, normalize_def,
    pair_objects, pk_columns, CompareOptions, DiffStatus,
};
use crate::model::{CheckConstraint, Column, ForeignKey, Identity, Index, Routine, RoutineKind, Schema, Table, Trigger};

/// Dialecto SQL del destino (determina el entrecomillado y la sintaxis).
///
/// Los nombres serializados son los que usa la interfaz en sus desplegables
/// (`sqlserver`, `postgres`, `mysql`, `sqlite`), para que el dialecto elegido
/// llegue tal cual a los comandos que lo reciben.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    SqlServer,
    Sqlite,
    Postgres,
    /// MySQL y MariaDB: hablan el mismo SQL para todo lo que genera Schemir.
    MySql,
}

impl Dialect {
    /// Entrecomilla un identificador según el motor: `[x]` en SQL Server, `"x"` en
    /// SQLite/PostgreSQL y `` `x` `` en MySQL/MariaDB.
    pub(crate) fn quote(&self, ident: &str) -> String {
        match self {
            Dialect::SqlServer => format!("[{}]", ident.replace(']', "]]")),
            Dialect::MySql => format!("`{}`", ident.replace('`', "``")),
            Dialect::Sqlite | Dialect::Postgres => format!("\"{}\"", ident.replace('"', "\"\"")),
        }
    }
    /// Nombre calificado entrecomillado, p.ej. `[dbo].[Clientes]`.
    pub(crate) fn qname(&self, schema: &str, name: &str) -> String {
        format!("{}.{}", self.quote(schema), self.quote(name))
    }
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Dialect::SqlServer => "SQL Server",
            Dialect::Sqlite => "SQLite",
            Dialect::Postgres => "PostgreSQL",
            Dialect::MySql => "MySQL / MariaDB",
        }
    }
}

fn null_kw(c: &Column) -> &'static str {
    if c.is_nullable {
        "NULL"
    } else {
        "NOT NULL"
    }
}

/// Cómo declara cada motor que una columna se autogenera.
///
/// Devuelve `None` cuando el dialecto no puede expresarlo dentro de la
/// definición de la columna. Ese es el caso de SQLite: allí `AUTOINCREMENT` solo
/// vale pegado a `INTEGER PRIMARY KEY` (ver `sqlite_autoincrement`).
fn identity_clause(d: Dialect, id: &Identity) -> Option<String> {
    match d {
        Dialect::SqlServer => Some(format!("IDENTITY({},{})", id.seed, id.increment)),
        Dialect::Postgres => {
            let kind = if id.always { "ALWAYS" } else { "BY DEFAULT" };
            Some(if id.seed == 1 && id.increment == 1 {
                format!("GENERATED {kind} AS IDENTITY")
            } else {
                format!(
                    "GENERATED {kind} AS IDENTITY (START WITH {} INCREMENT BY {})",
                    id.seed, id.increment
                )
            })
        }
        // El inicio y el incremento no se pueden poner aquí: el primero es una
        // opción de la tabla y el segundo una variable de sesión. Si venían
        // distintos de 1, `identity_note` lo avisa.
        Dialect::MySql => Some("AUTO_INCREMENT".to_string()),
        Dialect::Sqlite => None,
    }
}

/// ¿Puede SQLite declarar que esta columna se autogenera?
///
/// Solo de una forma: `INTEGER PRIMARY KEY AUTOINCREMENT`, y escrita pegada a la
/// columna, no como restricción de tabla. Si la clave primaria es compuesta, si
/// la columna no es la clave primaria o si el tipo no es entero, SQLite no tiene
/// manera de expresarlo.
fn sqlite_autoincrement(t: &Table, c: &Column) -> bool {
    c.identity.is_some()
        && c.data_type.eq_ignore_ascii_case("integer")
        && pk_columns(t) == [c.name.clone()]
}

/// Aviso (como comentario SQL) cuando el destino NO va a poder autogenerar una
/// columna que en el origen sí se autogenera, o no va a poder hacerlo igual.
///
/// Es el punto del arreglo: antes el `IDENTITY` o el `AUTO_INCREMENT` se perdía
/// al recrear la tabla y nadie se enteraba. Ahora, o se emite, o se dice por qué
/// no se pudo.
fn identity_note(d: Dialect, t: &Table, c: &Column) -> Option<String> {
    let id = c.identity.as_ref()?;
    let mut out = String::new();
    if d == Dialect::Sqlite && !sqlite_autoincrement(t, c) {
        let _ = writeln!(
            out,
            "-- (!) «{}» se autogenera en el origen, pero SQLite solo lo admite en una columna «integer» que sea ella sola la clave primaria: en el destino NO se autogenerará.",
            c.name
        );
    }
    if d == Dialect::MySql && (id.seed != 1 || id.increment != 1) {
        let _ = writeln!(
            out,
            "-- (!) «{}» empieza en {} y avanza de {} en {}: MySQL / MariaDB no lo expresan por columna, así que en el destino empezará en 1 y avanzará de 1 en 1.",
            c.name, id.seed, id.increment, id.increment
        );
    }
    if d == Dialect::MySql && !c.is_primary_key {
        let _ = writeln!(
            out,
            "-- (!) MySQL / MariaDB exigen que la columna AUTO_INCREMENT sea clave: «{}» no lo es en el origen, así que este SQL fallará tal cual.",
            c.name
        );
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Todos los avisos de autogeneración de una tabla, uno detrás de otro.
fn identity_notes(d: Dialect, t: &Table) -> String {
    sorted_columns(t)
        .iter()
        .filter_map(|c| identity_note(d, t, c))
        .collect()
}

/// Definición de una columna: `[nombre] tipo NULL/NOT NULL [autogeneración] [DEFAULT x]`.
fn column_ddl(d: Dialect, c: &Column) -> String {
    let clause = c.identity.as_ref().and_then(|id| identity_clause(d, id));
    // SQL Server quiere `IDENTITY(..)` entre el tipo y la nulabilidad;
    // PostgreSQL y MySQL la quieren después.
    let mut s = match (&clause, d) {
        (Some(cl), Dialect::SqlServer) => {
            format!("{} {} {cl} {}", d.quote(&c.name), c.data_type, null_kw(c))
        }
        (Some(cl), _) => format!("{} {} {} {cl}", d.quote(&c.name), c.data_type, null_kw(c)),
        (None, _) => format!("{} {} {}", d.quote(&c.name), c.data_type, null_kw(c)),
    };
    // Una columna que se autogenera no lleva DEFAULT: los motores rechazan las
    // dos cosas juntas, y el valor lo pone el motor.
    if c.identity.is_none() {
        if let Some(def) = &c.default_value {
            let _ = write!(s, " DEFAULT {def}");
        }
    }
    s
}

/// Escapa las comillas simples de un literal de texto SQL.
fn sql_text(s: &str) -> String {
    s.replace('\'', "''")
}

/// Guardia que detiene el script antes de una operación destructiva si la tabla
/// afectada tiene filas.
///
/// Es el equivalente de «bloquear la implementación incremental si se puede
/// producir pérdida de datos» de SSDT: las operaciones que pueden destruir datos
/// (quitar una columna o una tabla, cambiar el tipo de una columna) se ejecutan
/// solo si la tabla está vacía; si tiene filas, el script aborta antes de tocar
/// nada. En SQLite, PostgreSQL y SQL Server lo ya hecho además se revierte,
/// porque el script se aplica dentro de una transacción (y en SQL Server el
/// guardia va en su propio lote, antes del destructivo). En MySQL no hay vuelta
/// atrás que esperar —cada DDL hace un *commit* implícito—, así que allí el valor
/// del guardia es impedir el daño, no deshacerlo.
fn data_loss_guard(d: Dialect, schema: &str, name: &str, reasons: &[String]) -> String {
    let qn = d.qname(schema, name);
    let plain = format!("{schema}.{name}");
    let mut s = String::new();
    let _ = writeln!(s, "-- ⚠ Posible pérdida de datos en {plain}: {}.", reasons.join("; "));
    let _ = writeln!(
        s,
        "-- El script se detiene si la tabla tiene filas (opción «bloquear ante posible pérdida de datos»)."
    );
    let msg = format!("Schemir: {plain} tiene filas; se detiene por posible perdida de datos.");
    match d {
        Dialect::SqlServer => {
            let _ = writeln!(s, "IF EXISTS (SELECT TOP 1 1 FROM {qn})");
            let _ = writeln!(s, "    RAISERROR (N'{}', 16, 127) WITH NOWAIT;", sql_text(&msg));
            let _ = writeln!(s, "GO");
        }
        Dialect::Postgres => {
            let _ = writeln!(s, "DO $$");
            let _ = writeln!(s, "BEGIN");
            let _ = writeln!(s, "    IF EXISTS (SELECT 1 FROM {qn}) THEN");
            let _ = writeln!(s, "        RAISE EXCEPTION '{}';", sql_text(&msg));
            let _ = writeln!(s, "    END IF;");
            let _ = writeln!(s, "END $$;");
        }
        Dialect::Sqlite => {
            // SQLite no permite lanzar un error con un mensaje propio fuera de un
            // disparador, así que insertamos una fila que viola un CHECK: si la
            // tabla tiene filas, el INSERT falla y la transacción se revierte; si
            // está vacía no inserta nada y el script continúa.
            let _ = writeln!(
                s,
                "CREATE TEMP TABLE schemir_guard (ok INTEGER CONSTRAINT {} CHECK (ok = 1));",
                d.quote(&msg)
            );
            let _ = writeln!(
                s,
                "INSERT INTO schemir_guard (ok) SELECT 0 WHERE EXISTS (SELECT 1 FROM {qn});"
            );
            let _ = writeln!(s, "DROP TABLE schemir_guard;");
        }
        Dialect::MySql => {
            // MySQL tampoco deja lanzar un error con mensaje propio fuera de una
            // rutina (`SIGNAL` solo vale dentro de un BEGIN…END), así que
            // forzamos la violación de un NOT NULL: si la tabla tiene filas, el
            // SELECT devuelve una y el INSERT falla, de modo que `apply_script`
            // se detiene antes de llegar a las sentencias destructivas; si está
            // vacía no devuelve ninguna, no inserta nada y el script sigue.
            //
            // El mensaje va en el NOMBRE de la columna porque los identificadores
            // de MySQL admiten 64 caracteres y el texto entero no cabe; el detalle
            // queda en el comentario de las dos primeras líneas.
            //
            // `STRICT_ALL_TABLES` se fuerza a propósito: sin modo estricto, un
            // NULL en una columna NOT NULL sería solo un aviso y el guardia no
            // frenaría nada. Se guarda el `sql_mode` anterior y se restaura.
            let _ = writeln!(s, "SET @schemir_sql_mode = @@SESSION.sql_mode;");
            let _ = writeln!(
                s,
                "SET SESSION sql_mode = CONCAT(@@SESSION.sql_mode, ',STRICT_ALL_TABLES');"
            );
            let _ = writeln!(
                s,
                "CREATE TEMPORARY TABLE schemir_guard ({} INT NOT NULL);",
                d.quote("Schemir: posible perdida de datos")
            );
            let _ = writeln!(
                s,
                "INSERT INTO schemir_guard SELECT NULL FROM {qn} LIMIT 1;"
            );
            let _ = writeln!(s, "DROP TEMPORARY TABLE schemir_guard;");
            let _ = writeln!(s, "SET SESSION sql_mode = @schemir_sql_mode;");
        }
    }
    s
}

fn sorted_columns(t: &Table) -> Vec<&Column> {
    let mut cols: Vec<&Column> = t.columns.iter().collect();
    cols.sort_by_key(|c| c.ordinal);
    cols
}

/// Lista de columnas de la PK ya entrecomillada, p.ej. `[Id], [Linea]`.
fn pk_list(d: Dialect, cols: &[String]) -> String {
    cols.iter().map(|c| d.quote(c)).collect::<Vec<_>>().join(", ")
}

/// Cláusula de clave primaria a nivel de tabla, p.ej. `PRIMARY KEY ([Id])`.
///
/// Lleva el nombre de la restricción del ORIGEN cuando el motor lo expone, para
/// que el destino quede igual que A; si no se conoce (SQLite, o una comparación
/// guardada antes de que se leyera el nombre) se emite la forma anónima, que los
/// cuatro motores aceptan y nombran ellos. En MySQL/MariaDB siempre se usa la
/// forma anónima: la clave primaria se llama siempre `PRIMARY` y el motor
/// ignora cualquier otro nombre. En SQLite, `PRIMARY KEY("id")` sobre
/// una columna `integer` sigue siendo alias de `rowid` (solo `DESC` rompería esa
/// equivalencia).
fn primary_key_ddl(d: Dialect, t: &Table) -> Option<String> {
    let cols = pk_columns(t);
    if cols.is_empty() {
        return None;
    }
    let list = pk_list(d, &cols);
    Some(match (&t.pk_name, d) {
        // En SQLite el nombre no se lee nunca y en MySQL no sirve de nada; en
        // los otros dos, si lo tenemos, nombramos la restricción igual que en A.
        (Some(n), Dialect::SqlServer | Dialect::Postgres) => {
            format!("CONSTRAINT {} PRIMARY KEY ({list})", d.quote(n))
        }
        _ => format!("PRIMARY KEY ({list})"),
    })
}

/// Cómo se muestra una lista de columnas de PK en los comentarios del script.
fn pk_shown(cols: &[String]) -> String {
    if cols.is_empty() {
        "(sin PK)".to_string()
    } else {
        cols.join(", ")
    }
}

/// SQL para que la clave primaria del DESTINO quede como la del ORIGEN.
/// Devuelve `None` cuando ya coinciden.
///
/// Es una operación delicada: en SQL Server y PostgreSQL hay que soltar la
/// restricción vieja por su nombre y crear la nueva, y el motor rechaza el
/// `DROP` mientras alguna clave foránea la referencie — por eso el script lo
/// avisa. En SQLite no se puede hacer con `ALTER`: habría que recrear la tabla,
/// así que solo se deja el comentario (igual que con los cambios de columna).
fn primary_key_change(d: Dialect, src: &Table, tgt: &Table) -> Option<String> {
    let (spk, tpk) = (pk_columns(src), pk_columns(tgt));
    if spk == tpk {
        return None;
    }
    let qn = d.qname(&src.schema, &src.name);
    let mut sql = String::new();

    if d == Dialect::Sqlite {
        let _ = writeln!(
            sql,
            "-- SQLite no permite cambiar la clave primaria de {qn} (de «{}» a «{}»): hay que recrear la tabla.",
            pk_shown(&tpk),
            pk_shown(&spk)
        );
        return Some(sql);
    }

    let _ = writeln!(sql, "-- Clave primaria de {qn}: «{}» → «{}».", pk_shown(&tpk), pk_shown(&spk));
    if !tpk.is_empty() {
        let _ = writeln!(sql, "-- (!) El motor rechaza el DROP mientras una clave foránea referencie esta PK.");
        let _ = match (d, &tgt.pk_name) {
            // MySQL/MariaDB no tienen `DROP CONSTRAINT` para la PK: siempre se
            // llama «PRIMARY» y se suelta con su propia sintaxis.
            (Dialect::MySql, _) => writeln!(sql, "ALTER TABLE {qn} DROP PRIMARY KEY;"),
            (_, Some(n)) => writeln!(sql, "ALTER TABLE {qn} DROP CONSTRAINT {};", d.quote(n)),
            (_, None) => writeln!(sql, "-- (!) No se conoce el nombre de la PK actual de {qn}: suéltala a mano antes de crear la nueva."),
        };
    }
    if !spk.is_empty() {
        let list = pk_list(d, &spk);
        let _ = match (d, &src.pk_name) {
            // En MySQL la PK no se puede nombrar: siempre la forma anónima.
            (Dialect::MySql, _) | (_, None) => writeln!(sql, "ALTER TABLE {qn} ADD PRIMARY KEY ({list});"),
            (_, Some(n)) => writeln!(sql, "ALTER TABLE {qn} ADD CONSTRAINT {} PRIMARY KEY ({list});", d.quote(n)),
        };
    }
    Some(sql)
}

fn create_table(d: Dialect, t: &Table) -> String {
    // Columnas primero y, si la tabla tiene clave primaria, su cláusula al final.
    // Sin esto la tabla nacía sin PK en el destino y cualquier clave foránea que
    // la referenciara fallaba después (SQL Server y PostgreSQL exigen la unicidad).
    //
    // La excepción es SQLite: si una columna va a llevar AUTOINCREMENT, su
    // PRIMARY KEY tiene que ir pegado a ella y la cláusula de tabla ya no cabe
    // (el motor rechaza que la clave primaria se declare dos veces).
    let cols = sorted_columns(t);
    let inline_pk = d == Dialect::Sqlite && cols.iter().any(|c| sqlite_autoincrement(t, c));
    let mut parts: Vec<String> = Vec::with_capacity(cols.len() + 1);
    for c in &cols {
        if d == Dialect::Sqlite && sqlite_autoincrement(t, c) {
            // NOT NULL es implícito: un «integer primary key» de SQLite es alias
            // de `rowid` y nunca es nulo.
            parts.push(format!(
                "    {} {} PRIMARY KEY AUTOINCREMENT",
                d.quote(&c.name),
                c.data_type
            ));
        } else {
            parts.push(format!("    {}", column_ddl(d, c)));
        }
    }
    if !inline_pk {
        if let Some(pk) = primary_key_ddl(d, t) {
            parts.push(format!("    {pk}"));
        }
    }
    format!(
        "{}CREATE TABLE {} (\n{}\n);",
        identity_notes(d, t),
        d.qname(&t.schema, &t.name),
        parts.join(",\n")
    )
}

fn routine_keyword(kind: RoutineKind) -> &'static str {
    match kind {
        RoutineKind::View => "VIEW",
        RoutineKind::Procedure => "PROCEDURE",
        RoutineKind::ScalarFunction | RoutineKind::TableFunction => "FUNCTION",
    }
}

fn drop_routine(d: Dialect, r: &Routine) -> String {
    let kw = routine_keyword(r.kind);
    match d {
        Dialect::SqlServer => format!("DROP {kw} IF EXISTS {};\nGO", d.qname(&r.schema, &r.name)),
        Dialect::Sqlite | Dialect::Postgres | Dialect::MySql => {
            format!("DROP {kw} IF EXISTS {};", d.qname(&r.schema, &r.name))
        }
    }
}

/// Delimitador alterno que se usa en MySQL/MariaDB para poder escribir cuerpos
/// con `;` dentro (procedimientos, funciones y disparadores).
const MYSQL_DELIMITER: &str = "$$";

/// Envuelve un `CREATE` de MySQL cuyo cuerpo lleva `;` entre directivas
/// `DELIMITER`, que es como lo esperan los clientes (`mysql`, Workbench,
/// DBeaver). Sin esto el cliente corta la sentencia en el primer `;` del cuerpo.
fn mysql_delimited(body: &str) -> String {
    let body = body.trim().trim_end_matches(';');
    format!("DELIMITER {MYSQL_DELIMITER}\n{body}{MYSQL_DELIMITER}\nDELIMITER ;")
}

fn create_routine(d: Dialect, r: &Routine) -> String {
    // La definición almacenada ya es el CREATE completo del objeto.
    let body = r.definition.trim();
    match d {
        Dialect::SqlServer => format!("{body}\nGO"),
        // Una vista de MySQL es una sola sentencia: no necesita DELIMITER.
        Dialect::MySql if r.kind == RoutineKind::View => format!("{};", body.trim_end_matches(';')),
        Dialect::MySql => mysql_delimited(body),
        Dialect::Sqlite | Dialect::Postgres => format!("{};", body.trim_end_matches(';')),
    }
}

/// Emite los ALTER para que la tabla del destino quede como la del origen.
/// Emite los ALTER de columnas y además anota en `risks` las operaciones que
/// pueden destruir datos, para que quien llama decida si antepone el guardia.
fn alter_table(
    d: Dialect,
    src: &Table,
    tgt: &Table,
    o: &CompareOptions,
    out: &mut String,
    changes: &mut usize,
    risks: &mut Vec<String>,
) {
    let qn = d.qname(&src.schema, &src.name);
    let tgt_cols: BTreeMap<&str, &Column> =
        tgt.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let src_names: BTreeSet<&str> = src.columns.iter().map(|c| c.name.as_str()).collect();

    // Columnas a agregar o modificar (recorremos las de A en orden).
    for c in sorted_columns(src) {
        match tgt_cols.get(c.name.as_str()) {
            None => {
                // Una columna que se autogenera no siempre se puede agregar así:
                // el aviso va delante del ALTER para que se vea al revisarlo.
                if d == Dialect::Sqlite && c.identity.is_some() {
                    // `AUTOINCREMENT` solo se declara al crear la tabla: ningún
                    // ALTER de SQLite lo admite, ni siquiera sobre la PK.
                    let _ = writeln!(
                        out,
                        "-- (!) «{}» se autogenera en el origen, pero SQLite no puede agregar una columna AUTOINCREMENT con ALTER: hay que recrear {qn}.",
                        c.name
                    );
                } else if let Some(note) = identity_note(d, src, c) {
                    out.push_str(&note);
                }
                match d {
                    Dialect::SqlServer => {
                        let _ = writeln!(out, "ALTER TABLE {qn} ADD {};", column_ddl(d, c));
                    }
                    Dialect::Sqlite | Dialect::Postgres | Dialect::MySql => {
                        let _ = writeln!(out, "ALTER TABLE {qn} ADD COLUMN {};", column_ddl(d, c));
                    }
                }
                *changes += 1;
            }
            Some(t) if !columns_equal(c, t, o) => {
                // Cambiar el tipo puede truncar lo que ya hay guardado. En SQLite
                // no emitimos ALTER (solo un comentario), así que allí no hay
                // ninguna sentencia destructiva que frenar.
                let type_changed = if o.ignore_case {
                    !c.data_type.eq_ignore_ascii_case(&t.data_type)
                } else {
                    c.data_type != t.data_type
                };
                if type_changed && d != Dialect::Sqlite {
                    risks.push(format!(
                        "cambia el tipo de «{}» ({} → {})",
                        c.name, t.data_type, c.data_type
                    ));
                }
                // Que una columna empiece o deje de autogenerarse es un cambio
                // aparte, y cada motor lo trata muy distinto: PostgreSQL lo hace
                // con un ALTER, MySQL redefiniendo la columna entera, y SQL
                // Server y SQLite no pueden sin recrear la tabla.
                let identity_changed = c.identity != t.identity;
                let col = d.quote(&c.name);
                match d {
                    Dialect::SqlServer => {
                        let _ = writeln!(
                            out,
                            "ALTER TABLE {qn} ALTER COLUMN {col} {} {};",
                            c.data_type,
                            null_kw(c)
                        );
                        if identity_changed {
                            let _ = writeln!(
                                out,
                                "-- (!) SQL Server no puede poner ni quitar IDENTITY en «{}» con ALTER: hay que recrear {qn} (o agregar una columna nueva y copiar los datos).",
                                c.name
                            );
                        }
                    }
                    Dialect::Sqlite => {
                        let _ = writeln!(
                            out,
                            "-- SQLite no permite modificar «{}» (de «{} {}» a «{} {}»): hay que recrear {qn}.",
                            c.name,
                            t.data_type,
                            null_kw(t),
                            c.data_type,
                            null_kw(c)
                        );
                        if identity_changed {
                            let _ = writeln!(
                                out,
                                "-- (!) Y lo mismo con su autogeneración: AUTOINCREMENT solo se puede declarar al crear la tabla."
                            );
                        }
                    }
                    Dialect::Postgres => {
                        // Solo si el tipo cambió de verdad: en PostgreSQL un
                        // ALTER COLUMN TYPE reescribe la tabla entera, incluso
                        // cuando el tipo nuevo es igual al viejo.
                        if type_changed {
                            let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} TYPE {};", c.data_type);
                        }
                        if !type_changed && !identity_changed {
                            let _ = writeln!(
                                out,
                                "-- (!) «{}» de {qn} difiere en la nulabilidad o en el valor por defecto, y Schemir todavía no sincroniza ninguna de las dos.",
                                c.name
                            );
                        }
                        if identity_changed {
                            match (&t.identity, &c.identity) {
                                (Some(_), None) => {
                                    let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} DROP IDENTITY;");
                                }
                                (None, Some(id)) => {
                                    // PostgreSQL exige que una columna de
                                    // identidad sea NOT NULL.
                                    if !c.is_nullable {
                                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} SET NOT NULL;");
                                    }
                                    if let Some(clause) = identity_clause(d, id) {
                                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} ADD {clause};");
                                    }
                                }
                                (Some(before), Some(after)) => {
                                    if before.always != after.always {
                                        let kind = if after.always { "ALWAYS" } else { "BY DEFAULT" };
                                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} SET GENERATED {kind};");
                                    }
                                    if before.seed != after.seed {
                                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} SET START WITH {};", after.seed);
                                    }
                                    if before.increment != after.increment {
                                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {col} SET INCREMENT BY {};", after.increment);
                                    }
                                }
                                (None, None) => {}
                            }
                        }
                    }
                    // MySQL/MariaDB redefinen la columna entera con MODIFY COLUMN:
                    // hay que repetir el tipo, la nulabilidad Y el AUTO_INCREMENT,
                    // porque lo que no se escribe vuelve a su valor por defecto.
                    Dialect::MySql => {
                        let auto = c
                            .identity
                            .as_ref()
                            .and_then(|id| identity_clause(d, id))
                            .map(|cl| format!(" {cl}"))
                            .unwrap_or_default();
                        if let Some(note) = identity_note(d, src, c) {
                            out.push_str(&note);
                        }
                        let _ = writeln!(
                            out,
                            "ALTER TABLE {qn} MODIFY COLUMN {col} {} {}{auto};",
                            c.data_type,
                            null_kw(c)
                        );
                    }
                }
                *changes += 1;
            }
            _ => {}
        }
    }

    // Columnas a eliminar (están en B pero no en A).
    for c in sorted_columns(tgt) {
        if !src_names.contains(c.name.as_str()) {
            let _ = writeln!(out, "ALTER TABLE {qn} DROP COLUMN {};", d.quote(&c.name));
            risks.push(format!("se elimina la columna «{}»", c.name));
            *changes += 1;
        }
    }
}

/// Genera una unidad por rutina (vista, proc o función) que cambia.
fn routine_units(d: Dialect, label: &str, source: &[Routine], target: &[Routine], o: &CompareOptions, units: &mut Vec<ScriptUnit>) {
    use DiffStatus::*;
    const SECTION: &str = "Vistas / Procedimientos / Funciones";
    let tgt_by_name: BTreeMap<String, &Routine> =
        target.iter().map(|r| (r.qualified_name(), r)).collect();
    let src_names: BTreeSet<String> = source.iter().map(|r| r.qualified_name()).collect();

    // Crear (nuevas en A) o recrear (distintas): DROP + CREATE.
    for r in source {
        match tgt_by_name.get(&r.qualified_name()) {
            None => units.push(ScriptUnit::new(label, r.qualified_name(), OnlyInLeft, SECTION, create_routine(d, r))),
            Some(t) if normalize_def(&r.definition, o) != normalize_def(&t.definition, o) => {
                units.push(ScriptUnit::new(label, r.qualified_name(), Different, SECTION, format!("{}\n{}", drop_routine(d, t), create_routine(d, r))));
            }
            _ => {}
        }
    }

    // Eliminar (están en B pero no en A).
    for t in target {
        if !src_names.contains(&t.qualified_name()) {
            units.push(ScriptUnit::new(label, t.qualified_name(), OnlyInRight, SECTION, drop_routine(d, t)));
        }
    }
}

/// Genera una unidad por disparador (trigger) que cambia.
fn trigger_units(d: Dialect, source: &[Trigger], target: &[Trigger], o: &CompareOptions, units: &mut Vec<ScriptUnit>) {
    use DiffStatus::*;
    const SECTION: &str = "Disparadores";
    let src: BTreeMap<String, &Trigger> = source.iter().map(|t| (t.key(), t)).collect();
    let tgt: BTreeMap<String, &Trigger> = target.iter().map(|t| (t.key(), t)).collect();
    for (k, t) in &src {
        match tgt.get(k) {
            None => units.push(ScriptUnit::new("Disparador", k.clone(), OnlyInLeft, SECTION, create_trigger(d, t))),
            Some(tt) if normalize_def(&t.definition, o) != normalize_def(&tt.definition, o) => {
                units.push(ScriptUnit::new("Disparador", k.clone(), Different, SECTION, format!("{}\n{}", drop_trigger(d, tt), create_trigger(d, t))));
            }
            _ => {}
        }
    }
    for (k, tt) in &tgt {
        if !src.contains_key(k) {
            units.push(ScriptUnit::new("Disparador", k.clone(), OnlyInRight, SECTION, drop_trigger(d, tt)));
        }
    }
}

/// Separa un nombre calificado "schema.tabla" en sus dos partes.
fn split_qualified(qn: &str) -> (&str, &str) {
    qn.split_once('.').unwrap_or(("", qn))
}

fn create_index(d: Dialect, idx: &Index) -> String {
    let (schema, table) = split_qualified(&idx.table);
    let uniq = if idx.is_unique { "UNIQUE " } else { "" };
    let cols = idx.columns.iter().map(|c| d.quote(c)).collect::<Vec<_>>().join(", ");
    match d {
        Dialect::Sqlite => format!("CREATE {uniq}INDEX {} ON {} ({});", d.quote(&idx.name), d.quote(table), cols),
        _ => format!("CREATE {uniq}INDEX {} ON {} ({});", d.quote(&idx.name), d.qname(schema, table), cols),
    }
}

fn drop_index(d: Dialect, idx: &Index) -> String {
    let (schema, table) = split_qualified(&idx.table);
    match d {
        // Ninguno de los dos admite IF EXISTS aquí, y ambos exigen la tabla.
        Dialect::SqlServer | Dialect::MySql => format!("DROP INDEX {} ON {};", d.quote(&idx.name), d.qname(schema, table)),
        Dialect::Postgres => format!("DROP INDEX IF EXISTS {};", d.qname(schema, &idx.name)),
        Dialect::Sqlite => format!("DROP INDEX IF EXISTS {};", d.quote(&idx.name)),
    }
}

fn add_fk(d: Dialect, fk: &ForeignKey) -> String {
    if matches!(d, Dialect::Sqlite) {
        return format!("-- SQLite: la FK «{}» requiere recrear la tabla (no soporta ADD CONSTRAINT).", fk.name);
    }
    let (fs, ft) = split_qualified(&fk.from_table);
    let (ts, tt) = split_qualified(&fk.to_table);
    let from_cols = fk.from_columns.iter().map(|c| d.quote(c)).collect::<Vec<_>>().join(", ");
    let to_cols = fk.to_columns.iter().map(|c| d.quote(c)).collect::<Vec<_>>().join(", ");
    format!(
        "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({});",
        d.qname(fs, ft),
        d.quote(&fk.name),
        from_cols,
        d.qname(ts, tt),
        to_cols
    )
}

fn drop_fk(d: Dialect, fk: &ForeignKey) -> String {
    if matches!(d, Dialect::Sqlite) {
        return format!("-- SQLite: eliminar la FK «{}» requiere recrear la tabla.", fk.name);
    }
    let (fs, ft) = split_qualified(&fk.from_table);
    if matches!(d, Dialect::MySql) {
        // DROP CONSTRAINT solo existe en MySQL 8.0.19+; DROP FOREIGN KEY vale en
        // todas las versiones de MySQL y de MariaDB.
        return format!("ALTER TABLE {} DROP FOREIGN KEY {};", d.qname(fs, ft), d.quote(&fk.name));
    }
    format!("ALTER TABLE {} DROP CONSTRAINT {};", d.qname(fs, ft), d.quote(&fk.name))
}

fn add_check(d: Dialect, chk: &CheckConstraint) -> String {
    let (s, t) = split_qualified(&chk.table);
    // En SQL Server y en MySQL la definición es solo la expresión; en Postgres
    // (y en SQLite) ya viene con el "CHECK (...)" delante.
    let body = match d {
        Dialect::SqlServer | Dialect::MySql => format!("CHECK {}", chk.definition),
        _ => chk.definition.clone(),
    };
    format!("ALTER TABLE {} ADD CONSTRAINT {} {};", d.qname(s, t), d.quote(&chk.name), body)
}

/// `ALTER TABLE … DROP CONSTRAINT` sirve en los cuatro motores (en MySQL desde
/// 8.0.19 y en MariaDB desde 10.2, prácticamente las mismas versiones que
/// exponen las restricciones CHECK en `information_schema`).
fn drop_check(d: Dialect, chk: &CheckConstraint) -> String {
    let (s, t) = split_qualified(&chk.table);
    format!("ALTER TABLE {} DROP CONSTRAINT {};", d.qname(s, t), d.quote(&chk.name))
}

fn create_trigger(d: Dialect, tr: &Trigger) -> String {
    // La definición almacenada ya es el CREATE TRIGGER completo.
    let body = tr.definition.trim();
    match d {
        Dialect::SqlServer => format!("{body}\nGO"),
        // El cuerpo de un disparador de MySQL lleva `;` dentro: hace falta DELIMITER.
        Dialect::MySql => mysql_delimited(body),
        Dialect::Sqlite | Dialect::Postgres => format!("{};", body.trim_end_matches(';')),
    }
}

fn drop_trigger(d: Dialect, tr: &Trigger) -> String {
    let (ts, tt) = split_qualified(&tr.table);
    match d {
        // PostgreSQL exige indicar la tabla; los demás no.
        Dialect::Postgres => format!("DROP TRIGGER IF EXISTS {} ON {};", d.quote(&tr.name), d.qname(ts, tt)),
        Dialect::SqlServer => format!("DROP TRIGGER IF EXISTS {};\nGO", d.qname(&tr.schema, &tr.name)),
        // En MySQL el disparador vive en la base, no en la tabla: se califica con ella.
        Dialect::MySql => format!("DROP TRIGGER IF EXISTS {};", d.qname(&tr.schema, &tr.name)),
        Dialect::Sqlite => format!("DROP TRIGGER IF EXISTS {};", d.quote(&tr.name)),
    }
}

fn index_key(i: &Index) -> String {
    format!("{}.{}", i.table, i.name)
}
fn same_index(a: &Index, b: &Index) -> bool {
    a.columns == b.columns && a.is_unique == b.is_unique
}
fn same_fk(a: &ForeignKey, b: &ForeignKey) -> bool {
    a.from_table == b.from_table && a.from_columns == b.from_columns && a.to_table == b.to_table && a.to_columns == b.to_columns
}

// Nota: la sincronización por categorías (índices, claves foráneas y CHECKs)
// ahora vive en `generate_sync_units`, que produce una unidad por objeto para
// que la interfaz pueda elegir qué se sincroniza.

/// Una "unidad" del script: todo el SQL necesario para sincronizar UN objeto.
/// La interfaz usa `(object_type, name)` para casar cada unidad con su fila de la
/// lista de diferencias, y así permitir marcar/desmarcar qué objetos entran.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptUnit {
    /// Tipo legible, igual que en el diff: "Tabla", "Índice", "Vista"…
    pub object_type: String,
    /// Nombre/clave del objeto, igual que en el diff (p.ej. "dbo.Clientes").
    pub name: String,
    pub status: DiffStatus,
    /// Sección a la que pertenece (para agrupar el script compuesto).
    pub section: String,
    /// El SQL de esta unidad (ya recortado de espacios sobrantes al final).
    pub sql: String,
}

impl ScriptUnit {
    fn new(object_type: &str, name: impl Into<String>, status: DiffStatus, section: &str, sql: String) -> Self {
        ScriptUnit {
            object_type: object_type.into(),
            name: name.into(),
            status,
            section: section.into(),
            sql: sql.trim_end().to_string(),
        }
    }
}

/// Genera el script separado por objeto con las opciones por defecto.
/// Los módulos que hay que refrescar, en orden de dependencia.
///
/// SQL Server resuelve las referencias de vistas y rutinas de forma diferida y
/// les guarda los metadatos en caché. Al cambiar la forma de una tabla esos
/// metadatos quedan viejos —una vista con `SELECT *` sigue anunciando las
/// columnas de antes— hasta que se los refresca. Es lo que hace SSDT.
///
/// `changed` son las tablas cuya forma cambia en este script (nuevas, o con
/// cambios de columnas) y `rewritten` los módulos que el script ya vuelve a
/// crear: esos nacen compilados y no hay que tocarlos.
///
/// El resultado va por capas: primero los que dependen directamente de una
/// tabla cambiada, luego los que dependen de esos, y así. Importa porque una
/// vista montada sobre otra tiene que refrescarse DESPUÉS de la de abajo. Se
/// itera hasta que una capa no agrega nada, así un ciclo de referencias no lo
/// cuelga. Dentro de cada capa, por nombre, para que el script sea estable.
fn modules_to_refresh<'a>(
    source: &'a Schema,
    changed: &BTreeSet<String>,
    rewritten: &BTreeSet<String>,
) -> Vec<&'a Routine> {
    // En SQL Server los identificadores no distinguen mayúsculas, y el catálogo
    // puede devolver una referencia escrita de otra forma: se compara en minúsculas.
    let baja = |s: &str| s.to_lowercase();
    let modules: Vec<&Routine> = source
        .views
        .iter()
        .chain(&source.procedures)
        .chain(&source.functions)
        .collect();

    let mut alcanzados: BTreeSet<String> = changed.iter().map(|t| baja(t)).collect();
    let mut orden: Vec<&Routine> = Vec::new();
    let mut ya: BTreeSet<String> = BTreeSet::new();

    loop {
        let mut capa: Vec<&Routine> = modules
            .iter()
            .copied()
            .filter(|m| !ya.contains(&baja(&m.qualified_name())))
            .filter(|m| m.references.iter().any(|r| alcanzados.contains(&baja(r))))
            .collect();
        if capa.is_empty() {
            break;
        }
        capa.sort_by_key(|m| m.qualified_name());
        for m in &capa {
            ya.insert(baja(&m.qualified_name()));
        }
        // Los de esta capa pasan a contar como "cambiados" para la siguiente.
        for m in &capa {
            alcanzados.insert(baja(&m.qualified_name()));
        }
        orden.extend(capa);
    }

    orden
        .into_iter()
        .filter(|m| !rewritten.contains(&m.qualified_name()))
        .collect()
}

/// `EXECUTE sp_refreshsqlmodule` para un módulo, igual que lo emite SSDT.
fn refresh_module(r: &Routine) -> String {
    let qn = Dialect::SqlServer.qname(&r.schema, &r.name);
    // El nombre viaja dentro de una cadena: hay que doblar la comilla simple.
    format!("EXECUTE sp_refreshsqlmodule N'{}';", qn.replace('\'', "''"))
}

pub fn generate_sync_units(source: &Schema, target: &Schema, d: Dialect) -> Vec<ScriptUnit> {
    generate_sync_units_opts(source, target, d, &CompareOptions::default())
}

/// Genera el script de sincronización SEPARADO por objeto. Cada unidad trae el
/// SQL de un solo objeto, para que la interfaz pueda componer el script final
/// con solo lo que el usuario deja marcado. Van en orden seguro de dependencias:
/// primero crear/alterar, luego índices/FK/CHECK/rutinas, y al final los DROP.
/// Respeta las mismas opciones que el diff (qué ignorar y qué tipos sincronizar).
pub fn generate_sync_units_opts(source: &Schema, target: &Schema, d: Dialect, o: &CompareOptions) -> Vec<ScriptUnit> {
    use DiffStatus::*;
    let mut units: Vec<ScriptUnit> = Vec::new();

    let tgt_tables: BTreeMap<String, &Table> =
        target.tables.iter().map(|t| (t.qualified_name(), t)).collect();
    let src_tnames: BTreeSet<String> = source.tables.iter().map(|t| t.qualified_name()).collect();
    // Tablas cuya FORMA cambia en este script: las nuevas y las que cambian de
    // columnas. Son las que dejan con metadatos viejos a quien las referencia.
    let mut reshaped: BTreeSet<String> = BTreeSet::new();

    if o.tables {
        // ── Tablas nuevas ──
        for t in &source.tables {
            if !tgt_tables.contains_key(&t.qualified_name()) {
                reshaped.insert(t.qualified_name());
                units.push(ScriptUnit::new("Tabla", t.qualified_name(), OnlyInLeft, "Tablas nuevas", create_table(d, t)));
            }
        }

        // ── Cambios en columnas (tablas presentes en ambos) ──
        for src in &source.tables {
            if let Some(tgt) = tgt_tables.get(&src.qualified_name()) {
                let mut sql = String::new();
                let mut changes = 0usize;
                let mut risks: Vec<String> = Vec::new();
                alter_table(d, src, tgt, o, &mut sql, &mut changes, &mut risks);
                if changes > 0 {
                    if o.block_on_data_loss && !risks.is_empty() {
                        sql.insert_str(0, &data_loss_guard(d, &src.schema, &src.name, &risks));
                    }
                    reshaped.insert(src.qualified_name());
                    units.push(ScriptUnit::new("Tabla", src.qualified_name(), Different, "Cambios en columnas", sql));
                }
            }
        }

        // ── Cambios de clave primaria (tablas presentes en ambos) ──
        // El tipo y el nombre son los mismos que usa `diff_primary_keys`, para
        // que la interfaz pueda emparejar esta unidad con su fila y permitir
        // marcarla o desmarcarla como cualquier otro objeto.
        for src in &source.tables {
            if let Some(tgt) = tgt_tables.get(&src.qualified_name()) {
                if let Some(sql) = primary_key_change(d, src, tgt) {
                    units.push(ScriptUnit::new(
                        "Clave primaria",
                        format!("PK · {}", src.qualified_name()),
                        Different,
                        "Claves primarias",
                        sql,
                    ));
                }
            }
        }
    }

    // ── Índices ──
    if o.indexes {
        // Emparejados por nombre y, los que bautizó el motor, por su forma: así un
        // `UQ__…` con otro sufijo en cada base no se reporta como quitar uno y
        // crear otro. El orden se mantiene por clave, como antes.
        let mut pares = pair_objects(&source.indexes, &target.indexes, index_name_key, index_shape, |i| i.system_named);
        pares.sort_by_key(|(s, t)| s.or(*t).map(index_key).unwrap_or_default());
        for (s, t) in pares {
            match (s, t) {
                (Some(i), None) => units.push(ScriptUnit::new("Índice", index_key(i), OnlyInLeft, "Índices", create_index(d, i))),
                (Some(i), Some(ti)) if !same_index(i, ti) => units.push(ScriptUnit::new("Índice", index_key(i), Different, "Índices", format!("{}\n{}", drop_index(d, ti), create_index(d, i)))),
                (None, Some(ti)) => units.push(ScriptUnit::new("Índice", index_key(ti), OnlyInRight, "Índices", drop_index(d, ti))),
                _ => {}
            }
        }
    }

    // ── Claves foráneas ──
    if o.foreign_keys {
        let mut pares = pair_objects(&source.foreign_keys, &target.foreign_keys, |f| f.name.clone(), fk_sig, |f| f.system_named);
        pares.sort_by_key(|(s, t)| s.or(*t).map(|f| f.name.clone()).unwrap_or_default());
        for (s, t) in pares {
            match (s, t) {
                (Some(f), None) => units.push(ScriptUnit::new("Clave foránea", f.name.clone(), OnlyInLeft, "Claves foráneas", add_fk(d, f))),
                (Some(f), Some(tf)) if !same_fk(f, tf) => units.push(ScriptUnit::new("Clave foránea", f.name.clone(), Different, "Claves foráneas", format!("{}\n{}", drop_fk(d, tf), add_fk(d, f)))),
                (None, Some(tf)) => units.push(ScriptUnit::new("Clave foránea", tf.name.clone(), OnlyInRight, "Claves foráneas", drop_fk(d, tf))),
                _ => {}
            }
        }
    }

    // ── Restricciones CHECK ──
    if o.checks {
        let mut pares = pair_objects(&source.checks, &target.checks, |c| c.name.clone(), |c| check_shape(c, o), |c| c.system_named);
        pares.sort_by_key(|(s, t)| s.or(*t).map(|c| c.name.clone()).unwrap_or_default());
        for (s, t) in pares {
            match (s, t) {
                (Some(c), None) => units.push(ScriptUnit::new("Restricción", c.name.clone(), OnlyInLeft, "Restricciones CHECK", add_check(d, c))),
                (Some(c), Some(tc)) if normalize_def(&c.definition, o) != normalize_def(&tc.definition, o) => units.push(ScriptUnit::new("Restricción", c.name.clone(), Different, "Restricciones CHECK", format!("{}\n{}", drop_check(d, tc), add_check(d, c)))),
                (None, Some(tc)) => units.push(ScriptUnit::new("Restricción", tc.name.clone(), OnlyInRight, "Restricciones CHECK", drop_check(d, tc))),
                _ => {}
            }
        }
    }

    // ── Vistas / Procedimientos / Funciones ──
    if o.views {
        routine_units(d, "Vista", &source.views, &target.views, o, &mut units);
    }
    if o.procedures {
        routine_units(d, "Procedimiento", &source.procedures, &target.procedures, o, &mut units);
    }
    if o.functions {
        routine_units(d, "Función", &source.functions, &target.functions, o, &mut units);
    }

    // ── Disparadores ──
    if o.triggers {
        trigger_units(d, &source.triggers, &target.triggers, o, &mut units);
    }

    // ── Refrescar módulos ──
    // Va después de crear y alterar todo, porque refresca contra el estado final.
    // Solo SQL Server lo necesita: PostgreSQL guarda las vistas ya resueltas (no
    // deja ni quitar una columna que una vista usa), y MySQL y SQLite resuelven
    // al crear o al consultar.
    if d == Dialect::SqlServer && !reshaped.is_empty() {
        let rewritten: BTreeSet<String> = units
            .iter()
            .filter(|u| u.section == "Vistas / Procedimientos / Funciones")
            .map(|u| u.name.clone())
            .collect();
        for m in modules_to_refresh(source, &reshaped, &rewritten) {
            units.push(ScriptUnit::new(
                "Módulo",
                m.qualified_name(),
                Different,
                "Refrescar módulos",
                refresh_module(m),
            ));
        }
    }

    // ── Tablas eliminadas en el destino ──
    if o.tables {
        for t in &target.tables {
            if !src_tnames.contains(&t.qualified_name()) {
                let drop = format!("DROP TABLE IF EXISTS {};", d.qname(&t.schema, &t.name));
                let sql = if o.block_on_data_loss {
                    let reasons = vec!["se elimina la tabla completa".to_string()];
                    format!("{}{drop}", data_loss_guard(d, &t.schema, &t.name, &reasons))
                } else {
                    drop
                };
                units.push(ScriptUnit::new("Tabla", t.qualified_name(), OnlyInRight, "Tablas eliminadas en el destino", sql));
            }
        }
    }

    units
}

/// Encabezado común del script (comentarios de advertencia).
fn script_header(d: Dialect) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "-- ============================================================");
    let _ = writeln!(out, "-- Script de sincronización generado por Schemir");
    let _ = writeln!(out, "-- Deja el DESTINO igual que el ORIGEN");
    let _ = writeln!(out, "-- Motor destino: {}", d.label());
    let _ = writeln!(out, "-- (!) Revísalo y respalda el destino antes de ejecutarlo.");
    let _ = writeln!(out, "-- ============================================================");
    out
}

/// Genera el script SQL completo con las opciones por defecto.
pub fn generate_sync_script(source: &Schema, target: &Schema, d: Dialect) -> String {
    generate_sync_script_opts(source, target, d, &CompareOptions::default())
}

/// Genera el script SQL completo que transforma el esquema `target` (B) en
/// `source` (A), componiendo todas las unidades (sin filtrar) según las opciones.
pub fn generate_sync_script_opts(source: &Schema, target: &Schema, d: Dialect, o: &CompareOptions) -> String {
    let units = generate_sync_units_opts(source, target, d, o);
    let mut out = script_header(d);
    out.push('\n');

    if units.is_empty() {
        let _ = writeln!(out, "-- Sin diferencias: el destino ya coincide con el origen.");
        return out;
    }

    let mut section: &str = "";
    for u in &units {
        if u.section != section {
            let _ = write!(out, "-- ─── {} ───\n", u.section);
            section = u.section.as_str();
        }
        out.push_str(&u.sql);
        out.push_str("\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Routine, RoutineKind, Schema, Table};

    fn col(name: &str, ty: &str, nullable: bool) -> Column {
        Column {
            name: name.into(),
            data_type: ty.into(),
            is_nullable: nullable,
            default_value: None,
            ordinal: 0,
            is_primary_key: false,
            identity: None,
        }
    }
    /// Columna que forma parte de la clave primaria (con su posición).
    fn pk_col(name: &str, ty: &str, ordinal: i32) -> Column {
        Column {
            name: name.into(),
            data_type: ty.into(),
            is_nullable: false,
            default_value: None,
            ordinal,
            is_primary_key: true,
            identity: None,
        }
    }
    /// Columna que es la clave primaria Y se autogenera: el caso normal de una
    /// tabla con `IDENTITY` / `AUTO_INCREMENT` / `AUTOINCREMENT`.
    fn id_col(name: &str, ty: &str, id: Identity) -> Column {
        Column {
            name: name.into(),
            data_type: ty.into(),
            is_nullable: false,
            default_value: None,
            ordinal: 0,
            is_primary_key: true,
            identity: Some(id),
        }
    }
    fn table(name: &str, cols: Vec<Column>) -> Table {
        Table { schema: "dbo".into(), name: name.into(), columns: cols, pk_name: None }
    }
    /// Igual que `table`, pero con el nombre de la restricción de PK.
    fn table_pk(name: &str, cols: Vec<Column>, pk_name: &str) -> Table {
        Table { schema: "dbo".into(), name: name.into(), columns: cols, pk_name: Some(pk_name.into()) }
    }

    #[test]
    fn crea_tabla_faltante() {
        let a = Schema { tables: vec![table("Clientes", vec![col("Id", "int", false)])], ..Default::default() };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("CREATE TABLE [dbo].[Clientes]"), "{s}");
        assert!(s.contains("[Id] int NOT NULL"), "{s}");
    }

    #[test]
    fn crea_tabla_con_clave_primaria() {
        let a = Schema {
            tables: vec![table("Clientes", vec![pk_col("Id", "int", 1), col("Nombre", "varchar(50)", true)])],
            ..Default::default()
        };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("PRIMARY KEY ([Id])"), "{s}");
    }

    #[test]
    fn clave_primaria_compuesta_en_orden_de_columna() {
        let a = Schema {
            tables: vec![table(
                "Detalle",
                vec![pk_col("PedidoId", "int", 1), pk_col("Linea", "int", 2), col("Cantidad", "int", true)],
            )],
            ..Default::default()
        };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::Postgres);
        assert!(s.contains("PRIMARY KEY (\"PedidoId\", \"Linea\")"), "{s}");
    }

    #[test]
    fn crea_tabla_con_la_pk_nombrada_igual_que_el_origen() {
        let a = Schema {
            tables: vec![table_pk("Clientes", vec![pk_col("Id", "int", 1)], "PK_Clientes")],
            ..Default::default()
        };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("CONSTRAINT [PK_Clientes] PRIMARY KEY ([Id])"), "{s}");
    }

    #[test]
    fn cambia_la_clave_primaria_en_sqlserver() {
        let a = Schema {
            tables: vec![table_pk("T", vec![pk_col("Id", "int", 1), col("Codigo", "varchar(10)", false)], "PK_T")],
            ..Default::default()
        };
        let b = Schema {
            tables: vec![table_pk("T", vec![col("Id", "int", false), pk_col("Codigo", "varchar(10)", 2)], "PK_T_viejo")],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("ALTER TABLE [dbo].[T] DROP CONSTRAINT [PK_T_viejo];"), "{s}");
        assert!(s.contains("ALTER TABLE [dbo].[T] ADD CONSTRAINT [PK_T] PRIMARY KEY ([Id]);"), "{s}");
        // El DROP falla si una FK apunta a la PK vieja: el script lo avisa.
        assert!(s.contains("clave foránea"), "{s}");
    }

    #[test]
    fn agrega_clave_primaria_sin_nombre_conocido() {
        let a = Schema { tables: vec![table("T", vec![pk_col("Id", "int", 1)])], ..Default::default() };
        let b = Schema { tables: vec![table("T", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::Postgres);
        assert!(s.contains("ALTER TABLE \"dbo\".\"T\" ADD PRIMARY KEY (\"Id\");"), "{s}");
        // El destino no tenía PK: no hay nada que soltar.
        assert!(!s.contains("DROP CONSTRAINT"), "{s}");
    }

    #[test]
    fn sqlite_no_puede_cambiar_la_clave_primaria() {
        let a = Schema { tables: vec![table("T", vec![pk_col("id", "integer", 1)])], ..Default::default() };
        let b = Schema { tables: vec![table("T", vec![col("id", "integer", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::Sqlite);
        assert!(s.contains("SQLite no permite cambiar la clave primaria"), "{s}");
        assert!(!s.contains("ALTER TABLE \"dbo\".\"T\" ADD PRIMARY KEY"), "{s}");
    }

    #[test]
    fn pk_igual_no_genera_unidad() {
        let a = Schema { tables: vec![table("T", vec![pk_col("Id", "int", 1)])], ..Default::default() };
        let b = Schema { tables: vec![table("T", vec![pk_col("Id", "int", 1)])], ..Default::default() };
        let units = generate_sync_units_opts(&a, &b, Dialect::SqlServer, &CompareOptions::default());
        assert!(!units.iter().any(|u| u.object_type == "Clave primaria"), "{units:?}");
    }

    #[test]
    fn tabla_sin_clave_primaria_no_emite_clausula() {
        let a = Schema { tables: vec![table("Log", vec![col("Msg", "text", true)])], ..Default::default() };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::Sqlite);
        assert!(!s.contains("PRIMARY KEY"), "{s}");
    }

    #[test]
    fn elimina_tabla_sobrante() {
        let a = Schema::default();
        let b = Schema { tables: vec![table("Temp", vec![])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("DROP TABLE IF EXISTS [dbo].[Temp]"), "{s}");
    }

    #[test]
    fn agrega_columna() {
        let a = Schema { tables: vec![table("C", vec![col("Id", "int", false), col("Email", "varchar(50)", true)])], ..Default::default() };
        let b = Schema { tables: vec![table("C", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("ALTER TABLE [dbo].[C] ADD [Email] varchar(50) NULL"), "{s}");
    }

    #[test]
    fn sqlite_entrecomilla_con_dobles() {
        let a = Schema { tables: vec![table("c", vec![col("id", "integer", false)])], ..Default::default() };
        let b = Schema::default();
        let s = generate_sync_script(&a, &b, Dialect::Sqlite);
        assert!(s.contains("CREATE TABLE \"dbo\".\"c\""), "{s}");
    }

    #[test]
    fn recrea_rutina_distinta() {
        let a = Schema { procedures: vec![Routine { schema: "dbo".into(), name: "sp".into(), kind: RoutineKind::Procedure, definition: "CREATE PROCEDURE dbo.sp AS SELECT 1".into(), references: vec![] }], ..Default::default() };
        let b = Schema { procedures: vec![Routine { schema: "dbo".into(), name: "sp".into(), kind: RoutineKind::Procedure, definition: "CREATE PROCEDURE dbo.sp AS SELECT 2".into(), references: vec![] }], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("DROP PROCEDURE IF EXISTS [dbo].[sp]"), "{s}");
        assert!(s.contains("SELECT 1"), "{s}");
    }

    #[test]
    fn script_crea_indice_faltante() {
        use crate::model::Index;
        let a = Schema {
            indexes: vec![Index { name: "ix1".into(), system_named: false, table: "dbo.C".into(), columns: vec!["a".into(), "b".into()], is_unique: true }],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::SqlServer);
        assert!(s.contains("CREATE UNIQUE INDEX [ix1] ON [dbo].[C] ([a], [b]);"), "{s}");
    }

    #[test]
    fn script_agrega_fk() {
        use crate::model::ForeignKey;
        let a = Schema {
            foreign_keys: vec![ForeignKey {
                name: "fk1".into(),
                system_named: false,
                from_table: "dbo.Pedidos".into(),
                from_columns: vec!["cliente_id".into()],
                to_table: "dbo.Clientes".into(),
                to_columns: vec!["id".into()],
            }],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Postgres);
        assert!(
            s.contains("ALTER TABLE \"dbo\".\"Pedidos\" ADD CONSTRAINT \"fk1\" FOREIGN KEY (\"cliente_id\") REFERENCES \"dbo\".\"Clientes\" (\"id\");"),
            "{s}"
        );
    }

    #[test]
    fn script_dropea_indice_sobrante() {
        use crate::model::Index;
        let b = Schema {
            indexes: vec![Index { name: "ix1".into(), system_named: false, table: "public.c".into(), columns: vec!["a".into()], is_unique: false }],
            ..Default::default()
        };
        let s = generate_sync_script(&Schema::default(), &b, Dialect::Postgres);
        assert!(s.contains("DROP INDEX IF EXISTS \"public\".\"ix1\";"), "{s}");
    }

    #[test]
    fn sin_cambios_lo_dice() {
        let t = table("C", vec![col("Id", "int", false)]);
        let a = Schema { tables: vec![t.clone()], ..Default::default() };
        let b = Schema { tables: vec![t], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("Sin diferencias"), "{s}");
    }

    #[test]
    fn unidades_usan_la_misma_clave_que_el_diff() {
        use crate::model::Index;
        // La clave (object_type, name) de cada unidad debe coincidir con la fila
        // del diff para que la interfaz pueda casarlas y permitir la selección.
        let a = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false)])],
            indexes: vec![Index { name: "ix1".into(), system_named: false, table: "dbo.Clientes".into(), columns: vec!["Id".into()], is_unique: false }],
            ..Default::default()
        };
        let units = generate_sync_units(&a, &Schema::default(), Dialect::SqlServer);
        let tabla = units.iter().find(|u| u.object_type == "Tabla").expect("unidad de tabla");
        assert_eq!(tabla.name, "dbo.Clientes");
        assert!(tabla.sql.contains("CREATE TABLE [dbo].[Clientes]"), "{}", tabla.sql);
        let idx = units.iter().find(|u| u.object_type == "Índice").expect("unidad de índice");
        assert_eq!(idx.name, "dbo.Clientes.ix1"); // = tabla.nombre_del_índice, igual que en diff.rs
    }

    #[test]
    fn script_compuesto_equivale_a_unir_unidades() {
        let a = Schema { tables: vec![table("Nueva", vec![col("Id", "int", false)])], ..Default::default() };
        let b = Schema { tables: vec![table("Vieja", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("CREATE TABLE [dbo].[Nueva]"), "{s}");
        assert!(s.contains("DROP TABLE IF EXISTS [dbo].[Vieja]"), "{s}");
    }

    #[test]
    fn script_crea_y_dropea_trigger() {
        use crate::model::Trigger;
        let trg = Trigger {
            schema: "main".into(),
            name: "trg_x".into(),
            table: "main.t".into(),
            definition: "CREATE TRIGGER trg_x AFTER INSERT ON t BEGIN SELECT 1; END".into(),
        };
        let a = Schema { triggers: vec![trg.clone()], ..Default::default() };
        // Crear (solo en A).
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Sqlite);
        assert!(s.contains("CREATE TRIGGER trg_x"), "{s}");
        // Dropear en SQLite (solo en B): sin la tabla.
        let s2 = generate_sync_script(&Schema::default(), &a, Dialect::Sqlite);
        assert!(s2.contains("DROP TRIGGER IF EXISTS \"trg_x\";"), "{s2}");
        // Dropear en PostgreSQL: con la tabla (ON …).
        let s3 = generate_sync_script(&Schema::default(), &a, Dialect::Postgres);
        assert!(s3.contains("DROP TRIGGER IF EXISTS \"trg_x\" ON \"main\".\"t\";"), "{s3}");
    }

    // ── Bloqueo ante posible pérdida de datos ──────────────────────────────

    #[test]
    fn bloquea_al_quitar_una_columna() {
        let a = Schema { tables: vec![table("C", vec![col("Id", "int", false)])], ..Default::default() };
        let b = Schema {
            tables: vec![table("C", vec![col("Id", "int", false), col("Email", "varchar(50)", true)])],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("IF EXISTS (SELECT TOP 1 1 FROM [dbo].[C])"), "{s}");
        assert!(s.contains("RAISERROR"), "{s}");
        assert!(s.contains("se elimina la columna «Email»"), "{s}");
        // El guardia va ANTES de la sentencia destructiva.
        let guardia = s.find("RAISERROR").unwrap();
        let drop = s.find("DROP COLUMN").unwrap();
        assert!(guardia < drop, "el guardia debe preceder al DROP COLUMN:\n{s}");
    }

    #[test]
    fn bloquea_al_cambiar_el_tipo_de_una_columna() {
        let a = Schema { tables: vec![table("C", vec![col("Email", "varchar(50)", true)])], ..Default::default() };
        let b = Schema { tables: vec![table("C", vec![col("Email", "varchar(100)", true)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("cambia el tipo de «Email» (varchar(100) → varchar(50))"), "{s}");
        assert!(s.contains("RAISERROR"), "{s}");
    }

    #[test]
    fn bloquea_al_eliminar_una_tabla_en_postgres() {
        let a = Schema::default();
        let b = Schema { tables: vec![table("Vieja", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::Postgres);
        assert!(s.contains("IF EXISTS (SELECT 1 FROM \"dbo\".\"Vieja\") THEN"), "{s}");
        assert!(s.contains("RAISE EXCEPTION"), "{s}");
        assert!(s.contains("se elimina la tabla completa"), "{s}");
    }

    #[test]
    fn en_sqlite_el_guardia_usa_un_check() {
        let a = Schema::default();
        let b = Schema { tables: vec![table("Vieja", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::Sqlite);
        assert!(s.contains("CREATE TEMP TABLE schemir_guard"), "{s}");
        assert!(s.contains("WHERE EXISTS (SELECT 1 FROM \"dbo\".\"Vieja\")"), "{s}");
    }

    #[test]
    fn en_mysql_el_guardia_fuerza_un_not_null() {
        let a = Schema::default();
        let b = Schema { tables: vec![table("Vieja", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::MySql);
        assert!(s.contains("CREATE TEMPORARY TABLE schemir_guard"), "{s}");
        // Sin modo estricto el NULL sería un aviso y el guardia no frenaría.
        assert!(s.contains("STRICT_ALL_TABLES"), "{s}");
        assert!(s.contains("INSERT INTO schemir_guard SELECT NULL FROM `dbo`.`Vieja` LIMIT 1;"), "{s}");
    }

    #[test]
    fn agregar_una_columna_no_bloquea() {
        let a = Schema {
            tables: vec![table("C", vec![col("Id", "int", false), col("Email", "varchar(50)", true)])],
            ..Default::default()
        };
        let b = Schema { tables: vec![table("C", vec![col("Id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("ADD [Email]"), "{s}");
        assert!(!s.contains("RAISERROR"), "agregar no destruye nada:\n{s}");
    }

    #[test]
    fn sin_guardia_cuando_la_opcion_esta_apagada() {
        let a = Schema { tables: vec![table("C", vec![col("Id", "int", false)])], ..Default::default() };
        let b = Schema {
            tables: vec![table("C", vec![col("Id", "int", false), col("Email", "varchar(50)", true)])],
            ..Default::default()
        };
        let o = CompareOptions { block_on_data_loss: false, ..Default::default() };
        let s = generate_sync_script_opts(&a, &b, Dialect::SqlServer, &o);
        assert!(s.contains("DROP COLUMN [Email]"), "{s}");
        assert!(!s.contains("RAISERROR"), "{s}");
    }

    // ───────────────────────── MySQL / MariaDB ─────────────────────────

    #[test]
    fn mysql_entrecomilla_con_acentos_graves() {
        let a = Schema { tables: vec![table("clientes", vec![col("id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("CREATE TABLE `dbo`.`clientes`"), "{s}");
        assert!(s.contains("`id` int NOT NULL"), "{s}");
    }

    #[test]
    fn mysql_duplica_el_acento_grave_del_identificador() {
        // Un nombre con ` dentro debe salir escapado al duplicarlo, igual que
        // SQL Server duplica el ] y PostgreSQL la comilla doble.
        let a = Schema { tables: vec![table("ra`ro", vec![col("id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("CREATE TABLE `dbo`.`ra``ro`"), "{s}");
    }

    #[test]
    fn mysql_agrega_columna_con_add_column() {
        let a = Schema { tables: vec![table("c", vec![col("id", "int", false), col("email", "varchar(50)", true)])], ..Default::default() };
        let b = Schema { tables: vec![table("c", vec![col("id", "int", false)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::MySql);
        assert!(s.contains("ALTER TABLE `dbo`.`c` ADD COLUMN `email` varchar(50) NULL;"), "{s}");
    }

    #[test]
    fn mysql_modifica_columna_con_modify_column() {
        // MySQL no tiene «ALTER COLUMN … TYPE»: redefine la columna entera, así
        // que el ALTER tiene que repetir también la nulabilidad.
        let a = Schema { tables: vec![table("c", vec![col("email", "varchar(100)", false)])], ..Default::default() };
        let b = Schema { tables: vec![table("c", vec![col("email", "varchar(50)", true)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::MySql);
        assert!(s.contains("ALTER TABLE `dbo`.`c` MODIFY COLUMN `email` varchar(100) NOT NULL;"), "{s}");
        assert!(!s.contains("ALTER COLUMN"), "{s}");
    }

    #[test]
    fn mysql_cambia_la_clave_primaria_sin_nombrarla() {
        // La PK de MySQL se llama siempre «PRIMARY»: ni el DROP ni el ADD
        // pueden usar el nombre que traiga el origen.
        let a = Schema {
            tables: vec![table_pk("t", vec![pk_col("id", "int", 1), col("codigo", "varchar(10)", false)], "PK_t")],
            ..Default::default()
        };
        let b = Schema {
            tables: vec![table_pk("t", vec![col("id", "int", false), pk_col("codigo", "varchar(10)", 2)], "PK_t_viejo")],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &b, Dialect::MySql);
        assert!(s.contains("ALTER TABLE `dbo`.`t` DROP PRIMARY KEY;"), "{s}");
        assert!(s.contains("ALTER TABLE `dbo`.`t` ADD PRIMARY KEY (`id`);"), "{s}");
        assert!(!s.contains("DROP CONSTRAINT"), "{s}");
        assert!(!s.contains("ADD CONSTRAINT `PK_t`"), "{s}");
    }

    #[test]
    fn mysql_crea_tabla_con_la_pk_anonima() {
        let a = Schema {
            tables: vec![table_pk("clientes", vec![pk_col("id", "int", 1)], "PK_clientes")],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("PRIMARY KEY (`id`)"), "{s}");
        assert!(!s.contains("CONSTRAINT `PK_clientes`"), "{s}");
    }

    #[test]
    fn mysql_envuelve_el_procedimiento_en_delimiter() {
        // Sin DELIMITER el cliente cortaría el cuerpo en su primer «;».
        let a = Schema {
            procedures: vec![Routine {
                references: vec![],
                schema: "tienda".into(),
                name: "sp".into(),
                kind: RoutineKind::Procedure,
                definition: "CREATE PROCEDURE `tienda`.`sp`() BEGIN SELECT 1; END".into(),
            }],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("DELIMITER $$"), "{s}");
        assert!(s.contains("END$$"), "{s}");
        assert!(s.contains("DELIMITER ;"), "{s}");
    }

    #[test]
    fn mysql_no_envuelve_las_vistas_en_delimiter() {
        let a = Schema {
            views: vec![Routine {
                references: vec![],
                schema: "tienda".into(),
                name: "v".into(),
                kind: RoutineKind::View,
                definition: "CREATE VIEW `tienda`.`v` AS SELECT 1".into(),
            }],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("CREATE VIEW `tienda`.`v` AS SELECT 1;"), "{s}");
        assert!(!s.contains("DELIMITER"), "{s}");
    }

    #[test]
    fn mysql_envuelve_el_disparador_en_delimiter() {
        use crate::model::Trigger;
        let trg = Trigger {
            schema: "tienda".into(),
            name: "trg_x".into(),
            table: "tienda.t".into(),
            definition: "CREATE TRIGGER `trg_x` AFTER INSERT ON `t` FOR EACH ROW BEGIN SET @a = 1; END".into(),
        };
        let a = Schema { triggers: vec![trg.clone()], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("DELIMITER $$"), "{s}");
        // Y al soltarlo se califica con la base, no con la tabla.
        let s2 = generate_sync_script(&Schema::default(), &a, Dialect::MySql);
        assert!(s2.contains("DROP TRIGGER IF EXISTS `tienda`.`trg_x`;"), "{s2}");
    }

    #[test]
    fn mysql_dropea_indice_indicando_la_tabla() {
        use crate::model::Index;
        let b = Schema {
            indexes: vec![Index { name: "ix1".into(), system_named: false, table: "tienda.c".into(), columns: vec!["a".into()], is_unique: false }],
            ..Default::default()
        };
        let s = generate_sync_script(&Schema::default(), &b, Dialect::MySql);
        assert!(s.contains("DROP INDEX `ix1` ON `tienda`.`c`;"), "{s}");
    }

    #[test]
    fn mysql_suelta_la_foranea_con_drop_foreign_key() {
        use crate::model::ForeignKey;
        let b = Schema {
            foreign_keys: vec![ForeignKey {
                name: "fk1".into(),
                system_named: false,
                from_table: "tienda.pedidos".into(),
                from_columns: vec!["cliente_id".into()],
                to_table: "tienda.clientes".into(),
                to_columns: vec!["id".into()],
            }],
            ..Default::default()
        };
        let s = generate_sync_script(&Schema::default(), &b, Dialect::MySql);
        // DROP CONSTRAINT solo existe desde MySQL 8.0.19; esta forma vale siempre.
        assert!(s.contains("ALTER TABLE `tienda`.`pedidos` DROP FOREIGN KEY `fk1`;"), "{s}");
    }

    #[test]
    fn mysql_antepone_check_a_la_expresion() {
        use crate::model::CheckConstraint;
        // information_schema.CHECK_CONSTRAINTS da solo la expresión, igual que
        // SQL Server: el ALTER tiene que poner la palabra CHECK delante.
        let a = Schema {
            checks: vec![CheckConstraint { name: "ck_total".into(), system_named: false, table: "tienda.pedidos".into(), definition: "(`total` > 0)".into() }],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("ALTER TABLE `tienda`.`pedidos` ADD CONSTRAINT `ck_total` CHECK (`total` > 0);"), "{s}");
    }

    #[test]
    fn mysql_se_anuncia_en_el_encabezado() {
        let s = generate_sync_script(&Schema::default(), &Schema::default(), Dialect::MySql);
        assert!(s.contains("Motor destino: MySQL / MariaDB"), "{s}");
    }

    // ── Columnas que se autogeneran (IDENTITY / AUTO_INCREMENT / …) ─────────

    #[test]
    fn sql_server_emite_identity_al_crear_la_tabla() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "int", Identity::simple())])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::SqlServer);
        // IDENTITY va entre el tipo y la nulabilidad, y la tabla nace con su PK.
        assert!(s.contains("[Id] int IDENTITY(1,1) NOT NULL"), "{s}");
        assert!(s.contains("PRIMARY KEY ([Id])"), "{s}");
    }

    #[test]
    fn sql_server_respeta_el_inicio_y_el_incremento() {
        let id = Identity { seed: 100, increment: 5, always: false };
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "bigint", id)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::SqlServer);
        assert!(s.contains("IDENTITY(100,5)"), "{s}");
    }

    #[test]
    fn postgres_emite_generated_by_default() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "integer", Identity::simple())])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Postgres);
        assert!(s.contains("\"Id\" integer NOT NULL GENERATED BY DEFAULT AS IDENTITY"), "{s}");
    }

    #[test]
    fn postgres_distingue_generated_always() {
        let id = Identity { seed: 1, increment: 1, always: true };
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "integer", id)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Postgres);
        assert!(s.contains("GENERATED ALWAYS AS IDENTITY"), "{s}");
    }

    #[test]
    fn postgres_lleva_el_inicio_y_el_incremento_entre_parentesis() {
        let id = Identity { seed: 7, increment: 2, always: false };
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "integer", id)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Postgres);
        assert!(s.contains("GENERATED BY DEFAULT AS IDENTITY (START WITH 7 INCREMENT BY 2)"), "{s}");
    }

    #[test]
    fn mysql_emite_auto_increment() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "int", Identity::simple())])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("`Id` int NOT NULL AUTO_INCREMENT"), "{s}");
        // Siendo la clave primaria, no hace falta ningún aviso. (El "(!)" a
        // secas no sirve: el encabezado del script ya lleva uno.)
        assert!(!s.contains("exigen que la columna AUTO_INCREMENT"), "{s}");
        assert!(!s.contains("empieza en"), "{s}");
    }

    #[test]
    fn mysql_avisa_que_no_puede_con_el_inicio() {
        let id = Identity { seed: 100, increment: 1, always: false };
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "int", id)])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("AUTO_INCREMENT"), "{s}");
        assert!(s.contains("empieza en 100"), "{s}");
    }

    #[test]
    fn mysql_avisa_si_la_columna_autogenerada_no_es_clave() {
        let mut c = col("N", "int", false);
        c.identity = Some(Identity::simple());
        let a = Schema { tables: vec![table("C", vec![c])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::MySql);
        assert!(s.contains("exigen que la columna AUTO_INCREMENT sea clave"), "{s}");
    }

    #[test]
    fn sqlite_pega_autoincrement_a_la_clave_primaria() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "integer", Identity::simple())])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Sqlite);
        assert!(s.contains("\"Id\" integer PRIMARY KEY AUTOINCREMENT"), "{s}");
        // Y la clave primaria NO se declara dos veces: SQLite lo rechaza.
        assert_eq!(s.matches("PRIMARY KEY").count(), 1, "{s}");
        // Sin avisos: aquí SQLite sí puede con lo que pide el origen.
        assert!(!s.contains("NO se autogenerará"), "{s}");
    }

    #[test]
    fn sqlite_avisa_cuando_no_puede_autogenerar() {
        // Clave primaria compuesta: AUTOINCREMENT no cabe en ninguna parte.
        let mut primera = pk_col("Id", "integer", 0);
        primera.identity = Some(Identity::simple());
        let a = Schema {
            tables: vec![table("C", vec![primera, pk_col("Linea", "integer", 1)])],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::Sqlite);
        assert!(s.contains("NO se autogenerará"), "{s}");
        assert!(!s.contains("AUTOINCREMENT"), "{s}");
    }

    #[test]
    fn una_columna_que_se_autogenera_no_lleva_default() {
        let mut c = id_col("Id", "int", Identity::simple());
        c.default_value = Some("0".into());
        let a = Schema { tables: vec![table("C", vec![c])], ..Default::default() };
        let s = generate_sync_script(&a, &Schema::default(), Dialect::SqlServer);
        assert!(s.contains("IDENTITY(1,1)"), "{s}");
        assert!(!s.contains("DEFAULT"), "{s}");
    }

    #[test]
    fn agregar_una_columna_que_se_autogenera() {
        let mut nueva = col("Num", "int", false);
        nueva.identity = Some(Identity::simple());
        nueva.is_primary_key = true;
        let a = Schema { tables: vec![table("C", vec![nueva])], ..Default::default() };
        let b = Schema { tables: vec![table("C", vec![col("Otra", "int", true)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("ALTER TABLE [dbo].[C] ADD [Num] int IDENTITY(1,1) NOT NULL;"), "{s}");
    }

    #[test]
    fn sqlite_no_puede_agregar_autoincrement_con_alter() {
        let a = Schema {
            tables: vec![table("C", vec![id_col("Id", "integer", Identity::simple()), col("N", "integer", true)])],
            ..Default::default()
        };
        let b = Schema { tables: vec![table("C", vec![col("N", "integer", true)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::Sqlite);
        assert!(s.contains("SQLite no puede agregar una columna AUTOINCREMENT con ALTER"), "{s}");
    }

    #[test]
    fn postgres_pone_y_quita_la_identidad_con_alter() {
        let plano = table("C", vec![pk_col("Id", "integer", 0)]);
        let auto = table("C", vec![id_col("Id", "integer", Identity::simple())]);
        // B sin identidad y A con ella: hay que ponerla.
        let s = generate_sync_script(
            &Schema { tables: vec![auto.clone()], ..Default::default() },
            &Schema { tables: vec![plano.clone()], ..Default::default() },
            Dialect::Postgres,
        );
        assert!(s.contains("ALTER COLUMN \"Id\" ADD GENERATED BY DEFAULT AS IDENTITY;"), "{s}");
        // Y al revés: hay que quitarla.
        let s2 = generate_sync_script(
            &Schema { tables: vec![plano], ..Default::default() },
            &Schema { tables: vec![auto], ..Default::default() },
            Dialect::Postgres,
        );
        assert!(s2.contains("ALTER COLUMN \"Id\" DROP IDENTITY;"), "{s2}");
    }

    #[test]
    fn sql_server_avisa_que_no_puede_cambiar_identity_con_alter() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "int", Identity::simple())])], ..Default::default() };
        let b = Schema { tables: vec![table("C", vec![pk_col("Id", "int", 0)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("no puede poner ni quitar IDENTITY"), "{s}");
    }

    #[test]
    fn mysql_repite_el_auto_increment_al_modificar() {
        let a = Schema { tables: vec![table("C", vec![id_col("Id", "bigint", Identity::simple())])], ..Default::default() };
        let b = Schema { tables: vec![table("C", vec![pk_col("Id", "int", 0)])], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::MySql);
        assert!(s.contains("MODIFY COLUMN `Id` bigint NOT NULL AUTO_INCREMENT;"), "{s}");
    }

    #[test]
    fn postgres_no_reescribe_la_tabla_si_solo_cambia_la_identidad() {
        let plano = table("C", vec![pk_col("Id", "integer", 0)]);
        let auto = table("C", vec![id_col("Id", "integer", Identity::simple())]);
        let s = generate_sync_script(
            &Schema { tables: vec![auto], ..Default::default() },
            &Schema { tables: vec![plano], ..Default::default() },
            Dialect::Postgres,
        );
        assert!(s.contains("ADD GENERATED BY DEFAULT AS IDENTITY"), "{s}");
        // Un `ALTER COLUMN ... TYPE` con el mismo tipo reescribe la tabla entera
        // por nada: no debe salir.
        assert!(!s.contains("TYPE integer"), "{s}");
    }

    // ── Objetos que bautiza el motor ────────────────────────────────────────

    #[test]
    fn el_script_no_toca_un_unique_que_el_motor_nombro_distinto() {
        use crate::model::Index;
        let uq = |name: &str| Index {
            name: name.into(),
            system_named: true,
            table: "dbo.Producto".into(),
            columns: vec!["Codigo".into()],
            is_unique: true,
        };
        let a = Schema { indexes: vec![uq("UQ__Producto__AAAA1111")], ..Default::default() };
        let b = Schema { indexes: vec![uq("UQ__Producto__BBBB2222")], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        // Es la misma restricción: el script no tiene nada que hacer con ella.
        assert!(!s.contains("DROP INDEX"), "{s}");
        assert!(!s.contains("CREATE UNIQUE INDEX"), "{s}");
    }

    #[test]
    fn el_script_si_renombra_un_indice_con_nombre_propio() {
        use crate::model::Index;
        let ix = |name: &str| Index {
            name: name.into(),
            system_named: false,
            table: "dbo.Producto".into(),
            columns: vec!["Codigo".into()],
            is_unique: false,
        };
        let a = Schema { indexes: vec![ix("IX_Nuevo")], ..Default::default() };
        let b = Schema { indexes: vec![ix("IX_Viejo")], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("CREATE INDEX [IX_Nuevo]"), "{s}");
        assert!(s.contains("DROP INDEX [IX_Viejo]"), "{s}");
    }

    #[test]
    fn el_script_no_toca_una_foranea_que_el_motor_nombro_distinto() {
        use crate::model::ForeignKey;
        let fk = |name: &str| ForeignKey {
            name: name.into(),
            system_named: true,
            from_table: "dbo.Pedido".into(),
            from_columns: vec!["IdCliente".into()],
            to_table: "dbo.Cliente".into(),
            to_columns: vec!["Id".into()],
        };
        let a = Schema { foreign_keys: vec![fk("FK__Pedido__AAAA")], ..Default::default() };
        let b = Schema { foreign_keys: vec![fk("FK__Pedido__BBBB")], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(!s.contains("DROP CONSTRAINT"), "{s}");
        assert!(!s.contains("ADD CONSTRAINT"), "{s}");
    }

    // ── Refrescar módulos (sp_refreshsqlmodule) ─────────────────────────────

    /// Un módulo con sus dependencias declaradas, como las lee el catálogo.
    fn modulo(name: &str, cuerpo: &str, refs: &[&str]) -> Routine {
        Routine {
            schema: "dbo".into(),
            name: name.into(),
            kind: RoutineKind::View,
            definition: format!("CREATE VIEW dbo.{name} AS {cuerpo}"),
            references: refs.iter().map(|r| r.to_string()).collect(),
        }
    }

    /// Origen y destino que difieren SOLO en una columna de `Clientes`, con los
    /// módulos que se le pasen (iguales en los dos lados, para que el script no
    /// los recree).
    fn par_con_modulos(mods: Vec<Routine>) -> (Schema, Schema) {
        let a = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false), col("Nuevo", "int", true)])],
            views: mods.clone(),
            ..Default::default()
        };
        let b = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false)])],
            views: mods,
            ..Default::default()
        };
        (a, b)
    }

    #[test]
    fn refresca_el_modulo_que_depende_de_una_tabla_que_cambia() {
        let (a, b) = par_con_modulos(vec![modulo("VistaClientes", "SELECT * FROM Clientes", &["dbo.Clientes"])]);
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("EXECUTE sp_refreshsqlmodule N'[dbo].[VistaClientes]';"), "{s}");
    }

    #[test]
    fn no_refresca_un_modulo_que_no_depende_de_nada_cambiado() {
        let (a, b) = par_con_modulos(vec![modulo("VistaOtra", "SELECT 1", &["dbo.Productos"])]);
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(!s.contains("sp_refreshsqlmodule"), "{s}");
    }

    #[test]
    fn el_refresco_va_por_capas_de_dependencia() {
        // VistaB se monta sobre VistaA, y VistaA sobre la tabla que cambia: hay
        // que refrescar la de abajo primero.
        let (a, b) = par_con_modulos(vec![
            modulo("VistaB", "SELECT * FROM VistaA", &["dbo.VistaA"]),
            modulo("VistaA", "SELECT * FROM Clientes", &["dbo.Clientes"]),
        ]);
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        let pa = s.find("N'[dbo].[VistaA]'").expect("falta VistaA");
        let pb = s.find("N'[dbo].[VistaB]'").expect("falta VistaB");
        assert!(pa < pb, "VistaA tiene que refrescarse antes que VistaB:\n{s}");
    }

    #[test]
    fn no_refresca_un_modulo_que_el_script_vuelve_a_crear() {
        // La vista cambia de cuerpo, así que el script la recrea: nace compilada
        // y refrescarla sería ruido.
        let v = |cuerpo: &str| modulo("VistaClientes", cuerpo, &["dbo.Clientes"]);
        let a = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false), col("Nuevo", "int", true)])],
            views: vec![v("SELECT Id, Nuevo FROM Clientes")],
            ..Default::default()
        };
        let b = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false)])],
            views: vec![v("SELECT Id FROM Clientes")],
            ..Default::default()
        };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("CREATE VIEW"), "la vista tiene que recrearse:\n{s}");
        assert!(!s.contains("sp_refreshsqlmodule"), "{s}");
    }

    #[test]
    fn una_tabla_nueva_tambien_obliga_a_refrescar() {
        // Un módulo puede referenciar una tabla que todavía no existe en el
        // destino (SQL Server resuelve tarde): al crearla hay que refrescarlo.
        let m = modulo("VistaPedidos", "SELECT * FROM Pedidos", &["dbo.Pedidos"]);
        let a = Schema {
            tables: vec![table("Pedidos", vec![col("Id", "int", false)])],
            views: vec![m.clone()],
            ..Default::default()
        };
        let b = Schema { views: vec![m], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("EXECUTE sp_refreshsqlmodule N'[dbo].[VistaPedidos]';"), "{s}");
    }

    #[test]
    fn solo_sql_server_refresca_modulos() {
        let (a, b) = par_con_modulos(vec![modulo("VistaClientes", "SELECT * FROM Clientes", &["dbo.Clientes"])]);
        for d in [Dialect::Postgres, Dialect::MySql, Dialect::Sqlite] {
            let s = generate_sync_script(&a, &b, d);
            assert!(!s.contains("sp_refreshsqlmodule"), "{d:?} no debería refrescar:\n{s}");
        }
    }

    #[test]
    fn un_ciclo_de_referencias_no_cuelga_el_refresco() {
        // Dos módulos que se referencian entre sí (en SQL Server se puede con
        // procedimientos). El cierre tiene que terminar y nombrar a los dos.
        let (a, b) = par_con_modulos(vec![
            modulo("Uno", "SELECT 1", &["dbo.Clientes", "dbo.Dos"]),
            modulo("Dos", "SELECT 2", &["dbo.Uno"]),
        ]);
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("N'[dbo].[Uno]'"), "{s}");
        assert!(s.contains("N'[dbo].[Dos]'"), "{s}");
        assert_eq!(s.matches("sp_refreshsqlmodule").count(), 2, "{s}");
    }

    #[test]
    fn el_refresco_va_despues_de_los_cambios_de_la_tabla() {
        // Refresca contra el estado final, así que tiene que ir después del ALTER.
        let (a, b) = par_con_modulos(vec![modulo("VistaClientes", "SELECT * FROM Clientes", &["dbo.Clientes"])]);
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        let alter = s.find("ALTER TABLE [dbo].[Clientes] ADD").expect("falta el ALTER");
        let refresco = s.find("sp_refreshsqlmodule").expect("falta el refresco");
        assert!(alter < refresco, "{s}");
    }
}
