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
  let saved = $state(false);
  let result = $state<OllamaTestResult | null>(null);

  onMount(async () => {
    if (!settingsStore.state.loaded) await settingsStore.load();
    const s = settingsStore.state.settings;
    if (s) {
      host = s.ollamaHost ?? '';
      model = s.ollamaModel ?? '';
      embed = s.embedModel ?? '';
    }
    if (host.trim()) {
      await test();
    }
  });

  async function save() {
    await settingsStore.set('ollama_host', host.trim());
    await settingsStore.set('ollama_model', model.trim());
    await settingsStore.set('embed_model', embed.trim());
    saved = true;
    setTimeout(() => { saved = false; }, 2500);
  }

  async function test() {
    testing = true;
    try {
      result = await invoke<OllamaTestResult>('ollama_test', {
        payload: {
          host: host.trim(),
          model: model.trim() ? model.trim() : null
        }
      });
    } catch (e) {
      result = { ok: false, models: [], error: String(e) };
    } finally {
      testing = false;
    }
  }

  function selectModel(name: string) {
    model = name;
  }

  function selectEmbed(name: string) {
    embed = name;
  }

  const dotClass = $derived(
    !host.trim() ? 'unknown' : !result ? 'unknown' : result.ok ? 'ok' : 'err'
  );
</script>

<Panel title="LLM Configuration" description="Connect Argus to your local Ollama instance for session synthesis.">
  <section>
    <div class="section-label">Ollama Connection</div>
    <Field label="Host URL" hint="URL of your local Ollama daemon (e.g. http://localhost:11434).">
      <Input bind:value={host} mono full placeholder="http://localhost:11434" />
    </Field>
    <div class="status">
      <span class="dot {dotClass}"></span>
      {#if testing}
        Checking connection…
      {:else if !host.trim()}
        Not configured · Enter an Ollama host URL to connect
      {:else if !result}
        Untested · Click Test Connection below
      {:else if result.ok}
        Connected · {result.models.length} model{result.models.length === 1 ? '' : 's'} available
      {:else}
        {result.error}
      {/if}
    </div>
  </section>

  {#if result?.models && result.models.length > 0}
    <div class="model-picker-hint">
      <div class="hint-label">Installed in Ollama (click to select):</div>
      <div class="model-chips">
        {#each result.models as m}
          <button
            class="chip"
            type="button"
            class:active={model === m.name}
            onclick={() => selectModel(m.name)}
          >
            {m.name}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <section>
    <div class="section-label">Inference Model</div>
    <Field label="Model name" hint="Used for session concept extraction, action items, and note drafting.">
      <Input bind:value={model} mono full placeholder="e.g. qwen2.5-coder:14b or llama3.2" />
    </Field>
  </section>

  <section>
    <div class="section-label">Embedding Model (Optional)</div>
    <Field label="Model name" hint="Used for vector semantic search in your Obsidian vault.">
      <Input bind:value={embed} mono full placeholder="e.g. nomic-embed-text" />
    </Field>
  </section>

  <div class="actions">
    <Button variant="secondary" onclick={test} disabled={testing || !host.trim()}>
      {testing ? 'Checking…' : 'Test Connection'}
    </Button>
    <Button variant="primary" onclick={save}>
      {saved ? 'Saved!' : 'Save'}
    </Button>
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

  .model-picker-hint {
    margin: 12px 0 6px;
    padding: 10px 12px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
  }

  .hint-label {
    font-family: var(--font-body);
    font-size: var(--size-xs);
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }

  .model-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    font-family: var(--font-mono);
    font-size: var(--size-xs);
    padding: 4px 10px;
    border-radius: 12px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-default);
    color: var(--color-text-primary);
    cursor: pointer;
    transition: all var(--duration-fast);
  }

  .chip:hover {
    border-color: var(--color-brass);
    color: var(--color-brass);
  }

  .chip.active {
    background: var(--color-brass);
    border-color: var(--color-brass);
    color: var(--color-text-inverse);
    font-weight: 600;
  }
</style>
