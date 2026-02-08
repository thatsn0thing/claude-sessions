import { useEffect, useRef, useState } from 'react';
import { readSessionLogs, sendInput } from '../api';
import './ChatViewer.css';

interface Session {
  id: string;
  working_dir: string;
  log_path: string;
}

interface ChatViewerProps {
  session: Session;
}

interface Message {
  type: 'user' | 'system' | 'assistant' | 'command';
  content: string;
  timestamp: string;
  raw?: string;
}

export function ChatViewer({ session }: ChatViewerProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [offset, setOffset] = useState(0);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const isLoadingRef = useRef(false);

  useEffect(() => {
    // Reset when session changes
    setMessages([]);
    setOffset(0);
    setInput('');

    // Load initial messages
    loadMessages();
    const interval = setInterval(loadMessages, 1000);

    return () => clearInterval(interval);
  }, [session.id]);

  useEffect(() => {
    // Auto-scroll to bottom
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  async function loadMessages() {
    if (isLoadingRef.current) return;
    isLoadingRef.current = true;

    try {
      const lines = await readSessionLogs(session.log_path, offset);

      if (lines.length > 0) {
        const newMessages = parseLogLines(lines);
        setMessages((prev) => [...prev, ...newMessages]);
        setOffset((prev) => prev + lines.length);
      }
    } catch (err) {
      console.error('Failed to read logs:', err);
    } finally {
      isLoadingRef.current = false;
    }
  }

  function parseLogLines(lines: string[]): Message[] {
    const messages: Message[] = [];
    let currentGroup: { type: 'input' | 'output'; lines: string[]; timestamp: string } | null =
      null;

    for (const line of lines) {
      try {
        const entry = JSON.parse(line);
        const data = atob(entry.data);
        const timestamp = entry.timestamp;

        if (entry.direction === 'input') {
          // User input
          if (currentGroup) {
            messages.push(...groupToMessages(currentGroup));
            currentGroup = null;
          }
          messages.push({
            type: 'user',
            content: data.trim(),
            timestamp,
            raw: line,
          });
        } else if (entry.direction === 'output') {
          // Group consecutive output
          if (!currentGroup || currentGroup.type !== 'output') {
            if (currentGroup) {
              messages.push(...groupToMessages(currentGroup));
            }
            currentGroup = { type: 'output', lines: [], timestamp };
          }
          currentGroup.lines.push(data);
        }
      } catch (e) {
        console.error('Failed to parse log line:', e);
      }
    }

    if (currentGroup) {
      messages.push(...groupToMessages(currentGroup));
    }

    return messages;
  }

  function groupToMessages(group: {
    type: 'input' | 'output';
    lines: string[];
    timestamp: string;
  }): Message[] {
    const text = group.lines.join('');
    const trimmed = text.trim();

    if (!trimmed) return [];

    // Heuristic: detect commands (lines starting with $, >, #, etc.)
    if (trimmed.startsWith('$ ') || trimmed.startsWith('> ') || trimmed.startsWith('# ')) {
      return [
        {
          type: 'command',
          content: trimmed,
          timestamp: group.timestamp,
        },
      ];
    }

    // Heuristic: detect system messages (starting with ✓, ✗, [, etc.)
    if (
      trimmed.startsWith('✓') ||
      trimmed.startsWith('✗') ||
      trimmed.startsWith('[') ||
      trimmed.startsWith('→') ||
      trimmed.startsWith('•')
    ) {
      return [
        {
          type: 'system',
          content: trimmed,
          timestamp: group.timestamp,
        },
      ];
    }

    // Default: assistant message
    return [
      {
        type: 'assistant',
        content: trimmed,
        timestamp: group.timestamp,
      },
    ];
  }

  async function handleSend() {
    if (!input.trim()) return;

    const userMessage: Message = {
      type: 'user',
      content: input.trim(),
      timestamp: new Date().toISOString(),
    };

    // Optimistically add user message
    setMessages((prev) => [...prev, userMessage]);
    setInput('');

    try {
      // Send to daemon which forwards to PTY
      await sendInput(session.id, input.trim());
    } catch (err) {
      console.error('Failed to send input:', err);
      // Show error in UI
      setMessages((prev) => [
        ...prev,
        {
          type: 'system',
          content: `⚠️ Failed to send: ${err}`,
          timestamp: new Date().toISOString(),
        },
      ]);
    }
  }

  function handleKeyPress(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  return (
    <div className="chat-viewer">
      <div className="chat-header">
        <span className="session-id">Session: {session.id.substring(0, 12)}...</span>
        <span className="working-dir">{session.working_dir}</span>
      </div>

      <div className="chat-messages">
        {messages.length === 0 ? (
          <div className="chat-empty">
            <p>Waiting for messages...</p>
            <p className="hint">Type a message below to start interacting with Claude</p>
          </div>
        ) : (
          messages.map((msg, idx) => (
            <div key={idx} className={`message message-${msg.type}`}>
              <div className="message-content">
                {msg.type === 'command' && <div className="command-prefix">$</div>}
                <pre className="message-text">{msg.content}</pre>
              </div>
              <div className="message-time">{new Date(msg.timestamp).toLocaleTimeString()}</div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder="Type a message... (Enter to send, Shift+Enter for new line)"
          rows={1}
        />
        <button onClick={handleSend} disabled={!input.trim()}>
          Send
        </button>
      </div>
    </div>
  );
}
