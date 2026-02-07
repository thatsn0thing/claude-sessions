mod daemon_client;

use daemon_client::{DaemonClient, SessionInfo};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[tauri::command]
async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client
        .list_sessions()
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))
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
async fn send_input(log_path: String, text: String) -> Result<(), String> {
    // Write input to log file (will be picked up by daemon)
    // For now, we'll write directly to a companion input file
    let input_path = PathBuf::from(&log_path).with_extension("input");
    
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&input_path)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    writeln!(file, "{}", text)
        .map_err(|e| format!("Failed to write input: {}", e))?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            read_session_logs,
            send_input
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
