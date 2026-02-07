use crate::ipc::{Request, Response, SessionInfo};
use crate::manager::SessionManager;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Daemon manages a long-running session manager and IPC server
pub struct Daemon {
    manager: Arc<SessionManager>,
    socket_path: PathBuf,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl Daemon {
    /// Create a new daemon instance
    pub fn new() -> Result<Self> {
        let socket_path = Self::socket_path()?;
        let manager = Arc::new(SessionManager::new());
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        Ok(Daemon {
            manager,
            socket_path,
            shutdown_tx,
        })
    }

    /// Get the Unix socket path for IPC
    fn socket_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Cannot determine home directory")?;
        Ok(PathBuf::from(home)
            .join(".claude-sessions")
            .join("daemon.sock"))
    }

    /// Check if daemon is already running
    pub fn is_running() -> bool {
        if let Ok(socket_path) = Self::socket_path() {
            socket_path.exists()
        } else {
            false
        }
    }

    /// Start the daemon (blocking)
    pub async fn run(&mut self) -> Result<()> {
        // Ensure socket directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove stale socket if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        // Bind Unix socket
        let listener = UnixListener::bind(&self.socket_path)
            .context("Failed to bind Unix socket")?;

        println!("✅ Daemon started. Socket: {:?}", self.socket_path);

        // Accept connections in a loop
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            let manager = Arc::clone(&self.manager);
                            let shutdown_tx = self.shutdown_tx.clone();
                            // Handle connection sequentially (no need to spawn for local IPC)
                            if let Err(e) = Self::handle_connection(stream, manager, shutdown_tx).await {
                                eprintln!("Connection error: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    println!("Daemon shutting down...");
                    break;
                }
            }
        }

        // Cleanup socket on shutdown
        let _ = std::fs::remove_file(&self.socket_path);
        println!("✅ Daemon stopped");

        Ok(())
    }

    /// Handle a single client connection
    async fn handle_connection(
        stream: UnixStream,
        manager: Arc<SessionManager>,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // Read one request per connection (simple protocol)
        reader.read_line(&mut line).await?;
        
        let request: Request = serde_json::from_str(&line)
            .context("Failed to parse request")?;

        let response = Self::handle_request(request, &manager, shutdown_tx).await;

        // Send response
        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        Ok(())
    }

    /// Process a request and generate a response
    async fn handle_request(
        request: Request,
        manager: &SessionManager,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
    ) -> Response {
        match request {
            Request::StartSession { working_dir } => {
                match manager.start_session(working_dir).await {
                    Ok(session_id) => {
                        let sessions = manager.list_sessions().await;
                        let session = sessions.iter()
                            .find(|s| s.id == session_id.to_string());
                        
                        if let Some(s) = session {
                            Response::SessionStarted {
                                session_id: s.id.clone(),
                                log_path: s.log_path.clone(),
                            }
                        } else {
                            Response::Error {
                                message: "Session started but not found in list".to_string(),
                            }
                        }
                    }
                    Err(e) => Response::Error {
                        message: format!("Failed to start session: {}", e),
                    },
                }
            }
            Request::ListSessions => {
                let sessions: Vec<SessionInfo> = manager
                    .list_sessions()
                    .await
                    .into_iter()
                    .map(|s| SessionInfo {
                        id: s.id,
                        working_dir: s.working_dir,
                        created_at: s.created_at,
                        status: s.status,
                        log_path: s.log_path,
                    })
                    .collect();
                Response::SessionList { sessions }
            }
            Request::StopSession { session_id } => {
                match Uuid::parse_str(&session_id) {
                    Ok(uuid) => match manager.stop_session(uuid).await {
                        Ok(_) => Response::SessionStopped { session_id },
                        Err(e) => Response::Error {
                            message: format!("Failed to stop session: {}", e),
                        },
                    },
                    Err(_) => Response::Error {
                        message: "Invalid session ID format".to_string(),
                    },
                }
            }
            Request::AttachSession { session_id: _ } => {
                // TODO: Implement log streaming
                Response::Error {
                    message: "Attach not implemented yet".to_string(),
                }
            }
            Request::Ping => Response::Pong,
            Request::Shutdown => {
                let _ = shutdown_tx.send(());
                Response::Ok
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = Daemon::socket_path().unwrap();
        assert!(path.to_str().unwrap().contains(".claude-sessions"));
        assert!(path.to_str().unwrap().ends_with("daemon.sock"));
    }
}
