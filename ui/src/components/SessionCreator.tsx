import { useState } from 'react';
import { createSession, isTauriMode } from '../api';
import './SessionCreator.css';

interface SessionCreatorProps {
  onSessionCreated: (sessionId: string) => void;
  onCancel: () => void;
}

export function SessionCreator({ onSessionCreated, onCancel }: SessionCreatorProps) {
  const [directory, setDirectory] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handlePickDirectory() {
    if (!isTauriMode()) {
      setError('Directory picker only available in desktop app. Please type the path manually.');
      return;
    }
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const selected = await invoke<string | null>('pick_directory');
      if (selected) {
        setDirectory(selected);
        setError(null);
      }
    } catch (err) {
      setError(`Failed to pick directory: ${err}`);
    }
  }

  async function handleCreate() {
    if (!directory.trim()) {
      setError('Please select a directory');
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      const result = await createSession(directory);
      onSessionCreated(result.id);
    } catch (err) {
      setError(`Failed to create session: ${err}`);
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <div className="session-creator-overlay">
      <div className="session-creator-modal">
        <div className="modal-header">
          <h2>Create New Session</h2>
          <button className="close-btn" onClick={onCancel} disabled={isCreating}>
            ✕
          </button>
        </div>

        <div className="modal-body">
          <div className="form-group">
            <label>Project Directory</label>
            <div className="directory-picker">
              <input
                type="text"
                value={directory}
                onChange={(e) => setDirectory(e.target.value)}
                placeholder="/path/to/your/project"
                disabled={isCreating}
              />
              <button 
                onClick={handlePickDirectory}
                disabled={isCreating}
                className="browse-btn"
              >
                Browse...
              </button>
            </div>
            <p className="hint">
              Select the directory where you want Claude to work
            </p>
          </div>

          {error && (
            <div className="error-message">
              ⚠️ {error}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button 
            onClick={onCancel} 
            disabled={isCreating}
            className="cancel-btn"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={isCreating || !directory.trim()}
            className="create-btn"
          >
            {isCreating ? 'Creating...' : 'Create Session'}
          </button>
        </div>
      </div>
    </div>
  );
}
