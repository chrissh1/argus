<script lang="ts">
  import Panel from '$lib/components/settings/Panel.svelte';
  import Field from '$lib/components/settings/Field.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import type { IndexProgress } from '$lib/types';

  let threshold = $state('0.75');
  let unsubscribe: (() => void) | null = null;

  onMount(async () => {
    if (!settingsStore.state.loaded) await settingsStore.load();
    const s = settingsStore.state.settings;
    if (s) threshold = String(s.similarityThreshold);
    const off = await listen<IndexProgress>('vault-index-progress', () => {
      settingsStore.refreshIndex();
    });
    unsubscribe = off;
  });
  onDestroy(() => unsubscribe?.());

  async function chooseVault() {
    const path = await openDialog({
      title: 'Choose Obsidian Vault',
      directory: true,
      multiple: false,
    });
    if (typeof path === 'string') await settingsStore.chooseVault(path);
  }
  async function saveThreshold() {
    await settingsStore.set('similarity_threshold', String(parseFloat(threshold) || 0.75));
  }

  const idx = $derived(settingsStore.state.index);
  const indexed = $derived(!!idx && idx.chunkCount > 0);
</script>

<Panel title="Vault Settings" description="Configure your Obsidian vault integration.">
  <section>
    <div class="label">Vault Path</div>
    <div class="path-row">
      <Input
        value={settingsStore.state.settings?.vaultPath ?? ''}
        mono
        full
        readonly
        placeholder="No vault selected"
      />
      <Button variant="secondary" onclick={chooseVault}>
        {#snippet leading()}<Icon name="folder" size={12} />{/snippet}
        Choose…
      </Button>
    </div>
  </section>

  <section>
    <div class="label">Smart Append</div>
    <Field label="Similarity threshold" hint="Higher values = only append to very closely related notes. Range: 0.5 – 0.95">
      <Input bind:value={threshold} onchange={saveThreshold} mono />
    </Field>
  </section>

  <section>
    <div class="label">Vector Index</div>
    <div class="status">
      <span class="dot" class:on={indexed} class:indexing={idx?.indexing}></span>
      {#if idx?.indexing}
        Indexing… {idx.progressCurrent}/{idx.progressTotal}
      {:else if indexed}
        Indexed · {idx?.noteCount ?? 0} notes · {idx?.chunkCount ?? 0} chunks
      {:else}
        Not indexed
      {/if}
    </div>
    {#if idx?.indexing && idx.progressTotal > 0}
      <div class="progress">
        <ProgressBar value={idx.progressCurrent / idx.progressTotal} />
      </div>
    {/if}
    <div class="reindex">
      <Button variant="secondary" onclick={() => settingsStore.reindex()} disabled={!settingsStore.state.settings?.vaultPath}>
        Re-index Vault
      </Button>
    </div>
  </section>

  <section>
    <div class="label">Session Warnings</div>
    <Field label="Missing vault confirmation" hint="Prompt for confirmation before starting a session if no Obsidian vault is selected.">
      <label class="toggle-label">
        <input
          type="checkbox"
          checked={settingsStore.state.settings?.warnMissingVault ?? true}
          onchange={(e) => settingsStore.set('warn_missing_vault', String((e.target as HTMLInputElement).checked))}
        />
        <span>Warn before starting a session without an Obsidian vault</span>
      </label>
    </Field>
  </section>
</Panel>

<style>
  .label {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 10px;
  }
  .path-row { display: flex; gap: 8px; }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--color-text-tertiary); }
  .dot.on { background: var(--color-active); }
  .dot.indexing {
    background: var(--color-synthesizing);
    animation: synthesizing-dot 1.2s ease-in-out infinite;
  }
  .progress { margin-top: 12px; }
  .reindex { margin-top: 14px; }
  .toggle-label {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-primary);
    cursor: pointer;
  }
  .toggle-label input[type="checkbox"] {
    accent-color: var(--color-brass);
    width: 16px;
    height: 16px;
    cursor: pointer;
  }
</style>
