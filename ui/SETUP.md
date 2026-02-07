# Phase 4: Desktop UI Setup Guide

## Prerequisites

### 1. Update Rust Toolchain

The Tauri app requires a newer Rust version:

```bash
# Update Rust
rustup update stable
rustup default stable

# Verify version (need 1.77.0 or newer)
cargo --version
rustc --version
```

### 2. Install System Dependencies (Linux)

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# macOS (via Homebrew) - usually already installed
brew install gtk+3
```

## Installation

### 1. Install Node Dependencies

```bash
cd ui
npm install
```

### 2. Build Tauri Backend

```bash
cd ui/src-tauri
cargo build
```

## Running the App

### Step 1: Start the Daemon

In one terminal:

```bash
cd ..  # Back to claude-sessions root
./target/debug/claude-sessions daemon --foreground
```

### Step 2: Create a Test Session

In another terminal:

```bash
./target/debug/claude-sessions start /tmp/test-project
```

### Step 3: Run the Desktop App

In a third terminal:

```bash
cd ui
npm run tauri dev
```

## What You'll See

1. **Sidebar (Left)**
   - List of active Claude sessions
   - Session ID (truncated)
   - Working directory
   - Created timestamp

2. **Main Area (Right)**
   - Terminal viewer with xterm.js
   - Live PTY output from selected session
   - Terminal-accurate rendering

## Features

✅ **List Sessions** - See all active sessions from daemon  
✅ **Select Session** - Click to view terminal output  
✅ **Live Updates** - Sessions list refreshes every 5s  
✅ **Log Streaming** - Terminal output polls every 1s  
✅ **Terminal Fidelity** - xterm.js renders ANSI codes correctly  

## Troubleshooting

### "Failed to connect to daemon"

**Problem**: UI can't reach the daemon.

**Solution**:
1. Check daemon is running: `claude-sessions status`
2. Start daemon if needed: `claude-sessions daemon --foreground`
3. Verify socket exists: `ls ~/.claude-sessions/daemon.sock`

### "No sessions shown"

**Problem**: No sessions active.

**Solution**:
1. Create a session: `claude-sessions start /tmp/test`
2. Verify with CLI: `claude-sessions list`
3. Refresh UI (should auto-refresh every 5s)

### Terminal not rendering

**Problem**: Terminal appears blank.

**Solution**:
1. Check log file exists: `ls ~/.claude-sessions/logs/*.jsonl`
2. Open browser dev tools (F12) and check console for errors
3. Try selecting another session
4. Restart the UI app

### Build errors (Rust edition2024)

**Problem**: Cargo version too old.

**Solution**:
```bash
rustup update stable
rustup default stable
cargo --version  # Should be 1.77.0+
```

## Architecture

```
┌─────────────────────────────────┐
│    Desktop UI (Tauri + React)   │
│  ┌──────────────────────────┐   │
│  │   React Frontend         │   │
│  │   • SessionList          │   │
│  │   • TerminalViewer       │   │
│  │   • xterm.js             │   │
│  └───────────┬──────────────┘   │
│              │ invoke()          │
│  ┌───────────▼──────────────┐   │
│  │   Tauri Backend (Rust)   │   │
│  │   • list_sessions()      │   │
│  │   • read_session_logs()  │   │
│  └───────────┬──────────────┘   │
└──────────────┼──────────────────┘
               │ Unix Socket
┌──────────────▼──────────────────┐
│        Daemon (Phase 3)         │
│        SessionManager           │
└─────────────────────────────────┘
```

## File Structure

```
ui/
├── src/
│   ├── App.tsx                    # Main app component
│   ├── App.css                    # Styling
│   ├── components/
│   │   ├── SessionList.tsx        # Session list sidebar
│   │   └── TerminalViewer.tsx     # xterm.js terminal
│   └── main.tsx                   # React entry point
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                 # Tauri commands
│   │   └── daemon_client.rs       # IPC to daemon
│   └── Cargo.toml                 # Rust dependencies
├── package.json                   # Node dependencies
└── SETUP.md                       # This file
```

## Development

### Hot Reload

Changes to React code will hot-reload automatically.

Changes to Rust code require:
```bash
# Kill the app (Ctrl+C)
# Rebuild
cd ui
npm run tauri dev
```

### Debugging

**React**: Open dev tools in the Tauri window (F12 or Cmd+Option+I on macOS)

**Rust**: Add `println!` statements or use `dbg!` macro

## Next Steps

### Phase 4 Extensions

- [ ] Add session creation from UI
- [ ] Add input support (send commands to PTY)
- [ ] Add search/filter for sessions
- [ ] Add split view (multiple sessions)
- [ ] Add keyboard shortcuts
- [ ] Add themes (dark/light)

### Phase 5 Ideas

- [ ] Session grouping/workspace
- [ ] Log export functionality
- [ ] Session templates
- [ ] Performance monitoring
- [ ] Remote session support

## Performance

- **UI Refresh**: 5 seconds (session list)
- **Log Polling**: 1 second (terminal output)
- **Memory**: ~50MB (UI) + ~10MB per session
- **CPU**: Minimal when idle, spikes during log polling

## Known Limitations

1. **No input support**: Read-only terminal viewer
2. **Polling-based**: Uses polling instead of true streaming
3. **Sequential log reads**: Could be optimized with file watching
4. **No history replay**: Can't seek backward in logs

---

**Status**: Implementation Complete  
**Estimated Setup Time**: 10-15 minutes  
**Difficulty**: Medium (requires Rust toolchain update)
