<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    children: Snippet;
    size?: number;
    label: string;
  }
  let { children, size = 28, label, ...rest }: Props = $props();
</script>

<button
  type="button"
  class="icon-btn"
  style:width="{size}px"
  style:height="{size}px"
  aria-label={label}
  title={label}
  {...rest}
>
  {@render children()}
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    background: transparent;
    transition: background var(--duration-fast) var(--ease-default),
                color var(--duration-fast) var(--ease-default),
                transform var(--duration-instant) var(--ease-default);
  }
  .icon-btn:hover:not(:disabled) {
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }
  .icon-btn:active { transform: scale(0.93); }
  .icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
