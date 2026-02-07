import { useState } from 'react';
import { SessionList } from './components/SessionList';
import { TerminalViewer } from './components/TerminalViewer';
import { ChatViewer } from './components/ChatViewer';
import './App.css';

interface Session {
  id: string;
  working_dir: string;
  created_at: string;
  status: string;
  log_path: string;
}

type ViewMode = 'terminal' | 'chat';

function App() {
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);
  const [viewMode, setViewMode] = useState<ViewMode>('chat');

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>🤖 Claude Sessions</h1>
          <p className="subtitle">
            {viewMode === 'chat' ? 'Chat Interface' : 'Terminal Viewer'}
          </p>
        </div>
        <SessionList
          onSelectSession={setSelectedSession}
          selectedSessionId={selectedSession?.id}
        />
      </aside>
      <main className="main">
        {selectedSession ? (
          <>
            <div className="view-toggle">
              <button
                className={viewMode === 'chat' ? 'active' : ''}
                onClick={() => setViewMode('chat')}
              >
                💬 Chat
              </button>
              <button
                className={viewMode === 'terminal' ? 'active' : ''}
                onClick={() => setViewMode('terminal')}
              >
                🖥️ Terminal
              </button>
            </div>
            {viewMode === 'chat' ? (
              <ChatViewer session={selectedSession} />
            ) : (
              <TerminalViewer session={selectedSession} />
            )}
          </>
        ) : (
          <div className="empty-state">
            <div className="empty-content">
              <h2>No Session Selected</h2>
              <p>Select a session from the list to view its output</p>
              <div className="instructions">
                <h3>Quick Start:</h3>
                <ol>
                  <li>
                    Start the daemon: <code>claude-sessions daemon --foreground</code>
                  </li>
                  <li>
                    Create a session: <code>claude-sessions start /path/to/project</code>
                  </li>
                  <li>Select it from the list</li>
                </ol>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
