<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tone = 'complete' | 'synthesizing' | 'paused' | 'active' | 'interrupted' | 'neutral';

  interface Props {
    tone?: Tone;
    children: Snippet;
  }
  let { tone = 'neutral', children }: Props = $props();
</script>

<span class="badge {tone}">
  {#if tone === 'synthesizing'}
    <span class="dot"></span>
  {/if}
  {@render children()}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-2xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
  }
  .complete { color: var(--color-active); }
  .active { color: var(--color-active); }
  .paused { color: var(--color-paused); }
  .synthesizing { color: var(--color-synthesizing); }
  .interrupted { color: var(--color-interrupted); }
  .neutral { color: var(--color-text-tertiary); }
  .dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    background: currentColor;
    animation: synthesizing-dot 1.2s ease-in-out infinite;
  }
</style>
