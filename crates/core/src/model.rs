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
    /// Disparadores (triggers).
    #[serde(default)]
    pub triggers: Vec<Trigger>,
}

/// Una tabla con sus columnas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Esquema SQL del objeto, p.ej. "dbo".
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
    /// Nombre de la restricción de clave primaria, si el motor lo expone y la
    /// tabla tiene PK. Hace falta para poder emitir el `DROP CONSTRAINT` al
    /// cambiar la clave primaria del destino. `None` en SQLite (no la nombra)
    /// y en comparaciones guardadas antes de que existiera este campo.
    #[serde(default)]
    pub pk_name: Option<String>,
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
    /// Cómo se autogenera el valor de la columna, si el motor lo hace solo.
    /// `None` = la columna no se autogenera.
    #[serde(default)]
    pub identity: Option<Identity>,
}

/// Cómo genera el motor, por sí solo, el valor de una columna: el `IDENTITY` de
/// SQL Server, el `AUTO_INCREMENT` de MySQL / MariaDB, el `AUTOINCREMENT` de
/// SQLite y las columnas de identidad (o `serial`) de PostgreSQL.
///
/// Solo SQL Server expone el inicio y el incremento columna por columna. En los
/// otros tres se leen como `1` porque el catálogo no los da ahí: en MySQL el
/// inicio es una opción de la tabla y el incremento una variable de sesión, y en
/// PostgreSQL viven en la secuencia, que este modelo todavía no lee. Para una
/// tabla que nace vacía en el destino, empezar en 1 es justo lo que se quiere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Primer valor que entrega la secuencia.
    pub seed: i64,
    /// Cuánto avanza en cada fila.
    pub increment: i64,
    /// `true` cuando el motor PROHÍBE escribir el valor a mano
    /// (`GENERATED ALWAYS AS IDENTITY` de PostgreSQL). En SQL Server, MySQL y
    /// SQLite siempre es `false`: los tres permiten insertarlo.
    pub always: bool,
}

impl Identity {
    /// La forma habitual: empieza en 1, avanza de 1 en 1 y se puede escribir.
    /// Es lo único que saben decir MySQL, SQLite y PostgreSQL por columna.
    pub fn simple() -> Self {
        Self { seed: 1, increment: 1, always: false }
    }
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

/// Un disparador (trigger) asociado a una tabla.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub schema: String,
    pub name: String,
    /// Tabla a la que pertenece, calificada ("schema.tabla").
    pub table: String,
    /// Cuerpo SQL completo (CREATE TRIGGER …) tal como lo da el catálogo.
    pub definition: String,
}

impl Trigger {
    /// Clave única para emparejar: "schema.tabla.disparador".
    pub fn key(&self) -> String {
        format!("{}.{}", self.table, self.name)
    }
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
