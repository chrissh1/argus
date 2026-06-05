<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import SummaryScreen from '$lib/components/session/SummaryScreen.svelte';
  import type { SessionRecord } from '$lib/types';

  let session = $state<SessionRecord | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      session = await invoke<SessionRecord>('session_get', { id: $page.params.id });
    } catch (e) {
      error = String(e);
    }
  });
</script>

{#if error}
  <div class="err">{error}</div>
{:else if session}
  <SummaryScreen bind:session />
{:else}
  <div class="load">Loading…</div>
{/if}

<style>
  .err, .load {
    padding: 32px 24px;
    color: var(--color-text-secondary);
    font-family: var(--font-body);
  }
  .err { color: var(--color-interrupted); }
</style>
