# Claude Sessions

A local session manager for Claude Code (macOS only).

## Problem

Users currently open many terminal windows and run multiple Claude Code sessions manually. This tool provides a single local app to manage multiple Claude Code sessions.

## Architecture

### Phase 1 (Current)
- **CLI-only**: Direct session management via command-line interface
- **SessionManager**: Core logic to start/stop/list Claude Code sessions
- **PTY-backed**: Each session runs as a PTY subprocess
- **No daemon yet**: Each `start` command runs its own process

### Phase 2 (Future)
- **Daemon process**: Long-running background process managing all sessions
- **IPC**: Unix socket communication between CLI and daemon
- **UI**: Chat-style interface to interact with sessions

## Project Structure

```
claude-sessions/
├── Cargo.toml              # Dependencies
├── src/
│   ├── main.rs             # CLI entry point (clap)
│   ├── daemon.rs           # Daemon wrapper (Phase 2)
│   ├── manager.rs          # SessionManager (start/stop/list)
│   ├── session.rs          # Session data model
│   └── pty.rs              # PTY subprocess handling
└── README.md
```

## Core Data Models

### `Session`
- `id`: Unique UUID
- `working_dir`: Path where Claude runs
- `created_at`: Timestamp

### `SessionManager`
- `sessions`: HashMap of session metadata
- `processes`: HashMap of PTY handles
- Methods: `start_session`, `stop_session`, `list_sessions`

## Installation

```bash
cargo build --release
```

## Usage

### Start a new Claude Code session

```bash
claude-sessions start /path/to/project
```

This spawns `claude` in the specified directory as a PTY subprocess.

### List active sessions

```bash
claude-sessions list
```

Shows all running Claude Code sessions with their IDs, directories, and timestamps.

### Stop a session

```bash
claude-sessions stop <SESSION_ID>
```

Terminates the Claude Code subprocess and removes the session.

## Important Constraints

1. **Black box**: Claude CLI is NOT modified or reimplemented
2. **PTY**: Each session MUST run as a PTY subprocess
3. **macOS only**: Uses macOS-specific PTY handling (portable-pty)
4. **Local only**: No cloud, no authentication
5. **Explicit > Clever**: Code prioritizes clarity and correctness

## Dependencies

- `portable-pty`: PTY abstraction (used by WezTerm)
- `uuid`: Session IDs
- `clap`: CLI argument parsing
- `serde`/`serde_json`: Serialization
- `anyhow`: Error handling
- `chrono`: Timestamps

## Design Decisions

### Why PTY instead of stdout/stderr pipes?
Claude Code likely expects a TTY for interactive features. PTY gives us a proper terminal interface.

### Why not a daemon in Phase 1?
Keeping it simple. Each `start` command blocks and keeps the PTY alive. This works for testing and is easy to debug.

### Why Rust?
- Systems-level control over processes and PTY
- Strong type system prevents bugs
- Excellent async/concurrency support (for Phase 2)
- Native performance

## Phase 2 Roadmap

1. **Daemon mode**: Background process that owns all PTY handles
2. **IPC**: Unix socket server in daemon, CLI as thin client
3. **Session persistence**: Store session state to disk
4. **Session reconnection**: Attach/detach from running sessions
5. **UI**: Chat-style interface (separate app)

## Known Limitations (Phase 1)

- Sessions die when CLI process exits (need daemon for persistence)
- No inter-session communication
- No session history/logs
- No way to "attach" to a running session
- List command won't work across different CLI invocations

## Testing

Create a test directory and try:

```bash
# Terminal 1
claude-sessions start /tmp/test-project

# Terminal 2 (currently won't see the session from Terminal 1)
claude-sessions list

# Terminal 1
# Ctrl+C to stop
```

Once the daemon is implemented, `list` will work across terminals.

## Contributing

This is an MVP. Prefer simple, explicit code over clever abstractions.

## License

MIT
