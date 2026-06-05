<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    description?: string;
    children: Snippet;
  }
  let { title, description, children }: Props = $props();
</script>

<div class="panel">
  <div class="head">
    <div class="title section-label">{title}</div>
    {#if description}<p class="desc">{description}</p>{/if}
  </div>
  <div class="rule"></div>
  <div class="body">
    {@render children()}
  </div>
</div>

<style>
  .panel {
    max-width: 560px;
    margin: 0 auto;
    padding: 32px 24px 48px;
    animation: panel-in var(--duration-normal) var(--ease-default);
  }
  .head { margin-bottom: 0; }
  .title { font-size: var(--size-xs); }
  .desc {
    font-family: var(--font-body);
    font-size: var(--size-base);
    color: var(--color-text-secondary);
    margin-top: 4px;
  }
  .rule {
    height: 1px;
    background: var(--color-border-subtle);
    margin: 14px 0 20px;
  }
  .body :global(section + section) { margin-top: 28px; }
</style>
