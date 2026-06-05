<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'ghost' | 'destructive';

  interface Props extends HTMLButtonAttributes {
    variant?: Variant;
    full?: boolean;
    children: Snippet;
    leading?: Snippet;
    trailing?: Snippet;
  }
  let {
    variant = 'secondary',
    full = false,
    type = 'button',
    children,
    leading,
    trailing,
    ...rest
  }: Props = $props();
</script>

<button
  {type}
  class="btn {variant}"
  class:full
  {...rest}
>
  {#if leading}<span class="lead">{@render leading()}</span>{/if}
  <span class="label">{@render children()}</span>
  {#if trailing}<span class="trail">{@render trailing()}</span>{/if}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: 34px;
    padding: 0 16px;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-base);
    letter-spacing: var(--tracking-normal);
    border: 1px solid transparent;
    transition: background var(--duration-fast) var(--ease-default),
                color var(--duration-fast) var(--ease-default),
                box-shadow var(--duration-fast) var(--ease-default),
                border-color var(--duration-fast) var(--ease-default),
                filter var(--duration-fast) var(--ease-default);
  }
  .full { width: 100%; }
  .btn:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn:active { transform: translateY(0); filter: brightness(0.92); }
  .lead, .trail { display: inline-flex; align-items: center; }

  .primary {
    background: var(--color-brass);
    color: var(--color-text-inverse);
    font-weight: 600;
  }
  .primary:hover:not(:disabled) {
    filter: brightness(1.12);
    box-shadow: var(--shadow-glow-brass);
  }

  .secondary {
    background: transparent;
    color: var(--color-text-secondary);
    border-color: var(--color-border-default);
  }
  .secondary:hover:not(:disabled) {
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
    border-color: var(--color-border-active);
  }

  .ghost {
    background: transparent;
    color: var(--color-text-secondary);
    padding: 0 8px;
  }
  .ghost:hover:not(:disabled) {
    color: var(--color-text-primary);
  }

  .destructive {
    background: transparent;
    color: var(--color-interrupted);
    border-color: rgba(194,68,68,0.3);
  }
  .destructive:hover:not(:disabled) {
    background: rgba(194,68,68,0.10);
  }
</style>
