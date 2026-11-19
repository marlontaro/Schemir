// En Windows, en modo release, evita abrir una consola detrás de la ventana.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use schemir_core::{
    compare_schemas_opts, compare_table_data, data_types, default_schema, default_version,
    engine_versions, generate_sync_script_opts, generate_sync_units_opts, identity_suffix,
    identity_types, supports_schemas, uuid_default, uuid_types,
    to_diagram_model, to_diff_diagram_opts, CompareOptions, ComparisonResult, DataComparison,
    DataType, Dialect, DiagramModel, ScriptUnit, Version,
};
use db::{ConnectionConfig, EngineKind};
use serde::Serialize;

/// Lo que devuelve la comparación a la interfaz: las diferencias y el script.
#[derive(Serialize)]
struct CompareOutput {
    comparison: ComparisonResult,
    script: String,
    /// El script separado por objeto, para que la interfaz elija qué sincronizar.
    script_units: Vec<ScriptUnit>,
    diagram_a: DiagramModel,
    diagram_b: DiagramModel,
    diagram_diff: DiagramModel,
}

/// Todo lo que el diseñador necesita saber de un motor: sus tipos de datos, las
/// versiones entre las que se puede elegir y si agrupa las tablas en esquemas.
///
/// Va en una sola respuesta porque la interfaz lo pide de golpe cada vez que se
/// cambia de dialecto, y así no hay tres idas y vueltas para pintar un
/// desplegable.
#[derive(Serialize)]
struct DialectInfo {
    types: Vec<DataType>,
    versions: Vec<Version>,
    default_version: Version,
    supports_schemas: bool,
    default_schema: String,
    /// Tipos a los que se les puede poner un contador, y lo que se escribe para
    /// declararlo.
    identity_types: Vec<String>,
    identity_suffix: String,
    /// Tipos a los que el motor sabe generarles un identificador único, y la
    /// expresión que lo genera (vacía si el motor no trae ninguna).
    uuid_types: Vec<String>,
    uuid_default: String,
}

/// El catálogo de un motor. No toca ninguna base de datos: es conocimiento que
/// vive en el núcleo.
#[tauri::command]
fn dialect_info(dialect: Dialect) -> DialectInfo {
    DialectInfo {
        types: data_types(dialect),
        versions: engine_versions(dialect),
        default_version: default_version(dialect),
        supports_schemas: supports_schemas(dialect),
        default_schema: default_schema(dialect).to_string(),
        identity_types: identity_types(dialect).into_iter().map(String::from).collect(),
        identity_suffix: identity_suffix(dialect).to_string(),
        uuid_types: uuid_types(dialect).into_iter().map(String::from).collect(),
        uuid_default: uuid_default(dialect).unwrap_or("").to_string(),
    }
}

fn dialect_of(engine: EngineKind) -> Dialect {
    match engine {
        EngineKind::SqlServer => Dialect::SqlServer,
        EngineKind::Postgres => Dialect::Postgres,
        EngineKind::MySql => Dialect::MySql,
        EngineKind::Sqlite => Dialect::Sqlite,
    }
}

/// Comando principal: lee ambos esquemas, los compara y genera el script de
/// sincronización (para dejar el DESTINO igual que el ORIGEN).
#[tauri::command]
async fn compare_databases(
    left: ConnectionConfig,
    right: ConnectionConfig,
    options: Option<CompareOptions>,
) -> Result<CompareOutput, String> {
    let opts = options.unwrap_or_default();
    let source = db::read_schema(&left)
        .await
        .map_err(|e| format!("ORIGEN: {e}"))?;
    let target = db::read_schema(&right)
        .await
        .map_err(|e| format!("DESTINO: {e}"))?;

    let comparison = compare_schemas_opts(&source, &target, &opts);
    let dialect = dialect_of(right.engine);
    let script = generate_sync_script_opts(&source, &target, dialect, &opts);
    let script_units = generate_sync_units_opts(&source, &target, dialect, &opts);
    let diagram_a = to_diagram_model(&source);
    let diagram_b = to_diagram_model(&target);
    let diagram_diff = to_diff_diagram_opts(&source, &target, &opts);
    Ok(CompareOutput { comparison, script, script_units, diagram_a, diagram_b, diagram_diff })
}

/// Lista las bases de datos del servidor, para que se puedan elegir.
#[tauri::command]
async fn list_databases(config: ConnectionConfig) -> Result<Vec<String>, String> {
    db::list_databases(&config).await
}

/// Prueba una conexión y devuelve un mensaje legible (versión del motor, etc.).
#[tauri::command]
async fn test_connection(config: ConnectionConfig) -> Result<String, String> {
    db::test_connection(&config).await
}

/// Guarda texto en un archivo (lo usa el botón "Guardar .sql" y "Exportar SVG").
#[tauri::command]
async fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("No se pudo guardar el archivo: {e}"))
}

/// Guarda bytes en un archivo (lo usa "Exportar PNG" del diagrama).
#[tauri::command]
async fn save_binary_file(path: String, data: Vec<u8>) -> Result<(), String> {
    std::fs::write(&path, data).map_err(|e| format!("No se pudo guardar el archivo: {e}"))
}

/// Lee un archivo de texto (lo usa "Abrir comparación").
#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("No se pudo leer el archivo: {e}"))
}

/// Aplica (ejecuta) el script de sincronización contra la base indicada.
#[tauri::command]
async fn apply_script(config: ConnectionConfig, script: String) -> Result<String, String> {
    db::apply_script(&config, &script).await
}

/// Compara los DATOS de una tabla entre A y B (v1: solo SQLite) y arma el DML.
#[tauri::command]
async fn compare_data(
    left: ConnectionConfig,
    right: ConnectionConfig,
    table: String,
) -> Result<DataComparison, String> {
    if left.engine != EngineKind::Sqlite || right.engine != EngineKind::Sqlite {
        return Ok(DataComparison {
            table,
            note: "El comparador de datos admite SQLite por ahora.".into(),
            ..Default::default()
        });
    }
    let lt = db::read_table_data(&left, &table)
        .await
        .map_err(|e| format!("ORIGEN: {e}"))?;
    let rt = db::read_table_data(&right, &table)
        .await
        .map_err(|e| format!("DESTINO: {e}"))?;
    let dialect = dialect_of(right.engine);
    Ok(compare_table_data(&lt, &rt, dialect))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            compare_databases,
            test_connection,
            list_databases,
            save_text_file,
            save_binary_file,
            read_text_file,
            apply_script,
            compare_data,
            dialect_info
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Schemir");
}
