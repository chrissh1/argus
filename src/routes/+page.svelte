<script lang="ts">
  import { sessionStore } from '$lib/stores/session.svelte';
  import SessionRow from '$lib/components/session/SessionRow.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import ArgusEye from '$lib/components/eye/ArgusEye.svelte';

  const sessions = $derived(sessionStore.state.history);

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

<div class="header">
  <div class="title section-label">Sessions</div>
  <Button variant="primary" onclick={startSession}>
    {#snippet leading()}<Icon name="play" size={12} />{/snippet}
    Start New Session
  </Button>
</div>

<div class="list">
  {#if !sessionStore.state.loaded}
    <div class="empty"></div>
  {:else if sessions.length === 0}
    <div class="empty">
      <div class="eye"><ArgusEye state="idle" size={56} /></div>
      <div class="big">No sessions yet.</div>
      <div class="sub">Start a session to begin capturing your work.</div>
      <div class="cta">
        <Button variant="primary" onclick={startSession}>
          {#snippet leading()}<Icon name="play" size={12} />{/snippet}
          Start Your First Session
        </Button>
      </div>
    </div>
  {:else}
    {#each sessions as s (s.id)}
      <SessionRow session={s} synth={synthFor(s.id)} />
    {/each}
  {/if}
</div>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    height: 48px;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .title { font-size: var(--size-xs); }

  .list { padding: 0; }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 80px 16px;
    color: var(--color-text-secondary);
  }
  .empty .eye { color: var(--color-text-tertiary); margin-bottom: 24px; }
  .empty .big {
    font-family: var(--font-body);
    font-size: 15px;
    color: var(--color-text-secondary);
    margin-bottom: 6px;
  }
  .empty .sub {
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-tertiary);
    margin-bottom: 24px;
  }
  .cta { margin-top: 8px; }
</style>
