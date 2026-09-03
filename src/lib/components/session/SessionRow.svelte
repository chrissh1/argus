<script lang="ts">
  import type { SessionRecord } from '$lib/types';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import SynthesisProgress from './SynthesisProgress.svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { formatDateLong, formatTimeRange, formatDurationLong, basename } from '$lib/format';

  interface Props {
    session: SessionRecord;
    /** Tauri-emitted synthesis status for this row, when active. */
    synth?: { message: string; progress: number; total: number } | null;
  }
  let { session, synth = null }: Props = $props();

  let expanded = $state(false);

  const hasVault = $derived(Boolean(settingsStore.state.settings?.vaultPath));

  const isSynthesizing = $derived(session.status === 'synthesizing');
  const dotTone = $derived(
    session.status === 'complete' ? 'complete'
    : session.status === 'interrupted' ? 'interrupted'
    : session.status === 'synthesizing' ? 'synthesizing'
    : 'neutral'
  );

  const title = $derived(session.displayName || session.id);

  async function openInObsidian(path?: string) {
    if (!hasVault) return;
    await invoke('open_in_obsidian', { notePath: path ?? null });
  }
  function viewSummary() {
    goto(`/summary/${session.id}`);
  }
</script>

<div class="row" class:open={expanded}>
  <button
    type="button"
    class="header"
    aria-expanded={expanded}
    onclick={() => (expanded = !expanded)}
  >
    <span class="dot tone-{dotTone}" aria-hidden="true"></span>
    <span class="title">{title}</span>
    <span class="duration mono">
      {session.endedAt ? formatDurationLong(session.durationSecs) : 'in progress'}
    </span>
    <span class="chev" class:rot={expanded}>
      <Icon name="chevron-down" size={14} />
    </span>
  </button>

  <div class="meta">
    <span class="mono">{formatDateLong(session.startedAt)}{session.endedAt ? ` · ${formatTimeRange(session.startedAt, session.endedAt).split(' – ')[1] ?? ''}` : ''}</span>
    <Badge tone={dotTone}>{session.status}</Badge>
  </div>

  {#if isSynthesizing}
    <div class="synth">
      <SynthesisProgress
        message={synth?.message ?? 'Synthesizing…'}
        progress={synth?.progress ?? 0}
        total={synth?.total ?? 0}
      />
    </div>
  {/if}

  {#if expanded}
    <div class="body">
      {#if session.actionItems.length}
        <section>
          <div class="section-label">Action Items</div>
          <ul class="items">
            {#each session.actionItems as item}
              <li><span class="bullet">▸</span>{item}</li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if session.vaultFilesAffected.length}
        <section>
          <div class="section-label">{hasVault ? 'Vault Notes' : 'Generated Notes'}</div>
          <ul class="files">
            {#each session.vaultFilesAffected as f}
              <li>
                <span class="arrow">↳</span>
                <span class="path mono">{basename(f.path)}</span>
                <span class="action">· {hasVault ? f.action : 'generated'}</span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      <div class="actions">
        <Button variant="ghost" onclick={viewSummary}>View Summary</Button>
        {#if hasVault}
          <Button variant="secondary" onclick={() => openInObsidian()}>
            {#snippet leading()}<Icon name="arrow-up-right" size={14} />{/snippet}
            Open in Obsidian
          </Button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 1fr;
    padding: 14px 16px;
    border-bottom: 1px solid var(--color-border-subtle);
    transition: background var(--duration-fast) var(--ease-default);
  }
  .row:hover { background: var(--color-bg-elevated); }

  .header {
    display: grid;
    grid-template-columns: 16px 1fr auto 18px;
    align-items: center;
    gap: 10px;
    background: none;
    text-align: left;
    padding: 0;
  }

  .dot {
    width: 8px; height: 8px;
    border-radius: 50%;
    background: var(--color-text-tertiary);
  }
  .dot.tone-complete    { background: var(--color-active); }
  .dot.tone-interrupted { background: var(--color-interrupted); }
  .dot.tone-synthesizing {
    background: var(--color-synthesizing);
    animation: synthesizing-dot 1.2s ease-in-out infinite;
  }

  .title {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: var(--size-md);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .duration {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }
  .chev {
    color: var(--color-text-tertiary);
    transition: transform var(--duration-normal) var(--ease-default);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .chev.rot { transform: rotate(180deg); }

  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 4px;
    margin-left: 26px;
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
  }
  .meta .mono {
    font-family: var(--font-mono);
    font-size: var(--size-xs);
  }

  .synth {
    margin: 12px 0 0 26px;
  }

  .body {
    margin-top: 16px;
    margin-left: 26px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-subtle);
    animation: panel-in var(--duration-normal) var(--ease-default);
  }
  section + section { margin-top: 12px; }

  .section-label {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 6px;
  }

  ul { list-style: none; padding: 0; margin: 0; }

  .items li {
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-primary);
    padding: 2px 0;
    border-left: 2px solid var(--color-brass-dim);
    padding-left: 10px;
    margin-bottom: 2px;
  }
  .items .bullet {
    color: var(--color-brass);
    margin-right: 6px;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: var(--size-xs);
    color: var(--color-text-secondary);
  }
  .files .arrow { color: var(--color-obsidian); }
  .files .action { color: var(--color-text-tertiary); }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
