mod daemon;
mod manager;
mod pty;
mod session;

use clap::{Parser, Subcommand};
use manager::SessionManager;
use std::path::PathBuf;
use uuid::Uuid;

/// Claude Sessions - A local session manager for Claude Code
#[derive(Parser)]
#[command(name = "claude-sessions")]
#[command(about = "Manage multiple Claude Code sessions locally", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new Claude Code session in a directory
    Start {
        /// Working directory for the session
        #[arg(value_name = "DIR")]
        directory: PathBuf,
    },
    /// List all active sessions
    List,
    /// Stop a running session
    Stop {
        /// Session ID to stop
        #[arg(value_name = "SESSION_ID")]
        session_id: String,
    },
    /// Run the daemon (Phase 2 - not implemented yet)
    Daemon,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Phase 1: Create a session manager directly in the CLI process
    // Phase 2: This will communicate with a separate daemon process via IPC
    let manager = SessionManager::new();

    match cli.command {
        Commands::Start { directory } => {
            println!("🚀 Starting Claude Code session in {:?}", directory);
            let session_id = manager.start_session(directory)?;
            println!("✅ Session started: {}", session_id);
            println!("💡 Use `claude-sessions list` to see all sessions");
            println!("💡 Use `claude-sessions stop {}` to stop this session", session_id);
            
            // Keep the CLI process alive so the PTY doesn't die
            println!("\n📍 Session running... (Press Ctrl+C to stop)");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
        Commands::List => {
            let sessions = manager.list_sessions();
            if sessions.is_empty() {
                println!("No active sessions");
            } else {
                println!("📋 Active sessions:");
                for session in sessions {
                    println!("  • {} | {} | {}", 
                        session.id, 
                        session.working_dir,
                        session.created_at
                    );
                }
            }
        }
        Commands::Stop { session_id } => {
            let uuid = Uuid::parse_str(&session_id)
                .map_err(|_| anyhow::anyhow!("Invalid session ID format"))?;
            manager.stop_session(uuid)?;
            println!("✅ Session stopped: {}", session_id);
        }
        Commands::Daemon => {
            println!("⚠️  Daemon mode not implemented yet (Phase 2)");
            println!("For now, each `start` command runs its own process");
        }
    }

    Ok(())
}
