<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ArgusEye from '$lib/components/eye/ArgusEye.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Waveform from '$lib/components/ui/Waveform.svelte';
  import SessionTimer from '$lib/components/session/SessionTimer.svelte';
  import { sessionStore } from '$lib/stores/session.svelte';

  onMount(async () => {
    await sessionStore.ensureSubscribed();
    await sessionStore.refresh();
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
</script>

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
  </div>

  {#if status === 'idle' || !cur}
    <div class="actions single">
      <Button variant="primary" full onclick={() => sessionStore.start()}>
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
      <Button variant="destructive" onclick={() => sessionStore.stop()}>
        {#snippet leading()}<Icon name="stop" size={12} />{/snippet}
        Stop
      </Button>
    </div>
  {:else}
    <div class="actions split">
      <Button variant="secondary" onclick={() => sessionStore.pause()}>
        {#snippet leading()}<Icon name="pause" size={12} />{/snippet}
        Pause
      </Button>
      <Button variant="destructive" onclick={() => sessionStore.stop()}>
        {#snippet leading()}<Icon name="stop" size={12} />{/snippet}
        Stop
      </Button>
    </div>
  {/if}

  <div class="divider"></div>

  <button class="link" type="button" onclick={() => invoke('open_dashboard')}>
    <Icon name="chevron-right" size={12} />
    <span>Open Dashboard</span>
    <span class="trail"><Icon name="arrow-up-right" size={12} /></span>
  </button>
  <button class="link" type="button" onclick={() => location.reload()}>
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
</style>
