# Argus

A privacy-first macOS desktop app that records screen and audio **only** during user-initiated deep work sessions and synthesizes the data into structured Obsidian vault notes. 100% local, no cloud, zero idle overhead.

See [`SPEC.md`](./SPEC.md) for the formal product spec and [`DESIGN.md`](./DESIGN.md) for the frontend design system. [`CLAUDE.md`](./CLAUDE.md) is the working brief for agent collaborators.

---

## Stack

| Layer | Choice |
|---|---|
| App framework | Tauri v2 (Rust backend) |
| Frontend | Svelte 5 (runes) + SvelteKit + TypeScript |
| Capture engine | Screenpipe (bundled binary, MIT) |
| Local DB | SQLite via `rusqlite` (bundled) |
| Vector DB | `sqlite-vec` |
| LLM inference | Ollama (local, loopback only) |
| Vault integration | Direct filesystem writes |

Platform: macOS 12+ only for v1.

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Node.js | 20+ | `brew install node` |
| Rust | stable (1.75+) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Ollama | latest | `brew install --cask ollama` or [ollama.com](https://ollama.com) |

Pull the default models before first synthesis:

```bash
ollama pull llama3.2
ollama pull nomic-embed-text
```

Models are configurable in **Settings → LLM Config** — any model exposed by your local Ollama instance works.

### macOS permissions

The first session will prompt for:

- **Screen Recording** — required for OCR capture
- **Microphone** — required for voice transcription
- **Accessibility** — required for foreground-app detection (used by the auto-pause exclusion list)

Grant all three in **System Settings → Privacy & Security**.

---

## Development

```bash
git clone <this-repo>
cd argus

npm install                 # JS deps (Svelte 5, Tauri JS API)
npm run check               # svelte-check (types + a11y)
npm run build               # static SvelteKit build → ./build
npm run tauri:dev           # launch the desktop dev shell (Rust + Vite)
npm run tauri:build         # signed .app/.dmg production build
```

The first `tauri:dev` will compile ~400 crates and takes 1–3 minutes. Subsequent rebuilds are incremental and sub-second for frontend, sub-10s for Rust changes.

### Data locations

Everything Argus writes lives under `~/.argus/`:

```
~/.argus/
├── app.db                    # session history + settings
├── sessions/
│   └── <session_id>/raw.db   # per-session OCR + audio (TTL-vacuumed)
└── vault-index/
    └── vectors.db            # sqlite-vec embedding store
```

Wipe `~/.argus/` to fully reset state. The Obsidian vault is never touched outside of synthesis writes.

### Project layout

```
src/                # SvelteKit frontend
  routes/
    +layout.svelte           # sidebar chrome
    +page.svelte             # session history
    summary/[id]/+page.svelte
    settings/{llm,capture,vault}/+page.svelte
    menubar/+page.svelte
  lib/
    components/eye           # ArgusEye SVG + state animations
    components/session       # SessionRow, SessionTimer, SynthesisProgress, SummaryScreen
    components/settings      # Panel, Field
    components/ui            # Button, Input, Badge, Waveform, ProgressBar, Icon, IconButton
    stores/                  # Svelte 5 rune-based stores (session, settings)
    styles/                  # design tokens, typography, base, animations

src-tauri/          # Rust backend
  src/
    lib.rs           # Tauri builder + invoke handler registration
    state.rs         # shared AppState
    db.rs            # rusqlite app.db
    settings.rs      # typed settings accessor
    session.rs       # lifecycle + DB persistence
    screenpipe.rs    # spawn / SIGSTOP / SIGCONT / SIGTERM ladder
    ollama.rs        # /api/{tags,generate,embeddings}
    vault/
      chunk.rs       # 400-token chunker w/ 50 overlap
      index.rs       # sqlite-vec vector store
      watcher.rs     # notify-debouncer-full, 10s debounce
      writer.rs      # atomic temp+rename, append-under-heading
    synthesis.rs     # 6-step post-session pipeline
    commands.rs      # all #[tauri::command] handlers
    events.rs        # typed event emitters
    tray.rs          # system-tray + menubar popover
    paths.rs         # canonical ~/.argus paths
    error.rs         # ArgusError + IPC serialization
```

---

## Current capabilities

### Working
- **Session lifecycle** — `Idle → Active → Paused → Synthesizing → Complete | Interrupted`. One session at a time, configurable minimum duration (default 60s).
- **Menubar tray** — left-click toggles a 280px popover with eye-state indicator, live timer, animated waveform, and primary/secondary controls. Right-click → dashboard/quit.
- **Dashboard window** — session history (newest first, expandable rows), post-synthesis summary (editable title, action items, vault notes affected), sidebar navigation to settings panels.
- **Screenpipe process management** — spawn per session with `--ocr --start-audio --db <path>`; SIGSTOP/SIGCONT for pause; SIGTERM with 2s grace then SIGKILL on stop. Idle CPU is 0% because the process is fully terminated between sessions.
- **Ollama integration** — HTTP client for `/api/tags`, `/api/generate` (JSON mode for structured output), `/api/embeddings`. Settings panel has a live connection tester.
- **Vault vector index** — sqlite-vec backed, registered via `sqlite3_auto_extension`. 400-token chunks with 50-token overlap. Incremental updates via SHA-256 + mtime comparison.
- **Vault file watcher** — `notify-debouncer-full` watcher, 10s debounce, re-embeds changed `.md` files in-place.
- **Synthesis pipeline** — 6 steps emitting live progress events:
  1. Temporal pairing of OCR + audio on a ±2s window
  2. JSON-mode concept extraction
  3. Embed-and-top-k vault retrieval (threshold configurable)
  4. Per-concept append-vs-new decision via LLM
  5. Atomic vault writes (temp file → fsync → rename)
  6. Session record finalization + UI navigation to the summary screen
- **Settings persistence** — Ollama host/model, embedding model, similarity threshold, data retention TTL, minimum session duration, app exclusion list — all in `app.db` with sane defaults.
- **TTL housekeeping** — on app launch, raw session DBs past their retention window are deleted; the summary row in `app.db` is kept forever.
- **Design system** — 100% of DESIGN.md §2 (colors, typography scale, spacing, radius, shadow, motion) implemented as CSS custom properties. Dark-mode only; honors `prefers-reduced-motion`.

### Build status
- `cargo build` (dev profile): ✅ clean (5 dead-code warnings)
- `npm run check` (svelte-check): ✅ 0 errors, 0 warnings
- `npm run build` (vite/SvelteKit static): ✅ clean

---

## Next steps

### Critical before first real use
- **Real Screenpipe binary** — `src-tauri/bin/screenpipe` is currently a shell stub that exits with an error. Drop in a pinned MIT-licensed Screenpipe binary and verify it accepts our `--start-audio --ocr --db` flags. Add a checksum verification step on app launch.
- **App icons** — replace the solid-color placeholder PNGs in `src-tauri/icons/` with the eye glyph from DESIGN.md §2.5. Generate `.icns` for the Dock and a template-mode `18×18` glyph optimized for the menubar (1x and 2x).
- **Accessibility-API foreground-app poller** — exclusion-list CRUD is wired through, but the actual 1-second `NSWorkspace.frontmostApplication` polling loop that triggers `SIGSTOP`/`SIGCONT` isn't implemented. Add it as a tokio task started in `commands::session_start` and cancelled on stop.
- **macOS permission preflight** — surface a one-shot onboarding flow that checks Screen Recording / Microphone / Accessibility and links to the right Settings panes.

### v1 polish
- **Bundle fonts** — currently loaded from Google Fonts CDN. Spec wants offline-first via `@font-face` from `src-tauri/Resources/fonts/`. Pull DM Sans, Syne, JetBrains Mono.
- **Native vibrancy on the menubar popover** — `NSVisualEffectView` material `.hudWindow`. Solid `--color-bg-overlay` fallback is in place.
- **Custom dashboard titlebar** — hidden traffic lights at `12px 14px`. The current `TitleBarStyle: Overlay` is close but not pixel-accurate per DESIGN.md §8.
- **Bundle-ID extraction** — `commands::exclusion_add` accepts a payload from the UI, but the UI currently derives the bundle ID from the app name. Read `CFBundleIdentifier` from the chosen `.app`'s `Info.plist` (Rust side, via `plist` crate).
- **Synthesis retries + partial recovery** — if Ollama dies mid-pipeline, the session is marked complete with empty results. Add per-step error capture and a "Retry synthesis" button on the failed session row.
- **Vault writer wikilinks** — the synthesis prompt asks the LLM to include `[[wikilinks]]`, but we don't yet cross-reference them against a vault title index (spec §3.3, step 4). Build a simple `Map<title, path>` and post-process to fix dangling links.

### v2 (per SPEC.md §7)
- Inbox summarizer (Gmail OAuth + IMAP)
- Cloud API tier (BYOK Claude / OpenAI)
- Windows support
- HNSW index auto-upgrade for large vaults
- AES-256 encrypted raw session DBs at rest

---

## Privacy

- No network calls from the app in v1 except to `localhost:11434` (Ollama).
- No telemetry, analytics, or crash reporting.
- All data in `~/.argus/` — fully user-owned and deletable.
- The Obsidian vault path is the only filesystem location outside `~/.argus/` that Argus writes to, and only during the final step of synthesis.

---

## License

TBD.
