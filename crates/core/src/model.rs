//! Modelo normalizado de un esquema de base de datos.
//!
//! La idea clave: todos los motores (SQL Server hoy, PostgreSQL mañana) leen sus
//! catálogos del sistema y los traducen a estas mismas estructuras. El motor de
//! comparación trabaja solo sobre este modelo, sin importar de dónde vino.

use serde::{Deserialize, Serialize};

/// Motor de base de datos del que proviene un esquema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    SqlServer,
    Postgres,
}

/// Un esquema completo: todos los objetos leídos de una base de datos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schema {
    /// Etiqueta informativa para la interfaz, p.ej. "SQL Server · ventas".
    pub engine_label: String,
    pub tables: Vec<Table>,
    pub views: Vec<Routine>,
    pub procedures: Vec<Routine>,
    pub functions: Vec<Routine>,
    /// Relaciones de clave foránea del esquema (para el diagrama ER).
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKey>,
    /// Índices (sin contar el de la clave primaria).
    #[serde(default)]
    pub indexes: Vec<Index>,
    /// Restricciones CHECK.
    #[serde(default)]
    pub checks: Vec<CheckConstraint>,
}

/// Una tabla con sus columnas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Esquema SQL del objeto, p.ej. "dbo".
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
}

impl Table {
    /// Nombre calificado usado como clave para emparejar objetos, p.ej. "dbo.Clientes".
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

/// Una columna (campo) de una tabla.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    /// Tipo ya formateado al estilo SQL, p.ej. "varchar(50)" o "decimal(10,2)".
    pub data_type: String,
    pub is_nullable: bool,
    /// Expresión del valor por defecto, si existe.
    pub default_value: Option<String>,
    /// Posición de la columna (solo para mostrarla en orden; no se compara).
    pub ordinal: i32,
    /// ¿Forma parte de la clave primaria? (se usa para marcarla en el diagrama).
    #[serde(default)]
    pub is_primary_key: bool,
}

/// Una relación de clave foránea entre dos tablas (para el diagrama ER).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub name: String,
    /// Tabla que contiene la FK, calificada ("schema.tabla").
    pub from_table: String,
    pub from_columns: Vec<String>,
    /// Tabla referenciada, calificada.
    pub to_table: String,
    pub to_columns: Vec<String>,
}

/// Un índice sobre una tabla (no incluye el que respalda la clave primaria).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    /// Tabla del índice, calificada ("schema.tabla").
    pub table: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

/// Una restricción CHECK sobre una tabla.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConstraint {
    pub name: String,
    pub table: String,
    /// Expresión de la restricción.
    pub definition: String,
}

/// Objeto cuyo cuerpo es código SQL: vistas, procedimientos y funciones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routine {
    pub schema: String,
    pub name: String,
    pub kind: RoutineKind,
    /// Cuerpo SQL completo del objeto (tal como lo devuelve el catálogo).
    pub definition: String,
}

impl Routine {
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

/// Subtipo de un objeto con cuerpo SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutineKind {
    View,
    Procedure,
    ScalarFunction,
    TableFunction,
}
