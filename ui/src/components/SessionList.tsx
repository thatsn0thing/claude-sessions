import { useEffect, useState } from 'react';
import { listSessions, deleteSession } from '../api';

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
  onNewSession: () => void;
  onDeleteSession: (sessionId: string) => void;
}

export function SessionList({ onSelectSession, selectedSessionId, onNewSession, onDeleteSession }: SessionListProps) {
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
      const result = await listSessions();
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

  async function handleDelete(sessionId: string, e: React.MouseEvent) {
    e.stopPropagation(); // Prevent session selection
    
    if (confirm('Are you sure you want to stop this session?')) {
      try {
        await deleteSession(sessionId);
        onDeleteSession(sessionId);
      } catch (err) {
        alert(`Failed to delete session: ${err}`);
      }
    }
  }

  return (
    <div className="session-list">
      <div className="session-list-header">
        <h2>Active Sessions ({sessions.length})</h2>
        <button className="new-session-btn" onClick={onNewSession} title="Create New Session">
          + New
        </button>
      </div>
      {sessions.map((session) => (
        <div
          key={session.id}
          className={`session-item ${selectedSessionId === session.id ? 'selected' : ''}`}
          onClick={() => onSelectSession(session)}
        >
          <div className="session-info">
            <div className="session-id">{session.id.substring(0, 8)}...</div>
            <div className="session-dir">{session.working_dir}</div>
            <div className="session-time">
              {new Date(session.created_at).toLocaleString()}
            </div>
          </div>
          <button
            className="delete-session-btn"
            onClick={(e) => handleDelete(session.id, e)}
            title="Stop Session"
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  );
}
