mod daemon_client;

use daemon_client::{DaemonClient, SessionInfo};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tauri::api::dialog::blocking::FileDialogBuilder;

#[tauri::command]
async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client
        .list_sessions()
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))
}

#[tauri::command]
async fn create_session(working_dir: String) -> Result<SessionCreatedResponse, String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client
        .create_session(working_dir)
        .await
        .map_err(|e| format!("Failed to create session: {}", e))
}

#[tauri::command]
async fn delete_session(session_id: String) -> Result<(), String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client
        .delete_session(session_id)
        .await
        .map_err(|e| format!("Failed to delete session: {}", e))
}

#[tauri::command]
async fn read_session_logs(log_path: String, offset: usize) -> Result<Vec<String>, String> {
    let file = File::open(&log_path)
        .map_err(|e| format!("Failed to open log file {}: {}", log_path, e))?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .skip(offset)
        .filter_map(|line| line.ok())
        .collect();

    Ok(lines)
}

#[tauri::command]
async fn send_input(session_id: String, text: String) -> Result<(), String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client
        .send_input(session_id, text)
        .await
        .map_err(|e| format!("Failed to send input: {}", e))
}

#[tauri::command]
fn pick_directory() -> Result<Option<String>, String> {
    let dialog = FileDialogBuilder::new()
        .set_title("Select Project Directory")
        .pick_folder();
    
    Ok(dialog.map(|path| path.to_string_lossy().to_string()))
}

#[derive(serde::Serialize)]
struct SessionCreatedResponse {
    session_id: String,
    log_path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            create_session,
            delete_session,
            read_session_logs,
            send_input,
            pick_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
