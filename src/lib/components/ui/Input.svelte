<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends Omit<HTMLInputAttributes, 'value'> {
    value?: string;
    mono?: boolean;
    full?: boolean;
  }
  let {
    value = $bindable(''),
    mono = false,
    full = false,
    type = 'text',
    ...rest
  }: Props = $props();
</script>

<input
  bind:value
  {type}
  class:mono
  class:full
  {...rest}
/>

<style>
  input {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    font-family: var(--font-body);
    font-size: var(--size-base);
    height: 34px;
    padding: 0 10px;
    width: auto;
    transition: border-color var(--duration-fast) var(--ease-default),
                box-shadow var(--duration-fast) var(--ease-default);
  }
  input.full { width: 100%; }
  input.mono { font-family: var(--font-mono); font-size: var(--size-base); }
  input::placeholder { color: var(--color-text-tertiary); }
  input:focus {
    outline: none;
    border-color: var(--color-brass-dim);
    box-shadow: 0 0 0 3px rgba(200,146,42,0.10);
  }
  input:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
