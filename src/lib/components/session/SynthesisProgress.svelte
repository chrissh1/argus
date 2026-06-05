<script lang="ts">
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

  interface Props {
    message?: string;
    progress?: number;
    total?: number;
  }
  let { message = 'Synthesizing…', progress = 0, total = 0 }: Props = $props();
  const value = $derived(total > 0 ? progress / total : null);
</script>

<div class="synthesis">
  <ProgressBar value={value} />
  <p class="msg">{message}</p>
</div>

<style>
  .synthesis {
    padding: 8px 0 4px;
  }
  .msg {
    margin-top: 8px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }
</style>
