# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Argus is a macOS desktop app that records screen/audio exclusively during user-initiated deep work sessions and synthesizes the data into structured Obsidian vault notes. It is privacy-first: 100% local, no cloud, zero idle overhead.

Full design decisions are documented in `DESIGN.md`. The original brief is in `argus-assistant.pdf`.

## Stack

| Layer | Choice |
|---|---|
| App framework | Tauri v2 (Rust backend) |
| Frontend | Svelte 5 + TypeScript |
| Capture engine | Screenpipe (bundled binary, MIT-licensed) |
| Local DB | SQLite via `rusqlite` |
| Vector DB | `sqlite-vec` |
| LLM inference | Ollama (local only in v1) |
| Vault integration | Direct filesystem write to user-configured Obsidian vault path |

**Platform:** macOS only for v1.

## Key Architecture Decisions

- Screenpipe binary is bundled inside the Tauri app bundle — not a Homebrew prerequisite
- Sessions are bounded and user-initiated; Screenpipe process is fully terminated between sessions (0% idle CPU)
- Synthesis runs async post-session via Ollama (`llama3.2` default, `nomic-embed-text` for embeddings)
- Vault "smart append": sqlite-vec vector index finds semantically related existing notes; LLM decides to append or create new
- Raw session data stored in per-session SQLite DBs under `~/.argus/sessions/`; configurable TTL (default 30 days)
- App state DB: `~/.argus/app.db`
- Vault vector index: `~/.argus/vault-index/vectors.db` with incremental file watcher

## UI Structure

- **Menubar tray**: quick start/stop/pause toggle
- **Dashboard window**: session history (primary view); LLM config, capture settings, vault settings accessible via sidebar icon buttons
- **Post-synthesis summary**: auto-shown after export; displays vault notes affected, action items, LLM-suggested session title (editable)

## Session Lifecycle

`Idle → Active → Paused → Synthesizing → Complete`

- Pause is both manual and automatic (app exclusion list via Accessibility API)
- One active session at a time; minimum 60s duration

## Data Boundaries

- No network calls from the app in v1 (Ollama on loopback only)
- No telemetry or analytics
- All data in `~/.argus/` — user-owned

## Setup

Prerequisites: Node 20+, Rust stable (rustup), and a local Ollama install.

```bash
npm install                # install JS deps (Svelte 5, Tauri JS API)
npm run check              # svelte-check (types + a11y lint)
npm run build              # vite/SvelteKit static build → ./build
npm run tauri:dev          # launch the Tauri dev shell (Rust backend + Vite frontend)
npm run tauri:build        # produce a signed .app/.dmg
```

The Screenpipe binary is a stub at `src-tauri/bin/screenpipe` for dev. Replace it with a real Screenpipe build before shipping; the bundler will pick it up via `bundle.resources` in `tauri.conf.json`.

App state lives in `~/.argus/` (`app.db`, `sessions/<id>/raw.db`, `vault-index/vectors.db`). Delete that directory to start from a clean slate.

## Code layout

```
src/                # SvelteKit frontend
  app.html
  routes/
    +layout.svelte           # sidebar chrome
    +page.svelte             # session history
    summary/[id]/+page.svelte
    settings/{llm,capture,vault}/+page.svelte
    menubar/+page.svelte     # 280px popover view
  lib/
    components/
      eye/ArgusEye.svelte
      session/{SessionRow,SessionTimer,SynthesisProgress,SummaryScreen}.svelte
      settings/{Panel,Field}.svelte
      ui/{Button,Input,Badge,Waveform,ProgressBar,Icon,IconButton}.svelte
    stores/{session,settings}.svelte.ts   # Svelte 5 rune-based stores
    styles/{tokens,typography,base,animations,global}.css
    format.ts                 # date / duration formatters
    types.ts                  # shared TS types mirroring Rust structs

src-tauri/          # Rust backend
  src/
    lib.rs           # Tauri builder + invoke handler registration
    state.rs         # shared AppState (Db, Screenpipe, VaultIndex, current session)
    db.rs            # rusqlite app.db
    settings.rs      # typed settings accessor
    session.rs       # session lifecycle + DB persistence
    screenpipe.rs    # spawn / SIGSTOP / SIGCONT / SIGTERM
    ollama.rs        # /api/tags + /api/generate + /api/embeddings
    vault/
      mod.rs
      chunk.rs       # 400-token chunker w/ 50 overlap
      index.rs       # sqlite-vec backed vector store
      watcher.rs     # notify-debouncer-full, 10s debounce
      writer.rs      # atomic temp+rename, append-under-heading
    synthesis.rs     # 6-step pipeline (pair → extract → retrieve → decide → write → finalize)
    commands.rs      # all #[tauri::command] handlers
    events.rs        # typed event emitters (synthesis-progress, etc.)
    tray.rs          # macOS system-tray + menubar popover toggle
    paths.rs         # canonical ~/.argus paths
    error.rs         # ArgusError + serialization for IPC
```
