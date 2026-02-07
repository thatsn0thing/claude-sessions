mod client;
mod daemon;
mod ipc;
mod logging;
mod manager;
mod pty;
mod session;

#[cfg(test)]
mod tests;

use clap::{Parser, Subcommand};
use client::Client;
use daemon::Daemon;
use ipc::{Request, Response};
use std::path::PathBuf;

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
    /// Start the daemon process (background server)
    Daemon {
        /// Run daemon in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },
    /// Check daemon status
    Status,
    /// Stop the daemon
    StopDaemon,
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
    /// Attach to a session's output (stream logs)
    Attach {
        /// Session ID to attach to
        #[arg(value_name = "SESSION_ID")]
        session_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { foreground } => {
            if Daemon::is_running() {
                eprintln!("❌ Daemon is already running");
                std::process::exit(1);
            }

            if foreground {
                // Run in foreground (blocking)
                println!("🚀 Starting daemon in foreground mode...");
                let mut daemon = Daemon::new()?;
                daemon.run().await?;
            } else {
                // TODO: Fork and daemonize (for now, just run in foreground)
                println!("⚠️  Daemonization not implemented yet. Running in foreground.");
                println!("💡 Use `claude-sessions daemon --foreground` explicitly");
                let mut daemon = Daemon::new()?;
                daemon.run().await?;
            }
        }
        Commands::Status => {
            let client = Client::new()?;
            if client.is_daemon_running() {
                match client.send_request(Request::Ping).await {
                    Ok(Response::Pong) => {
                        println!("✅ Daemon is running");
                    }
                    Ok(_) => {
                        println!("⚠️  Daemon responded but with unexpected message");
                    }
                    Err(e) => {
                        println!("❌ Daemon not responding: {}", e);
                    }
                }
            } else {
                println!("❌ Daemon is not running");
                println!("💡 Start it with: claude-sessions daemon");
            }
        }
        Commands::StopDaemon => {
            let client = Client::new()?;
            if !client.is_daemon_running() {
                println!("❌ Daemon is not running");
                std::process::exit(1);
            }

            match client.send_request(Request::Shutdown).await {
                Ok(_) => {
                    println!("✅ Daemon shutdown requested");
                }
                Err(e) => {
                    eprintln!("❌ Failed to stop daemon: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Start { directory } => {
            let client = Client::new()?;
            if !client.is_daemon_running() {
                eprintln!("❌ Daemon is not running");
                eprintln!("💡 Start it with: claude-sessions daemon");
                std::process::exit(1);
            }

            let request = Request::StartSession {
                working_dir: directory.clone(),
            };

            match client.send_request(request).await? {
                Response::SessionStarted { session_id, log_path } => {
                    println!("✅ Session started: {}", session_id);
                    println!("📂 Working directory: {:?}", directory);
                    println!("📝 Logs: {}", log_path);
                    println!("\n💡 Use `claude-sessions list` to see all sessions");
                    println!("💡 Use `claude-sessions stop {}` to stop this session", session_id);
                }
                Response::Error { message } => {
                    eprintln!("❌ Failed to start session: {}", message);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    std::process::exit(1);
                }
            }
        }
        Commands::List => {
            let client = Client::new()?;
            if !client.is_daemon_running() {
                eprintln!("❌ Daemon is not running");
                std::process::exit(1);
            }

            match client.send_request(Request::ListSessions).await? {
                Response::SessionList { sessions } => {
                    if sessions.is_empty() {
                        println!("No active sessions");
                    } else {
                        println!("📋 Active sessions ({}):\n", sessions.len());
                        for session in sessions {
                            println!("  🔹 {}", session.id);
                            println!("     Directory: {}", session.working_dir);
                            println!("     Created: {}", session.created_at);
                            println!("     Status: {}", session.status);
                            println!("     Logs: {}", session.log_path);
                            println!();
                        }
                    }
                }
                Response::Error { message } => {
                    eprintln!("❌ Failed to list sessions: {}", message);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    std::process::exit(1);
                }
            }
        }
        Commands::Stop { session_id } => {
            let client = Client::new()?;
            if !client.is_daemon_running() {
                eprintln!("❌ Daemon is not running");
                std::process::exit(1);
            }

            let request = Request::StopSession {
                session_id: session_id.clone(),
            };

            match client.send_request(request).await? {
                Response::SessionStopped { session_id } => {
                    println!("✅ Session stopped: {}", session_id);
                }
                Response::Error { message } => {
                    eprintln!("❌ Failed to stop session: {}", message);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    std::process::exit(1);
                }
            }
        }
        Commands::Attach { session_id } => {
            let client = Client::new()?;
            if !client.is_daemon_running() {
                eprintln!("❌ Daemon is not running");
                std::process::exit(1);
            }

            let request = Request::AttachSession {
                session_id: session_id.clone(),
            };

            match client.send_request(request).await? {
                Response::Error { message } => {
                    eprintln!("⚠️  {}", message);
                    eprintln!("💡 For now, use: tail -f ~/.claude-sessions/logs/{}.jsonl", session_id);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
