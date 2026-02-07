use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a single Claude Code session.
/// 
/// Each session has:
/// - A unique ID (UUID)
/// - A working directory where `claude` runs
/// - A log file path for capturing PTY I/O
/// - A reference to the PTY subprocess (stored separately by the manager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub working_dir: PathBuf,
    pub created_at: String,
    pub log_path: PathBuf,
}

impl Session {
    /// Create a new session for a given working directory
    pub fn new(working_dir: PathBuf) -> Self {
        let id = Uuid::new_v4();
        let log_path = Self::log_path_for_session(id);
        
        Session {
            id,
            working_dir,
            created_at: chrono::Utc::now().to_rfc3339(),
            log_path,
        }
    }

    /// Get the log file path for a session
    fn log_path_for_session(session_id: Uuid) -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| String::from("."));
        PathBuf::from(home)
            .join(".claude-sessions")
            .join("logs")
            .join(format!("{}.jsonl", session_id))
    }
}

/// Session metadata for list operations (without PTY handles)
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub working_dir: String,
    pub created_at: String,
    pub status: String,
    pub log_path: String,
}
