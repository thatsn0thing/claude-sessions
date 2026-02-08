/**
 * API client that works in both Tauri and browser contexts
 */

// Check if we're running in Tauri or browser
const isTauri = typeof (window as any).__TAURI_INTERNALS__ !== 'undefined';

// API base URL for browser mode
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3030/api';

interface Session {
  id: string;
  working_dir: string;
  created_at: string;
  status: string;
  log_path: string;
}

interface LogEntry {
  timestamp: string;
  event_type: string;
  content?: string;
}

/**
 * List all active sessions
 */
export async function listSessions(): Promise<Session[]> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<Session[]>('list_sessions');
  } else {
    const response = await fetch(`${API_BASE}/sessions`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }
}

/**
 * Create a new session
 */
export async function createSession(workingDir: string): Promise<{ id: string }> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<{ id: string }>('create_session', { workingDir });
  } else {
    const response = await fetch(`${API_BASE}/sessions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ working_dir: workingDir })
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }
}

/**
 * Delete/stop a session
 */
export async function deleteSession(sessionId: string): Promise<void> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('delete_session', { sessionId });
  } else {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}`, {
      method: 'DELETE'
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
  }
}

/**
 * Get session logs
 */
export async function getSessionLogs(sessionId: string): Promise<LogEntry[]> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<LogEntry[]>('get_session_logs', { sessionId });
  } else {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}/logs`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();
    return data.logs || [];
  }
}

/**
 * Send input to a session
 */
export async function sendInput(sessionId: string, input: string): Promise<void> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke('send_input', { sessionId, input });
  } else {
    const response = await fetch(`${API_BASE}/sessions/${sessionId}/input`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ input })
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
  }
}

/**
 * Read session log lines
 */
export async function readSessionLogs(logPath: string, offset: number): Promise<string[]> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<string[]>('read_session_logs', { logPath, offset });
  } else {
    // For browser mode, we'll need to implement this differently
    // For now, return empty array
    console.warn('readSessionLogs not yet implemented for browser mode');
    return [];
  }
}

/**
 * Check if running in Tauri desktop mode
 */
export function isTauriMode(): boolean {
  return isTauri;
}
