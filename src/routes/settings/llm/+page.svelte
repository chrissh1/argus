<script lang="ts">
  import Panel from '$lib/components/settings/Panel.svelte';
  import Field from '$lib/components/settings/Field.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { OllamaTestResult } from '$lib/types';

  let host = $state('');
  let model = $state('');
  let embed = $state('');
  let testing = $state(false);
  let result = $state<OllamaTestResult | null>(null);

  onMount(async () => {
    if (!settingsStore.state.loaded) await settingsStore.load();
    const s = settingsStore.state.settings;
    if (s) {
      host = s.ollamaHost;
      model = s.ollamaModel;
      embed = s.embedModel;
    }
  });

  async function save() {
    await settingsStore.set('ollama_host', host);
    await settingsStore.set('ollama_model', model);
    await settingsStore.set('embed_model', embed);
  }
  async function test() {
    testing = true;
    try {
      result = await invoke<OllamaTestResult>('ollama_test', { payload: { host } });
    } finally {
      testing = false;
    }
  }

  const dotClass = $derived(
    !result ? 'unknown' : result.ok ? 'ok' : 'err'
  );
</script>

<Panel title="LLM Configuration" description="Connect Argus to your local Ollama instance.">
  <section>
    <div class="section-label">Ollama Connection</div>
    <Field label="Host URL">
      <Input bind:value={host} mono full placeholder="http://localhost:11434" />
    </Field>
    <div class="status">
      <span class="dot {dotClass}"></span>
      {#if testing}
        Checking…
      {:else if !result}
        Untested
      {:else if result.ok}
        Connected · {result.models.length} model{result.models.length === 1 ? '' : 's'} detected
      {:else}
        Unreachable · {result.error}
      {/if}
    </div>
  </section>

  <section>
    <div class="section-label">Inference Model</div>
    <Field label="Model name" hint="Used for synthesis, extraction, and smart-append decisions.">
      <Input bind:value={model} mono full placeholder="llama3.2" />
    </Field>
  </section>

  <section>
    <div class="section-label">Embedding Model</div>
    <Field label="Model name" hint="Used for vault vector indexing.">
      <Input bind:value={embed} mono full placeholder="nomic-embed-text" />
    </Field>
  </section>

  <div class="actions">
    <Button variant="secondary" onclick={test} disabled={testing}>Test Connection</Button>
    <Button variant="primary" onclick={save}>Save</Button>
  </div>
</Panel>

<style>
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-secondary);
  }
  .dot { width: 8px; height: 8px; border-radius: 50%; }
  .dot.ok      { background: var(--color-active); }
  .dot.err     { background: var(--color-interrupted); }
  .dot.unknown { background: var(--color-text-tertiary); }

  .section-label {
    font-family: var(--font-body);
    font-weight: 500;
    font-size: var(--size-xs);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--color-text-secondary);
    margin-bottom: 10px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 24px;
  }
</style>
