use crate::ipc::{Request, Response};
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// IPC Client for communicating with the daemon
pub struct Client {
    socket_path: PathBuf,
}

impl Client {
    /// Create a new client
    pub fn new() -> Result<Self> {
        let socket_path = Self::socket_path()?;
        Ok(Client { socket_path })
    }

    /// Get the Unix socket path
    fn socket_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Cannot determine home directory")?;
        Ok(PathBuf::from(home)
            .join(".claude-sessions")
            .join("daemon.sock"))
    }

    /// Send a request to the daemon and get a response
    pub async fn send_request(&self, request: Request) -> Result<Response> {
        // Connect to daemon
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to daemon. Is it running?")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request (JSON + newline)
        let request_json = serde_json::to_string(&request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: Response = serde_json::from_str(&line)
            .context("Failed to parse daemon response")?;

        Ok(response)
    }

    /// Check if daemon is running
    pub fn is_daemon_running(&self) -> bool {
        self.socket_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_socket_path() {
        let path = Client::socket_path().unwrap();
        assert!(path.to_str().unwrap().contains(".claude-sessions"));
    }
}
