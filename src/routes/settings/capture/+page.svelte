<script lang="ts">
  import Panel from '$lib/components/settings/Panel.svelte';
  import Field from '$lib/components/settings/Field.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';

  let retention = $state('30');
  let minSecs = $state('60');

  onMount(async () => {
    if (!settingsStore.state.loaded) await settingsStore.load();
    const s = settingsStore.state.settings;
    if (s) {
      retention = String(s.dataRetentionDays);
      minSecs = String(s.minSessionSeconds);
    }
  });

  async function saveRetention() {
    await settingsStore.set('data_retention_days', String(parseInt(retention, 10) || 30));
  }
  async function saveMinSecs() {
    await settingsStore.set('min_session_seconds', String(parseInt(minSecs, 10) || 60));
  }

  async function addApp() {
    const path = await openDialog({
      title: 'Choose .app bundle',
      multiple: false,
      directory: false,
      filters: [{ name: 'macOS Apps', extensions: ['app'] }],
    });
    if (!path || typeof path !== 'string') return;
    const name = path.split('/').pop()?.replace(/\.app$/, '') ?? path;
    // Bundle ID derivation would be done in Rust; here we use the app name as a placeholder identifier.
    await settingsStore.addExclusion({ name, bundleId: name.toLowerCase().replace(/\s+/g, '.') });
  }

  const exclusion = $derived(settingsStore.state.settings?.exclusionList ?? []);
</script>

<Panel title="Capture Settings" description="Control what Argus records during a session.">
  <section>
    <div class="label">App Exclusion List</div>
    <p class="desc">These apps will auto-pause recording when focused.</p>
    <div class="list">
      {#if exclusion.length === 0}
        <div class="empty mono">No apps excluded.</div>
      {/if}
      {#each exclusion as e}
        <div class="row">
          <div class="name">{e.name}</div>
          <div class="bid mono">{e.bundleId}</div>
          <button class="rm" type="button" aria-label="Remove" onclick={() => settingsStore.removeExclusion(e.bundleId)}>
            <Icon name="x" size={12} />
          </button>
        </div>
      {/each}
    </div>
    <div class="add"><Button variant="secondary" onclick={addApp}>
      {#snippet leading()}<Icon name="plus" size={12} />{/snippet}
      Add App
    </Button></div>
  </section>

  <section>
    <div class="label">Data Retention</div>
    <Field label="Delete raw session data after" hint="Session summaries and vault notes are kept permanently.">
      <Input bind:value={retention} onchange={saveRetention} mono />
      <span class="unit">days</span>
    </Field>
  </section>

  <section>
    <div class="label">Minimum Session Duration</div>
    <Field label="Ignore sessions shorter than">
      <Input bind:value={minSecs} onchange={saveMinSecs} mono />
      <span class="unit">seconds</span>
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
    margin-bottom: 8px;
  }
  .desc {
    font-family: var(--font-body);
    font-size: var(--size-sm);
    color: var(--color-text-tertiary);
    margin-bottom: 10px;
  }
  .list {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .empty {
    padding: 14px;
    color: var(--color-text-tertiary);
    font-size: var(--size-sm);
    text-align: center;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr 28px;
    gap: 12px;
    align-items: center;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  .row:last-child { border-bottom: 0; }
  .name { font-size: var(--size-base); color: var(--color-text-primary); }
  .bid { font-size: var(--size-xs); color: var(--color-text-tertiary); }
  .rm {
    border-radius: var(--radius-sm);
    color: var(--color-text-tertiary);
    background: transparent;
    width: 24px; height: 24px;
    display: inline-flex; align-items: center; justify-content: center;
  }
  .rm:hover { color: var(--color-interrupted); background: rgba(194,68,68,0.10); }

  .add { margin-top: 10px; }
  .unit { color: var(--color-text-tertiary); font-size: var(--size-sm); }
</style>
