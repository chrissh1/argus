<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    label: string;
    hint?: string;
    children: Snippet;
  }
  let { label, hint, children }: Props = $props();
</script>

<div class="field" role="group" aria-label={label}>
  <div class="label">{label}</div>
  <div class="control">{@render children()}</div>
  {#if hint}<p class="hint">{hint}</p>{/if}
</div>

<style>
  .field { margin-bottom: 14px; }
  .label {
    display: block;
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 6px;
  }
  .control { display: flex; gap: 8px; align-items: center; }
  .hint {
    margin-top: 6px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-tertiary);
  }
</style>
