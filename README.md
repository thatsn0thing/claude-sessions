# Claude Sessions

A local session manager for Claude Code with daemon architecture and IPC.

## 🎯 What It Does

Manage multiple long-lived Claude Code sessions from a single interface:
- **Daemon** runs in the background managing all sessions
- **CLI** is a thin client over Unix sockets
- **Logging** captures all PTY I/O to structured JSONL files
- **macOS** compatible (Linux/BSD should work too)

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│      Daemon Process                 │
│  ┌───────────────────────────────┐  │
│  │   SessionManager              │  │
│  │   • Sessions (metadata)       │  │
│  │   • Processes (PTY handles)   │  │
│  │   • Logging (per-session)     │  │
│  └───────────────────────────────┘  │
│              │                       │
│  ┌───────────▼────────────┐         │
│  │   IPC Server           │         │
│  │   (Unix Socket)        │         │
│  └────────────────────────┘         │
└───────────┬────────────────────────┘
            │
   ~/.claude-sessions/daemon.sock
            │
┌───────────▼──────────┐
│   CLI (Client)       │
│   • start            │
│   • list             │
│   • stop             │
│   • attach           │
└──────────────────────┘
```

## 🚀 Quick Start

### 1. Build

```bash
cargo build --release
```

### 2. Start the Daemon

```bash
# Start daemon in foreground (for testing)
./target/release/claude-sessions daemon --foreground

# Or run in background (using nohup)
nohup ./target/release/claude-sessions daemon --foreground > /tmp/daemon.log 2>&1 &
```

### 3. Manage Sessions

```bash
# Check daemon status
./target/release/claude-sessions status

# Start a new session
./target/release/claude-sessions start /path/to/project

# List all sessions
./target/release/claude-sessions list

# Stop a session
./target/release/claude-sessions stop <session-id>

# Stop the daemon
./target/release/claude-sessions stop-daemon
```

## 📋 Commands

### Daemon Management

| Command | Description |
|---------|-------------|
| `daemon [--foreground]` | Start the daemon process |
| `status` | Check if daemon is running |
| `stop-daemon` | Shutdown the daemon gracefully |

### Session Management

| Command | Description |
|---------|-------------|
| `start <dir>` | Start a Claude session in directory |
| `list` | List all active sessions |
| `stop <id>` | Stop a running session |
| `attach <id>` | Attach to session logs (TODO) |

## 📂 File Structure

```
~/.claude-sessions/
├── daemon.sock           # IPC Unix socket
└── logs/
    ├── <uuid-1>.jsonl    # Session 1 logs
    ├── <uuid-2>.jsonl    # Session 2 logs
    └── ...
```

## 📝 Log Format

Logs are stored as **JSON Lines** (newline-delimited JSON):

```json
{"timestamp":"2026-02-07T18:15:32.123Z","session_id":"abc-123...","direction":"output","data":"SGVsbG8=","size":5}
{"timestamp":"2026-02-07T18:15:35.456Z","session_id":"abc-123...","direction":"input","data":"Y2xhdWRl","size":6}
```

**Fields:**
- `timestamp`: RFC3339 UTC timestamp
- `session_id`: Session UUID
- `direction`: `"input"` (user) or `"output"` (Claude)
- `data`: Base64-encoded raw bytes (PTY I/O)
- `size`: Byte count

### Viewing Logs

```bash
# Raw logs
cat ~/.claude-sessions/logs/<session-id>.jsonl

# Pretty-print with jq
jq '.' ~/.claude-sessions/logs/<session-id>.jsonl

# Filter only Claude output
jq 'select(.direction == "output")' ~/.claude-sessions/logs/<session-id>.jsonl

# Decode data
jq -r '.data' ~/.claude-sessions/logs/<session-id>.jsonl | base64 -d
```

## 🔧 Development

### Build & Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run with ignored tests (requires `claude` command)
cargo test -- --ignored
```

### Test Results

```
running 18 tests
✓ 16 passed
○ 2 ignored (integration tests)
```

### Project Structure

```
src/
├── main.rs          # CLI entry point (client)
├── daemon.rs        # Daemon server
├── client.rs        # IPC client
├── ipc.rs           # Protocol definitions
├── manager.rs       # SessionManager (async)
├── pty.rs           # PTY spawning & I/O
├── logging.rs       # Log format & writer
├── session.rs       # Session data models
└── tests.rs         # Test suite
```

## 📚 Documentation

- **`README.md`** - This file (overview)
- **`ARCHITECTURE.md`** - Design decisions
- **`PHASE2.md`** - PTY streaming & logging
- **`PHASE3.md`** - Daemon & IPC architecture
- **`TESTING.md`** - Test coverage
- **`SUMMARY.md`** - Complete feature summary

## 🎨 Features

### ✅ Implemented

- [x] **Phase 1**: Basic session management
- [x] **Phase 2**: PTY streaming & logging
- [x] **Phase 3**: Daemon + IPC architecture
- [x] Structured JSONL logging
- [x] Non-blocking async I/O
- [x] Unix socket IPC
- [x] Session lifecycle management
- [x] Comprehensive test suite

### 🚧 TODO

- [ ] Proper daemonization (fork + detach)
- [ ] Real-time log streaming (`attach` command)
- [ ] Session persistence across daemon restarts
- [ ] Terminal UI (TUI)
- [ ] Log replay tool
- [ ] Session grouping/tagging
- [ ] Resource limits per session

## 🔒 Security

### Socket Permissions

- Unix socket: `~/.claude-sessions/daemon.sock`
- Inherits user permissions (no world access)
- Local-only (no network exposure)

### Log Privacy

⚠️ **Logs may contain sensitive data:**
- API keys
- Passwords
- Private code

**Recommendations:**
- Restrict permissions: `chmod 600 ~/.claude-sessions/logs/*`
- Add `.claude-sessions/` to `.gitignore`
- Consider log encryption at rest
- Implement auto-delete policy

## 🐛 Troubleshooting

### Daemon won't start

```bash
# Check if socket exists (stale)
ls ~/.claude-sessions/daemon.sock

# Remove and retry
rm ~/.claude-sessions/daemon.sock
claude-sessions daemon --foreground
```

### CLI says "Daemon not running"

```bash
# Check status
claude-sessions status

# Start daemon
claude-sessions daemon --foreground
```

### Session won't start

```bash
# Ensure `claude` command exists
which claude

# Check Claude is authenticated
claude --version
```

## 📦 Dependencies

```toml
[dependencies]
uuid = { version = "1.6", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
portable-pty = "0.8"              # PTY abstraction
clap = { version = "4.4", features = ["derive"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.21"                   # Log encoding
tokio = { version = "1.35", features = ["full"] }  # Async runtime
```

## 🤝 Contributing

This is an MVP. Contributions welcome!

**Guidelines:**
- Prefer simple, explicit code
- Follow existing architecture
- Add tests for new features
- Update documentation

## 📜 License

MIT

## 🙏 Credits

- Built with [portable-pty](https://github.com/wez/wezterm/tree/main/pty) (WezTerm)
- Inspired by tmux, screen, and other terminal multiplexers
- Claude Code by Anthropic

---

**Status**: Phase 3 Complete ✅  
**Version**: 0.1.0  
**Last Updated**: February 7, 2026  
**Author**: Nitanshu Lokhande
