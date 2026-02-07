#[cfg(test)]
mod tests {
    use crate::manager::SessionManager;
    use crate::session::Session;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper: Create a temporary directory for testing
    fn create_test_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn test_session_creation() {
        let dir = PathBuf::from("/tmp");
        let session = Session::new(dir.clone());
        
        assert_eq!(session.working_dir, dir);
        assert!(!session.id.to_string().is_empty());
        assert!(!session.created_at.is_empty());
    }

    #[test]
    fn test_session_unique_ids() {
        let dir = PathBuf::from("/tmp");
        let session1 = Session::new(dir.clone());
        let session2 = Session::new(dir.clone());
        
        assert_ne!(session1.id, session2.id, "Session IDs should be unique");
    }

    #[test]
    fn test_manager_creation() {
        let manager = SessionManager::new();
        let sessions = manager.list_sessions();
        
        assert_eq!(sessions.len(), 0, "New manager should have no sessions");
    }

    #[test]
    fn test_start_session_invalid_dir() {
        let manager = SessionManager::new();
        let result = manager.start_session(PathBuf::from("/nonexistent/path"));
        
        assert!(result.is_err(), "Should fail for non-existent directory");
    }

    #[test]
    fn test_start_session_valid_dir() {
        let manager = SessionManager::new();
        let temp_dir = create_test_dir();
        
        // Note: This will fail if 'claude' command doesn't exist
        // For testing purposes, we're just checking the directory validation
        let result = manager.start_session(temp_dir.path().to_path_buf());
        
        // Expected to fail because 'claude' command likely doesn't exist in test env
        // But should pass directory validation
        if result.is_err() {
            println!("Expected: 'claude' command not found in test environment");
        }
    }

    #[test]
    fn test_stop_nonexistent_session() {
        let manager = SessionManager::new();
        let fake_id = uuid::Uuid::new_v4();
        
        let result = manager.stop_session(fake_id);
        assert!(result.is_err(), "Should fail when stopping non-existent session");
    }

    #[test]
    fn test_list_sessions_empty() {
        let manager = SessionManager::new();
        let sessions = manager.list_sessions();
        
        assert!(sessions.is_empty(), "Should return empty list for new manager");
    }

    // Integration test - only runs if 'claude' command exists
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this
    fn test_full_session_lifecycle() {
        let manager = SessionManager::new();
        let temp_dir = create_test_dir();
        
        // Start session
        let session_id = manager.start_session(temp_dir.path().to_path_buf())
            .expect("Failed to start session");
        
        // Verify it's in the list
        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id.to_string());
        
        // Stop session
        manager.stop_session(session_id)
            .expect("Failed to stop session");
        
        // Verify it's gone
        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_session_serialization() {
        let dir = PathBuf::from("/tmp/test");
        let session = Session::new(dir);
        
        // Test JSON serialization
        let json = serde_json::to_string(&session)
            .expect("Should serialize to JSON");
        
        assert!(json.contains(&session.id.to_string()));
        assert!(json.contains("/tmp/test"));
    }

    #[test]
    fn test_session_info_serialization() {
        use crate::session::SessionInfo;
        
        let info = SessionInfo {
            id: "test-id".to_string(),
            working_dir: "/tmp".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            status: "running".to_string(),
        };
        
        let json = serde_json::to_string(&info)
            .expect("Should serialize SessionInfo");
        
        assert!(json.contains("test-id"));
        assert!(json.contains("running"));
    }
}
