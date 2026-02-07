use crate::pty::{spawn_claude_pty, SessionProcess};
use crate::session::{Session, SessionInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// SessionManager owns all active Claude Code sessions.
/// 
/// It maintains:
/// - A map of session metadata (Session objects)
/// - A map of running PTY processes (SessionProcess objects)
/// 
/// The manager provides methods to start, stop, and list sessions.
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    processes: Arc<Mutex<HashMap<Uuid, SessionProcess>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a new Claude Code session in the given working directory.
    /// 
    /// Returns the session ID on success.
    pub fn start_session(&self, working_dir: PathBuf) -> Result<Uuid> {
        // Validate that the directory exists
        if !working_dir.exists() {
            anyhow::bail!("Working directory does not exist: {:?}", working_dir);
        }

        // Create session metadata
        let session = Session::new(working_dir.clone());
        let session_id = session.id;

        // Spawn Claude as a PTY subprocess
        let pty_pair = spawn_claude_pty(&working_dir)
            .context("Failed to spawn Claude Code PTY")?;
        let process = SessionProcess::new(session_id, pty_pair)
            .context("Failed to create session process with logging")?;

        // Store session and process
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id, session);
        }
        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(session_id, process);
        }

        println!("✅ Started session {} in {:?}", session_id, working_dir);
        Ok(session_id)
    }

    /// Stop a running session by ID.
    /// 
    /// This removes the session metadata and drops the PTY process,
    /// which should terminate the Claude subprocess.
    pub fn stop_session(&self, session_id: Uuid) -> Result<()> {
        {
            let mut sessions = self.sessions.lock().unwrap();
            if !sessions.contains_key(&session_id) {
                anyhow::bail!("Session not found: {}", session_id);
            }
            sessions.remove(&session_id);
        }
        {
            let mut processes = self.processes.lock().unwrap();
            processes.remove(&session_id);
            // Dropping the PTY should terminate the child process
        }

        println!("✅ Stopped session {}", session_id);
        Ok(())
    }

    /// List all active sessions.
    /// 
    /// Returns a vector of SessionInfo structs (without PTY handles).
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .map(|s| SessionInfo {
                id: s.id.to_string(),
                working_dir: s.working_dir.display().to_string(),
                created_at: s.created_at.clone(),
                status: "running".to_string(),
            })
            .collect()
    }
}
