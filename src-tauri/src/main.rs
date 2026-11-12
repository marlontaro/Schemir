// En Windows, en modo release, evita abrir una consola detrás de la ventana.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use schemir_core::{
    compare_schemas_opts, compare_table_data, generate_sync_script_opts, generate_sync_units_opts,
    to_diagram_model, to_diff_diagram_opts, CompareOptions, ComparisonResult, DataComparison,
    Dialect, DiagramModel, ScriptUnit,
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

fn dialect_of(engine: EngineKind) -> Dialect {
    match engine {
        EngineKind::SqlServer => Dialect::SqlServer,
        EngineKind::Postgres => Dialect::Postgres,
        EngineKind::Sqlite => Dialect::Sqlite,
    }
}

/// Comando principal: lee ambos esquemas, los compara y genera el script de
/// sincronización (para dejar el DESTINO (B) igual que el ORIGEN (A)).
#[tauri::command]
async fn compare_databases(
    left: ConnectionConfig,
    right: ConnectionConfig,
    options: Option<CompareOptions>,
) -> Result<CompareOutput, String> {
    let opts = options.unwrap_or_default();
    let source = db::read_schema(&left)
        .await
        .map_err(|e| format!("ORIGEN (A): {e}"))?;
    let target = db::read_schema(&right)
        .await
        .map_err(|e| format!("DESTINO (B): {e}"))?;

    let comparison = compare_schemas_opts(&source, &target, &opts);
    let dialect = dialect_of(right.engine);
    let script = generate_sync_script_opts(&source, &target, dialect, &opts);
    let script_units = generate_sync_units_opts(&source, &target, dialect, &opts);
    let diagram_a = to_diagram_model(&source);
    let diagram_b = to_diagram_model(&target);
    let diagram_diff = to_diff_diagram_opts(&source, &target, &opts);
    Ok(CompareOutput { comparison, script, script_units, diagram_a, diagram_b, diagram_diff })
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
        .map_err(|e| format!("ORIGEN (A): {e}"))?;
    let rt = db::read_table_data(&right, &table)
        .await
        .map_err(|e| format!("DESTINO (B): {e}"))?;
    let dialect = dialect_of(right.engine);
    Ok(compare_table_data(&lt, &rt, dialect))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            compare_databases,
            test_connection,
            save_text_file,
            save_binary_file,
            read_text_file,
            apply_script,
            compare_data
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Schemir");
}
