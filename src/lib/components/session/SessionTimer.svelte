<script lang="ts">
  import { formatDuration } from '$lib/format';
  import { onMount } from 'svelte';

  interface Props {
    /** Initial elapsed seconds (carried over after pause/resume). */
    seconds: number;
    /** If false, timer pauses. */
    running?: boolean;
    /** Reference time in ms when `seconds` was sampled. Defaults to now. */
    sampledAt?: number;
  }
  let { seconds, running = true, sampledAt }: Props = $props();

  let now = $state(Date.now());
  const ref = $derived(sampledAt ?? Date.now());

  const liveSeconds = $derived(
    running ? seconds + Math.floor((now - ref) / 1000) : seconds
  );

  onMount(() => {
    const id = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(id);
  });
</script>

<span class="timer mono">{formatDuration(liveSeconds)}</span>

<style>
  .timer {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--size-md);
    color: var(--color-text-primary);
  }
</style>
