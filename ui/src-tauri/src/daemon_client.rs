use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub working_dir: String,
    pub created_at: String,
    pub status: String,
    pub log_path: String,
}

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Cannot determine home directory")?;
        let socket_path = PathBuf::from(home)
            .join(".claude-sessions")
            .join("daemon.sock");
        Ok(DaemonClient { socket_path })
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let request = serde_json::json!({"type": "list_sessions"});
        let response = self.send_request(&request).await?;

        match response.get("type").and_then(|v| v.as_str()) {
            Some("session_list") => {
                let sessions: Vec<SessionInfo> = serde_json::from_value(
                    response
                        .get("sessions")
                        .cloned()
                        .unwrap_or(serde_json::json!([])),
                )?;
                Ok(sessions)
            }
            Some("error") => {
                let msg = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                anyhow::bail!("Daemon error: {}", msg)
            }
            _ => anyhow::bail!("Unexpected response type"),
        }
    }

    async fn send_request(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to daemon. Is it running?")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: serde_json::Value =
            serde_json::from_str(&line).context("Failed to parse daemon response")?;

        Ok(response)
    }
}
