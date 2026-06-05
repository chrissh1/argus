<script lang="ts">
  interface Props {
    bars?: number;
    width?: number;
    height?: number;
  }
  let { bars = 20, width = 60, height = 14 }: Props = $props();
  const items = $derived(Array.from({ length: bars }, (_, i) => i));
</script>

<div class="waveform" style:width="{width}px" style:height="{height}px">
  {#each items as i (i)}
    <span
      class="bar"
      style:animation-delay="{(i * 90) % 1800}ms"
      style:height="{height}px"
    ></span>
  {/each}
</div>

<style>
  .waveform {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1px;
  }
  .bar {
    flex: 1;
    background: var(--color-active);
    opacity: 0.3;
    border-radius: 1px;
    transform-origin: center;
    transform: scaleY(0.3);
    animation: waveform-bar 1.8s ease-in-out infinite;
  }
</style>
