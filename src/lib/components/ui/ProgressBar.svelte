<script lang="ts">
  interface Props {
    /** 0..1 value, or null for indeterminate shimmer. */
    value?: number | null;
    height?: number;
  }
  let { value = null, height = 4 }: Props = $props();
</script>

<div class="track" style:height="{height}px">
  {#if value === null}
    <div class="fill shimmer"></div>
  {:else}
    <div class="fill" style:width="{Math.max(0, Math.min(1, value)) * 100}%"></div>
  {/if}
</div>

<style>
  .track {
    width: 100%;
    background: var(--color-bg-surface);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--color-synthesizing);
    transition: width var(--duration-normal) var(--ease-default);
  }
  .shimmer {
    width: 100%;
    background: linear-gradient(
      90deg,
      var(--color-synthesizing) 0%,
      rgba(123,108,240,0.5) 50%,
      var(--color-synthesizing) 100%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s linear infinite;
  }
</style>
