// En Windows, en modo release, evita abrir una consola detrás de la ventana.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use datadiff_core::{compare_schemas, generate_sync_script, to_diagram_model, to_diff_diagram, ComparisonResult, Dialect, DiagramModel};
use db::{ConnectionConfig, EngineKind};
use serde::Serialize;

/// Lo que devuelve la comparación a la interfaz: las diferencias y el script.
#[derive(Serialize)]
struct CompareOutput {
    comparison: ComparisonResult,
    script: String,
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
) -> Result<CompareOutput, String> {
    let source = db::read_schema(&left)
        .await
        .map_err(|e| format!("ORIGEN (A): {e}"))?;
    let target = db::read_schema(&right)
        .await
        .map_err(|e| format!("DESTINO (B): {e}"))?;

    let comparison = compare_schemas(&source, &target);
    let script = generate_sync_script(&source, &target, dialect_of(right.engine));
    let diagram_a = to_diagram_model(&source);
    let diagram_b = to_diagram_model(&target);
    let diagram_diff = to_diff_diagram(&source, &target);
    Ok(CompareOutput { comparison, script, diagram_a, diagram_b, diagram_diff })
}

/// Prueba una conexión y devuelve un mensaje legible (versión del motor, etc.).
#[tauri::command]
async fn test_connection(config: ConnectionConfig) -> Result<String, String> {
    db::test_connection(&config).await
}

/// Guarda texto en un archivo (lo usa el botón "Guardar .sql").
#[tauri::command]
async fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("No se pudo guardar el archivo: {e}"))
}

/// Aplica (ejecuta) el script de sincronización contra la base indicada.
#[tauri::command]
async fn apply_script(config: ConnectionConfig, script: String) -> Result<String, String> {
    db::apply_script(&config, &script).await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            compare_databases,
            test_connection,
            save_text_file,
            apply_script
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar DataDiff");
}
