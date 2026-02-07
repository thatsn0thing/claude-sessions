# Architecture & Design Decisions

## Overview

This is a **local-only** session manager for Claude Code. The goal is to manage multiple long-lived Claude Code sessions from a single interface.

## Design Principles

1. **Treat Claude as a black box**: We don't modify or reimplement the `claude` CLI
2. **PTY-backed sessions**: Each session runs in a proper PTY for full terminal emulation
3. **Local-only**: No cloud, no authentication, no network dependencies
4. **Explicit > Clever**: Code prioritizes clarity and maintainability
5. **macOS first**: Using macOS-compatible PTY handling (portable-pty)

## Data Flow (Phase 1)

```
User → CLI (clap) → SessionManager → PTY spawn → Claude subprocess
                         ↓
                     Sessions HashMap
                     Processes HashMap
```

## Core Components

### 1. Session (session.rs)
**Purpose**: Data model for session metadata

**Fields**:
- `id`: UUID (unique identifier)
- `working_dir`: PathBuf (where Claude runs)
- `created_at`: String (RFC3339 timestamp)

**Design choice**: Keep it simple. Just the essential metadata. No state machine, no complex lifecycle.

### 2. SessionProcess (pty.rs)
**Purpose**: Wrapper around PTY pair

**Fields**:
- `pty_pair`: Arc<PtyPair> (shared PTY handle)

**Design choice**: Use `portable-pty` which is battle-tested (WezTerm uses it). Arc allows future sharing across threads.

### 3. SessionManager (manager.rs)
**Purpose**: Core business logic

**State**:
- `sessions`: Arc<Mutex<HashMap<Uuid, Session>>>
- `processes`: Arc<Mutex<HashMap<Uuid, SessionProcess>>>

**Methods**:
- `start_session(dir)` → Result<Uuid>
- `stop_session(id)` → Result<()>
- `list_sessions()` → Vec<SessionInfo>

**Design choices**:
- Two separate HashMaps: metadata vs PTY handles
- Arc<Mutex<T>> for thread-safe access (needed for Phase 2 daemon)
- Dropping SessionProcess terminates the child (RAII cleanup)

### 4. CLI (main.rs)
**Purpose**: User interface

**Commands**:
- `start <dir>`: Spawn new session, block to keep PTY alive
- `list`: Show all sessions (within same CLI process only, for now)
- `stop <id>`: Terminate session
- `daemon`: Placeholder for Phase 2

**Design choice**: Phase 1 keeps it simple - each `start` is its own process. This makes debugging easy and avoids IPC complexity.

## Why These Technologies?

### Rust
- Systems-level control (process, PTY, signals)
- Memory safety without GC overhead
- Excellent concurrency (for Phase 2)
- Strong typing prevents many classes of bugs

### portable-pty
- Cross-platform PTY abstraction
- Used by production terminal emulators (WezTerm)
- Handles macOS-specific quirks

### clap
- Standard Rust CLI framework
- Derives CLI from structs (less boilerplate)
- Good error messages

### UUID
- Universally unique session IDs
- No collision risk
- Easy to copy/paste

## Known Limitations (Phase 1)

1. **No persistence**: Sessions exist only while CLI process runs
2. **No daemon**: Can't list sessions from other CLI invocations
3. **No attach/detach**: Can't reconnect to running session
4. **No I/O capture**: PTY output isn't logged or stored
5. **No session communication**: Sessions are isolated

These are **intentional tradeoffs** for Phase 1 simplicity.

## Phase 2: Daemon Architecture

```
┌─────────────┐
│  UI (TUI)   │
└──────┬──────┘
       │ IPC (Unix socket)
┌──────▼──────────────────┐
│  Daemon Process         │
│  ┌─────────────────┐    │
│  │ SessionManager  │    │
│  │  • Sessions     │    │
│  │  • PTY handles  │    │
│  └─────────────────┘    │
│          │              │
│  ┌───────▼────────┐     │
│  │ IPC Server     │     │
│  │ (Unix socket)  │     │
│  └────────────────┘     │
└─────────────────────────┘
         │
    ┌────▼────┐ ┌────────┐
    │ Claude  │ │ Claude │ ...
    └─────────┘ └────────┘
```

**Changes needed**:
1. Daemon runs as background process
2. CLI becomes thin client (sends commands via IPC)
3. Sessions persist as long as daemon runs
4. Need session state serialization/deserialization
5. Need PTY multiplexing (read/write to multiple PTYs)

## Security Considerations

### Phase 1
- Local-only: No network attack surface
- PTY isolation: Sessions can't see each other
- No authentication: Single-user assumption

### Phase 2
- Unix socket permissions: Only user can connect
- No remote access: Still local-only
- Consider sandboxing Claude subprocesses (future)

## Performance Considerations

### Phase 1
- Minimal overhead: Direct PTY spawning
- Memory: ~1-2MB per session (PTY buffers)
- CPU: Negligible (just subprocess management)

### Phase 2
- IPC latency: <1ms (Unix socket is fast)
- Memory: Daemon holds all PTY handles
- Need async I/O to handle multiple PTYs efficiently

## Testing Strategy

Phase 1:
- Manual testing: `start`, `list`, `stop`
- Verify PTY works: Input/output to Claude
- Verify cleanup: Session processes actually die

Phase 2:
- Unit tests: SessionManager logic
- Integration tests: Daemon + CLI communication
- Stress tests: 10+ concurrent sessions

## Future Enhancements (Post Phase 2)

1. **Session history**: Log all PTY I/O to files
2. **Session replay**: Rerun previous session commands
3. **Session sharing**: Multiple UIs attach to same session
4. **Session templates**: Predefined session configs
5. **Resource limits**: CPU/memory caps per session
6. **Health checks**: Auto-restart failed sessions
7. **Remote sessions**: SSH tunneling for remote Claude instances

## Questions & Design Tradeoffs

### Q: Why not use tmux/screen?
A: We want programmatic control and a custom UI. Terminal multiplexers are user-facing tools.

### Q: Why Rust instead of Go/Python?
A: Rust gives us:
- Zero-cost abstractions
- No GC pauses
- Better FFI (if we need to call C libraries)
- Stronger type system

### Q: Why UUID instead of sequential IDs?
A: UUID:
- No collision risk across machines (future remote sessions?)
- No need for central ID allocator
- Easy to generate

### Q: Why separate metadata and process HashMaps?
A: Separation of concerns:
- Metadata is serializable (for disk persistence)
- PTY handles are not (kernel resources)
- Makes it clear what needs to persist vs what's runtime-only

---

**Last updated**: Phase 1 implementation
**Status**: MVP complete, ready for testing
