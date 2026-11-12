//! Núcleo de DataDiff: el modelo de esquema (independiente del motor de base de
//! datos) y el motor que compara dos esquemas y reporta las diferencias.
//!
//! Este crate NO sabe nada de SQL Server ni de PostgreSQL: solo trabaja con el
//! modelo normalizado. Cada motor (en `src-tauri`) se encarga de leer su base y
//! traducirla a este modelo. Así, agregar PostgreSQL luego solo requiere escribir
//! su capa de lectura; el diff y la interfaz se reutilizan tal cual.

pub mod data;
pub mod diagram;
pub mod diff;
pub mod model;
pub mod script;

pub use data::*;
pub use diagram::*;
pub use diff::*;
pub use model::*;
pub use script::*;
