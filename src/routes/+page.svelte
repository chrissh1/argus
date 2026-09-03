<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { sessionStore } from '$lib/stores/session.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import SessionRow from '$lib/components/session/SessionRow.svelte';
  import SessionTimer from '$lib/components/session/SessionTimer.svelte';
  import SynthesisProgress from '$lib/components/session/SynthesisProgress.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import type { OllamaTestResult } from '$lib/types';

  const sessions = $derived(sessionStore.state.history);
  const cur = $derived(sessionStore.state.current);
  const status = $derived(cur?.record.status ?? 'idle');

  let showLlmModal = $state(false);
  let llmError = $state<string | null>(null);
  let showVaultModal = $state(false);
  let dontWarnVaultAgain = $state(false);
  let checkingLlm = $state(false);
  let llmConnected = $state<boolean | null>(null);

  onMount(async () => {
    if (!settingsStore.state.loaded) await settingsStore.load();
    await checkLlmStatus();
  });

  async function checkLlmStatus(): Promise<boolean> {
    try {
      const res = await invoke<OllamaTestResult>('llm_status');
      llmConnected = res.ok;
      if (!res.ok) {
        llmError = res.error || 'Ollama is unreachable. Please ensure the local service is running.';
      } else {
        llmError = null;
      }
      return res.ok;
    } catch (e) {
      llmConnected = false;
      llmError = String(e);
      return false;
    }
  }

  async function handleStartSession() {
    checkingLlm = true;
    const ok = await checkLlmStatus();
    checkingLlm = false;

    if (!ok) {
      showLlmModal = true;
      return;
    }

    const s = settingsStore.state.settings;
    if (!s?.vaultPath && (s?.warnMissingVault ?? true)) {
      showVaultModal = true;
      return;
    }

    await proceedStartSession();
  }

  async function proceedStartSession() {
    showVaultModal = false;
    if (dontWarnVaultAgain) {
      await settingsStore.set('warn_missing_vault', 'false');
    }
    try {
      await sessionStore.start();
    } catch (e) {
      console.error(e);
      await checkLlmStatus();
    }
  }

  function synthFor(id: string) {
    const s = sessionStore.state.synthesis[id];
    return s ? { message: s.message, progress: s.progress, total: s.total } : null;
  }
</script>

<div class="main-page">
  <!-- Centered Hero Section (Lowered & Focused) -->
  <section class="hero-section">
    <h1 class="app-title">ARGUS</h1>

    <!-- Centered Action Area -->
    <div class="cta-container">
      {#if status === 'idle' || !cur}
        <div class="start-btn-group">
          <button class="hero-start-btn" type="button" onclick={handleStartSession} disabled={checkingLlm}>
            <span class="btn-icon-circle">
              <Icon name="play" size={16} />
            </span>
            <span class="btn-title">{checkingLlm ? 'Connecting…' : 'Start Session'}</span>
          </button>

          {#if llmConnected === false}
            <button class="llm-status-pill" type="button" onclick={() => goto('/settings/llm/')}>
              <span class="dot-pill"></span>
              <span>No LLM connected · Configure</span>
              <Icon name="chevron-right" size={10} />
            </button>
          {/if}
        </div>
      {:else if status === 'active'}
        <div class="active-session-card">
          <div class="status-indicator">
            <span class="pulse-dot"></span>
            <span class="status-label">RECORDING ACTIVE</span>
          </div>
          <div class="timer-display">
            <SessionTimer seconds={cur.durationSecs} running={true} />
          </div>
          <div class="hero-controls">
            <Button variant="secondary" onclick={() => sessionStore.pause()}>
              {#snippet leading()}<Icon name="pause" size={14} />{/snippet}
              Pause
            </Button>
            <Button variant="destructive" onclick={() => sessionStore.stop()}>
              {#snippet leading()}<Icon name="stop" size={14} />{/snippet}
              Stop & Synthesize
            </Button>
          </div>
        </div>
      {:else if status === 'paused'}
        <div class="active-session-card paused">
          <div class="status-indicator paused">
            <span class="pause-dot"></span>
            <span class="status-label">SESSION PAUSED</span>
          </div>
          <div class="timer-display">
            <SessionTimer seconds={cur.durationSecs} running={false} />
          </div>
          <div class="hero-controls">
            <Button variant="primary" onclick={() => sessionStore.resume()}>
              {#snippet leading()}<Icon name="play" size={14} />{/snippet}
              Resume
            </Button>
            <Button variant="destructive" onclick={() => sessionStore.stop()}>
              {#snippet leading()}<Icon name="stop" size={14} />{/snippet}
              Stop & Synthesize
            </Button>
          </div>
        </div>
      {:else if status === 'synthesizing'}
        <div class="active-session-card synthesizing">
          <div class="status-indicator synthesizing">
            <span class="status-label">AI SYNTHESIZING NOTES</span>
          </div>
          {#if cur && synthFor(cur.record.id)}
            {@const synth = synthFor(cur.record.id)!}
            <SynthesisProgress message={synth.message} progress={synth.progress} total={synth.total} />
          {:else}
            <p class="synth-waiting">Extracting concepts and updating Obsidian vault…</p>
          {/if}
        </div>
      {/if}
    </div>
  </section>

  <!-- Compact Session History Section Below -->
  <section class="sessions-section">
    <div class="section-header">
      <div class="section-heading-group">
        <span class="section-title">Session History</span>
        {#if sessions.length > 0}
          <span class="session-count-badge">{sessions.length}</span>
        {/if}
      </div>
    </div>

    <div class="session-list">
      {#if !sessionStore.state.loaded}
        <div class="loading-placeholder">Loading sessions…</div>
      {:else if sessions.length === 0}
        <div class="empty-state">
          <p class="empty-title">No sessions yet</p>
        </div>
      {:else}
        {#each sessions as s (s.id)}
          <SessionRow session={s} synth={synthFor(s.id)} />
        {/each}
      {/if}
    </div>
  </section>

  {#if showLlmModal}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div class="modal-backdrop" onclick={() => (showLlmModal = false)} role="dialog" aria-modal="true" tabindex="-1">
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="modal-card" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span class="modal-icon tone-error">
            <Icon name="alert-circle" size={20} />
          </span>
          <h2 class="modal-title">LLM Connection Required</h2>
        </div>
        <p class="modal-body">
          Argus requires an active local LLM (Ollama) to extract concepts, generate action items, and synthesize session notes. No LLM is currently reachable.
        </p>
        {#if llmError}
          <div class="modal-error-box mono">
            {llmError}
          </div>
        {/if}
        <p class="modal-hint">
          Please ensure Ollama is running (`ollama serve`) and has your configured model installed before starting a session.
        </p>
        <div class="modal-actions">
          <Button variant="ghost" onclick={() => (showLlmModal = false)}>Dismiss</Button>
          <Button variant="primary" onclick={() => goto('/settings/llm/')}>Configure LLM</Button>
        </div>
      </div>
    </div>
  {/if}

  {#if showVaultModal}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div class="modal-backdrop" onclick={() => (showVaultModal = false)} role="dialog" aria-modal="true" tabindex="-1">
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="modal-card" onclick={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <span class="modal-icon tone-warn">
            <Icon name="book-open" size={20} />
          </span>
          <h2 class="modal-title">No Obsidian Vault Selected</h2>
        </div>
        <p class="modal-body">
          No Obsidian vault is selected. Argus will record and synthesize notes internally without syncing to an Obsidian vault.
        </p>
        <p class="modal-question">
          Would you like to proceed with the session anyway?
        </p>
        <label class="modal-checkbox-row">
          <input type="checkbox" bind:checked={dontWarnVaultAgain} />
          <span>Don't ask me again</span>
        </label>
        <div class="modal-actions">
          <Button variant="ghost" onclick={() => (showVaultModal = false)}>Cancel</Button>
          <Button variant="secondary" onclick={() => goto('/settings/vault/')}>Select Vault</Button>
          <Button variant="primary" onclick={proceedStartSession}>Proceed</Button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .main-page {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    padding: 0 48px 48px;
    max-width: 860px;
    margin: 0 auto;
  }

  /* Hero Section: lowered with comfortable vertical breathing room */
  .hero-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding-top: 88px;
    padding-bottom: 56px;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  /* Bold solid display title (no gradient) */
  .app-title {
    font-family: 'Brassie', 'Montserrat', var(--font-display);
    font-size: 60px;
    font-weight: 900;
    line-height: 1;
    letter-spacing: 0.16em;
    color: var(--color-text-primary);
    margin: 0 0 28px;
    text-shadow: 0 4px 20px rgba(0, 0, 0, 0.6);
  }

  /* Centered Action Button Area */
  .cta-container {
    width: 100%;
    max-width: 280px;
    display: flex;
    justify-content: center;
  }

  .hero-start-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    width: 100%;
    height: 48px;
    padding: 0 24px;
    background: var(--color-brass);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    box-shadow: 0 4px 18px rgba(196, 180, 129, 0.22), 0 2px 6px rgba(0, 0, 0, 0.4);
    transition: all var(--duration-fast) var(--ease-default);
  }

  .hero-start-btn:hover {
    transform: translateY(-1px);
    filter: brightness(1.08);
    box-shadow: 0 6px 24px rgba(196, 180, 129, 0.32), 0 2px 8px rgba(0, 0, 0, 0.5);
  }

  .hero-start-btn:active {
    transform: translateY(0);
    filter: brightness(0.96);
  }

  .btn-icon-circle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: rgba(24, 25, 31, 0.22);
    color: var(--color-text-inverse);
    flex-shrink: 0;
  }

  .btn-title {
    font-family: var(--font-body);
    font-size: var(--size-md);
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  /* Active / Paused / Synthesizing Card */
  .active-session-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    padding: 18px 22px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-surface);
  }

  .active-session-card.paused {
    border-color: var(--color-paused);
  }

  .active-session-card.synthesizing {
    border-color: var(--color-synthesizing);
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .status-label {
    font-family: var(--font-body);
    font-size: var(--size-2xs);
    font-weight: 600;
    letter-spacing: var(--tracking-wide);
    color: var(--color-active);
  }

  .status-indicator.paused .status-label {
    color: var(--color-paused);
  }

  .status-indicator.synthesizing .status-label {
    color: var(--color-synthesizing);
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-active);
    box-shadow: 0 0 8px var(--color-active);
    animation: recording-pulse 2s infinite ease-in-out;
  }

  .pause-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-paused);
  }

  .timer-display {
    font-size: 26px;
    margin-bottom: 14px;
  }

  .hero-controls {
    display: flex;
    gap: 10px;
    width: 100%;
    justify-content: center;
  }

  .synth-waiting {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-secondary);
    margin: 6px 0 0;
  }

  /* Compact Session History Section Below */
  .sessions-section {
    padding-top: 24px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .section-heading-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .section-title {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    font-weight: 600;
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-tertiary);
  }

  .session-count-badge {
    font-family: var(--font-mono);
    font-size: var(--size-2xs);
    padding: 0 6px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
    border-radius: 10px;
    color: var(--color-text-tertiary);
  }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 320px;
    overflow-y: auto;
  }

  .loading-placeholder, .empty-state {
    padding: 24px 16px;
    text-align: center;
    color: var(--color-text-tertiary);
    background: var(--color-bg-surface);
    border: 1px dashed var(--color-border-subtle);
    border-radius: var(--radius-md);
  }

  .empty-title {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
    margin: 0;
  }

  /* Start Button & Warning Pill */
  .start-btn-group {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    gap: 12px;
  }

  .llm-status-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(217, 83, 79, 0.12);
    border: 1px solid rgba(217, 83, 79, 0.35);
    border-radius: 20px;
    color: var(--color-interrupted);
    font-family: var(--font-body);
    font-size: var(--size-xs);
    cursor: pointer;
    transition: all var(--duration-fast);
  }

  .llm-status-pill:hover {
    background: rgba(217, 83, 79, 0.20);
    border-color: rgba(217, 83, 79, 0.60);
  }

  .dot-pill {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-interrupted);
    box-shadow: 0 0 6px var(--color-interrupted);
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 11, 14, 0.75);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 24px;
    animation: fade-in 150ms var(--ease-default);
  }

  .modal-card {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-modal, 0 12px 36px rgba(0, 0, 0, 0.6));
    width: 100%;
    max-width: 440px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: panel-in 180ms var(--ease-default);
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .modal-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-md);
  }
  .modal-icon.tone-error {
    background: rgba(217, 83, 79, 0.15);
    color: var(--color-interrupted);
  }
  .modal-icon.tone-warn {
    background: rgba(196, 180, 129, 0.15);
    color: var(--color-brass);
  }

  .modal-title {
    font-family: var(--font-display);
    font-size: var(--size-lg);
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
    letter-spacing: -0.01em;
  }

  .modal-body {
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .modal-error-box {
    padding: 8px 12px;
    background: var(--color-bg-base);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    font-size: var(--size-xs);
    color: var(--color-interrupted);
    word-break: break-word;
    max-height: 100px;
    overflow-y: auto;
  }

  .modal-hint {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .modal-question {
    font-family: var(--font-body);
    font-size: var(--size-sm);
    font-weight: 500;
    color: var(--color-text-primary);
    margin: 4px 0 0;
  }

  .modal-checkbox-row {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
    cursor: pointer;
    margin-top: 4px;
  }

  .modal-checkbox-row input[type="checkbox"] {
    accent-color: var(--color-brass);
    cursor: pointer;
  }

  .modal-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 8px;
  }
</style>
