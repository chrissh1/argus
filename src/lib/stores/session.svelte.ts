import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  CurrentSessionSnapshot,
  SessionRecord,
  SessionStateChanged,
  SynthesisStep,
} from '$lib/types';

export interface SessionStore {
  current: CurrentSessionSnapshot | null;
  history: SessionRecord[];
  synthesis: Record<string, SynthesisStep>;
  loaded: boolean;
}

function createStore() {
  const state = $state<SessionStore>({
    current: null,
    history: [],
    synthesis: {},
    loaded: false,
  });

  let subscribed = false;

  async function refresh() {
    const [cur, hist] = await Promise.all([
      invoke<CurrentSessionSnapshot | null>('session_current'),
      invoke<SessionRecord[]>('session_list', { limit: 100 }),
    ]);
    state.current = cur;
    state.history = hist;
    state.loaded = true;
  }

  async function start() {
    const rec = await invoke<SessionRecord>('session_start');
    state.current = {
      record: rec,
      durationSecs: 0,
      pausedSecs: 0,
    };
  }

  async function pause() {
    const rec = await invoke<SessionRecord>('session_pause');
    if (state.current) state.current = { ...state.current, record: rec };
  }

  async function resume() {
    const rec = await invoke<SessionRecord>('session_resume');
    if (state.current) state.current = { ...state.current, record: rec };
  }

  async function stop() {
    try {
      await invoke<SessionRecord>('session_stop');
    } finally {
      state.current = null;
      await refresh();
    }
  }

  async function ensureSubscribed() {
    if (subscribed) return;
    subscribed = true;
    await listen<SessionStateChanged>('session-state', (e) => {
      const s = e.payload;
      if (s.status === 'idle' || !s.sessionId) {
        state.current = null;
      } else if (state.current) {
        state.current = {
          ...state.current,
          record: { ...state.current.record, status: s.status as any },
        };
      }
      refresh();
    });
    await listen<SynthesisStep>('synthesis-progress', (e) => {
      state.synthesis = { ...state.synthesis, [e.payload.sessionId]: e.payload };
    });
    await listen<SynthesisStep>('synthesis-complete', (e) => {
      const { [e.payload.sessionId]: _drop, ...rest } = state.synthesis;
      state.synthesis = rest;
      refresh();
    });
  }

  return {
    get state() {
      return state;
    },
    refresh,
    ensureSubscribed,
    start,
    pause,
    resume,
    stop,
  };
}

export const sessionStore = createStore();
