import { useState } from 'react';
import { SessionList } from './components/SessionList';
import { TerminalViewer } from './components/TerminalViewer';
import { ChatViewer } from './components/ChatViewer';
import { SessionCreator } from './components/SessionCreator';
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
  const [showCreator, setShowCreator] = useState(false);

  function handleNewSession() {
    setShowCreator(true);
  }

  function handleSessionCreated(sessionId: string) {
    setShowCreator(false);
    // Session will appear in the list automatically on next refresh
    console.log('Session created:', sessionId);
  }

  function handleDeleteSession(sessionId: string) {
    // If deleted session was selected, deselect it
    if (selectedSession?.id === sessionId) {
      setSelectedSession(null);
    }
  }

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
          onNewSession={handleNewSession}
          onDeleteSession={handleDeleteSession}
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
              <p>Click "+ New" to create a session or select an existing one</p>
              <div className="instructions">
                <h3>Quick Start:</h3>
                <ol>
                  <li>Click the <strong>+ New</strong> button in the sidebar</li>
                  <li>Select a project directory</li>
                  <li>Start chatting with Claude!</li>
                </ol>
              </div>
            </div>
          </div>
        )}
      </main>

      {showCreator && (
        <SessionCreator
          onSessionCreated={handleSessionCreated}
          onCancel={() => setShowCreator(false)}
        />
      )}
    </div>
  );
}

export default App;
