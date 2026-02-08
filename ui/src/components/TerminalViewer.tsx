import { useEffect, useRef, useState } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { readSessionLogs } from '../api';
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
  const isLoadingRef = useRef(false);

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
        cursor: '#d4d4d4',
      },
      rows: 30,
      cols: 100,
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(terminalRef.current);

    // Fit after a short delay to ensure container is sized
    setTimeout(() => fit.fit(), 100);

    terminalInstance.current = term;
    fitAddon.current = fit;

    // Handle resize
    const handleResize = () => {
      if (fitAddon.current) {
        fitAddon.current.fit();
      }
    };
    window.addEventListener('resize', handleResize);

    // Reset offset when session changes
    setOffset(0);

    // Load logs
    loadLogs();
    const interval = setInterval(loadLogs, 1000); // Poll every 1s

    return () => {
      window.removeEventListener('resize', handleResize);
      clearInterval(interval);
      term.dispose();
      terminalInstance.current = null;
    };
  }, [session.id]);

  async function loadLogs() {
    if (!terminalInstance.current || isLoadingRef.current) return;

    isLoadingRef.current = true;

    try {
      const lines = await readSessionLogs(session.log_path, offset);

      if (lines.length > 0) {
        for (const line of lines) {
          try {
            const entry = JSON.parse(line);
            if (entry.direction === 'output' && entry.data) {
              // Decode base64 data
              const data = atob(entry.data);
              terminalInstance.current?.write(data);
            }
          } catch (e) {
            console.error('Failed to parse log entry:', e, line);
          }
        }
        setOffset((prev) => prev + lines.length);
      }
    } catch (err) {
      console.error('Failed to read logs:', err);
    } finally {
      isLoadingRef.current = false;
    }
  }

  return (
    <div className="terminal-viewer">
      <div className="terminal-header">
        <span className="session-id">Session: {session.id.substring(0, 12)}...</span>
        <span className="working-dir">{session.working_dir}</span>
      </div>
      <div ref={terminalRef} className="terminal-container" />
    </div>
  );
}
