//! Motor de comparación: dado un esquema "izquierdo" (origen) y uno "derecho"
//! (destino), reporta qué objetos sobran, cuáles faltan y cuáles difieren.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{CheckConstraint, Column, ForeignKey, Index, Routine, Schema, Table};

/// Estado de un objeto al comparar los dos esquemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatus {
    /// Existe solo en el esquema izquierdo (origen).
    OnlyInLeft,
    /// Existe solo en el esquema derecho (destino).
    OnlyInRight,
    /// Existe en ambos pero con diferencias.
    Different,
    /// Existe en ambos y es idéntico.
    Equal,
}

/// Diferencia puntual dentro de un objeto (una columna, o el cuerpo de un SP).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    /// Qué cambió, p.ej. "columna: Email" o "definición".
    pub field: String,
    /// Valor en el esquema izquierdo (`None` = no existe de ese lado).
    pub left: Option<String>,
    /// Valor en el esquema derecho.
    pub right: Option<String>,
}

/// Resultado de comparar un objeto entre ambos esquemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDiff {
    /// Tipo legible: "Tabla", "Vista", "Procedimiento", "Función".
    pub object_type: String,
    /// Nombre calificado, p.ej. "dbo.Clientes".
    pub name: String,
    pub status: DiffStatus,
    /// Detalle de las diferencias (vacío salvo cuando `status == Different`).
    pub details: Vec<FieldDiff>,
}

/// Resultado completo de comparar dos esquemas.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub objects: Vec<ObjectDiff>,
}

impl ComparisonResult {
    /// Conteo por estado, para el resumen de la interfaz.
    pub fn summary(&self) -> Summary {
        let mut s = Summary::default();
        for o in &self.objects {
            match o.status {
                DiffStatus::OnlyInLeft => s.only_in_left += 1,
                DiffStatus::OnlyInRight => s.only_in_right += 1,
                DiffStatus::Different => s.different += 1,
                DiffStatus::Equal => s.equal += 1,
            }
        }
        s
    }
}

/// Conteo de objetos por estado.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub only_in_left: usize,
    pub only_in_right: usize,
    pub different: usize,
    pub equal: usize,
}

/// Opciones de comparación (al estilo de SSDT): qué ignorar y qué tipos comparar.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct CompareOptions {
    /// Ignorar mayúsculas/minúsculas en definiciones y tipos de datos.
    pub ignore_case: bool,
    /// Colapsar los espacios en blanco de las definiciones.
    pub ignore_whitespace: bool,
    /// Quitar los comentarios SQL de las definiciones antes de comparar.
    pub ignore_comments: bool,
    /// Tratar las columnas como iguales aunque cambie su valor por defecto.
    pub ignore_column_default: bool,
    // Qué tipos de objeto comparar.
    pub tables: bool,
    pub views: bool,
    pub procedures: bool,
    pub functions: bool,
    pub indexes: bool,
    pub foreign_keys: bool,
    pub checks: bool,
    pub primary_keys: bool,
}

impl Default for CompareOptions {
    fn default() -> Self {
        CompareOptions {
            ignore_case: true,
            ignore_whitespace: true,
            ignore_comments: false,
            ignore_column_default: false,
            tables: true,
            views: true,
            procedures: true,
            functions: true,
            indexes: true,
            foreign_keys: true,
            checks: true,
            primary_keys: true,
        }
    }
}

/// Compara dos esquemas completos y devuelve todas las diferencias de objetos.
pub fn compare_schemas(left: &Schema, right: &Schema) -> ComparisonResult {
    compare_schemas_opts(left, right, &CompareOptions::default())
}

/// Igual que `compare_schemas`, pero con opciones (qué ignorar y qué comparar).
pub fn compare_schemas_opts(left: &Schema, right: &Schema, o: &CompareOptions) -> ComparisonResult {
    let mut objects = Vec::new();

    if o.tables {
        diff_tables(&left.tables, &right.tables, o, &mut objects);
    }
    if o.views {
        diff_routines("Vista", &left.views, &right.views, o, &mut objects);
    }
    if o.procedures {
        diff_routines("Procedimiento", &left.procedures, &right.procedures, o, &mut objects);
    }
    if o.functions {
        diff_routines("Función", &left.functions, &right.functions, o, &mut objects);
    }
    if o.primary_keys {
        diff_primary_keys(&left.tables, &right.tables, &mut objects);
    }
    if o.indexes {
        diff_indexes(&left.indexes, &right.indexes, &mut objects);
    }
    if o.foreign_keys {
        diff_foreign_keys(&left.foreign_keys, &right.foreign_keys, &mut objects);
    }
    if o.checks {
        diff_checks(&left.checks, &right.checks, o, &mut objects);
    }

    // Orden estable y predecible: primero por tipo, luego por nombre.
    objects.sort_by(|a, b| a.object_type.cmp(&b.object_type).then(a.name.cmp(&b.name)));
    ComparisonResult { objects }
}

/// Devuelve los nombres presentes en cualquiera de los dos lados, ordenados y sin repetir.
fn union_keys<'a, T>(
    left: &'a BTreeMap<String, T>,
    right: &'a BTreeMap<String, T>,
) -> Vec<String> {
    let mut names: Vec<String> = left.keys().chain(right.keys()).cloned().collect();
    names.sort();
    names.dedup();
    names
}

fn diff_tables(left: &[Table], right: &[Table], o: &CompareOptions, out: &mut Vec<ObjectDiff>) {
    let left_map: BTreeMap<String, &Table> =
        left.iter().map(|t| (t.qualified_name(), t)).collect();
    let right_map: BTreeMap<String, &Table> =
        right.iter().map(|t| (t.qualified_name(), t)).collect();

    for name in union_keys(&left_map, &right_map) {
        let diff = match (left_map.get(&name), right_map.get(&name)) {
            (Some(_), None) => ObjectDiff {
                object_type: "Tabla".into(),
                name,
                status: DiffStatus::OnlyInLeft,
                details: vec![],
            },
            (None, Some(_)) => ObjectDiff {
                object_type: "Tabla".into(),
                name,
                status: DiffStatus::OnlyInRight,
                details: vec![],
            },
            (Some(l), Some(r)) => {
                let details = diff_columns(&l.columns, &r.columns, o);
                let status = if details.is_empty() {
                    DiffStatus::Equal
                } else {
                    DiffStatus::Different
                };
                ObjectDiff { object_type: "Tabla".into(), name, status, details }
            }
            (None, None) => unreachable!("una clave de la unión siempre existe en algún lado"),
        };
        out.push(diff);
    }
}

fn diff_columns(left: &[Column], right: &[Column], o: &CompareOptions) -> Vec<FieldDiff> {
    let left_map: BTreeMap<String, &Column> =
        left.iter().map(|c| (c.name.clone(), c)).collect();
    let right_map: BTreeMap<String, &Column> =
        right.iter().map(|c| (c.name.clone(), c)).collect();

    let mut diffs = Vec::new();
    for name in union_keys(&left_map, &right_map) {
        match (left_map.get(&name), right_map.get(&name)) {
            (Some(l), None) => diffs.push(FieldDiff {
                field: format!("columna: {name}"),
                left: Some(describe_column(l)),
                right: None,
            }),
            (None, Some(r)) => diffs.push(FieldDiff {
                field: format!("columna: {name}"),
                left: None,
                right: Some(describe_column(r)),
            }),
            (Some(l), Some(r)) if !columns_equal(l, r, o) => diffs.push(FieldDiff {
                field: format!("columna: {name}"),
                left: Some(describe_column(l)),
                right: Some(describe_column(r)),
            }),
            _ => {} // igual: no se reporta
        }
    }
    diffs
}

/// Dos columnas son "iguales" si coinciden tipo, nulabilidad y valor por defecto.
/// La posición (`ordinal`) no se compara para no generar ruido por reordenamientos.
/// Las opciones permiten ignorar may/min del tipo y el valor por defecto.
pub(crate) fn columns_equal(a: &Column, b: &Column, o: &CompareOptions) -> bool {
    let type_eq = if o.ignore_case {
        a.data_type.eq_ignore_ascii_case(&b.data_type)
    } else {
        a.data_type == b.data_type
    };
    let default_eq = o.ignore_column_default || a.default_value == b.default_value;
    type_eq && a.is_nullable == b.is_nullable && default_eq
}

fn describe_column(c: &Column) -> String {
    let null = if c.is_nullable { "NULL" } else { "NOT NULL" };
    match &c.default_value {
        Some(d) => format!("{} {} DEFAULT {}", c.data_type, null, d),
        None => format!("{} {}", c.data_type, null),
    }
}

fn diff_routines(type_label: &str, left: &[Routine], right: &[Routine], o: &CompareOptions, out: &mut Vec<ObjectDiff>) {
    let left_map: BTreeMap<String, &Routine> =
        left.iter().map(|r| (r.qualified_name(), r)).collect();
    let right_map: BTreeMap<String, &Routine> =
        right.iter().map(|r| (r.qualified_name(), r)).collect();

    for name in union_keys(&left_map, &right_map) {
        let diff = match (left_map.get(&name), right_map.get(&name)) {
            (Some(_), None) => ObjectDiff {
                object_type: type_label.into(),
                name,
                status: DiffStatus::OnlyInLeft,
                details: vec![],
            },
            (None, Some(_)) => ObjectDiff {
                object_type: type_label.into(),
                name,
                status: DiffStatus::OnlyInRight,
                details: vec![],
            },
            (Some(l), Some(r)) => {
                let same = normalize_def(&l.definition, o) == normalize_def(&r.definition, o);
                let details = if same {
                    vec![]
                } else {
                    vec![FieldDiff {
                        field: "definición".into(),
                        left: Some(l.definition.clone()),
                        right: Some(r.definition.clone()),
                    }]
                };
                let status = if same { DiffStatus::Equal } else { DiffStatus::Different };
                ObjectDiff { object_type: type_label.into(), name, status, details }
            }
            (None, None) => unreachable!(),
        };
        out.push(diff);
    }
}

fn yesno(b: bool) -> String {
    if b {
        "sí".to_string()
    } else {
        "no".to_string()
    }
}

/// Compara índices por nombre calificado (tabla + nombre del índice).
fn diff_indexes(left: &[Index], right: &[Index], out: &mut Vec<ObjectDiff>) {
    let key = |i: &Index| format!("{}.{}", i.table, i.name);
    let lm: BTreeMap<String, &Index> = left.iter().map(|i| (key(i), i)).collect();
    let rm: BTreeMap<String, &Index> = right.iter().map(|i| (key(i), i)).collect();
    for name in union_keys(&lm, &rm) {
        let diff = match (lm.get(&name), rm.get(&name)) {
            (Some(_), None) => ObjectDiff { object_type: "Índice".into(), name, status: DiffStatus::OnlyInLeft, details: vec![] },
            (None, Some(_)) => ObjectDiff { object_type: "Índice".into(), name, status: DiffStatus::OnlyInRight, details: vec![] },
            (Some(l), Some(r)) => {
                let mut details = vec![];
                if l.columns != r.columns {
                    details.push(FieldDiff { field: "columnas".into(), left: Some(l.columns.join(", ")), right: Some(r.columns.join(", ")) });
                }
                if l.is_unique != r.is_unique {
                    details.push(FieldDiff { field: "único".into(), left: Some(yesno(l.is_unique)), right: Some(yesno(r.is_unique)) });
                }
                let status = if details.is_empty() { DiffStatus::Equal } else { DiffStatus::Different };
                ObjectDiff { object_type: "Índice".into(), name, status, details }
            }
            (None, None) => unreachable!(),
        };
        out.push(diff);
    }
}

fn fk_sig(f: &ForeignKey) -> String {
    format!("{}({}) -> {}({})", f.from_table, f.from_columns.join(","), f.to_table, f.to_columns.join(","))
}

/// Compara claves foráneas por nombre.
fn diff_foreign_keys(left: &[ForeignKey], right: &[ForeignKey], out: &mut Vec<ObjectDiff>) {
    let lm: BTreeMap<String, &ForeignKey> = left.iter().map(|f| (f.name.clone(), f)).collect();
    let rm: BTreeMap<String, &ForeignKey> = right.iter().map(|f| (f.name.clone(), f)).collect();
    for name in union_keys(&lm, &rm) {
        let diff = match (lm.get(&name), rm.get(&name)) {
            (Some(_), None) => ObjectDiff { object_type: "Clave foránea".into(), name, status: DiffStatus::OnlyInLeft, details: vec![] },
            (None, Some(_)) => ObjectDiff { object_type: "Clave foránea".into(), name, status: DiffStatus::OnlyInRight, details: vec![] },
            (Some(l), Some(r)) => {
                let (ls, rs) = (fk_sig(l), fk_sig(r));
                let details = if ls != rs { vec![FieldDiff { field: "definición".into(), left: Some(ls), right: Some(rs) }] } else { vec![] };
                let status = if details.is_empty() { DiffStatus::Equal } else { DiffStatus::Different };
                ObjectDiff { object_type: "Clave foránea".into(), name, status, details }
            }
            (None, None) => unreachable!(),
        };
        out.push(diff);
    }
}

/// Compara restricciones CHECK por nombre.
fn diff_checks(left: &[CheckConstraint], right: &[CheckConstraint], o: &CompareOptions, out: &mut Vec<ObjectDiff>) {
    let lm: BTreeMap<String, &CheckConstraint> = left.iter().map(|c| (c.name.clone(), c)).collect();
    let rm: BTreeMap<String, &CheckConstraint> = right.iter().map(|c| (c.name.clone(), c)).collect();
    for name in union_keys(&lm, &rm) {
        let diff = match (lm.get(&name), rm.get(&name)) {
            (Some(_), None) => ObjectDiff { object_type: "Restricción".into(), name, status: DiffStatus::OnlyInLeft, details: vec![] },
            (None, Some(_)) => ObjectDiff { object_type: "Restricción".into(), name, status: DiffStatus::OnlyInRight, details: vec![] },
            (Some(l), Some(r)) => {
                let same = normalize_def(&l.definition, o) == normalize_def(&r.definition, o);
                let details = if same { vec![] } else { vec![FieldDiff { field: "definición".into(), left: Some(l.definition.clone()), right: Some(r.definition.clone()) }] };
                let status = if same { DiffStatus::Equal } else { DiffStatus::Different };
                ObjectDiff { object_type: "Restricción".into(), name, status, details }
            }
            (None, None) => unreachable!(),
        };
        out.push(diff);
    }
}

fn pk_columns(t: &Table) -> Vec<String> {
    let mut v: Vec<(i32, &str)> = t.columns.iter().filter(|c| c.is_primary_key).map(|c| (c.ordinal, c.name.as_str())).collect();
    v.sort();
    v.into_iter().map(|(_, n)| n.to_string()).collect()
}

/// Compara la clave primaria de cada tabla presente en ambos esquemas.
fn diff_primary_keys(left: &[Table], right: &[Table], out: &mut Vec<ObjectDiff>) {
    let lm: BTreeMap<String, &Table> = left.iter().map(|t| (t.qualified_name(), t)).collect();
    let rm: BTreeMap<String, &Table> = right.iter().map(|t| (t.qualified_name(), t)).collect();
    for name in union_keys(&lm, &rm) {
        if let (Some(l), Some(r)) = (lm.get(&name), rm.get(&name)) {
            let (lpk, rpk) = (pk_columns(l), pk_columns(r));
            if lpk.is_empty() && rpk.is_empty() {
                continue;
            }
            let same = lpk == rpk;
            let details = if same {
                vec![]
            } else {
                vec![FieldDiff {
                    field: "columnas PK".into(),
                    left: Some(if lpk.is_empty() { "(sin PK)".into() } else { lpk.join(", ") }),
                    right: Some(if rpk.is_empty() { "(sin PK)".into() } else { rpk.join(", ") }),
                }]
            };
            let status = if same { DiffStatus::Equal } else { DiffStatus::Different };
            out.push(ObjectDiff { object_type: "Clave primaria".into(), name: format!("PK · {name}"), status, details });
        }
    }
}

/// Normaliza una definición SQL para compararla según las opciones activas
/// (quitar comentarios, colapsar espacios e ignorar may/min).
pub(crate) fn normalize_def(sql: &str, o: &CompareOptions) -> String {
    let mut s = if o.ignore_comments {
        strip_sql_comments(sql)
    } else {
        sql.to_string()
    };
    s = if o.ignore_whitespace {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        s.trim().to_string()
    };
    if o.ignore_case {
        s = s.to_lowercase();
    }
    s
}

/// Quita los comentarios SQL (`-- …` de línea y `/* … */` de bloque), respetando
/// las cadenas entre comillas simples para no borrar texto dentro de ellas.
fn strip_sql_comments(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut in_str = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if c == '\'' {
                in_str = false;
            }
        } else if c == '\'' {
            in_str = true;
            out.push(c);
        } else if c == '-' && chars.peek() == Some(&'-') {
            // Comentario de línea: salta hasta el fin de línea (conserva el salto).
            for n in chars.by_ref() {
                if n == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            // Comentario de bloque: salta hasta "*/".
            chars.next();
            let mut prev_star = false;
            for n in chars.by_ref() {
                if prev_star && n == '/' {
                    break;
                }
                prev_star = n == '*';
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RoutineKind, Schema, Table};

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

    fn table(name: &str, columns: Vec<Column>) -> Table {
        Table { schema: "dbo".into(), name: name.into(), columns }
    }

    fn proc(name: &str, body: &str) -> Routine {
        Routine {
            schema: "dbo".into(),
            name: name.into(),
            kind: RoutineKind::Procedure,
            definition: body.into(),
        }
    }

    #[test]
    fn tabla_solo_en_izquierda() {
        let left = Schema { tables: vec![table("Clientes", vec![col("Id", "int", false)])], ..Default::default() };
        let right = Schema::default();
        let r = compare_schemas(&left, &right);
        assert_eq!(r.objects.len(), 1);
        assert_eq!(r.objects[0].status, DiffStatus::OnlyInLeft);
        assert_eq!(r.objects[0].name, "dbo.Clientes");
    }

    #[test]
    fn columna_con_tipo_distinto() {
        let left = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false), col("Email", "varchar(50)", true)])],
            ..Default::default()
        };
        let right = Schema {
            tables: vec![table("Clientes", vec![col("Id", "int", false), col("Email", "varchar(100)", true)])],
            ..Default::default()
        };
        let r = compare_schemas(&left, &right);
        assert_eq!(r.objects[0].status, DiffStatus::Different);
        assert_eq!(r.objects[0].details.len(), 1);
        assert!(r.objects[0].details[0].field.contains("Email"));
    }

    #[test]
    fn tablas_identicas_son_equal() {
        let t = table("Clientes", vec![col("Id", "int", false)]);
        let left = Schema { tables: vec![t.clone()], ..Default::default() };
        let right = Schema { tables: vec![t], ..Default::default() };
        let r = compare_schemas(&left, &right);
        assert_eq!(r.objects[0].status, DiffStatus::Equal);
    }

    #[test]
    fn procedimiento_ignora_espacios_y_mayusculas() {
        let left = Schema { procedures: vec![proc("sp_x", "SELECT   1")], ..Default::default() };
        let right = Schema { procedures: vec![proc("sp_x", "select 1")], ..Default::default() };
        let r = compare_schemas(&left, &right);
        assert_eq!(r.objects[0].status, DiffStatus::Equal);
    }

    #[test]
    fn resumen_cuenta_por_estado() {
        let left = Schema {
            tables: vec![table("A", vec![]), table("B", vec![])],
            ..Default::default()
        };
        let right = Schema {
            tables: vec![table("B", vec![]), table("C", vec![])],
            ..Default::default()
        };
        let s = compare_schemas(&left, &right).summary();
        assert_eq!(s.only_in_left, 1); // A
        assert_eq!(s.only_in_right, 1); // C
        assert_eq!(s.equal, 1); // B
        assert_eq!(s.different, 0);
    }

    #[test]
    fn indice_solo_en_izquierda() {
        use crate::model::Index;
        let left = Schema {
            indexes: vec![Index { name: "ix1".into(), table: "dbo.C".into(), columns: vec!["a".into()], is_unique: true }],
            ..Default::default()
        };
        let r = compare_schemas(&left, &Schema::default());
        let ix = r.objects.iter().find(|o| o.object_type == "Índice").unwrap();
        assert_eq!(ix.status, DiffStatus::OnlyInLeft);
    }

    #[test]
    fn clave_foranea_diferente() {
        use crate::model::ForeignKey;
        let mk = |to: &str| ForeignKey {
            name: "fk".into(),
            from_table: "dbo.P".into(),
            from_columns: vec!["cid".into()],
            to_table: to.into(),
            to_columns: vec!["id".into()],
        };
        let left = Schema { foreign_keys: vec![mk("dbo.Clientes")], ..Default::default() };
        let right = Schema { foreign_keys: vec![mk("dbo.Otros")], ..Default::default() };
        let r = compare_schemas(&left, &right);
        let fk = r.objects.iter().find(|o| o.object_type == "Clave foránea").unwrap();
        assert_eq!(fk.status, DiffStatus::Different);
    }

    #[test]
    fn opcion_ignorar_mayusculas_en_tipos() {
        let left = Schema { tables: vec![table("C", vec![col("Id", "INT", false)])], ..Default::default() };
        let right = Schema { tables: vec![table("C", vec![col("Id", "int", false)])], ..Default::default() };
        // Con ignore_case (por defecto) son iguales; sin él, distintas.
        assert_eq!(compare_schemas(&left, &right).objects[0].status, DiffStatus::Equal);
        let strict = CompareOptions { ignore_case: false, ..Default::default() };
        assert_eq!(compare_schemas_opts(&left, &right, &strict).objects[0].status, DiffStatus::Different);
    }

    #[test]
    fn opcion_ignorar_comentarios() {
        let a = Schema { procedures: vec![proc("sp", "CREATE PROC sp AS SELECT 1 -- activo")], ..Default::default() };
        let b = Schema { procedures: vec![proc("sp", "CREATE PROC sp AS SELECT 1 -- inactivo")], ..Default::default() };
        // Por defecto el comentario cuenta → distintas; ignorándolos → iguales.
        assert_eq!(compare_schemas(&a, &b).objects[0].status, DiffStatus::Different);
        let o = CompareOptions { ignore_comments: true, ..Default::default() };
        assert_eq!(compare_schemas_opts(&a, &b, &o).objects[0].status, DiffStatus::Equal);
    }

    #[test]
    fn opcion_filtra_tipos_de_objeto() {
        let left = Schema { tables: vec![table("Solo", vec![])], ..Default::default() };
        // Sin comparar tablas, no debe reportar ningún objeto de tipo "Tabla".
        let o = CompareOptions { tables: false, ..Default::default() };
        let r = compare_schemas_opts(&left, &Schema::default(), &o);
        assert!(r.objects.iter().all(|x| x.object_type != "Tabla"));
    }
}
