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
  selectedSessionId?: string;
}

export function SessionList({ onSelectSession, selectedSessionId }: SessionListProps) {
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
    return <div className="session-list loading">Loading sessions...</div>;
  }

  if (error) {
    return (
      <div className="session-list error">
        <p><strong>Error:</strong> {error}</p>
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
        <code>claude-sessions start /path</code>
      </div>
    );
  }

  return (
    <div className="session-list">
      <h2>Active Sessions ({sessions.length})</h2>
      {sessions.map((session) => (
        <div
          key={session.id}
          className={`session-item ${selectedSessionId === session.id ? 'selected' : ''}`}
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
