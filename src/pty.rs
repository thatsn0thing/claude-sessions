use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty, PtyPair, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::logging::{Direction, SessionLogger};

/// Spawns a Claude Code session as a PTY subprocess.
/// 
/// Important: We treat `claude` CLI as a black box.
/// We simply spawn it in the given working directory and let it run.
pub fn spawn_claude_pty(working_dir: &Path) -> Result<PtyPair> {
    // Create a PTY pair (master + slave)
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to create PTY pair")?;

    // Build the command to spawn `claude`
    let mut cmd = CommandBuilder::new("claude");
    cmd.cwd(working_dir);

    // Spawn the process in the PTY slave
    let _child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn claude process")?;

    // Note: We return the PtyPair. The caller is responsible for:
    // - Keeping the master alive to interact with the PTY
    // - Managing the child process lifecycle
    Ok(pair)
}

/// SessionProcess holds the PTY pair for a running Claude session
/// and manages I/O logging.
pub struct SessionProcess {
    pub pty_pair: Arc<PtyPair>,
    session_id: Uuid,
    output_task: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SessionProcess {
    /// Create a new session process with logging enabled
    pub fn new(session_id: Uuid, pty_pair: PtyPair) -> Result<Self> {
        let pty_pair = Arc::new(pty_pair);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        // Spawn PTY output reader task
        let output_task = Self::spawn_output_reader(
            session_id,
            Arc::clone(&pty_pair),
            shutdown_rx,
        )?;

        Ok(SessionProcess {
            pty_pair,
            session_id,
            output_task: Some(output_task),
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Spawn a background task to read PTY output and log it
    fn spawn_output_reader(
        session_id: Uuid,
        pty_pair: Arc<PtyPair>,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        // Clone the master reader for the background task
        let mut reader = pty_pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let handle = tokio::task::spawn_blocking(move || {
            let mut logger = match SessionLogger::new(session_id) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to create logger for session {}: {}", session_id, e);
                    return;
                }
            };

            let mut buffer = [0u8; 8192];

            loop {
                // Check for shutdown signal (non-blocking)
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }

                // Read from PTY (blocking, but with timeout via buffer size)
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - Claude process exited
                        break;
                    }
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        if let Err(e) = logger.log(Direction::Output, data) {
                            eprintln!("Failed to log output for session {}: {}", session_id, e);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Non-blocking read, no data available yet
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error reading from PTY for session {}: {}", session_id, e);
                        break;
                    }
                }
            }

            println!("PTY output reader stopped for session {}", session_id);
        });

        Ok(handle)
    }

    /// Write input to the PTY and log it
    pub fn write_input(&self, data: &[u8]) -> Result<()> {
        // Write to PTY master
        let mut writer = self
            .pty_pair
            .master
            .take_writer()
            .context("Failed to get PTY writer")?;
        writer.write_all(data)?;
        writer.flush()?;

        // Log the input
        let mut logger = SessionLogger::new(self.session_id)?;
        logger.log(Direction::Input, data.to_vec())?;

        Ok(())
    }

    /// Get the session ID
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }
}

impl Drop for SessionProcess {
    fn drop(&mut self) {
        // Signal the output reader to shutdown (non-blocking)
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }

        // Note: We don't wait for the output reader to finish here
        // to avoid blocking during Drop. The task will clean up on its own.
        // The PTY will be closed when pty_pair is dropped.
        if let Some(_handle) = self.output_task.take() {
            // Task will notice PTY closure and exit naturally
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[ignore] // Requires `claude` to be installed
    fn test_pty_spawn() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = spawn_claude_pty(temp_dir.path());
        assert!(result.is_ok());
    }
}
