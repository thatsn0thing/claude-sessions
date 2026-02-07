use crate::manager::SessionManager;
use std::sync::Arc;

/// The Daemon owns the SessionManager and keeps it alive.
/// 
/// Phase 1: This is just a wrapper around SessionManager.
/// Phase 2: Will add IPC server (Unix socket) for CLI/UI communication.
pub struct Daemon {
    pub manager: Arc<SessionManager>,
}

impl Daemon {
    pub fn new() -> Self {
        Daemon {
            manager: Arc::new(SessionManager::new()),
        }
    }

    /// Run the daemon (blocking).
    /// 
    /// For now, this just keeps the process alive.
    /// Later, we'll add a server loop here to handle IPC commands.
    pub fn run(&self) {
        println!("🚀 Claude Sessions daemon started");
        println!("📍 Daemon running... (Press Ctrl+C to stop)");

        // For Phase 1, we just block forever
        // In Phase 2, this will be the IPC server loop
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
