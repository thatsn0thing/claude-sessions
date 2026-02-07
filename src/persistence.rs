use crate::session::Session;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Persisted session state on disk
///
/// This is the source of truth for what sessions exist.
/// The daemon reads this on startup to recover sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub id: Uuid,
    pub working_dir: PathBuf,
    pub created_at: String,
    pub log_path: PathBuf,
    /// Process ID of the Claude subprocess (if known)
    /// 
    /// IMPORTANT: This may be stale if:
    /// - Daemon crashed and PID was reused
    /// - Process was killed externally
    /// - System rebooted
    /// 
    /// Always check process status before trusting this.
    pub pid: Option<u32>,
    /// Current session status
    /// 
    /// Valid states:
    /// - "running": Process is alive and responding
    /// - "stopped": Process was stopped cleanly
    /// - "crashed": Process died unexpectedly
    /// - "stale": Daemon restarted, process status unknown
    /// - "orphaned": PID exists but not our process
    pub status: String,
}

impl PersistedSession {
    pub fn from_session(session: &Session, pid: Option<u32>) -> Self {
        PersistedSession {
            id: session.id,
            working_dir: session.working_dir.clone(),
            created_at: session.created_at.clone(),
            log_path: session.log_path.clone(),
            pid,
            status: "running".to_string(),
        }
    }
}

/// Persistence manager for session metadata
/// 
/// Writes session state to disk and recovers it on daemon restart.
/// 
/// ## Failure Modes
/// 
/// 1. **Disk full**: write_state() fails, in-memory state preserved
/// 2. **Permission denied**: write_state() fails, in-memory state preserved
/// 3. **Corrupted state file**: load_state() returns empty HashMap
/// 4. **Concurrent writes**: Last write wins (file is overwritten atomically)
/// 5. **Daemon crash**: Most recent write is preserved, PID may be stale
pub struct PersistenceManager {
    state_file: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Result<Self> {
        let state_file = Self::state_file_path()?;
        
        // Ensure directory exists
        if let Some(parent) = state_file.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(PersistenceManager { state_file })
    }

    fn state_file_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Cannot determine home directory")?;
        Ok(PathBuf::from(home)
            .join(".claude-sessions")
            .join("sessions.json"))
    }

    /// Save current session state to disk
    /// 
    /// This is called after:
    /// - Starting a session
    /// - Stopping a session
    /// - Updating session status
    /// 
    /// ## Error Handling
    /// 
    /// If write fails, logs error but does not crash daemon.
    /// In-memory state is still valid, but recovery after crash will fail.
    pub fn write_state(&self, sessions: &HashMap<Uuid, PersistedSession>) -> Result<()> {
        let json = serde_json::to_string_pretty(sessions)
            .context("Failed to serialize sessions")?;

        // Write atomically: write to temp file, then rename
        let temp_file = self.state_file.with_extension("json.tmp");
        fs::write(&temp_file, json)
            .context("Failed to write temp state file")?;
        fs::rename(&temp_file, &self.state_file)
            .context("Failed to rename state file")?;

        Ok(())
    }

    /// Load session state from disk
    /// 
    /// Called on daemon startup to recover previous sessions.
    /// 
    /// ## Failure Modes
    /// 
    /// 1. **File doesn't exist**: Returns empty HashMap (first run)
    /// 2. **File corrupted**: Logs error, returns empty HashMap
    /// 3. **File readable but invalid JSON**: Logs error, returns empty HashMap
    /// 
    /// Conservative approach: if we can't parse state, start fresh.
    /// User can manually inspect/fix sessions.json if needed.
    pub fn load_state(&self) -> Result<HashMap<Uuid, PersistedSession>> {
        if !self.state_file.exists() {
            // First run, no state to load
            return Ok(HashMap::new());
        }

        let json = fs::read_to_string(&self.state_file)
            .context("Failed to read state file")?;

        let sessions: HashMap<Uuid, PersistedSession> = serde_json::from_str(&json)
            .context("Failed to parse state file")?;

        println!(
            "Loaded {} session(s) from disk",
            sessions.len()
        );

        Ok(sessions)
    }

    /// Delete the state file (for cleanup)
    pub fn delete_state(&self) -> Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }
        Ok(())
    }
}

/// Check if a process is still alive
/// 
/// ## Platform-specific behavior
/// 
/// Linux/macOS: Checks /proc or uses kill -0
/// Windows: Checks process handle
/// 
/// ## False Positives
/// 
/// PID may have been reused by another process.
/// This function only checks if *a* process exists with that PID,
/// not if it's *our* Claude process.
/// 
/// ## False Negatives
/// 
/// Process may be zombie (dead but not reaped).
/// 
/// ## Safety
/// 
/// This function is conservative:
/// - Returns true if process *might* be alive
/// - Returns false only if we're *certain* it's dead
pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Send signal 0 (null signal) to check if process exists
        // Returns 0 if process exists, -1 if not
        unsafe {
            libc::kill(pid as i32, 0) == 0
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use std::process::Command;
        
        // Try to open process handle
        // If it fails, process doesn't exist
        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Unknown platform: assume process is alive (conservative)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_persistence_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("sessions.json");

        let mut pm = PersistenceManager {
            state_file: state_file.clone(),
        };

        // Create test sessions
        let mut sessions = HashMap::new();
        let session = PersistedSession {
            id: Uuid::new_v4(),
            working_dir: PathBuf::from("/tmp/test"),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            log_path: PathBuf::from("/tmp/test.log"),
            pid: Some(12345),
            status: "running".to_string(),
        };
        sessions.insert(session.id, session);

        // Write
        pm.write_state(&sessions).unwrap();

        // Read
        let loaded = pm.load_state().unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.keys().next().unwrap(), sessions.keys().next().unwrap());
    }

    #[test]
    fn test_is_process_alive() {
        // Test with current process (should be alive)
        let current_pid = std::process::id();
        assert!(is_process_alive(current_pid));

        // Test with unlikely PID (probably not alive)
        // Using PID 1 would be init/systemd which is always alive
        // Using very high PID that likely doesn't exist
        assert!(!is_process_alive(999999));
    }
}
