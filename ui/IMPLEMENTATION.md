# Phase 4: Desktop UI Implementation Guide

## Overview

Building a minimal Tauri + React + xterm.js desktop app to view Claude sessions.

## Architecture

```
React Frontend (TypeScript)
    ↓ (Tauri invoke)
Tauri Backend (Rust)
    ↓ (Unix Socket)
Existing Daemon (Phase 3)
```

## Step-by-Step Implementation

### 1. Complete npm install

```bash
cd ui
npm install
npm install xterm @xterm/addon-fit
```

### 2. Update Tauri Backend Dependencies

Edit `ui/src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

### 3. Create IPC Client Module

Create `ui/src-tauri/src/daemon_client.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub working_dir: String,
    pub created_at: String,
    pub status: String,
    pub log_path: String,
}

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .context("Cannot determine home directory")?;
        let socket_path = PathBuf::from(home)
            .join(".claude-sessions")
            .join("daemon.sock");
        Ok(DaemonClient { socket_path })
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let request = serde_json::json!({"type": "list_sessions"});
        let response = self.send_request(&request).await?;
        
        match response.get("type").and_then(|v| v.as_str()) {
            Some("session_list") => {
                let sessions: Vec<SessionInfo> = serde_json::from_value(
                    response.get("sessions").cloned().unwrap_or(serde_json::json!([]))
                )?;
                Ok(sessions)
            }
            Some("error") => {
                let msg = response.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                anyhow::bail!("Daemon error: {}", msg)
            }
            _ => anyhow::bail!("Unexpected response type")
        }
    }

    async fn send_request(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to daemon")?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send request
        let request_json = serde_json::to_string(request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: serde_json::Value = serde_json::from_str(&line)
            .context("Failed to parse daemon response")?;

        Ok(response)
    }
}
```

### 4. Create Tauri Commands

Edit `ui/src-tauri/src/lib.rs`:

```rust
mod daemon_client;

use daemon_client::{DaemonClient, SessionInfo};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tauri::State;

#[tauri::command]
async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let client = DaemonClient::new().map_err(|e| e.to_string())?;
    client.list_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_session_logs(log_path: String, offset: usize) -> Result<Vec<String>, String> {
    let file = File::open(&log_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    
    let lines: Vec<String> = reader
        .lines()
        .skip(offset)
        .filter_map(|line| line.ok())
        .collect();
    
    Ok(lines)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            read_session_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5. React Frontend - Session List

Create `ui/src/components/SessionList.tsx`:

```typescript
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Session {
  id: string;
  working_dir: string;
  created_at: string;
  status: string;
  log_path: string;
}

interface SessionListProps {
  onSelectSession: (session: Session) => void;
}

export function SessionList({ onSelectSession }: SessionListProps) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSessions();
    const interval = setInterval(loadSessions, 5000); // Refresh every 5s
    return () => clearInterval(interval);
  }, []);

  async function loadSessions() {
    try {
      const result = await invoke<Session[]>('list_sessions');
      setSessions(result);
      setError(null);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return <div className="session-list">Loading sessions...</div>;
  }

  if (error) {
    return (
      <div className="session-list error">
        <p>Error: {error}</p>
        <p>Make sure the daemon is running:</p>
        <code>claude-sessions daemon --foreground</code>
      </div>
    );
  }

  if (sessions.length === 0) {
    return (
      <div className="session-list empty">
        <p>No active sessions</p>
        <p>Start one with:</p>
        <code>claude-sessions start /path/to/project</code>
      </div>
    );
  }

  return (
    <div className="session-list">
      <h2>Active Sessions ({sessions.length})</h2>
      {sessions.map((session) => (
        <div
          key={session.id}
          className="session-item"
          onClick={() => onSelectSession(session)}
        >
          <div className="session-id">{session.id.substring(0, 8)}...</div>
          <div className="session-dir">{session.working_dir}</div>
          <div className="session-time">
            {new Date(session.created_at).toLocaleString()}
          </div>
        </div>
      ))}
    </div>
  );
}
```

### 6. React Frontend - Terminal Viewer

Create `ui/src/components/TerminalViewer.tsx`:

```typescript
import { useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import '@xterm/xterm/css/xterm.css';

interface Session {
  id: string;
  working_dir: string;
  log_path: string;
}

interface TerminalViewerProps {
  session: Session;
}

export function TerminalViewer({ session }: TerminalViewerProps) {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminalInstance = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon | null>(null);
  const [offset, setOffset] = useState(0);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Initialize terminal
    const term = new Terminal({
      cursorBlink: false,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
      },
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(terminalRef.current);
    fit.fit();

    terminalInstance.current = term;
    fitAddon.current = fit;

    // Handle resize
    const handleResize = () => fit.fit();
    window.addEventListener('resize', handleResize);

    // Load logs
    loadLogs();
    const interval = setInterval(loadLogs, 1000); // Poll every 1s

    return () => {
      window.removeEventListener('resize', handleResize);
      clearInterval(interval);
      term.dispose();
    };
  }, [session.id]);

  async function loadLogs() {
    if (!terminalInstance.current) return;

    try {
      const lines = await invoke<string[]>('read_session_logs', {
        logPath: session.log_path,
        offset,
      });

      if (lines.length > 0) {
        for (const line of lines) {
          try {
            const entry = JSON.parse(line);
            if (entry.direction === 'output') {
              // Decode base64 data
              const data = atob(entry.data);
              terminalInstance.current.write(data);
            }
          } catch (e) {
            console.error('Failed to parse log entry:', e);
          }
        }
        setOffset(offset + lines.length);
      }
    } catch (err) {
      console.error('Failed to read logs:', err);
    }
  }

  return (
    <div className="terminal-viewer">
      <div className="terminal-header">
        <span className="session-id">{session.id}</span>
        <span className="working-dir">{session.working_dir}</span>
      </div>
      <div ref={terminalRef} className="terminal-container" />
    </div>
  );
}
```

### 7. Main App Component

Edit `ui/src/App.tsx`:

```typescript
import { useState } from 'react';
import { SessionList } from './components/SessionList';
import { TerminalViewer } from './components/TerminalViewer';
import './App.css';

interface Session {
  id: string;
  working_dir: string;
  created_at: string;
  status: string;
  log_path: string;
}

function App() {
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);

  return (
    <div className="app">
      <aside className="sidebar">
        <h1>Claude Sessions</h1>
        <SessionList onSelectSession={setSelectedSession} />
      </aside>
      <main className="main">
        {selectedSession ? (
          <TerminalViewer session={selectedSession} />
        ) : (
          <div className="empty-state">
            <p>Select a session to view its output</p>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
```

### 8. Basic Styling

Edit `ui/src/App.css`:

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  background: #1e1e1e;
  color: #d4d4d4;
}

.app {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 300px;
  background: #252526;
  border-right: 1px solid #3e3e42;
  overflow-y: auto;
  padding: 20px;
}

.sidebar h1 {
  font-size: 18px;
  margin-bottom: 20px;
  color: #fff;
}

.session-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.session-item {
  padding: 12px;
  background: #2d2d30;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.session-item:hover {
  background: #3e3e42;
}

.session-id {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #4ec9b0;
}

.session-dir {
  margin-top: 4px;
  font-size: 13px;
  color: #d4d4d4;
}

.session-time {
  margin-top: 4px;
  font-size: 11px;
  color: #858585;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.terminal-viewer {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.terminal-header {
  background: #2d2d30;
  padding: 10px 20px;
  border-bottom: 1px solid #3e3e42;
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.terminal-container {
  flex: 1;
  padding: 10px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #858585;
}

.error {
  color: #f48771;
  padding: 20px;
}

code {
  background: #2d2d30;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
}
```

## Testing

### 1. Start the daemon

```bash
cd ..  # Back to claude-sessions root
./target/debug/claude-sessions daemon --foreground
```

### 2. Start a session (in another terminal)

```bash
./target/debug/claude-sessions start /tmp/test-project
```

### 3. Run the Tauri app

```bash
cd ui
npm run tauri dev
```

You should see:
- Session list on the left
- Terminal output on the right when you click a session

## Troubleshooting

### "Failed to connect to daemon"
- Make sure the daemon is running
- Check socket exists: `ls ~/.claude-sessions/daemon.sock`

### "No sessions shown"
- Create a session with the CLI first
- Check `claude-sessions list` works

### Terminal not rendering
- Check browser console for errors
- Verify log file exists at the log_path
- Try refreshing the app

## Next Steps

- [ ] Add session creation from UI
- [ ] Add input support (send commands to session)
- [ ] Better error handling
- [ ] Session search/filter
- [ ] Dark/light theme toggle

---

**Status**: Implementation guide complete  
**Estimated time**: 1-2 hours to implement  
**Complexity**: Medium (IPC + React + xterm.js integration)
