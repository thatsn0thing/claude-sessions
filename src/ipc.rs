use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// IPC Request messages sent from CLI to Daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Start a new Claude session
    StartSession {
        working_dir: PathBuf,
    },
    /// List all active sessions
    ListSessions,
    /// Stop a running session
    StopSession {
        session_id: String,
    },
    /// Send input to a running session
    SendInput {
        session_id: String,
        text: String,
    },
    /// Attach to session output stream (streaming logs)
    AttachSession {
        session_id: String,
    },
    /// Ping the daemon (health check)
    Ping,
    /// Shutdown the daemon gracefully
    Shutdown,
}

/// IPC Response messages sent from Daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Success response with session ID
    SessionStarted {
        session_id: String,
        log_path: String,
    },
    /// Success response with session list
    SessionList {
        sessions: Vec<SessionInfo>,
    },
    /// Success response for stop
    SessionStopped {
        session_id: String,
    },
    /// Streaming log chunk (for attach)
    LogChunk {
        session_id: String,
        data: String, // Base64 encoded
    },
    /// Pong response
    Pong,
    /// Generic success
    Ok,
    /// Error response
    Error {
        message: String,
    },
}

/// Session info for list responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub working_dir: String,
    pub created_at: String,
    pub status: String,
    pub log_path: String,
}

impl From<crate::session::Session> for SessionInfo {
    fn from(session: crate::session::Session) -> Self {
        SessionInfo {
            id: session.id.to_string(),
            working_dir: session.working_dir.display().to_string(),
            created_at: session.created_at,
            status: "running".to_string(),
            log_path: session.log_path.display().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request::StartSession {
            working_dir: PathBuf::from("/tmp"),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();
        
        match parsed {
            Request::StartSession { working_dir } => {
                assert_eq!(working_dir, PathBuf::from("/tmp"));
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let resp = Response::SessionStarted {
            session_id: "test-123".to_string(),
            log_path: "/tmp/test.log".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: Response = serde_json::from_str(&json).unwrap();
        
        match parsed {
            Response::SessionStarted { session_id, .. } => {
                assert_eq!(session_id, "test-123");
            }
            _ => panic!("Wrong response type"),
        }
    }
}
