use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a single Claude Code session.
/// 
/// Each session has:
/// - A unique ID (UUID)
/// - A working directory where `claude` runs
/// - A reference to the PTY subprocess (stored separately by the manager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub working_dir: PathBuf,
    pub created_at: String,
}

impl Session {
    /// Create a new session for a given working directory
    pub fn new(working_dir: PathBuf) -> Self {
        Session {
            id: Uuid::new_v4(),
            working_dir,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Session metadata for list operations (without PTY handles)
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub working_dir: String,
    pub created_at: String,
    pub status: String,
}
