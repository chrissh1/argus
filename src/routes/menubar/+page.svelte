<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import ArgusEye from '$lib/components/eye/ArgusEye.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Waveform from '$lib/components/ui/Waveform.svelte';
  import SessionTimer from '$lib/components/session/SessionTimer.svelte';
  import { sessionStore } from '$lib/stores/session.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { OllamaTestResult } from '$lib/types';

  let llmError = $state<string | null>(null);
  let showVaultConfirm = $state(false);

  onMount(async () => {
    await sessionStore.ensureSubscribed();
    await Promise.all([sessionStore.refresh(), settingsStore.load()]);
  });

  const cur = $derived(sessionStore.state.current);
  const status = $derived(cur?.record.status ?? 'idle');
  const eyeState = $derived(
    status === 'active' ? 'active'
    : status === 'paused' ? 'paused'
    : status === 'synthesizing' ? 'synthesizing'
    : status === 'interrupted' ? 'interrupted'
    : 'idle'
  );

  const startedAtLabel = $derived(
    cur ? new Date(cur.record.startedAt * 1000).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' }) : ''
  );

  async function closePopover() {
    try {
      await getCurrentWebviewWindow().hide();
    } catch {
      await invoke('hide_menubar');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePopover();
    }
  }

  async function handleOpenDashboard() {
    await invoke('open_dashboard');
    await closePopover();
  }

  async function handleStop() {
    await sessionStore.stop();
    await invoke('open_dashboard');
    await closePopover();
  }

  async function tryStartSession() {
    llmError = null;
    try {
      const res = await invoke<OllamaTestResult>('llm_status');
      if (!res.ok) {
        llmError = res.error || 'Ollama is unreachable. Please connect an LLM.';
        return;
      }
    } catch (e) {
      llmError = `Cannot reach LLM: ${e}`;
      return;
    }

    const settings = settingsStore.state.settings;
    if (!settings?.vaultPath && (settings?.warnMissingVault ?? true)) {
      showVaultConfirm = true;
      return;
    }

    await startSessionDirectly();
  }

  async function startSessionDirectly() {
    showVaultConfirm = false;
    llmError = null;
    try {
      await sessionStore.start();
    } catch (e) {
      llmError = String(e);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="popover" data-tauri-drag-region>
  <header class="head">
    <div class="left">
      <ArgusEye state={eyeState} size={14} glyph />
      <span class="brand">ARGUS</span>
      {#if status === 'active' || status === 'paused'}
        <Waveform width={70} height={12} />
      {/if}
    </div>
    {#if cur}
      <SessionTimer seconds={cur.durationSecs} running={status === 'active'} />
    {/if}
  </header>

  <div class="card">
    <div class="eye-big">
      <ArgusEye state={eyeState} size={68} />
    </div>
    <div class="state display">
      {#if status === 'active'}Recording
      {:else if status === 'paused'}Paused
      {:else if status === 'synthesizing'}Synthesizing
      {:else if status === 'interrupted'}Interrupted
      {:else}Idle{/if}
    </div>
    <div class="sub">
      {#if cur}
        Started {startedAtLabel}
      {:else}
        No active session
      {/if}
    </div>

    {#if llmError}
      <div class="menubar-warning">
        <div class="warning-title">
          <Icon name="alert-circle" size={13} />
          <span>LLM Disconnected</span>
        </div>
        <p class="warning-text">{llmError}</p>
        <button class="warning-action" type="button" onclick={handleOpenDashboard}>
          Open LLM Config &rarr;
        </button>
      </div>
    {/if}

    {#if showVaultConfirm}
      <div class="menubar-warning vault-warn">
        <div class="warning-title">
          <Icon name="book-open" size={13} />
          <span>No Vault Selected</span>
        </div>
        <p class="warning-text">Proceed without an Obsidian vault?</p>
        <div class="prompt-actions">
          <button class="prompt-btn confirm" type="button" onclick={startSessionDirectly}>Proceed</button>
          <button class="prompt-btn cancel" type="button" onclick={() => (showVaultConfirm = false)}>Cancel</button>
        </div>
      </div>
    {/if}
  </div>

  {#if status === 'idle' || !cur}
    <div class="actions single">
      <Button variant="primary" full onclick={tryStartSession}>
        {#snippet leading()}<Icon name="play" size={12} />{/snippet}
        Start Session
      </Button>
    </div>
  {:else if status === 'paused'}
    <div class="actions split">
      <Button variant="secondary" onclick={() => sessionStore.resume()}>
        {#snippet leading()}<Icon name="play" size={12} />{/snippet}
        Resume
      </Button>
      <Button variant="destructive" onclick={handleStop}>
        {#snippet leading()}<Icon name="stop" size={12} />{/snippet}
        Stop
      </Button>
    </div>
  {:else if status === 'synthesizing'}
    <div class="actions single">
      <Button variant="secondary" full onclick={handleOpenDashboard}>
        View Synthesis in Dashboard
      </Button>
    </div>
  {:else}
    <div class="actions split">
      <Button variant="secondary" onclick={() => sessionStore.pause()}>
        {#snippet leading()}<Icon name="pause" size={12} />{/snippet}
        Pause
      </Button>
      <Button variant="destructive" onclick={handleStop}>
        {#snippet leading()}<Icon name="stop" size={12} />{/snippet}
        Stop
      </Button>
    </div>
  {/if}

  <div class="divider"></div>

  <button class="link" type="button" onclick={handleOpenDashboard}>
    <Icon name="chevron-right" size={12} />
    <span>Open Dashboard</span>
    <span class="trail"><Icon name="arrow-up-right" size={12} /></span>
  </button>
  <button class="link" type="button" onclick={closePopover}>
    <Icon name="x" size={12} />
    <span>Close</span>
  </button>
</div>

<style>
  .popover {
    width: 280px;
    background: var(--color-bg-overlay);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-default);
    box-shadow: var(--shadow-float);
    overflow: hidden;
    font-family: var(--font-body);
    color: var(--color-text-primary);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    animation: pop-in 120ms var(--ease-default);
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border-subtle);
    height: 40px;
  }
  .head .left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wider);
    color: var(--color-text-secondary);
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 18px 16px 14px;
    gap: 8px;
  }
  .eye-big { margin-bottom: 4px; }
  .state {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: var(--size-xl);
    color: var(--color-text-primary);
    letter-spacing: var(--tracking-tight);
  }
  .sub {
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }

  .actions { padding: 8px 16px 14px; }
  .actions.single { display: block; }
  .actions.split { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .actions :global(.btn) { height: 36px; }

  .divider {
    height: 1px;
    background: var(--color-border-subtle);
    margin: 0 16px;
  }

  .link {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 16px;
    background: none;
    color: var(--color-text-secondary);
    font-family: var(--font-body);
    font-size: var(--size-sm);
  }
  .link:hover { color: var(--color-text-primary); background: var(--color-bg-elevated); }
  .link .trail { margin-left: auto; color: var(--color-text-tertiary); }

  .menubar-warning {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: rgba(194, 68, 68, 0.12);
    border: 1px solid rgba(194, 68, 68, 0.35);
    text-align: left;
    width: 100%;
  }
  .menubar-warning.vault-warn {
    background: rgba(196, 180, 129, 0.12);
    border-color: rgba(196, 180, 129, 0.35);
  }
  .warning-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--size-xs);
    font-weight: 600;
    color: var(--color-interrupted);
  }
  .menubar-warning.vault-warn .warning-title {
    color: var(--color-brass);
  }
  .warning-text {
    font-size: var(--size-xs);
    color: var(--color-text-secondary);
    margin: 4px 0 6px;
    line-height: 1.3;
  }
  .warning-action {
    display: inline-flex;
    font-size: var(--size-xs);
    font-weight: 600;
    color: var(--color-brass);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }
  .warning-action:hover {
    text-decoration: underline;
  }
  .prompt-actions {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }
  .prompt-btn {
    flex: 1;
    height: 26px;
    font-size: var(--size-xs);
    font-weight: 600;
    border-radius: var(--radius-sm);
    cursor: pointer;
    border: none;
  }
  .prompt-btn.confirm {
    background: var(--color-brass);
    color: var(--color-text-inverse);
  }
  .prompt-btn.cancel {
    background: var(--color-bg-elevated);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border-subtle);
  }
</style>
