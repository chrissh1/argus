<script lang="ts">
  import { sessionStore } from '$lib/stores/session.svelte';
  import SessionRow from '$lib/components/session/SessionRow.svelte';
  import SessionTimer from '$lib/components/session/SessionTimer.svelte';
  import SynthesisProgress from '$lib/components/session/SynthesisProgress.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import ArgusEye from '$lib/components/eye/ArgusEye.svelte';

  const sessions = $derived(sessionStore.state.history);
  const cur = $derived(sessionStore.state.current);
  const status = $derived(cur?.record.status ?? 'idle');

  const eyeState = $derived(
    status === 'active' ? 'active'
    : status === 'paused' ? 'paused'
    : status === 'synthesizing' ? 'synthesizing'
    : status === 'interrupted' ? 'interrupted'
    : 'idle'
  );

  async function startSession() {
    try {
      await sessionStore.start();
    } catch (e) {
      console.error(e);
    }
  }

  function synthFor(id: string) {
    const s = sessionStore.state.synthesis[id];
    return s ? { message: s.message, progress: s.progress, total: s.total } : null;
  }
</script>

<div class="main-page">
  <!-- Hero Header Section -->
  <section class="hero-section">
    <div class="logo-wrapper">
      <ArgusEye state={eyeState} size={84} />
    </div>

    <div class="hero-text">
      <h1 class="app-title">ARGUS</h1>
      <p class="app-tagline">Privacy-First Deep Work Capture & Knowledge Synthesizer</p>
    </div>

    <!-- Centered Primary Action Area -->
    <div class="cta-container">
      {#if status === 'idle' || !cur}
        <button class="hero-start-btn" type="button" onclick={startSession}>
          <span class="btn-icon-circle">
            <Icon name="play" size={18} />
          </span>
          <span class="btn-text-block">
            <span class="btn-title">Start Deep Work Session</span>
            <span class="btn-subtitle">Captures screen & voice · Synthesizes to Obsidian</span>
          </span>
        </button>
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

  <!-- Recent Sessions Section Below -->
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
          <p class="empty-title">No sessions recorded yet</p>
          <p class="empty-desc">When you complete a deep-work session, your knowledge digests will appear here.</p>
        </div>
      {:else}
        {#each sessions as s (s.id)}
          <SessionRow session={s} synth={synthFor(s.id)} />
        {/each}
      {/if}
    </div>
  </section>
</div>

<style>
  .main-page {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    padding: 32px 40px;
    max-width: 960px;
    margin: 0 auto;
  }

  /* Hero Section */
  .hero-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 24px 0 36px;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .logo-wrapper {
    margin-bottom: 16px;
    filter: drop-shadow(0 4px 20px rgba(0, 0, 0, 0.4));
  }

  .hero-text {
    margin-bottom: 24px;
  }

  .app-title {
    font-family: var(--font-display);
    font-size: 52px;
    font-weight: 800;
    line-height: 1.1;
    letter-spacing: 0.16em;
    color: var(--color-brass);
    background: linear-gradient(180deg, #FFFFFF 15%, var(--color-brass) 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0 0 10px;
    filter: drop-shadow(0 4px 24px rgba(196, 180, 129, 0.30)) drop-shadow(0 2px 10px rgba(0, 0, 0, 0.8));
  }

  .app-tagline {
    font-family: var(--font-body);
    font-size: var(--size-md);
    font-weight: 400;
    letter-spacing: 0.02em;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* Hero Call-To-Action */
  .cta-container {
    width: 100%;
    max-width: 440px;
    display: flex;
    justify-content: center;
  }

  .hero-start-btn {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
    padding: 14px 22px;
    background: var(--color-brass);
    color: var(--color-text-inverse);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    box-shadow: 0 4px 20px rgba(196, 180, 129, 0.25), 0 2px 6px rgba(0, 0, 0, 0.4);
    transition: all var(--duration-fast) var(--ease-default);
    text-align: left;
  }

  .hero-start-btn:hover {
    transform: translateY(-1px);
    filter: brightness(1.08);
    box-shadow: 0 6px 28px rgba(196, 180, 129, 0.35), 0 2px 8px rgba(0, 0, 0, 0.5);
  }

  .hero-start-btn:active {
    transform: translateY(0);
    filter: brightness(0.95);
  }

  .btn-icon-circle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(24, 25, 31, 0.2);
    color: var(--color-text-inverse);
    flex-shrink: 0;
  }

  .btn-text-block {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .btn-title {
    font-family: var(--font-body);
    font-size: var(--size-md);
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .btn-subtitle {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    opacity: 0.85;
  }

  /* Active / Paused / Synth Card */
  .active-session-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    padding: 20px 24px;
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
    margin-bottom: 8px;
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
    font-size: 28px;
    margin-bottom: 16px;
  }

  .hero-controls {
    display: flex;
    gap: 12px;
    width: 100%;
    justify-content: center;
  }

  .synth-waiting {
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
    margin: 8px 0 0;
  }

  /* Sessions List Section */
  .sessions-section {
    padding-top: 28px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
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
    color: var(--color-text-secondary);
  }

  .session-count-badge {
    font-family: var(--font-mono);
    font-size: var(--size-2xs);
    padding: 1px 6px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
    border-radius: 10px;
    color: var(--color-text-tertiary);
  }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .loading-placeholder, .empty-state {
    padding: 36px 16px;
    text-align: center;
    color: var(--color-text-tertiary);
    background: var(--color-bg-surface);
    border: 1px dashed var(--color-border-subtle);
    border-radius: var(--radius-md);
  }

  .empty-title {
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-secondary);
    margin: 0 0 4px;
    font-weight: 500;
  }

  .empty-desc {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-tertiary);
    margin: 0;
  }
</style>
