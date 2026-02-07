# Phase 2: PTY Streaming & Logging

## ✅ Completed Implementation

Phase 2 adds comprehensive PTY I/O capture and logging to disk for all Claude Code sessions.

## What Was Added

### 1. **Logging Module** (`src/logging.rs`)

Handles structured logging of all PTY I/O:

```rust
pub struct LogEntry {
    timestamp: String,      // RFC3339 UTC timestamp
    session_id: String,     // Session UUID
    direction: Direction,   // "input" or "output"
    data: Vec<u8>,          // Raw bytes (base64 encoded in JSON)
    size: usize,            // Byte count
}
```

**Direction enum:**
- `Input`: User → Claude (keyboard input, commands)
- `Output`: Claude → User (terminal output, responses)

### 2. **SessionLogger**

Manages log files per session:

- **Log location**: `~/.claude-sessions/logs/<session_id>.jsonl`
- **Format**: JSON Lines (newline-delimited JSON)
- **Encoding**: Base64 for binary safety
- **Flushing**: Immediate (every write is flushed)

### 3. **Async PTY Reader**

Background task per session that:
- Continuously reads PTY output
- Logs each chunk immediately
- Handles graceful shutdown
- Runs in a blocking thread (via `tokio::task::spawn_blocking`)

### 4. **Updated SessionProcess**

Now manages:
- PTY pair
- Background output reader task
- Shutdown signaling
- Input logging (via `write_input` method)

## Log Format

### JSONL (JSON Lines)

Each log entry is a single-line JSON object:

```json
{"timestamp":"2026-02-07T18:15:32.123Z","session_id":"a1b2c3d4-...","direction":"output","data":"SGVsbG8sIHdvcmxkIQ==","size":13}
{"timestamp":"2026-02-07T18:15:35.456Z","session_id":"a1b2c3d4-...","direction":"input","data":"Y2xhdWRlIGhlbHA=","size":11}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | string | RFC3339 UTC timestamp (ISO 8601) |
| `session_id` | string | UUID of the session |
| `direction` | string | `"input"` or `"output"` |
| `data` | string | Base64-encoded raw bytes |
| `size` | integer | Number of bytes (before encoding) |

### Why Base64?

- **Binary safety**: PTY output may contain non-UTF8 bytes (control codes, colors)
- **JSON compatibility**: No escaping issues
- **Replayable**: Can decode and write raw bytes to a terminal

## Example: Reading Logs

```bash
# View logs for a session
cat ~/.claude-sessions/logs/<session-id>.jsonl

# Decode a single entry
jq -r '.data' <<< '{"data":"SGVsbG8="}' | base64 -d
# Output: Hello

# Filter only output (Claude responses)
jq 'select(.direction == "output")' ~/.claude-sessions/logs/<session-id>.jsonl

# Count input vs output
jq -s 'group_by(.direction) | map({direction: .[0].direction, count: length})' ~/.claude-sessions/logs/<session-id>.jsonl
```

## Performance Characteristics

### Memory
- **Buffer size**: 8KB per read
- **Log file**: Append-only, unbuffered
- **Overhead**: Minimal (just serialization + disk I/O)

### Latency
- **PTY responsiveness**: Not blocked by logging
- **Async design**: Output reader runs in separate thread
- **Flush policy**: Immediate (tradeoff: reliability over throughput)

### Disk Usage
- **Raw logs**: Typical session ~1-10 MB
- **Base64 overhead**: +33% size vs raw bytes
- **Compression**: Could be added later (gzip jsonl files)

## Architecture

```
┌─────────────────────┐
│  SessionProcess     │
│  ┌───────────────┐  │
│  │   PTY Pair    │  │
│  │  Master/Slave │  │
│  └───────┬───────┘  │
│          │          │
│  ┌───────▼───────┐  │
│  │ Output Reader │  │  (async task)
│  │   Thread      │  │
│  └───────┬───────┘  │
│          │          │
│  ┌───────▼───────┐  │
│  │ SessionLogger │  │
│  │  Write JSONL  │  │
│  └───────────────┘  │
└─────────────────────┘
          │
          ▼
~/.claude-sessions/logs/<uuid>.jsonl
```

## Testing

### Unit Tests

```bash
cargo test
```

Tests cover:
- LogEntry serialization/deserialization
- Direction enum JSON format
- Base64 encoding round-trip

### Manual Testing

```bash
# Start a session
./target/debug/claude-sessions start /tmp/test-project

# In another terminal, tail the logs
tail -f ~/.claude-sessions/logs/*.jsonl

# Interact with Claude, watch logs appear in real-time
```

## Known Limitations

1. **No log rotation**: Files grow unbounded (add max size later)
2. **No compression**: Logs are stored raw (could gzip old files)
3. **No replay tool**: Logs are replayable format, but no CLI tool yet
4. **No search**: Logs are append-only, no indexing
5. **Blocking I/O**: File writes are synchronous (could use async I/O)

## Future Enhancements

### Short-term
- [ ] Add log rotation (max size per file)
- [ ] Compress old log files (gzip)
- [ ] Add session metadata to log file header

### Medium-term
- [ ] Build `claude-sessions replay <session-id>` command
- [ ] Add search/filter CLI: `claude-sessions logs <session-id> --grep "error"`
- [ ] Session export: combine logs + metadata into archive

### Long-term
- [ ] Real-time log streaming API
- [ ] Web UI with log viewer
- [ ] Log analytics dashboard

## Security Considerations

- **Sensitive data**: Logs may contain API keys, passwords, secrets
- **Permissions**: Log files should be user-readable only (`chmod 600`)
- **Retention**: Add auto-delete policy for old logs
- **Encryption**: Consider encrypting logs at rest

## Migration from Phase 1

No breaking changes:
- Existing SessionManager API unchanged
- Sessions still work without reading logs
- Logs are opt-in feature (automatic, but not required)

## Dependencies Added

- `base64 = "0.21"`: Base64 encoding for binary-safe JSON
- `tokio = { version = "1.35", features = ["full"] }`: Async runtime

## Files Modified

- `src/logging.rs`: New module
- `src/pty.rs`: Added async reader, input logging
- `src/session.rs`: Added `log_path` field
- `src/manager.rs`: Updated `SessionProcess::new` call
- `src/main.rs`: Added `logging` module, made `main` async
- `Cargo.toml`: Added dependencies

---

**Status**: ✅ Implemented and tested  
**Build**: `cargo build` successful  
**Next**: Phase 3 (Daemon + IPC) or UI development
