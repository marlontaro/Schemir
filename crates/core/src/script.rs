//! Generación del "script de sincronización": el SQL que deja el DESTINO (B)
//! igual que el ORIGEN (A). Es la función estrella de SSDT Schema Compare.
//!
//! El script es un BORRADOR para revisar: DataDiff nunca ejecuta ni modifica las
//! bases, solo genera el texto para que tú lo revises y lo corras.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::diff::{columns_equal, normalize_def, CompareOptions, DiffStatus};
use crate::model::{CheckConstraint, Column, ForeignKey, Index, Routine, RoutineKind, Schema, Table};

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

/// Emite los ALTER para que la tabla del destino quede como la del origen.
fn alter_table(d: Dialect, src: &Table, tgt: &Table, o: &CompareOptions, out: &mut String, changes: &mut usize) {
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
            Some(t) if !columns_equal(c, t, o) => {
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
        Dialect::SqlServer => format!("DROP INDEX {} ON {};", d.quote(&idx.name), d.qname(schema, table)),
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
    format!("ALTER TABLE {} DROP CONSTRAINT {};", d.qname(fs, ft), d.quote(&fk.name))
}

fn add_check(d: Dialect, chk: &CheckConstraint) -> String {
    let (s, t) = split_qualified(&chk.table);
    // En SQL Server la definición es solo la expresión; en Postgres ya trae "CHECK (...)".
    let body = match d {
        Dialect::SqlServer => format!("CHECK {}", chk.definition),
        _ => chk.definition.clone(),
    };
    format!("ALTER TABLE {} ADD CONSTRAINT {} {};", d.qname(s, t), d.quote(&chk.name), body)
}

fn drop_check(d: Dialect, chk: &CheckConstraint) -> String {
    let (s, t) = split_qualified(&chk.table);
    format!("ALTER TABLE {} DROP CONSTRAINT {};", d.qname(s, t), d.quote(&chk.name))
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

    if o.tables {
        // ── Tablas nuevas ──
        for t in &source.tables {
            if !tgt_tables.contains_key(&t.qualified_name()) {
                units.push(ScriptUnit::new("Tabla", t.qualified_name(), OnlyInLeft, "Tablas nuevas", create_table(d, t)));
            }
        }

        // ── Cambios en columnas (tablas presentes en ambos) ──
        for src in &source.tables {
            if let Some(tgt) = tgt_tables.get(&src.qualified_name()) {
                let mut sql = String::new();
                let mut changes = 0usize;
                alter_table(d, src, tgt, o, &mut sql, &mut changes);
                if changes > 0 {
                    units.push(ScriptUnit::new("Tabla", src.qualified_name(), Different, "Cambios en columnas", sql));
                }
            }
        }
    }

    // ── Índices ──
    if o.indexes {
        let src: BTreeMap<String, &Index> = source.indexes.iter().map(|i| (index_key(i), i)).collect();
        let tgt: BTreeMap<String, &Index> = target.indexes.iter().map(|i| (index_key(i), i)).collect();
        for (k, i) in &src {
            match tgt.get(k) {
                None => units.push(ScriptUnit::new("Índice", k.clone(), OnlyInLeft, "Índices", create_index(d, i))),
                Some(ti) if !same_index(i, ti) => units.push(ScriptUnit::new("Índice", k.clone(), Different, "Índices", format!("{}\n{}", drop_index(d, ti), create_index(d, i)))),
                _ => {}
            }
        }
        for (k, ti) in &tgt {
            if !src.contains_key(k) {
                units.push(ScriptUnit::new("Índice", k.clone(), OnlyInRight, "Índices", drop_index(d, ti)));
            }
        }
    }

    // ── Claves foráneas ──
    if o.foreign_keys {
        let src: BTreeMap<String, &ForeignKey> = source.foreign_keys.iter().map(|f| (f.name.clone(), f)).collect();
        let tgt: BTreeMap<String, &ForeignKey> = target.foreign_keys.iter().map(|f| (f.name.clone(), f)).collect();
        for (k, f) in &src {
            match tgt.get(k) {
                None => units.push(ScriptUnit::new("Clave foránea", k.clone(), OnlyInLeft, "Claves foráneas", add_fk(d, f))),
                Some(tf) if !same_fk(f, tf) => units.push(ScriptUnit::new("Clave foránea", k.clone(), Different, "Claves foráneas", format!("{}\n{}", drop_fk(d, tf), add_fk(d, f)))),
                _ => {}
            }
        }
        for (k, tf) in &tgt {
            if !src.contains_key(k) {
                units.push(ScriptUnit::new("Clave foránea", k.clone(), OnlyInRight, "Claves foráneas", drop_fk(d, tf)));
            }
        }
    }

    // ── Restricciones CHECK ──
    if o.checks {
        let src: BTreeMap<String, &CheckConstraint> = source.checks.iter().map(|c| (c.name.clone(), c)).collect();
        let tgt: BTreeMap<String, &CheckConstraint> = target.checks.iter().map(|c| (c.name.clone(), c)).collect();
        for (k, c) in &src {
            match tgt.get(k) {
                None => units.push(ScriptUnit::new("Restricción", k.clone(), OnlyInLeft, "Restricciones CHECK", add_check(d, c))),
                Some(tc) if normalize_def(&c.definition, o) != normalize_def(&tc.definition, o) => units.push(ScriptUnit::new("Restricción", k.clone(), Different, "Restricciones CHECK", format!("{}\n{}", drop_check(d, tc), add_check(d, c)))),
                _ => {}
            }
        }
        for (k, tc) in &tgt {
            if !src.contains_key(k) {
                units.push(ScriptUnit::new("Restricción", k.clone(), OnlyInRight, "Restricciones CHECK", drop_check(d, tc)));
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

    // ── Tablas eliminadas en el destino ──
    if o.tables {
        for t in &target.tables {
            if !src_tnames.contains(&t.qualified_name()) {
                units.push(ScriptUnit::new("Tabla", t.qualified_name(), OnlyInRight, "Tablas eliminadas en el destino", format!("DROP TABLE IF EXISTS {};", d.qname(&t.schema, &t.name))));
            }
        }
    }

    units
}

/// Encabezado común del script (comentarios de advertencia).
fn script_header(d: Dialect) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "-- ============================================================");
    let _ = writeln!(out, "-- Script de sincronización generado por DataDiff");
    let _ = writeln!(out, "-- Deja el DESTINO (B) igual que el ORIGEN (A)");
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
    fn script_crea_indice_faltante() {
        use crate::model::Index;
        let a = Schema {
            indexes: vec![Index { name: "ix1".into(), table: "dbo.C".into(), columns: vec!["a".into(), "b".into()], is_unique: true }],
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
            indexes: vec![Index { name: "ix1".into(), table: "public.c".into(), columns: vec!["a".into()], is_unique: false }],
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
            indexes: vec![Index { name: "ix1".into(), table: "dbo.Clientes".into(), columns: vec!["Id".into()], is_unique: false }],
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
}
