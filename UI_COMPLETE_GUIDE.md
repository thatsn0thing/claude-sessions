# Complete UI Guide - Full CRUD Operations

## ✅ What's Now Possible

You can now **fully manage Claude sessions** directly from the UI without touching the CLI!

### Features

✅ **Create** sessions with directory picker  
✅ **Read** / View sessions and their output  
✅ **Update** / Send messages to Claude in real-time  
✅ **Delete** / Stop sessions from UI  

---

## 🚀 Quick Start

### Step 1: Start the Daemon

```bash
cd claude-sessions
./target/release/claude-sessions daemon --foreground
```

### Step 2: Launch UI

```bash
cd ui
npm run tauri dev
```

### Step 3: Create Your First Session

1. **Click "+ New" button** in the sidebar
2. **Browse** or type a directory path
3. **Click "Create Session"**
4. **Start chatting with Claude!**

That's it! No CLI needed. 🎉

---

## 📖 Detailed Usage

### Creating a Session

**Method 1: Directory Picker (Recommended)**

1. Click **"+ New"** button at the top of the sidebar
2. A modal window appears
3. Click **"Browse..."** button
4. Navigate to your project directory
5. Click **"Select Folder"**
6. Click **"Create Session"**

**Method 2: Manual Path Entry**

1. Click **"+ New"** button
2. Type the directory path directly: `/home/user/my-project`
3. Click **"Create Session"**

**What Happens:**
- Session is created on the daemon
- New session appears in the sidebar immediately
- Session is automatically selected
- You can start chatting right away

---

### Chatting with Claude

Once a session is selected:

**Sending Messages:**
1. Type your message in the input box at the bottom
2. Press **Enter** or click **"Send"**
3. Your message appears as a blue bubble (right side)
4. Claude's response appears as gray bubble (left side)

**Features:**
- **Auto-scroll:** Conversation scrolls to latest message
- **Multi-line:** Press **Shift+Enter** for new line
- **Real-time:** Messages reach Claude instantly via PTY
- **Persistent:** All messages logged to disk

**Example Conversation:**
```
You: Can you list the files in this directory?

Claude: Sure! Let me check...
$ ls -la
total 48
drwxr-xr-x 5 user user 4096 Feb 8 03:00 .
...

You: What's in the README?

Claude: Let me read it for you...
$ cat README.md
# My Project
...
```

---

### Viewing Modes

**Chat View (Default):**
- Message bubbles (user + Claude)
- Command blocks for shell commands
- System messages for status
- Input box for sending messages

**Terminal View:**
- Raw PTY output (xterm.js)
- ANSI colors and formatting
- Scrollback buffer
- Read-only (for now)

**Toggle:** Click **💬 Chat** or **🖥️ Terminal** buttons

---

### Deleting a Session

**Option 1: From Session Card**
1. Hover over a session in the sidebar
2. Click the **✕** button (appears on the right)
3. Confirm deletion
4. Session stops immediately

**Option 2: Before Deleting**
- Session automatically deselected if it was active
- Logs remain on disk at `~/.claude-sessions/logs/`
- State file updated immediately

**What Gets Deleted:**
- ✅ Session metadata
- ✅ PTY process (Claude subprocess terminated)
- ✅ Entry in sessions list

**What's Preserved:**
- ✅ Log files (for future reference)
- ✅ Project directory (untouched)

---

## 🎨 UI Elements Explained

### Sidebar

```
┌─────────────────────────────┐
│ 🤖 Claude Sessions          │
│ Chat Interface              │
├─────────────────────────────┤
│ ACTIVE SESSIONS (2)  [+ New]│ ← Create button
├─────────────────────────────┤
│ ┌─────────────────────────┐ │
│ │ a1b2c3d4...          [✕]│ │ ← Delete button
│ │ /tmp/my-project         │ │
│ │ 9:30 AM                 │ │
│ └─────────────────────────┘ │
│                             │
│ ┌─────────────────────────┐ │
│ │ e5f6g7h8...          [✕]│ │
│ │ /home/user/other        │ │
│ │ 9:45 AM                 │ │
│ └─────────────────────────┘ │
└─────────────────────────────┘
```

### Session Creator Modal

```
┌────────────────────────────────────────┐
│ Create New Session                  [✕]│
├────────────────────────────────────────┤
│                                        │
│ Project Directory                      │
│ [/home/user/project    ] [Browse...  ] │
│                                        │
│ Select the directory where you want   │
│ Claude to work                         │
│                                        │
│            [Cancel] [Create Session]   │
└────────────────────────────────────────┘
```

### Chat Interface

```
┌────────────────────────────────────────┐
│ Session: abc123...    /tmp/project     │
├────────────────────────────────────────┤
│                                        │
│          Can you help me debug?        │
│                         9:30 AM        │
│                                        │
│  Sure! Let me check the code...        │
│  9:31 AM                               │
│                                        │
│  $ npm test                            │
│  9:32 AM                               │
│                                        │
│         ✓ All tests passed             │
│         9:32 AM                        │
│                                        │
├────────────────────────────────────────┤
│ [Type a message...]           [Send]   │
└────────────────────────────────────────┘
```

---

## ⌨️ Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Send message | **Enter** |
| New line in input | **Shift + Enter** |
| Close modal | **Escape** (planned) |

---

## 💡 Pro Tips

### 1. **Multiple Sessions**
Open multiple projects and switch between them:
```
Click session 1 → Chat with Claude about project 1
Click session 2 → Chat with Claude about project 2
```

### 2. **Command Output**
Commands executed by Claude appear as special blocks:
```
╔═══════════════════════════════╗
║ $ npm install express         ║
║ added 57 packages in 3.2s     ║
╚═══════════════════════════════╝
```

### 3. **Error Messages**
If input fails, you'll see:
```
⚠️ Failed to send: Session not found or not active
```

### 4. **Directory Validation**
- Directory must exist
- Can be relative or absolute path
- Home directory expansion not supported (use full paths)

### 5. **Session Status**
Session cards show status:
- **Green highlight** = Selected
- **Gray** = Available
- **Hover** = Shows delete button

---

## 🔧 Troubleshooting

### "Failed to create session"

**Possible Causes:**
1. Directory doesn't exist
2. No permission to access directory
3. Daemon not running

**Solutions:**
```bash
# Check directory exists
ls /path/to/directory

# Check daemon status
./target/release/claude-sessions status

# Restart daemon if needed
./target/release/claude-sessions daemon --foreground
```

### "Failed to send input"

**Possible Causes:**
1. Session is stale (no PTY)
2. Claude process crashed
3. Daemon connection lost

**Solutions:**
```bash
# Check session status in terminal
./target/release/claude-sessions list

# If session is stale, delete and recreate it
```

### Delete Button Not Working

**Check:**
1. Click the **✕** button specifically (not the whole card)
2. Confirm the prompt
3. Check if daemon is still running

### Directory Picker Not Opening

**Platform Issues:**
- Linux: May need Zenity installed: `sudo apt install zenity`
- macOS: Should work out of the box
- Windows: Should work with native dialog

**Workaround:** Type path manually instead of browsing

---

## 🎯 Common Workflows

### Workflow 1: Quick Debugging Session

1. Click **"+ New"**
2. Select project directory
3. Create session
4. Ask Claude: "What's in this project?"
5. Claude lists files and explains structure
6. Ask follow-up questions
7. When done, click **✕** to delete session

### Workflow 2: Multi-Project Development

1. Create session for project A
2. Create session for project B  
3. Create session for project C
4. Switch between them as needed
5. Keep all sessions running simultaneously
6. Each has independent context

### Workflow 3: Code Review

1. Create session in repository
2. Ask Claude to review recent changes
3. Claude runs `git diff` and analyzes
4. Discuss improvements
5. Claude can suggest fixes
6. Keep session for ongoing review

---

## 📊 Feature Comparison

| Feature | Phase 4 | Phase 5 | Current |
|---------|---------|---------|---------|
| View sessions | ✅ | ✅ | ✅ |
| View output | ✅ | ✅ | ✅ |
| Chat interface | ❌ | ✅ | ✅ |
| Send input | ❌ | ⚠️ (partial) | ✅ (full) |
| Create sessions | ❌ | ❌ | ✅ |
| Delete sessions | ❌ | ❌ | ✅ |
| Directory picker | ❌ | ❌ | ✅ |

---

## 🔮 What's Next?

### Planned Features

- [ ] **Session templates:** Pre-configured project types
- [ ] **Batch operations:** Delete multiple sessions
- [ ] **Search:** Find sessions by directory
- [ ] **Favorites:** Pin important sessions
- [ ] **Session rename:** Custom session names
- [ ] **Export conversation:** Save chat history
- [ ] **Keyboard shortcuts:** Full keyboard navigation

---

## 📁 File Locations

```bash
# Daemon socket
~/.claude-sessions/daemon.sock

# Session state
~/.claude-sessions/sessions.json

# Logs
~/.claude-sessions/logs/<session-id>.jsonl

# UI config (Tauri)
~/.config/ui/
```

---

## 🎓 Behind the Scenes

### What Happens When You Create a Session

```
UI: Click "Create Session"
  ↓
Tauri: invoke('create_session', { working_dir })
  ↓
DaemonClient: Send IPC request
  ↓
Daemon: Receive StartSession request
  ↓
SessionManager: start_session(working_dir)
  ↓
1. Validate directory exists
2. Create Session metadata (UUID)
3. Spawn Claude PTY in directory
4. Start background log reader
5. Save to sessions.json
  ↓
Response: { session_id, log_path }
  ↓
UI: Session appears in sidebar
```

### What Happens When You Send a Message

```
UI: Type "Hello" → Press Enter
  ↓
Tauri: invoke('send_input', { session_id, text })
  ↓
DaemonClient: Send IPC request
  ↓
Daemon: Receive SendInput request
  ↓
SessionManager: send_input(session_id, "Hello")
  ↓
SessionProcess: write_input(b"Hello\n")
  ↓
PTY Master: Write to PTY
  ↓
Claude subprocess: Receives "Hello\n" on stdin
  ↓
Claude processes and responds
  ↓
PTY: Output captured by background reader
  ↓
Log file: Appended as JSONL entry
  ↓
UI: Polls log file → Sees new output → Renders in chat
```

---

## ✅ Complete Example Session

```bash
# Terminal 1: Start daemon
./target/release/claude-sessions daemon --foreground
# Output: ✅ Daemon started

# Terminal 2: Launch UI
cd ui && npm run tauri dev
# UI window opens

# In UI:
1. Click "+ New"
2. Browse to /home/user/my-app
3. Click "Create Session"
4. Session appears: "a1b2c3d4... /home/user/my-app"

# Chat with Claude:
You: "What files are in this directory?"
Claude: "Let me check..."
        $ ls -la
        [output shows files]

You: "Can you explain what app.js does?"
Claude: "Sure! Let me read it..."
        $ cat app.js
        [shows content and explains]

You: "Add a new feature to handle user login"
Claude: [Suggests code changes, shows diffs]

# When done:
5. Click ✕ button on session card
6. Confirm deletion
7. Session stops

# Logs remain at:
# ~/.claude-sessions/logs/a1b2c3d4-....jsonl
```

---

**Status**: ✅ Full CRUD Implementation Complete  
**Repository**: https://github.com/thatsn0thing/claude-sessions  
**Latest Commit**: `7250ad7` - Add full CRUD operations in UI  

**You can now manage Claude sessions entirely from the UI!** 🎉
