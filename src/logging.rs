use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::Result;

/// Direction of PTY data flow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Input,  // User → Claude
    Output, // Claude → User
}

/// A single log entry capturing PTY I/O
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in RFC3339 format (UTC)
    pub timestamp: String,
    /// Session UUID
    pub session_id: String,
    /// Direction of data flow
    pub direction: Direction,
    /// Raw bytes (base64 encoded for safe JSON storage)
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
    /// Optional: size of data in bytes
    pub size: usize,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(session_id: Uuid, direction: Direction, data: Vec<u8>) -> Self {
        let size = data.len();
        LogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: session_id.to_string(),
            direction,
            data,
            size,
        }
    }
}

/// Custom serde module for base64 encoding/decoding
mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{Engine as _, engine::general_purpose};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = general_purpose::STANDARD.encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(s.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

/// Session logger - handles writing log entries to disk
pub struct SessionLogger {
    session_id: Uuid,
    log_file: File,
    log_path: PathBuf,
}

impl SessionLogger {
    /// Create a new session logger
    /// 
    /// Log files are stored in: `~/.claude-sessions/logs/<session_id>.jsonl`
    pub fn new(session_id: Uuid) -> Result<Self> {
        let log_dir = Self::log_directory()?;
        std::fs::create_dir_all(&log_dir)?;

        let log_path = log_dir.join(format!("{}.jsonl", session_id));
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(SessionLogger {
            session_id,
            log_file,
            log_path,
        })
    }

    /// Get the log directory path
    fn log_directory() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))?;
        Ok(PathBuf::from(home).join(".claude-sessions").join("logs"))
    }

    /// Log an entry (non-blocking write)
    pub fn log(&mut self, direction: Direction, data: Vec<u8>) -> Result<()> {
        let entry = LogEntry::new(self.session_id, direction, data);
        let json = serde_json::to_string(&entry)?;
        writeln!(self.log_file, "{}", json)?;
        // Flush to ensure data is written immediately
        self.log_file.flush()?;
        Ok(())
    }

    /// Get the path to this session's log file
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry::new(
            Uuid::new_v4(),
            Direction::Output,
            b"Hello, world!".to_vec(),
        );

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: LogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.session_id, parsed.session_id);
        assert_eq!(entry.size, parsed.size);
        assert_eq!(entry.data, parsed.data);
    }

    #[test]
    fn test_direction_serialization() {
        let input_json = serde_json::to_string(&Direction::Input).unwrap();
        let output_json = serde_json::to_string(&Direction::Output).unwrap();

        assert_eq!(input_json, "\"input\"");
        assert_eq!(output_json, "\"output\"");
    }
}
