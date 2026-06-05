import { invoke } from '@tauri-apps/api/core';
import type { ExclusionEntry, IndexStatus, Settings } from '$lib/types';

function createStore() {
  const state = $state<{
    settings: Settings | null;
    index: IndexStatus | null;
    loaded: boolean;
  }>({
    settings: null,
    index: null,
    loaded: false,
  });

  async function load() {
    state.settings = await invoke<Settings>('settings_get_all');
    state.index = await invoke<IndexStatus>('vault_index_status');
    state.loaded = true;
  }

  async function set(key: string, value: string) {
    state.settings = await invoke<Settings>('settings_set', {
      payload: { key, value },
    });
  }

  async function chooseVault(path: string) {
    state.settings = await invoke<Settings>('vault_choose', { path });
  }

  async function reindex() {
    await invoke('vault_reindex');
  }

  async function refreshIndex() {
    state.index = await invoke<IndexStatus>('vault_index_status');
  }

  async function addExclusion(entry: ExclusionEntry) {
    const list = await invoke<ExclusionEntry[]>('exclusion_add', { entry });
    if (state.settings) state.settings = { ...state.settings, exclusionList: list };
  }
  async function removeExclusion(bundleId: string) {
    const list = await invoke<ExclusionEntry[]>('exclusion_remove', { bundleId });
    if (state.settings) state.settings = { ...state.settings, exclusionList: list };
  }

  return {
    get state() {
      return state;
    },
    load,
    set,
    chooseVault,
    reindex,
    refreshIndex,
    addExclusion,
    removeExclusion,
  };
}

export const settingsStore = createStore();
