# Argus — Frontend Design Document
**Version:** 1.0 | **Date:** 2026-06-04 | **Status:** Pre-development

---

## 1. Design Philosophy

Argus is a precision instrument for the knowledge worker. It sits at the intersection of surveillance and scholarship — a guardian that watches so you don't have to. The UI should feel like an **observatory terminal**: purposeful, intimate, dark-adapted, technical without being cold. Every pixel earns its place.

**Core principles:**
- **Invisible when idle.** The app lives in the menubar. The dashboard only appears when the user summons it. Never demand attention.
- **Signal over chrome.** Recording state is the single most important status in the UI. Everything else is secondary. The eye is always watching or not watching — nothing in between.
- **Knowledge as craft.** Session history and vault notes are permanent artifacts of thought. Treat them with typographic respect — not like rows in a spreadsheet, but like entries in a ledger.
- **Density with clarity.** Power users want information. Don't artificially spacious-ify. Pack intelligently.

---

## 2. Visual Language

### 2.1 Color Palette

The palette is drawn from a darkened observatory: near-black as the sky, brass and amber for instruments, a clean green pulse for life.

```
--color-bg-base:        #09090D   /* deepest background — near void */
--color-bg-surface:     #111218   /* card/panel backgrounds */
--color-bg-elevated:    #191B24   /* hover states, active rows */
--color-bg-overlay:     #1E2130   /* modals, popovers */

--color-border-subtle:  #1E2030   /* hairline separators */
--color-border-default: #2A2D3A   /* component borders */
--color-border-active:  #3D4157   /* focused/interactive borders */

--color-text-primary:   #E4DFCF   /* warm parchment — main text */
--color-text-secondary: #8B8578   /* metadata, labels, de-emphasized */
--color-text-tertiary:  #524F4A   /* timestamps, disabled states */
--color-text-inverse:   #0A0B0E   /* text on bright surfaces */

--color-brass:          #C8922A   /* primary accent — brass instrument */
--color-brass-dim:      #7A5515   /* secondary accent, borders */
--color-brass-glow:     #E5B54E   /* highlights, hover emphasis */

--color-active:         #3EC87A   /* recording — clean green pulse */
--color-active-dim:     #1B5C38   /* recording bg tint */
--color-paused:         #C8922A   /* paused — same brass; intentional */
--color-synthesizing:   #7B6CF0   /* AI working — indigo, otherworldly */
--color-synthesizing-dim:#30275A  /* synthesizing bg tint */
--color-interrupted:    #C24444   /* error/interrupted */

--color-obsidian:       #4E8BC0   /* vault-related actions — Obsidian purple-blue */
```

**Usage rules:**
- Use `--color-bg-base` for the window background. Never pure `#000000`.
- `--color-brass` is the only warm accent — use it sparingly (one CTA per screen maximum).
- Status colors (`--color-active`, `--color-paused`, `--color-synthesizing`) appear only in state indicators and status badges. Never decoratively.
- All backgrounds use a `0.6% noise overlay` via SVG filter to avoid flat digital look.

### 2.2 Typography

Three typefaces, each with a clear role.

| Role | Family | Weights | Notes |
|---|---|---|---|
| Display / Headers | **Syne** | 600, 700, 800 | Geometric, cuts sharp. Session titles, section headers. |
| Interface / Body | **DM Sans** | 300, 400, 500 | Clean humanist, reads at 12px. Labels, descriptions, body. |
| Data / Mono | **JetBrains Mono** | 400, 500 | Timestamps, paths, IDs, code snippets. Variable font. |

```
--font-display:   'Syne', system-ui, sans-serif;
--font-body:      'DM Sans', system-ui, sans-serif;
--font-mono:      'JetBrains Mono', 'Menlo', monospace;

--size-2xs:  10px   /* status badge text */
--size-xs:   11px   /* tertiary metadata */
--size-sm:   12px   /* secondary labels, table data */
--size-base: 13px   /* primary UI text (macOS density) */
--size-md:   14px   /* slightly emphasized labels */
--size-lg:   16px   /* section subtitles */
--size-xl:   18px   /* panel titles */
--size-2xl:  22px   /* screen titles */
--size-3xl:  28px   /* large display, session timer */
--size-4xl:  42px   /* hero numerics (session duration, note counts) */

--leading-tight:  1.2
--leading-normal: 1.5
--leading-loose:  1.7

--tracking-tight:  -0.03em  /* display headers */
--tracking-normal: -0.01em  /* body */
--tracking-wide:   0.06em   /* all-caps labels, badges */
--tracking-wider:  0.12em   /* menubar context labels */
```

**Typographic rules:**
- Session titles: `Syne 600, 16px, tracking -0.02em`
- Section headers: `DM Sans 500, 11px, tracking 0.08em, ALL CAPS, --color-text-secondary`
- Timestamps and paths: always `JetBrains Mono, 11px, --color-text-tertiary`
- Action items: `DM Sans 400, 13px` with a subtle left border in `--color-brass-dim`

### 2.3 Spacing System

8px base unit. All spacing in multiples of 4.

```
--space-1:   4px
--space-2:   8px
--space-3:  12px
--space-4:  16px
--space-5:  20px
--space-6:  24px
--space-8:  32px
--space-10: 40px
--space-12: 48px
--space-16: 64px
```

### 2.4 Shape & Elevation

Argus is a desktop app — corners are tight, not bubbly. Border radii are conservative.

```
--radius-sm:  3px   /* buttons, badges, inputs */
--radius-md:  6px   /* cards, panels, popovers */
--radius-lg: 10px   /* main window, modal overlays */

--shadow-surface:   0 1px 3px rgba(0,0,0,0.5), 0 1px 2px rgba(0,0,0,0.6);
--shadow-elevated:  0 4px 16px rgba(0,0,0,0.7), 0 1px 4px rgba(0,0,0,0.5);
--shadow-float:     0 12px 40px rgba(0,0,0,0.85), 0 2px 8px rgba(0,0,0,0.6);
--shadow-glow-brass: 0 0 20px rgba(200,146,42,0.15);
--shadow-glow-active: 0 0 16px rgba(62,200,122,0.2);
```

### 2.5 The Argus Eye

The app icon and primary state indicator is an **eye glyph** — stylized, geometric, not literal. The eye is constructed from two arcs forming a vesica piscis lens shape with a centered circle (iris). No eyelashes. No realism.

**Eye states:**
- **Idle:** Monochrome `--color-text-tertiary`. Still. The eye is closed (lids closed to a thin horizontal line).
- **Active/Recording:** Full open eye, iris filled `--color-active`, animated radial pulse every 3s (subtle expand + fade, not distracting).
- **Paused:** Half-open (top lid at center). `--color-paused` (amber). Slow breathe animation: opacity 0.6 → 1.0 → 0.6 over 4s.
- **Synthesizing:** Full open, iris cycling through indigo `--color-synthesizing`, slow clockwise rotation of a subtle iris texture, 20s loop.

The eye SVG is inline in all contexts — never an `<img>` tag, to allow CSS animation and fill manipulation.

---

## 3. Menubar Popover

The primary interaction surface. Should open instantly and feel native.

### Layout

```
┌────────────────────────────────┐  width: 280px
│  ○ ARGUS          ⋯           │  header: 40px, border-bottom
│                                │
│  ┌──────────────────────────┐  │  state card: padding 16px
│  │  [EYE ICON]              │  │
│  │  Idle                    │  │  state label: Syne 600 18px
│  │  No active session       │  │  sub-label: DM Sans 400 12px
│  └──────────────────────────┘  │
│                                │
│  ┌──────────────────────────┐  │  primary CTA: full width
│  │  ▶  Start Session        │  │  height: 36px, --color-brass bg
│  └──────────────────────────┘  │
│                                │
│  ─────────────────────────── ─ │  divider
│  ⊞  Open Dashboard    ↗       │  14px, secondary action
│  ✕  Quit Argus                │  14px, secondary action
└────────────────────────────────┘
```

**Active session state:**

```
┌────────────────────────────────┐
│  ● ARGUS  [●●●●●●●●●●]  04:23 │  header: eye + waveform + timer
│                                │  timer: JetBrains Mono 400 14px
│  ┌──────────────────────────┐  │
│  │  [EYE — OPEN, PULSING]   │  │
│  │  Recording               │  │  --color-active, Syne 600 18px
│  │  Started 2:17 PM         │  │
│  └──────────────────────────┘  │
│                                │
│  ┌────────────┐┌─────────────┐ │  two buttons side-by-side
│  │  ⏸ Pause  ││  ■ Stop     │ │  Pause: secondary; Stop: destructive
│  └────────────┘└─────────────┘ │
│                                │
│  ─────────────────────────────  │
│  ⊞  Open Dashboard    ↗        │
└────────────────────────────────┘
```

**Waveform strip:** A live 60px-wide mini audio waveform in the header, rendered as 20 vertical bars from right to left. Bars are `--color-active` at 30% opacity. Updates every 500ms. Pure CSS animation if no audio data, using `animation-delay` stagger on a sine-curve keyframe sequence.

**Session timer:** Counts up from `00:00`. Format: `MM:SS` under 1 hour, `H:MM:SS` above. JetBrains Mono, tabular-nums. No label — just the number. Next to the eye in the header.

### Interaction

- Opens on left-click of menubar icon. Width fixed 280px. Anchors to right edge of icon.
- Popover background: `--color-bg-overlay` with `backdrop-filter: blur(20px)` on macOS (vibrancy-adjacent feel).
- Appears with a 120ms ease-out scale from 0.96 → 1.0 + fade. Dismisses on click-outside or Escape.
- No scrollbar. If content overflows — it doesn't in this design.

---

## 4. Dashboard Window

Main window. Opened via menubar → "Open Dashboard" or right-click context. Resizable, minimum 720×520px, default 900×600px.

### Overall Layout

```
┌──────────────────────────────────────────────────────────┐
│  [TITLEBAR — macOS native, hidden traffic lights]        │
├──────────┬───────────────────────────────────────────────┤
│          │                                               │
│  SIDEBAR │  CONTENT AREA                                 │
│   48px   │  fills remainder                              │
│          │                                               │
│  [eye]   │                                               │
│          │                                               │
│  ────    │                                               │
│          │                                               │
│  [hist]  │                                               │  Nav icons: 28px,
│  [llm]   │                                               │  centered in 48px
│  [cap]   │                                               │  column. Active:
│  [vault] │                                               │  --color-brass
│          │                                               │  Inactive: --color-text-tertiary
│  ────    │                                               │
│          │                                               │
│  [gear]  │                                               │
└──────────┴───────────────────────────────────────────────┘
```

**Sidebar icons (top to bottom):**
1. Eye logo / Argus mark — not a nav item, just branding. 32px.
2. Separator
3. `ClockHistory` icon — Session History (default view)
4. `BrainCircuit` icon — LLM Config
5. `Aperture` icon — Capture Settings
6. `BookOpen` icon — Vault Settings
7. Separator
8. `Settings2` icon — General/About

All icons: Lucide icon set. No labels on sidebar — tooltip on hover only (native macOS tooltip, not custom).

### 4.1 Session History (Primary View)

**Header bar** (48px tall, full content-area width, border-bottom):
```
Sessions                                     [▶ Start New Session]
SECTION HEADER style                         brass CTA button, 32px height
```

**Session list area** — scrollable, padding `0 24px`.

**Empty state:** When no sessions exist:
```
          [large eye icon — 48px, --color-text-tertiary]

          No sessions yet.
          DM Sans 400, 15px, --color-text-secondary

          Start a session to begin capturing your work.
          DM Sans 400 13px, --color-text-tertiary

          [Start Your First Session] — brass button, centered
```

**Session row** (default, collapsed):
```
┌────────────────────────────────────────────────────────┐
│ ●  Refactoring Auth Middleware — Python    45m 12s  ▾  │
│    Wed Jun 4, 2026 · 2:17–3:02 PM  [complete]         │
└────────────────────────────────────────────────────────┘
```

Row anatomy:
- Status dot (8px circle, `--color-active` for complete, `--color-interrupted` for interrupted, `--color-synthesizing` animated for synthesizing)
- Title: `Syne 600, 14px, --color-text-primary`
- Duration: `JetBrains Mono 400, 12px, --color-text-secondary`, right-aligned
- Expand chevron: `--color-text-tertiary`, rotates 180° on expand
- Second line: timestamp range + status badge, `DM Sans 400, 11px, --color-text-tertiary`
- Row height: 56px collapsed, expands to fit content
- Hover: `--color-bg-elevated` background transition 80ms
- Active session row: left border `2px solid --color-active`, bg tint `--color-active-dim` at 15%

**Status badges:**
```
[complete]       — text-only, --color-active, tracking-wide, 10px
[synthesizing]   — text + indigo dot, animated
[paused]         — text-only, --color-paused
[interrupted]    — text-only, --color-interrupted
```
No background fill on badges — just colored text and tracking. They read as metadata, not alerts.

**Session row — expanded:**
```
┌────────────────────────────────────────────────────────┐
│ ●  Refactoring Auth Middleware — Python    45m 12s  ▴  │
│    Wed Jun 4, 2026 · 2:17–3:02 PM  [complete]         │
│ ─────────────────────────────────────────────────────  │
│                                                        │
│  ACTION ITEMS                                          │  section label style
│  ▸ Write tests for the new token refresh logic         │
│  ▸ Check session expiry edge case in staging           │
│  ▸ PR review with Mia by Thursday                      │
│                                                        │
│  VAULT NOTES                                           │
│  ↳ Projects/Auth Refactor.md          · appended      │
│  ↳ Reference/JWT Patterns.md          · created       │
│                                                        │
│          [View Summary]  [Open in Obsidian ↗]         │
└────────────────────────────────────────────────────────┘
```

Expanded section details:
- Action items: `DM Sans 400, 13px`, `--color-text-primary`, left-aligned. Triangle bullet `▸` in `--color-brass`.
- Vault files: `JetBrains Mono 400, 11px`, `--color-text-secondary`. Arrow prefix `↳` in `--color-obsidian`. Appended/created tag in `--color-text-tertiary`.
- Expand animation: 200ms ease-out height from 0 to content height.

**Row separator:** `1px solid --color-border-subtle`. Full width of list.

**Synthesizing in-progress row:**
```
┌────────────────────────────────────────────────────────┐
│ ◉  session_2026-06-04_1435          Synthesizing...    │
│    Started 2:35 PM                  [synthesizing]     │
│ ─────────────────────────────────────────────────────  │
│    [indigo progress bar — indeterminate, animated]     │
│    Extracting concepts from 342 frames...              │
└────────────────────────────────────────────────────────┘
```
Progress bar: `4px` tall, `--color-synthesizing`, shimmer animation (highlight sweeps left-to-right, 1.5s loop). Status text updates via Tauri event as synthesis pipeline steps complete.

---

### 4.2 Post-Synthesis Summary Screen

Shown automatically when synthesis completes (a Tauri event triggers navigation). Not a modal — a full content-area view. Can be returned to from session history.

```
┌────────────────────────────────────────────────────────┐
│  ← Back to Sessions                                    │  small back button
│                                                        │
│  Session Complete                                      │  DM Sans 400 11px caps label
│  ┌──────────────────────────────────────────────────┐  │
│  │  Refactoring Auth Middleware — Python            │  │  Syne 700 22px, editable
│  │  [edit cursor appears on hover/focus]            │  │  
│  └──────────────────────────────────────────────────┘  │
│  Wed Jun 4, 2026  ·  2:17 PM – 3:02 PM                 │  JetBrains Mono 12px secondary
│                                                        │
│  ┌──────────┬─────────────┬──────────────────┐         │  stat row
│  │  45:12   │      2      │        3         │         │
│  │ Duration │Notes Updated│  Action Items    │         │
│  └──────────┴─────────────┴──────────────────┘         │
│                                                        │
│  ─────────────────────────────────────────────────     │
│                                                        │
│  ACTION ITEMS                                          │  section label
│  ☐  Write tests for the new token refresh logic        │
│  ☐  Check session expiry edge case in staging          │
│  ☐  PR review with Mia by Thursday                     │
│                                                        │
│  OPEN QUESTIONS                                        │
│  ?  Should the refresh endpoint be rate-limited?       │
│  ?  Confirm Mia's branch naming convention             │
│                                                        │
│  VAULT NOTES AFFECTED                                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  ↳ Projects/Auth Refactor.md             appended │  │
│  │     "Added section: Token Refresh Logic"           │  │
│  │                                                    │  │
│  │  ↳ Reference/JWT Patterns.md              created │  │
│  │     "New note created with 4 sections"             │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│        [Open in Obsidian ↗]   [Start New Session]     │  action bar
└────────────────────────────────────────────────────────┘
```

**Stat row** (the three numbers):
- Values: `Syne 800, 32px, --color-text-primary`
- Labels: `DM Sans 400, 11px, --color-text-secondary, tracking-wide, ALL CAPS`
- Dividers between stats: `1px solid --color-border-subtle`, full height of row
- Row padding: `20px 0`, border-top and border-bottom `--color-border-subtle`

**Editable title:**
- Displays as styled text (Syne 700, 22px) until focused
- On hover: subtle cursor change, thin underline in `--color-brass-dim` appears
- On focus: inline text input reveals, styled identically (no visible input box), cursor `--color-brass`
- On blur/enter: saves via Tauri command, animates back to display state
- Pencil icon (`16px, --color-text-tertiary`) appears on row hover, disappears on focus

**Action items checkboxes:**
- Custom checkbox: `14px` square, `--color-border-default` border, `3px --radius-sm`
- Checked state: filled `--color-brass`, checkmark in `--color-text-inverse`
- These are ephemeral (session-scoped UI state), not synced to vault

**Vault notes panel:**
- Background: `--color-bg-surface`, `--radius-md`, `1px solid --color-border-subtle`
- Each note row: path in `JetBrains Mono 12px`, action tag right-aligned in `--color-text-tertiary`
- Subtitle under path: LLM-generated one-line summary of what was written, `DM Sans 400 12px --color-text-secondary`

---

## 5. Settings Panels

Accessed via sidebar icons. Full content-area replacement. Each panel has a consistent structure.

**Panel anatomy:**
```
┌────────────────────────────────────────────────────────┐
│  SECTION TITLE                                         │  DM Sans 500 11px caps
│  Description of what this panel controls.              │  DM Sans 400 13px secondary
│  ──────────────────────────────────────────────────    │
│                                                        │
│  SUBSECTION LABEL                                      │  DM Sans 500 11px caps
│  [form content]                                        │
│                                                        │
└────────────────────────────────────────────────────────┘
```

Max content width: `560px`, centered in the content area. Padding: `32px 24px`.

### 5.1 LLM Config

```
  LLM CONFIGURATION
  Connect Argus to your local Ollama instance.
  ──────────────────────────────────────────

  OLLAMA CONNECTION

  Host URL                       [http://localhost:11434      ]
                                 ○ Connected  ·  llama3.2 detected

  INFERENCE MODEL

  Model name                     [llama3.2                    ]
  Used for synthesis, extraction, and smart-append decisions.

  EMBEDDING MODEL

  Model name                     [nomic-embed-text            ]
  Used for vault vector indexing.

                    [Test Connection]   [Save]
```

**Connection status indicator:**
- `○ Connected` — `--color-active` dot, DM Sans 12px
- `○ Unreachable` — `--color-interrupted` dot
- `○ Checking...` — animated dot, `--color-text-tertiary`

**Input fields:**
- Height: `34px`, full width (minus padding), `--color-bg-surface` bg
- Border: `1px solid --color-border-default`, radius `--radius-sm`
- Focus: border `--color-brass-dim`, `box-shadow: 0 0 0 3px rgba(200,146,42,0.12)`
- Font: `JetBrains Mono 13px` (URLs and model names are technical strings)

### 5.2 Capture Settings

```
  CAPTURE SETTINGS
  Control what Argus records during a session.
  ──────────────────────────────────────────

  APP EXCLUSION LIST
  These apps will auto-pause recording when focused.

  ┌─────────────────────────────────────────────────┐
  │  1Password                com.1password.1password│
  │  Keychain Access          com.apple.keychainAccess│
  └─────────────────────────────────────────────────┘

  [+ Add App]

  DATA RETENTION

  Delete raw session data after   [30]  days
  Session summaries and vault notes are kept permanently.

  MINIMUM SESSION DURATION

  Ignore sessions shorter than    [60]  seconds
```

**Exclusion list:** Table with two columns (display name, bundle ID). DM Sans 13px. Each row has a `✕` remove button on hover (`--color-interrupted`, 12px). "Add App" opens a macOS file picker filtered to `.app` bundles — bundle ID extracted from `Info.plist`.

**Numeric inputs:** `48px` wide, centered text, `JetBrains Mono`. Arrow up/down controls hidden; use keyboard or direct input.

### 5.3 Vault Settings

```
  VAULT SETTINGS
  Configure your Obsidian vault integration.
  ──────────────────────────────────────────

  VAULT PATH

  [/Users/chris/Documents/Obsidian Vault    ] [Choose…]

  SMART APPEND

  Similarity threshold            [0.75]
  Higher values = only append to very closely related notes.
  Range: 0.5 – 0.95

  VECTOR INDEX

  Status:  ○ Indexed  ·  2,847 notes  ·  18,340 chunks
  Last updated: Today at 3:47 PM

  [Re-index Vault]

  ─────────────────────────────────────────────────────
  [progress bar — shown during indexing only]
  Indexing 847 of 2,847 notes...
```

**Path field:** Read-only text input showing the path; `[Choose…]` button triggers macOS folder picker. `JetBrains Mono 12px` for path.

**Index status:**
- `○ Indexed` — `--color-active` dot
- `○ Not indexed` — `--color-text-tertiary` dot
- `○ Indexing...` — animated dot, indigo

**Re-index progress bar:** Same style as synthesis progress bar in session rows. 4px height, `--color-synthesizing` fill, shimmer animation. Disappears when indexing complete.

---

## 6. Component Library

### Buttons

```
/* Primary (brass) */
background: --color-brass
color: --color-text-inverse
font: DM Sans 500 13px
padding: 0 16px, height 34px
radius: --radius-sm
hover: brightness(1.12), shadow-glow-brass
active: brightness(0.92)
transition: 120ms ease

/* Secondary */
background: transparent
color: --color-text-secondary
border: 1px solid --color-border-default
hover: --color-bg-elevated border, --color-text-primary text
active: brightness(0.9)

/* Ghost / Text */
background: transparent, no border
color: --color-text-secondary
hover: --color-text-primary
Underline animates in on hover (width 0 → 100%, 160ms ease)

/* Destructive */
color: --color-interrupted
border: 1px solid rgba(194,68,68,0.3)
hover: background rgba(194,68,68,0.1)

/* Icon button */
28×28px, --radius-sm
hover: --color-bg-elevated
active: scale(0.93)
```

### Form Inputs

```
height: 34px
background: --color-bg-surface
border: 1px solid --color-border-default
border-radius: --radius-sm
padding: 0 10px
font: DM Sans 400 13px, --color-text-primary
placeholder: --color-text-tertiary

:focus
  border-color: --color-brass-dim
  box-shadow: 0 0 0 3px rgba(200,146,42,0.10)
  outline: none

:disabled
  opacity: 0.45
  cursor: not-allowed
```

### Separators / Dividers

```
border: none
border-top: 1px solid --color-border-subtle
margin: 8px 0

/* Section divider (with label) */
display: flex, align-items: center, gap: 12px
label: DM Sans 400 11px, --color-text-tertiary, tracking-wider, ALL CAPS
lines: 1px solid --color-border-subtle, flex: 1
```

### Scrollbars

```
::-webkit-scrollbar
  width: 4px

::-webkit-scrollbar-track
  background: transparent

::-webkit-scrollbar-thumb
  background: --color-border-default
  border-radius: 2px

::-webkit-scrollbar-thumb:hover
  background: --color-border-active
```

Scrollbars appear only on hover of the scrollable container (CSS `overflow: hidden` → `overflow: auto` on parent `:hover`).

---

## 7. Motion & Animation

**Philosophy:** Motion communicates state, not decoration. Every animation has a functional reason.

### Durations

```
--duration-instant:  80ms   /* immediate feedback (button press) */
--duration-fast:    120ms   /* hover states, small transitions */
--duration-normal:  200ms   /* panel switches, row expand */
--duration-slow:    320ms   /* screen transitions, modal entrance */
--duration-deliberate: 500ms /* only for meaningful moments (synthesis complete) */
```

### Easing

```
--ease-default:     cubic-bezier(0.16, 1, 0.3, 1)   /* fast-out, ease-out */
--ease-spring:      cubic-bezier(0.34, 1.56, 0.64, 1) /* slight bounce */
--ease-linear:      linear                             /* loops, progress */
```

### Key Animations

**Recording pulse (eye iris):**
```css
@keyframes recording-pulse {
  0%   { transform: scale(1);   opacity: 1;   }
  60%  { transform: scale(1.3); opacity: 0.4; }
  100% { transform: scale(1);   opacity: 1;   }
}
/* Applied to eye iris ring: 3s ease-in-out infinite */
```

**Synthesis shimmer (progress bar):**
```css
@keyframes shimmer {
  0%   { background-position: -200% center; }
  100% { background-position:  200% center; }
}
background: linear-gradient(
  90deg,
  --color-synthesizing 0%,
  rgba(123,108,240,0.5) 50%,
  --color-synthesizing 100%
);
background-size: 200% 100%;
animation: shimmer 1.5s linear infinite;
```

**Panel entrance:**
```css
@keyframes panel-in {
  from { opacity: 0; transform: translateX(8px); }
  to   { opacity: 1; transform: translateX(0);   }
}
animation: panel-in 200ms var(--ease-default);
```

**Session row expand:**
Height from `56px` to `auto` using `grid-template-rows: 0fr → 1fr` trick. 200ms ease.

**Synthesis complete notification:**
When synthesis finishes, the session row in history briefly highlights: border flashes `--color-synthesizing` then settles to default. Duration: 500ms. Then the post-synthesis summary screen slides in from the right (translateX 24px → 0, 320ms).

**Waveform (menubar, active state):**
20 bars, each with independent `animation-delay` from 0 to 1.8s, sine curve between 30% and 100% height. `--color-active`, 30% opacity. CSS only — no JS needed.

---

## 8. macOS Integration Notes

### Window Chrome

- `decorations: false` in Tauri config — custom titlebar, no native title
- Draggable area: top 40px of window (data-tauri-drag-region)
- Traffic light buttons (close/minimize/zoom): positioned at `12px 14px`, native via `tauri_plugin_window::effects`
- Window background: `--color-bg-base` via Tauri `window.setBackground`

### Vibrancy

For the menubar popover only (not the main window):
- `NSVisualEffectView` with `material: .hudWindow` and `blendingMode: .behindWindow`
- Fallback: `--color-bg-overlay` solid if vibrancy unavailable

### Native Context Menus

Right-click on session rows: native macOS context menu via `tauri-plugin-shell` or custom Svelte context menu styled to match.

### Font Loading

All three typefaces loaded via `@font-face` from bundled assets in `Resources/fonts/`. No Google Fonts CDN — offline-first app.

---

## 9. Accessibility

- All interactive elements reachable via Tab. Focus rings: `2px solid --color-brass, 2px offset`.
- No color-only information: status badges include text labels, not just colored dots.
- `prefers-reduced-motion`: all looping/pulse animations replaced with static states. Entrance animations cut to 0ms.
- Minimum tap/click target: `28×28px` for icon buttons.
- `aria-label` on all icon-only controls.
- Contrast ratios: `--color-text-primary` on `--color-bg-base` = 11.2:1 (AAA). `--color-text-secondary` on `--color-bg-base` = 5.8:1 (AA).

---

## 10. File Organization

```
src/
├── lib/
│   ├── components/
│   │   ├── eye/
│   │   │   ├── ArgusEye.svelte      # SVG eye with state props
│   │   │   └── EyeStates.ts         # state → animation mapping
│   │   ├── session/
│   │   │   ├── SessionRow.svelte
│   │   │   ├── SessionTimer.svelte
│   │   │   ├── SynthesisProgress.svelte
│   │   │   └── SummaryScreen.svelte
│   │   ├── menubar/
│   │   │   └── MenubarPopover.svelte
│   │   ├── settings/
│   │   │   ├── LLMConfig.svelte
│   │   │   ├── CaptureSettings.svelte
│   │   │   └── VaultSettings.svelte
│   │   └── ui/
│   │       ├── Button.svelte
│   │       ├── Input.svelte
│   │       ├── Badge.svelte
│   │       ├── Waveform.svelte
│   │       └── ProgressBar.svelte
│   ├── stores/
│   │   ├── session.ts               # current session state
│   │   └── settings.ts              # app settings cache
│   └── styles/
│       ├── tokens.css               # all CSS custom properties
│       ├── typography.css           # font-face declarations
│       ├── base.css                 # reset + body defaults
│       └── animations.css           # shared keyframe definitions
├── routes/
│   ├── +layout.svelte               # sidebar + content frame
│   ├── sessions/+page.svelte        # session history
│   ├── summary/[id]/+page.svelte    # post-synthesis summary
│   └── settings/
│       ├── llm/+page.svelte
│       ├── capture/+page.svelte
│       └── vault/+page.svelte
└── app.html
```

---

## 11. Open Design Questions

- Should the waveform in the menubar be a real FFT visualization (requires audio data from Rust) or a stylized fake waveform? Real is more authentic but requires IPC. Recommendation: fake CSS waveform for v1, real FFT in v2.
- Session title edit: should unsaved changes be persisted on window close, or prompt to save? Recommendation: auto-save on blur.
- Should the synthesis progress step labels ("Extracting concepts…", "Querying vault…") be shown in the session row during synthesis or only in the post-synthesis summary? Recommendation: in the row, live-updated via Tauri events — it's a meaningful transparency moment.
- Menubar icon: the eye glyph needs to render cleanly at `18×18px` (standard macOS menubar icon size). Needs careful SVG optimization and testing at 1x and 2x (Retina).
