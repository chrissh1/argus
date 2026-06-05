/** Shapes that mirror Rust serde structs in src-tauri/src. */

export type SessionStatus =
  | 'idle'
  | 'active'
  | 'paused'
  | 'synthesizing'
  | 'complete'
  | 'interrupted';

export type VaultAction = 'appended' | 'created';

export interface VaultFileAffected {
  path: string;
  action: VaultAction;
  summary: string | null;
}

export interface SessionRecord {
  id: string;
  displayName: string | null;
  status: SessionStatus;
  startedAt: number;
  endedAt: number | null;
  durationSecs: number;
  pausedSecs: number;
  vaultFilesAffected: VaultFileAffected[];
  actionItems: string[];
  openQuestions: string[];
  rawDbPath: string | null;
  rawDbExpiresAt: number | null;
}

export interface CurrentSessionSnapshot {
  record: SessionRecord;
  durationSecs: number;
  pausedSecs: number;
}

export interface ExclusionEntry {
  name: string;
  bundleId: string;
}

export interface Settings {
  vaultPath: string | null;
  ollamaHost: string;
  ollamaModel: string;
  embedModel: string;
  dataRetentionDays: number;
  similarityThreshold: number;
  minSessionSeconds: number;
  exclusionList: ExclusionEntry[];
}

export interface IndexStatus {
  configured: boolean;
  noteCount: number;
  chunkCount: number;
  lastIndexedAt: number | null;
  indexing: boolean;
  progressCurrent: number;
  progressTotal: number;
}

export interface ModelTag {
  name: string;
  size: number | null;
}

export interface OllamaTestResult {
  ok: boolean;
  models: ModelTag[];
  error: string | null;
}

export interface SynthesisStep {
  sessionId: string;
  step: string;
  message: string;
  progress: number;
  total: number;
}

export interface SessionStateChanged {
  sessionId: string | null;
  status: SessionStatus;
}

export interface IndexProgress {
  current: number;
  total: number;
}
