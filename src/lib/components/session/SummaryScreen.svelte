<script lang="ts">
  import type { SessionRecord } from '$lib/types';
  import { invoke } from '@tauri-apps/api/core';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { goto } from '$app/navigation';
  import { formatDateLong, formatTimeRange, formatDurationLong, basename } from '$lib/format';

  interface Props { session: SessionRecord; }
  let { session = $bindable() }: Props = $props();

  import { tick, onMount } from 'svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  let editing = $state(false);
  let titleInput = $state(session.displayName ?? session.id);
  let titleEl = $state<HTMLInputElement | null>(null);
  let checks = $state<boolean[]>(session.actionItems.map(() => false));

  let expandedNote = $state<string | null>(null);
  let noteContents = $state<Record<string, string>>({});
  let loadingNote = $state(false);

  onMount(async () => {
    if (!settingsStore.state.loaded) {
      await settingsStore.load();
    }
  });

  const hasVault = $derived(Boolean(settingsStore.state.settings?.vaultPath));

  async function beginEdit() {
    editing = true;
    await tick();
    titleEl?.focus();
    titleEl?.select();
  }

  async function saveTitle() {
    editing = false;
    const next = titleInput.trim();
    if (!next || next === session.displayName) return;
    await invoke('session_rename', { id: session.id, name: next });
    session = { ...session, displayName: next };
  }

  async function openInObsidian(path?: string) {
    if (!hasVault) return;
    await invoke('open_in_obsidian', { notePath: path ?? null });
  }

  async function toggleNoteContent(path: string) {
    if (expandedNote === path) {
      expandedNote = null;
      return;
    }
    expandedNote = path;
    if (!noteContents[path]) {
      loadingNote = true;
      try {
        const text = await invoke<string>('note_read', { path });
        noteContents[path] = text;
      } catch (e) {
        noteContents[path] = `Unable to load note content: ${e}`;
      } finally {
        loadingNote = false;
      }
    }
  }
</script>

<div class="screen">
  <button class="back" type="button" onclick={() => goto('/')}>
    <Icon name="chevron-right" size={14} />
    <span>Back to Sessions</span>
  </button>

  <div class="kicker">Session Complete</div>

  <div class="title-row">
    {#if editing}
      <input
        class="title-input display"
        bind:value={titleInput}
        bind:this={titleEl}
        onblur={saveTitle}
        onkeydown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); if (e.key === 'Escape') { editing = false; titleInput = session.displayName ?? session.id; } }}
      />
    {:else}
      <button class="title-display display" type="button" onclick={beginEdit}>
        <span>{session.displayName ?? session.id}</span>
        <span class="edit-icon" aria-hidden="true"><Icon name="pencil" size={14} /></span>
      </button>
    {/if}
  </div>

  <div class="date mono">
    {formatDateLong(session.startedAt)}
    {#if session.endedAt}· {formatTimeRange(session.startedAt, session.endedAt).split(' – ')[1] ?? ''}{/if}
  </div>

  <div class="stats">
    <div class="stat">
      <div class="num display">{formatDurationLong(session.durationSecs)}</div>
      <div class="lab">Duration</div>
    </div>
    <div class="stat">
      <div class="num display">{session.vaultFilesAffected.length}</div>
      <div class="lab">{hasVault ? 'Notes Updated' : 'Notes Generated'}</div>
    </div>
    <div class="stat">
      <div class="num display">{session.actionItems.length}</div>
      <div class="lab">Action Items</div>
    </div>
  </div>

  {#if session.actionItems.length}
    <section>
      <div class="section-label">Action Items</div>
      <ul class="todos">
        {#each session.actionItems as item, i}
          <li>
            <label>
              <input type="checkbox" bind:checked={checks[i]} />
              <span class:checked={checks[i]}>{item}</span>
            </label>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if session.openQuestions.length}
    <section>
      <div class="section-label">Open Questions</div>
      <ul class="qs">
        {#each session.openQuestions as q}
          <li><span class="mark">?</span>{q}</li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if session.vaultFilesAffected.length}
    <section>
      <div class="section-label">{hasVault ? 'Vault Notes Affected' : 'Generated Notes Summary'}</div>
      <div class="files">
        {#each session.vaultFilesAffected as f}
          {#if hasVault}
            <button class="file clickable" type="button" onclick={() => openInObsidian(f.path)}>
              <div class="file-line">
                <span class="arrow">↳</span>
                <span class="path mono">{basename(f.path)}</span>
                <span class="action">{f.action}</span>
              </div>
              {#if f.summary}
                <div class="summary">{f.summary}</div>
              {/if}
            </button>
          {:else}
            <div class="file static">
              <div class="file-line">
                <span class="arrow">↳</span>
                <span class="path mono">{basename(f.path)}</span>
                <span class="action badge-pill">generated</span>
              </div>
              {#if f.summary}
                <div class="summary highlight-summary">{f.summary}</div>
              {/if}
              {#if expandedNote === f.path}
                <div class="note-preview">
                  {#if loadingNote}
                    <div class="note-loading">Loading note…</div>
                  {:else}
                    <pre class="note-content">{noteContents[f.path] || 'No content'}</pre>
                  {/if}
                </div>
              {/if}
              <button class="expand-btn" type="button" onclick={() => toggleNoteContent(f.path)}>
                <Icon name={expandedNote === f.path ? "chevron-down" : "chevron-right"} size={11} />
                <span>{expandedNote === f.path ? 'Hide full note' : 'View full note'}</span>
              </button>
            </div>
          {/if}
        {/each}
      </div>
    </section>
  {/if}

  <div class="actions">
    {#if hasVault}
      <Button variant="secondary" onclick={() => openInObsidian()}>
        {#snippet leading()}<Icon name="arrow-up-right" size={14} />{/snippet}
        Open in Obsidian
      </Button>
    {/if}
    <Button variant="primary" onclick={() => goto('/')}>Start New Session</Button>
  </div>
</div>

<style>
  .screen {
    max-width: 720px;
    margin: 0 auto;
    padding: 32px 24px 48px;
  }

  .back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--color-text-secondary);
    font-size: var(--size-sm);
    margin-bottom: 16px;
    background: none;
    padding: 0;
  }
  .back :global(svg) { transform: rotate(180deg); }
  .back:hover { color: var(--color-text-primary); }

  .kicker {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }

  .title-row {
    margin-bottom: 4px;
  }
  .title-display {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-display);
    font-weight: 700;
    font-size: var(--size-2xl);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
    background: transparent;
    padding: 0;
  }
  .title-display:hover { border-bottom: 1px solid var(--color-brass-dim); }
  .edit-icon { color: var(--color-text-tertiary); }
  .title-input {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: var(--size-2xl);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--color-brass-dim);
    padding: 0;
    height: auto;
    box-shadow: none;
    outline: none;
    caret-color: var(--color-brass);
    width: 100%;
  }

  .date {
    font-family: var(--font-mono);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
    margin-bottom: 20px;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    padding: 20px 0;
    border-top: 1px solid var(--color-border-subtle);
    border-bottom: 1px solid var(--color-border-subtle);
    margin-bottom: 24px;
  }
  .stat {
    text-align: center;
    border-left: 1px solid var(--color-border-subtle);
  }
  .stat:first-child { border-left: 0; }
  .num {
    font-family: var(--font-display);
    font-weight: 800;
    font-size: 28px;
    color: var(--color-text-primary);
    line-height: 1;
  }
  .lab {
    font-family: var(--font-body);
    font-weight: 400;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-top: 6px;
  }

  section { margin-bottom: 20px; }
  .section-label {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 10px;
  }
  ul { list-style: none; padding: 0; margin: 0; }

  .todos li { padding: 4px 0; }
  .todos label {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-primary);
    cursor: pointer;
  }
  .todos input[type="checkbox"] {
    appearance: none;
    width: 14px; height: 14px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    padding: 0;
    cursor: pointer;
    transition: background var(--duration-fast), border-color var(--duration-fast);
  }
  .todos input[type="checkbox"]:checked {
    background: var(--color-brass);
    border-color: var(--color-brass);
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 14 14'><path d='M3 7l2.5 2.5L11 4' fill='none' stroke='%230A0B0E' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'/></svg>");
    background-repeat: no-repeat;
    background-position: center;
  }
  .checked { color: var(--color-text-tertiary); text-decoration: line-through; }

  .qs li {
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-primary);
    padding: 4px 0;
  }
  .qs .mark {
    color: var(--color-brass);
    margin-right: 8px;
  }

  .files {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .file {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    padding: 12px 14px;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .file:last-child { border-bottom: 0; }
  .file:hover { background: var(--color-bg-elevated); }
  .file-line {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .arrow { color: var(--color-obsidian); }
  .path {
    font-family: var(--font-mono);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }
  .action {
    margin-left: auto;
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
    text-transform: lowercase;
  }
  .summary {
    margin-top: 4px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 24px;
  }

  .file.clickable {
    cursor: pointer;
  }
  .file.static {
    cursor: default;
  }
  .file.static:hover {
    background: var(--color-bg-surface);
  }
  .badge-pill {
    padding: 1px 8px;
    background: rgba(196, 180, 129, 0.15);
    color: var(--color-brass);
    border-radius: 10px;
    font-size: var(--size-2xs);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .highlight-summary {
    color: var(--color-text-primary);
    font-size: var(--size-sm);
    line-height: 1.5;
  }
  .expand-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    padding: 6px 0 0;
    color: var(--color-brass);
    font-family: var(--font-body);
    font-size: var(--size-xs);
    font-weight: 500;
    cursor: pointer;
    transition: opacity var(--duration-fast);
  }
  .expand-btn:hover {
    opacity: 0.8;
  }
  .note-preview {
    margin-top: 10px;
    padding: 12px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    animation: panel-in var(--duration-fast) var(--ease-default);
  }
  .note-content {
    font-family: var(--font-mono);
    font-size: var(--size-xs);
    color: var(--color-text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    max-height: 280px;
    overflow-y: auto;
  }
  .note-loading {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
  }
</style>
