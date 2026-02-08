use crate::persistence::{is_process_alive, PersistenceManager, PersistedSession};
use crate::pty::{spawn_claude_pty, SessionProcess};
use crate::session::{Session, SessionInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// SessionManager owns all active Claude Code sessions.
///
/// Responsibilities:
/// - Spawn and manage PTY processes
/// - Maintain session metadata
/// - Persist session state to disk
/// - Recover sessions on restart
///
/// ## Persistence Strategy
///
/// Session metadata is saved to disk after every change (start/stop).
/// On daemon restart, sessions are loaded from disk and their process
/// status is verified (PID check).
///
/// ## Recovery Logic
///
/// On startup:
/// 1. Load persisted sessions from disk
/// 2. For each session:
///    - If PID is unknown → mark as "stale"
///    - If PID is known but process is dead → mark as "crashed"
///    - If PID is known and process is alive → mark as "orphaned"
/// 3. Do NOT attempt to reattach to orphaned processes
///
/// Conservative approach: we don't try to reconnect to existing PTYs.
/// User must manually check orphaned sessions and stop them if needed.
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    processes: Arc<Mutex<HashMap<Uuid, SessionProcess>>>,
    persistence: Arc<Mutex<PersistenceManager>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let persistence = PersistenceManager::new()
            .expect("Failed to initialize persistence manager");

        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            persistence: Arc::new(Mutex::new(persistence)),
        }
    }

    /// Create a new session manager and recover persisted sessions
    /// 
    /// This should be called when starting the daemon.
    /// Use `new()` for testing without recovery.
    pub async fn with_recovery() -> Self {
        let manager = Self::new();
        
        // Attempt to recover sessions from disk
        if let Err(e) = manager.recover_sessions().await {
            eprintln!("⚠️  Failed to recover sessions: {}", e);
            eprintln!("Starting with empty session list");
        }

        manager
    }

    /// Recover sessions from disk on daemon startup
    ///
    /// ## Process Status Detection
    ///
    /// For each persisted session:
    /// 1. If no PID recorded → status = "stale" (unknown state)
    /// 2. If PID recorded but process dead → status = "crashed"
    /// 3. If PID recorded and process alive → status = "orphaned"
    ///
    /// ## Why "orphaned"?
    ///
    /// We mark alive processes as "orphaned" because:
    /// - We don't have a PTY handle to them
    /// - We can't send input to them
    /// - We can't reliably determine if they're our Claude processes
    ///   (PID could have been reused)
    ///
    /// User should manually verify and stop orphaned sessions.
    ///
    /// ## Conservative Approach
    ///
    /// We do NOT attempt to:
    /// - Reattach to existing PTY file descriptors
    /// - Parse /proc to verify process is Claude
    /// - Send signals to "test" the process
    ///
    /// Rationale: Reconnecting to processes is fragile and error-prone.
    /// Better to be explicit about what we don't know.
    async fn recover_sessions(&self) -> Result<()> {
        let persistence = self.persistence.lock().await;
        let persisted = persistence.load_state()?;
        drop(persistence); // Release lock early

        if persisted.is_empty() {
            println!("No sessions to recover");
            return Ok(());
        }

        println!("Attempting to recover {} session(s)...", persisted.len());

        let mut sessions = self.sessions.lock().await;
        let mut recovered_count = 0;
        let mut stale_count = 0;
        let mut crashed_count = 0;
        let mut orphaned_count = 0;

        for (id, mut persisted_session) in persisted {
            // Determine current status
            let status = if let Some(pid) = persisted_session.pid {
                if is_process_alive(pid) {
                    orphaned_count += 1;
                    "orphaned"
                } else {
                    crashed_count += 1;
                    "crashed"
                }
            } else {
                stale_count += 1;
                "stale"
            };

            persisted_session.status = status.to_string();

            // Reconstruct Session from PersistedSession
            let session = Session {
                id: persisted_session.id,
                working_dir: persisted_session.working_dir.clone(),
                created_at: persisted_session.created_at.clone(),
                log_path: persisted_session.log_path.clone(),
            };

            sessions.insert(id, session);
            recovered_count += 1;

            println!(
                "  • {} - {} (status: {})",
                id, persisted_session.working_dir.display(), status
            );
        }

        drop(sessions);

        // Save updated statuses back to disk
        self.save_state().await?;

        println!("\n✅ Recovered {} session(s):", recovered_count);
        if stale_count > 0 {
            println!("   • {} stale (unknown state)", stale_count);
        }
        if crashed_count > 0 {
            println!("   • {} crashed (process dead)", crashed_count);
        }
        if orphaned_count > 0 {
            println!("   • {} orphaned (process alive but not managed)", orphaned_count);
        }

        Ok(())
    }

    /// Save current session state to disk
    ///
    /// Called after:
    /// - Starting a session
    /// - Stopping a session
    /// - Recovering sessions
    ///
    /// ## Error Handling
    ///
    /// If save fails, logs error but does not crash daemon.
    /// Session continues to exist in memory, but won't survive restart.
    async fn save_state(&self) -> Result<()> {
        let sessions = self.sessions.lock().await;
        let processes = self.processes.lock().await;

        let mut persisted = HashMap::new();

        for (id, session) in sessions.iter() {
            // Get PID if process is active
            let pid = processes.get(id).and_then(|_| {
                // TODO: Extract PID from SessionProcess
                // For now, we don't track PID (would require PTY changes)
                None
            });

            let persisted_session = PersistedSession::from_session(session, pid);
            persisted.insert(*id, persisted_session);
        }

        drop(sessions);
        drop(processes);

        let persistence = self.persistence.lock().await;
        persistence.write_state(&persisted)?;

        Ok(())
    }

    /// Start a new Claude Code session in the given working directory.
    ///
    /// Returns the session ID on success.
    ///
    /// ## Persistence
    ///
    /// Session is saved to disk after successful start.
    /// If save fails, logs error but session remains active.
    pub async fn start_session(&self, working_dir: PathBuf) -> Result<Uuid> {
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
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session_id, session);
        }
        {
            let mut processes = self.processes.lock().await;
            processes.insert(session_id, process);
        }

        // Save to disk
        if let Err(e) = self.save_state().await {
            eprintln!("⚠️  Failed to save session state: {}", e);
            eprintln!("Session will be lost on daemon restart");
        }

        println!("✅ Started session {} in {:?}", session_id, working_dir);
        Ok(session_id)
    }

    /// Stop a running session by ID.
    ///
    /// This removes the session metadata and drops the PTY process,
    /// which should terminate the Claude subprocess.
    ///
    /// ## Persistence
    ///
    /// Session is removed from disk after successful stop.
    pub async fn stop_session(&self, session_id: Uuid) -> Result<()> {
        {
            let mut sessions = self.sessions.lock().await;
            if !sessions.contains_key(&session_id) {
                anyhow::bail!("Session not found: {}", session_id);
            }
            sessions.remove(&session_id);
        }
        {
            let mut processes = self.processes.lock().await;
            processes.remove(&session_id);
            // Dropping the PTY should terminate the child process
        }

        // Save to disk
        if let Err(e) = self.save_state().await {
            eprintln!("⚠️  Failed to save session state: {}", e);
        }

        println!("✅ Stopped session {}", session_id);
        Ok(())
    }

    /// List all active sessions.
    ///
    /// Returns a vector of SessionInfo structs (without PTY handles).
    ///
    /// ## Status Field
    ///
    /// - "running": Has active PTY process
    /// - "stale": Loaded from disk, no PTY (daemon restarted)
    /// - "crashed": Process was alive but died
    /// - "orphaned": Process is alive but not managed
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().await;
        let processes = self.processes.lock().await;

        sessions
            .values()
            .map(|s| {
                // Determine status based on whether we have an active process
                let status = if processes.contains_key(&s.id) {
                    "running".to_string()
                } else {
                    // Check persisted status
                    "stale".to_string()
                };

                SessionInfo {
                    id: s.id.to_string(),
                    working_dir: s.working_dir.display().to_string(),
                    created_at: s.created_at.clone(),
                    status,
                    log_path: s.log_path.display().to_string(),
                }
            })
            .collect()
    }

    /// Send input to a running session
    ///
    /// Writes the text to the session's PTY, which forwards it to Claude.
    ///
    /// ## Errors
    ///
    /// - Session not found
    /// - Session exists but no active PTY (stale/crashed)
    /// - PTY write failed
    pub async fn send_input(&self, session_id: Uuid, text: String) -> Result<()> {
        let processes = self.processes.lock().await;
        
        if let Some(process) = processes.get(&session_id) {
            // Add newline if not present
            let input = if text.ends_with('\n') {
                text
            } else {
                format!("{}\n", text)
            };
            
            process.write_input(input.as_bytes())
                .context("Failed to write to PTY")?;
            
            Ok(())
        } else {
            anyhow::bail!("Session not found or not active (no PTY handle)")
        }
    }
}
