# Argus — Operational Handoff & Session Guide (GEMINI.md)

**Date:** 2026-08-31 / 2026-09-01  
**Current Branch:** `main` (Clean & Synchronized with `origin/main`)  
**Status:** Phase 1 Complete (Backend Architecture, Map-Reduce Prototype, Mock Test Harness, UI Overhaul)

---

## 1. What We Accomplished in This Session

### Backend & Core Engine (Rust + Tauri v2)
1. **Decoupled `LlmClient` Trait & JSON Sanitizer** (`src-tauri/src/llm/`):
   - Created the async `LlmClient` trait (`complete`, `complete_json`, `embed`, `ping`).
   - Implemented `OllamaClient` as the local default provider.
   - Built `parse_llm_json` with automated markdown code-fence stripping and bracket extraction for resilient parsing with local LLMs.
2. **Map-Reduce Session Synthesis Pipeline** (`src-tauri/src/synthesis.rs`):
   - Chronological two-pointer merge for timeline pairing without dropping quiet screen frames.
   - Slices long sessions ($>10$ mins) into 10-minute windows, maps each to micro-summaries, and reduces them into unified concepts, titles, and action items without context truncation.
3. **No-Default LLM Fields & Strict Multi-Stage Preflight** (`src-tauri/src/settings.rs`, `db.rs`, `commands.rs`):
   - Removed hardcoded default seeds (`http://localhost:11434`, `llama3.2`, `nomic-embed-text`) from `Db::seed_defaults` and `settings::get_all`.
   - Converted `ollama_host`, `ollama_model`, and `embed_model` to `Option<String>`.
   - Implemented multi-stage verification in `commands::session_start` and `commands::llm_status`: verifies host configured, model configured, daemon reachability, and model presence against `/api/tags` via `model_matches`.
   - Purged legacy pre-seeded defaults from existing SQLite databases on boot.
4. **Pinned Screenpipe Auto-Downloader** (`src-tauri/src/screenpipe.rs`):
   - Resolves binary from `~/.argus/bin/screenpipe` or bundled fallback.
   - Implemented `download_pinned_screenpipe` for pinned release `v0.1.72` with SHA-256 validation, `chmod 0o755`, and macOS `xattr -d com.apple.quarantine`.
5. **Synthetic Mock Session Generator & Test Suite** (`src-tauri/src/mock.rs`, `src-tauri/tests/synthesis_test.rs`):
   - Created realistic multi-phase deep work session generator (`dev_seed_mock_session`).
   - 14 unit and integration tests passing (`cargo test`).
6. **RAII `Drop` Cleanup & Startup Sweep** (`src-tauri/src/vault/writer.rs`, `src-tauri/src/state.rs`):
   - Automatic temp file deletion on error/panic via `TempHandle::drop`.
   - On launch, Argus sweeps and cleans any orphaned `.argus-tmp-*` files from prior system crashes.
7. **Window Lifecycle & Dock Reopen** (`src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`):
   - Dashboard window starts visible (`"visible": true`).
   - Hooked up `RunEvent::Reopen` to bring the Argus window to the front when clicking the macOS Dock icon.

### Frontend & UI (Svelte 5 + SvelteKit)
1. **Custom Color Palette** (`src/lib/styles/tokens.css`):
   - **Slate Charcoal** (`#272931` surfaces, `#18191F` base void)
   - **Moss Green** (`#668727` active recording, `#233E1A` dark tint)
   - **Warm Sand / Brass** (`#C4B481` primary CTA, paused state, borders)
   - **Ice Blue** (`#B1D1EC` Obsidian & synthesis actions)
   - **Crisp White** (`#FFFFFF` high-contrast text)
2. **Centered Hero Home Page & Preflight Safeguards** (`src/routes/+page.svelte`):
   - Bold, solid display title (`ARGUS` in Brassie / Montserrat Black 900).
   - Centered large CTA button: **"Start Session"**.
   - LLM preflight guard: Prevents starting session if Ollama is unreachable or model is missing; displays warning dialog with direct link to LLM config.
   - Missing vault confirmation dialog: Warns user if no Obsidian vault is configured and asks whether to proceed.
3. **LLM Settings with Click-to-Select Model Chips** (`src/routes/settings/llm/+page.svelte`):
   - Replaced default inputs with clean empty states.
   - Live Ollama testing checks host connection and displays detected installed models as interactive chips (e.g. `qwen2.5-coder:14b`) for instant selection.
4. **Menubar Popover Dismissal & Lifecycle Fixes** (`src/routes/menubar/+page.svelte`, `src-tauri/src/tray.rs`):
   - Menubar popover automatically closes on focus loss (blur), clicking "Close", pressing Escape, stopping a session, or opening the dashboard.
   - Fixed glitch where Close button called `location.reload()` instead of hiding the window.
5. **Non-Obsidian Note Summary View** (`src/lib/components/session/SummaryScreen.svelte`, `SessionRow.svelte`):
   - When no Obsidian vault is configured, displays generated notes summary cleanly without broken Obsidian links.
   - Added `note_read` command allowing inline viewing of generated Markdown note content.
   - Added toggleable `warn_missing_vault` setting in Settings $\rightarrow$ Vault.

---

## 2. Key Codebase Map

| Subsystem | Primary Files | Key Components |
|---|---|---|
| **App Entry & IPC** | `src-tauri/src/lib.rs`, `commands.rs` | Tauri commands, Dock click reopen |
| **State Management** | `src-tauri/src/state.rs`, `session.rs` | `AppState`, `CurrentSession`, SQLite CRUD |
| **LLM Provider Layer** | `src-tauri/src/llm/mod.rs`, `ollama.rs` | `LlmClient` trait, `OllamaClient`, JSON cleaner |
| **Synthesis Pipeline** | `src-tauri/src/synthesis.rs` | Timeline pairing, Window Map, Reduce, Note generation |
| **Vector DB & Vault** | `src-tauri/src/vault/index.rs`, `writer.rs`, `watcher.rs` | `sqlite-vec` (vec0), atomic markdown writer, fs watcher |
| **Capture & Screenpipe** | `src-tauri/src/screenpipe.rs`, `mock.rs` | Pinned downloader, process manager, mock generator |
| **Frontend UI** | `src/routes/+page.svelte`, `+layout.svelte`, `menubar/+page.svelte` | Hero home, sidebar navigation, menubar popover |
| **Design System** | `src/lib/styles/tokens.css`, `typography.css` | Colors, fonts, shadows, animations |

---

## 3. Priority Next Steps for Next Session

1. **Screenpipe Download & Permission Preflight UX**:
   - Add a "Download Screenpipe Engine" button or first-launch prompt in Settings $\rightarrow$ Capture.
   - Verify screen recording and microphone permissions gracefully.
2. **App Exclusion List Background Poller**:
   - Add the 1-second polling loop (`NSWorkspace.shared.frontmostApplication`) to auto-pause (`SIGSTOP`) Screenpipe when an excluded app is in focus.
3. **Wikilinks Cross-Referencing**:
   - Build a vault note index cache during synthesis to resolve `[[wikilinks]]` against existing notes.
4. **Live End-to-End Recording Verification**:
   - Test a real recording session with local Ollama (`llama3.2` + `nomic-embed-text`) exporting into an Obsidian vault.

---

## 4. Commands Quick Reference

```bash
# Run the desktop app in live dev mode (hot reloading)
npm run tauri:dev

# Run automated backend test suite (all 14 tests)
cargo test --manifest-path src-tauri/Cargo.toml

# Run frontend type and diagnostic checks
npm run check

# Build production downloadable installer (.dmg / .app)
npm run tauri:build
```
