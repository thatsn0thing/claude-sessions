use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtyPair, PtySize};
use std::path::Path;
use std::sync::Arc;

/// Spawns a Claude Code session as a PTY subprocess.
/// 
/// Important: We treat `claude` CLI as a black box.
/// We simply spawn it in the given working directory and let it run.
pub fn spawn_claude_pty(working_dir: &Path) -> Result<PtyPair> {
    // Create a PTY pair (master + slave)
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to create PTY pair")?;

    // Build the command to spawn `claude`
    let mut cmd = CommandBuilder::new("claude");
    cmd.cwd(working_dir);

    // Spawn the process in the PTY slave
    let _child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn claude process")?;

    // Note: We return the PtyPair. The caller is responsible for:
    // - Keeping the master alive to interact with the PTY
    // - Managing the child process lifecycle
    Ok(pair)
}

/// SessionProcess holds the PTY pair for a running Claude session.
/// 
/// The PTY master allows us to read/write to the Claude subprocess.
/// We keep this alive for the lifetime of the session.
pub struct SessionProcess {
    pub pty_pair: Arc<PtyPair>,
}

impl SessionProcess {
    pub fn new(pty_pair: PtyPair) -> Self {
        SessionProcess {
            pty_pair: Arc::new(pty_pair),
        }
    }
}
