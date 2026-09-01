# Argus — Formal Design Document
**Version:** 1.0 | **Date:** 2026-06-04 | **Status:** Pre-development

---

## Context

Argus is a macOS desktop application that records user activity exclusively during bounded, user-initiated deep work sessions and synthesizes that data into structured Obsidian vault notes. It was conceived as a privacy-first alternative to tools like Rewind AI — no cloud, no persistent background recording, zero idle overhead. The target user is a high-intent knowledge worker (developer, researcher, Obsidian power user) who wants automated knowledge capture without surveillance anxiety.

The PDF brief established the concept, market positioning, and high-level architecture. This document translates those into concrete implementation decisions.

---

## 1. Technical Stack

| Layer | Choice | Rationale |
|---|---|---|
| App framework | Tauri v2 (Rust backend) | Native macOS, small bundle, safe IPC |
| Frontend | Svelte 5 + TypeScript | Lightest runtime, reactive by default, fits small focused app |
| Capture engine | Screenpipe (bundled binary) | MIT-licensed, handles OS screen/audio permissions, avoids kernel driver work |
| Local database | SQLite via `rusqlite` | Raw session storage, app config, session history |
| Vector database | `sqlite-vec` | Embedded, no separate process, Rust-native, upgradeable to HNSW |
| LLM inference | Ollama (local) | Privacy-first, zero marginal cost, user-managed models |
| Vault integration | Direct filesystem write | No plugin required; user sets vault path in settings |

**Platform:** macOS only for v1. No Windows or Linux until v2.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Argus (Tauri App)                    │
│                                                         │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────┐  │
│  │  Menubar    │   │  Dashboard   │   │  Settings   │  │
│  │  Toggle     │   │  Window      │   │  Panel      │  │
│  └──────┬──────┘   └──────┬───────┘   └──────┬──────┘  │
│         │                 │                  │         │
│  ┌──────▼─────────────────▼──────────────────▼──────┐  │
│  │              Tauri Rust Command Layer             │  │
│  └──────┬──────────────────────────────────┬────────┘  │
│         │                                  │           │
│  ┌──────▼──────┐                  ┌────────▼────────┐  │
│  │  Session    │                  │  Synthesis      │  │
│  │  Manager   │                  │  Engine         │  │
│  └──────┬──────┘                  └────────┬────────┘  │
│         │                                  │           │
│  ┌──────▼──────┐   ┌──────────┐   ┌────────▼────────┐  │
│  │  Screenpipe │   │  SQLite  │   │  Vault Writer   │  │
│  │  Process   ├───►  (raw)   ◄───┤  + Vec Index    │  │
│  └─────────────┘   └──────────┘   └─────────────────┘  │
└─────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼──────────┐
                    │  Obsidian Vault    │
                    │  (filesystem)      │
                    └────────────────────┘
```

The Rust backend owns all stateful operations. The Svelte frontend communicates exclusively via Tauri commands and events — no direct filesystem or process access from JS.

---

## 3. Component Specifications

### 3.1 Session Manager

Owns the full lifecycle of a recording session.

**States:** `Idle → Active → Paused → Synthesizing → Complete`

**Session start:**
- User clicks menubar icon or dashboard button
- App assigns a timestamp-based ID: `session_2026-06-04_1430`
- Screenpipe process is spawned (see §3.2)
- Session record created in SQLite with status `active`

**Session pause/resume:**
- Manual: user clicks pause in menubar or dashboard
- Automatic: app monitors active window via Accessibility API; if foreground app matches the exclusion list, Screenpipe is sent SIGSTOP (pauses capture), resumed with SIGCONT when focus leaves the excluded app
- Timer pauses with capture; the session remains the same session

**Session end:**
- User clicks stop
- Screenpipe process is terminated
- Session status set to `synthesizing`
- Synthesis engine is invoked asynchronously
- User is free to use the computer; synthesis runs in background

**Session naming:**
- Auto-named by timestamp on creation
- After synthesis completes, LLM suggests a descriptive title (e.g., "Refactoring Auth Middleware — Python")
- User can accept or type a custom name in the post-synthesis summary screen

**Constraints:**
- One active session at a time
- Minimum session duration: 60 seconds (guard against accidental triggers)

---

### 3.2 Screenpipe Integration

Screenpipe captures screen OCR and audio. The binary is resolved from:
1. `~/.argus/bin/screenpipe` (on-demand download with pinned versioning and SHA-256 validation).
2. `Contents/Resources/bin/screenpipe` (bundled app bundle fallback).

**Binary Acquisition & Verification:**
- Pinned release: `v0.1.72` (architecture-specific for `aarch64-apple-darwin` and `x86_64-apple-darwin`).
- SHA-256 cryptographic verification prior to execution.
- Auto-configuration of POSIX permissions (`chmod 0o755`) and macOS quarantine removal (`xattr -d com.apple.quarantine`).

**Process management (Rust):**
```rust
// Each session spawns a fresh screenpipe process
let child = Command::new(screenpipe_path)
    .args(["--start-audio", "--ocr", "--db", session_db_path])
    .spawn()?;
```

- Each session gets its own SQLite DB file: `~/.argus/sessions/{session_id}/raw.db`
- On session end: `child.kill()` (SIGTERM → SIGKILL after 2s grace period)
- On app quit during active session: same termination sequence, session marked `interrupted`

**App exclusion list:**
- Stored in app config as a list of bundle identifiers (e.g., `com.1password.1password`)
- Checked via `NSWorkspace.shared.frontmostApplication` polled at 1s intervals during active session
- User manages this list in Settings

**macOS permissions required:**
- Screen Recording (`com.apple.security.screen-recording`)
- Microphone (`NSMicrophoneUsageDescription`)
- Accessibility (for foreground app detection and exclusion list)

---

### 3.3 Synthesis Engine (Map-Reduce Architecture)

Runs post-session, async, in a background Rust task. Quality is prioritized over speed.

**Pipeline steps:**

**Step 1 — Temporal context pairing**
- Read OCR frames and audio transcript chunks from the session's raw SQLite DB (`raw.db`).
- Merge chronologically: OCR and audio segments within a ±2s window are joined into aligned `TimelineRow` entries without dropping non-overlapping frames.

**Step 2 — Map-Reduce concept extraction**
- **Map Phase**: Sessions longer than 10 minutes are sliced into 10-minute chronological windows. Each window is sent to the LLM to extract key activities, decisions, and evidence.
- **Reduce Phase**: Aggregated window summaries are distilled into a cohesive session title, 2–5 structured concepts, consolidated action items, and open questions.
- **LLM Abstraction**: Decoupled behind the `LlmClient` trait (Ollama default for local MVP; extensible for BYOK cloud providers in v2).

**Step 3 — Vault candidate retrieval (smart append)**
- Embed each extracted concept using `nomic-embed-text` via `LlmClient::embed`.
- Query `sqlite-vec` flat index for top-5 most semantically similar existing vault note chunks.
- For each candidate above similarity threshold (default 0.75): read candidate note body into context.
- LLM decides: append to matching note under heading or create new note.

**Step 4 — Content generation**
- For append decisions: LLM generates the section to add with Obsidian `[[wikilinks]]`.
- For new note decisions: LLM generates a complete note with YAML frontmatter, headers, and wikilinks.

**Step 5 — Write to vault**
- Atomic writes: temporary file write → fsync → rename.
- Appends target specific H2/H3 headings or EOF.
- Session record updated in `app.db`.

**Step 6 — In-app summary**
- Summary screen displays: session title (editable), duration, affected vault files, and action items.
- "Open in Obsidian" button launches the vault or specific note.

---

### 3.4 Vault Vector Index

Maintains semantic awareness of the user's existing Obsidian vault for smart append decisions.

**Index storage:** `~/.argus/vault-index/vectors.db`

**Initial indexing:**
- Triggered on first vault path configuration
- Chunks each `.md` file into ~400-token segments with 50-token overlap
- Embeds each chunk via `nomic-embed-text`
- Stores: `(chunk_id, note_path, chunk_text, embedding, content_hash, mtime)`
- Progress bar shown in Settings panel during first-time indexing

**Incremental updates:**
- Background file watcher (`notify` crate) monitors the vault directory
- On file change: compare `mtime` and content hash; re-embed only changed files
- Debounced at 10s to avoid thrashing during rapid edits

**Scalability:**
- Flat cosine similarity scan is sufficient for vaults up to ~80k chunks (~8,000 notes)
- HNSW index available as a Settings toggle for users with large vaults
- Future escape hatch: LanceDB (Rust-native, IVF/HNSW) as a drop-in replacement if sqlite-vec becomes a bottleneck

---

### 3.5 Vault Writer

Handles all filesystem writes to the Obsidian vault.

- Vault path configured by user via folder picker; stored in app settings
- Atomic writes: temp file → rename, preventing partial writes
- Appends inserted at end of matching note section (by H2/H3 header) or end-of-file
- New note YAML frontmatter includes: `date`, `session_id`, `tags` (LLM-extracted), `argus: true`
- Note output is separate from the in-app session summary — the vault gets structured knowledge notes, the app shows a session digest

---

## 4. Data Model

### App Database (`~/.argus/app.db`)

```sql
sessions (
  id TEXT PRIMARY KEY,          -- "session_2026-06-04_1430"
  display_name TEXT,            -- user-set or LLM-suggested title
  status TEXT,                  -- idle|active|paused|synthesizing|complete|interrupted
  started_at INTEGER,
  ended_at INTEGER,
  duration_secs INTEGER,        -- excludes paused time
  vault_files_affected TEXT,    -- JSON array of file paths
  action_items TEXT,            -- JSON array
  raw_db_path TEXT,
  raw_db_expires_at INTEGER     -- TTL timestamp; NULL = never
)

settings (
  key TEXT PRIMARY KEY,
  value TEXT
)
-- Keys: vault_path, ollama_host, ollama_model, embed_model,
--       data_retention_days, exclusion_list, similarity_threshold
```

### Per-Session Raw DB (`~/.argus/sessions/{id}/raw.db`)

Mirrors Screenpipe's output schema:
```sql
ocr_frames   (id, timestamp, app_name, window_title, text, frame_path)
audio_chunks (id, timestamp, duration_ms, transcript)
```

**Data retention:** Configurable TTL (default 30 days). A background job on app launch deletes expired raw DBs. The `app.db` session record (summary, affected files, action items) is retained permanently.

---

## 5. UI/UX Design

### Menubar

- Icon: Argus eye glyph — grey (idle), green pulse (active), yellow (paused)
- Left-click: popover with session status, start/stop/pause button, link to dashboard
- Right-click: open dashboard, settings, quit

### Dashboard Window

**Primary view — Session History:**
- Scrollable list of past sessions, newest first
- Each row: session title, date, duration, status badge, "View Summary" button
- Expanding a row shows: action items + vault files affected

**Post-synthesis summary screen:**
- Shown automatically after synthesis completes
- LLM-suggested title (editable inline), session duration, vault notes created/appended, action items, open questions
- "Open in Obsidian" button

**Secondary panels (sidebar icon access, not main screen):**
- LLM Config: Ollama host, inference model, embedding model, test connection
- Capture Settings: exclusion app list, data retention TTL
- Vault Settings: vault path picker, similarity threshold, re-index button, index progress

---

## 6. Privacy & Security

- No network calls from the app in v1 — Ollama runs on loopback only
- No telemetry, analytics, or crash reporting
- Raw session data lives entirely in `~/.argus/` — user-owned and deletable
- App exclusion list prevents capture of sensitive apps
- Bundled Screenpipe binary is a pinned version verified by checksum on launch
- **v2 consideration:** AES-256 encryption of raw session SQLite DBs at rest

---

## 7. Phased Roadmap

### v1 — MVP
- Session capture via bundled Screenpipe (screen OCR + audio)
- Manual + automatic pause/resume with app exclusion list
- Ollama synthesis: concept extraction, temporal pairing, smart append to vault
- sqlite-vec vault index with incremental file watcher
- Direct vault write (smart append to existing notes + create new notes)
- Menubar tray + full dashboard window
- Configurable data retention TTL

### v2 — Expansion
- Inbox summarizer: Gmail OAuth + IMAP header polling, cross-reference injection into synthesis
- Cloud API tier: BYOK Claude / OpenAI keys
- Managed API subscription tier
- Windows support
- HNSW index auto-upgrade path
- Encrypted raw session storage

---

## 8. Open Questions

- Which specific Screenpipe binary version to pin for v1 bundle
- Exact chunking strategy for OCR frames (by app window change vs. fixed time interval)
- Whether to expose prompt templates to advanced users in v1 or defer to v2
