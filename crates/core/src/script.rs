//! Generación del "script de sincronización": el SQL que deja el DESTINO (B)
//! igual que el ORIGEN (A). Es la función estrella de SSDT Schema Compare.
//!
//! El script es un BORRADOR para revisar: DataDiff nunca ejecuta ni modifica las
//! bases, solo genera el texto para que tú lo revises y lo corras.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::model::{Column, Routine, RoutineKind, Schema, Table};

/// Dialecto SQL del destino (determina el entrecomillado y la sintaxis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    SqlServer,
    Sqlite,
    Postgres,
}

impl Dialect {
    /// Entrecomilla un identificador según el motor: `[x]` en SQL Server, `"x"` en SQLite.
    fn quote(&self, ident: &str) -> String {
        match self {
            Dialect::SqlServer => format!("[{}]", ident.replace(']', "]]")),
            Dialect::Sqlite | Dialect::Postgres => format!("\"{}\"", ident.replace('"', "\"\"")),
        }
    }
    /// Nombre calificado entrecomillado, p.ej. `[dbo].[Clientes]`.
    fn qname(&self, schema: &str, name: &str) -> String {
        format!("{}.{}", self.quote(schema), self.quote(name))
    }
    fn label(&self) -> &'static str {
        match self {
            Dialect::SqlServer => "SQL Server",
            Dialect::Sqlite => "SQLite",
            Dialect::Postgres => "PostgreSQL",
        }
    }
}

fn same_column(a: &Column, b: &Column) -> bool {
    a.data_type == b.data_type && a.is_nullable == b.is_nullable && a.default_value == b.default_value
}

fn null_kw(c: &Column) -> &'static str {
    if c.is_nullable {
        "NULL"
    } else {
        "NOT NULL"
    }
}

/// Definición de una columna: `[nombre] tipo NULL/NOT NULL [DEFAULT x]`.
fn column_ddl(d: Dialect, c: &Column) -> String {
    let mut s = format!("{} {} {}", d.quote(&c.name), c.data_type, null_kw(c));
    if let Some(def) = &c.default_value {
        let _ = write!(s, " DEFAULT {def}");
    }
    s
}

fn sorted_columns(t: &Table) -> Vec<&Column> {
    let mut cols: Vec<&Column> = t.columns.iter().collect();
    cols.sort_by_key(|c| c.ordinal);
    cols
}

fn create_table(d: Dialect, t: &Table) -> String {
    let body = sorted_columns(t)
        .iter()
        .map(|c| format!("    {}", column_ddl(d, c)))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("CREATE TABLE {} (\n{}\n);", d.qname(&t.schema, &t.name), body)
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
        Dialect::Sqlite | Dialect::Postgres => format!("DROP {kw} IF EXISTS {};", d.qname(&r.schema, &r.name)),
    }
}

fn create_routine(d: Dialect, r: &Routine) -> String {
    // La definición almacenada ya es el CREATE completo del objeto.
    let body = r.definition.trim();
    match d {
        Dialect::SqlServer => format!("{body}\nGO"),
        Dialect::Sqlite | Dialect::Postgres => format!("{};", body.trim_end_matches(';')),
    }
}

fn normalize(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Emite los ALTER para que la tabla del destino quede como la del origen.
fn alter_table(d: Dialect, src: &Table, tgt: &Table, out: &mut String, changes: &mut usize) {
    let qn = d.qname(&src.schema, &src.name);
    let tgt_cols: BTreeMap<&str, &Column> =
        tgt.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let src_names: BTreeSet<&str> = src.columns.iter().map(|c| c.name.as_str()).collect();

    // Columnas a agregar o modificar (recorremos las de A en orden).
    for c in sorted_columns(src) {
        match tgt_cols.get(c.name.as_str()) {
            None => {
                match d {
                    Dialect::SqlServer => {
                        let _ = writeln!(out, "ALTER TABLE {qn} ADD {};", column_ddl(d, c));
                    }
                    Dialect::Sqlite | Dialect::Postgres => {
                        let _ = writeln!(out, "ALTER TABLE {qn} ADD COLUMN {};", column_ddl(d, c));
                    }
                }
                *changes += 1;
            }
            Some(t) if !same_column(c, t) => {
                match d {
                    Dialect::SqlServer => {
                        let _ = writeln!(
                            out,
                            "ALTER TABLE {qn} ALTER COLUMN {} {} {};",
                            d.quote(&c.name),
                            c.data_type,
                            null_kw(c)
                        );
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
                    }
                    Dialect::Postgres => {
                        let _ = writeln!(out, "ALTER TABLE {qn} ALTER COLUMN {} TYPE {};", d.quote(&c.name), c.data_type);
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
            *changes += 1;
        }
    }
}

/// Emite CREATE/DROP para una categoría de rutinas (vistas, procs o funciones).
fn routine_section(
    d: Dialect,
    source: &[Routine],
    target: &[Routine],
    out: &mut String,
    changes: &mut usize,
) {
    let tgt_by_name: BTreeMap<String, &Routine> =
        target.iter().map(|r| (r.qualified_name(), r)).collect();
    let src_names: BTreeSet<String> = source.iter().map(|r| r.qualified_name()).collect();

    // Crear (nuevas en A) o recrear (distintas): DROP + CREATE.
    for r in source {
        match tgt_by_name.get(&r.qualified_name()) {
            None => {
                let _ = writeln!(out, "{}\n", create_routine(d, r));
                *changes += 1;
            }
            Some(t) if normalize(&r.definition) != normalize(&t.definition) => {
                let _ = writeln!(out, "{}", drop_routine(d, t));
                let _ = writeln!(out, "{}\n", create_routine(d, r));
                *changes += 1;
            }
            _ => {}
        }
    }

    // Eliminar (están en B pero no en A).
    for t in target {
        if !src_names.contains(&t.qualified_name()) {
            let _ = writeln!(out, "{}\n", drop_routine(d, t));
            *changes += 1;
        }
    }
}

/// Genera el script SQL que transforma el esquema `target` (B) en `source` (A).
pub fn generate_sync_script(source: &Schema, target: &Schema, d: Dialect) -> String {
    let mut body = String::new();
    let mut changes = 0usize;

    // ── Tablas ──
    let tgt_tables: BTreeMap<String, &Table> =
        target.tables.iter().map(|t| (t.qualified_name(), t)).collect();
    let src_tnames: BTreeSet<String> = source.tables.iter().map(|t| t.qualified_name()).collect();

    let mut creates = String::new();
    for t in &source.tables {
        if !tgt_tables.contains_key(&t.qualified_name()) {
            let _ = writeln!(creates, "{}\n", create_table(d, t));
            changes += 1;
        }
    }
    if !creates.is_empty() {
        let _ = write!(body, "-- ─── Tablas nuevas ───\n{creates}\n");
    }

    let mut alters = String::new();
    for src in &source.tables {
        if let Some(tgt) = tgt_tables.get(&src.qualified_name()) {
            alter_table(d, src, tgt, &mut alters, &mut changes);
        }
    }
    if !alters.is_empty() {
        let _ = write!(body, "-- ─── Cambios en columnas ───\n{alters}\n");
    }

    // ── Vistas / Procedimientos / Funciones ──
    let mut routines = String::new();
    routine_section(d, &source.views, &target.views, &mut routines, &mut changes);
    routine_section(d, &source.procedures, &target.procedures, &mut routines, &mut changes);
    routine_section(d, &source.functions, &target.functions, &mut routines, &mut changes);
    if !routines.is_empty() {
        let _ = write!(body, "-- ─── Vistas / Procedimientos / Funciones ───\n{routines}\n");
    }

    // ── Tablas a eliminar en el destino ──
    let mut drops = String::new();
    for t in &target.tables {
        if !src_tnames.contains(&t.qualified_name()) {
            let _ = writeln!(drops, "DROP TABLE IF EXISTS {};", d.qname(&t.schema, &t.name));
            changes += 1;
        }
    }
    if !drops.is_empty() {
        let _ = write!(body, "-- ─── Tablas eliminadas en el destino ───\n{drops}\n");
    }

    // ── Encabezado + salida ──
    let mut out = String::new();
    let _ = writeln!(out, "-- ============================================================");
    let _ = writeln!(out, "-- Script de sincronización generado por DataDiff");
    let _ = writeln!(out, "-- Deja el DESTINO (B) igual que el ORIGEN (A)");
    let _ = writeln!(out, "-- Motor destino: {}", d.label());
    let _ = writeln!(out, "-- (!) Revísalo y respalda el destino antes de ejecutarlo.");
    let _ = writeln!(out, "-- ============================================================\n");

    if changes == 0 {
        let _ = writeln!(out, "-- Sin diferencias: el destino ya coincide con el origen.");
    } else {
        out.push_str(&body);
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
        }
    }
    fn table(name: &str, cols: Vec<Column>) -> Table {
        Table { schema: "dbo".into(), name: name.into(), columns: cols }
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
        let a = Schema { procedures: vec![Routine { schema: "dbo".into(), name: "sp".into(), kind: RoutineKind::Procedure, definition: "CREATE PROCEDURE dbo.sp AS SELECT 1".into() }], ..Default::default() };
        let b = Schema { procedures: vec![Routine { schema: "dbo".into(), name: "sp".into(), kind: RoutineKind::Procedure, definition: "CREATE PROCEDURE dbo.sp AS SELECT 2".into() }], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("DROP PROCEDURE IF EXISTS [dbo].[sp]"), "{s}");
        assert!(s.contains("SELECT 1"), "{s}");
    }

    #[test]
    fn sin_cambios_lo_dice() {
        let t = table("C", vec![col("Id", "int", false)]);
        let a = Schema { tables: vec![t.clone()], ..Default::default() };
        let b = Schema { tables: vec![t], ..Default::default() };
        let s = generate_sync_script(&a, &b, Dialect::SqlServer);
        assert!(s.contains("Sin diferencias"), "{s}");
    }
}
