//! Capa de acceso a datos: cada motor lee su base y la traduce al modelo común
//! de `datadiff-core`. Aquí viven la configuración de conexión y el despacho
//! según el motor elegido.

pub mod postgres;
pub mod sqlite;
pub mod sqlserver;

use datadiff_core::Schema;
use serde::Deserialize;

/// Error genérico de la capa de datos.
pub type DbError = Box<dyn std::error::Error + Send + Sync>;
pub type DbResult<T> = Result<T, DbError>;

/// Motor elegido por el usuario para una conexión.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    SqlServer,
    Sqlite,
    Postgres,
}

/// Datos de conexión que llegan desde la interfaz. Es un único objeto "plano"
/// con todos los campos posibles; según `engine` se usan unos u otros.
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionConfig {
    pub engine: EngineKind,

    // --- SQL Server ---
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,

    // --- SQLite ---
    /// Ruta al archivo .db / .sqlite.
    #[serde(default)]
    pub path: String,
}

fn default_port() -> u16 {
    1433
}

/// Lee el esquema completo del motor correspondiente.
pub async fn read_schema(cfg: &ConnectionConfig) -> Result<Schema, String> {
    match cfg.engine {
        EngineKind::SqlServer => sqlserver::read_schema(cfg).await.map_err(|e| e.to_string()),
        EngineKind::Postgres => postgres::read_schema(cfg).await.map_err(|e| e.to_string()),
        EngineKind::Sqlite => run_blocking(cfg.path.clone(), sqlite::read_schema).await,
    }
}

/// Prueba la conexión y devuelve un mensaje legible.
pub async fn test_connection(cfg: &ConnectionConfig) -> Result<String, String> {
    match cfg.engine {
        EngineKind::SqlServer => sqlserver::test_connection(cfg).await.map_err(|e| e.to_string()),
        EngineKind::Postgres => postgres::test_connection(cfg).await.map_err(|e| e.to_string()),
        EngineKind::Sqlite => run_blocking(cfg.path.clone(), sqlite::test_connection).await,
    }
}

/// Ejecuta un script SQL contra la base indicada (aplica la migración).
pub async fn apply_script(cfg: &ConnectionConfig, script: &str) -> Result<String, String> {
    match cfg.engine {
        EngineKind::SqlServer => sqlserver::apply_script(cfg, script).await.map_err(|e| e.to_string()),
        EngineKind::Postgres => postgres::apply_script(cfg, script).await.map_err(|e| e.to_string()),
        EngineKind::Sqlite => {
            let (path, script) = (cfg.path.clone(), script.to_string());
            tokio::task::spawn_blocking(move || sqlite::apply_script(&path, &script))
                .await
                .map_err(|e| format!("fallo en el hilo de SQLite: {e}"))?
                .map_err(|e| e.to_string())
        }
    }
}

/// rusqlite es síncrono: lo ejecutamos en un hilo aparte (`spawn_blocking`) para
/// no bloquear el runtime asíncrono de Tauri.
async fn run_blocking<T, F>(path: String, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&str) -> DbResult<T> + Send + 'static,
{
    tokio::task::spawn_blocking(move || f(path.as_str()))
        .await
        .map_err(|e| format!("fallo en el hilo de SQLite: {e}"))?
        .map_err(|e| e.to_string())
}
