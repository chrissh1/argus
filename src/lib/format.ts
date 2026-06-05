/** Format a Unix-seconds timestamp as `Wed Jun 4, 2026 · 2:17 PM`. */
export function formatDateLong(unix: number): string {
  const d = new Date(unix * 1000);
  const date = d.toLocaleDateString('en-US', {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  });
  const time = d.toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit'
  });
  return `${date} · ${time}`;
}

/** Format a (start, end) range as `2:17 – 3:02 PM`. */
export function formatTimeRange(start: number, end: number | null): string {
  const fmt = (n: number) =>
    new Date(n * 1000).toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
  if (!end) return fmt(start);
  return `${fmt(start)} – ${fmt(end)}`;
}

/** Format seconds as MM:SS under 1h, H:MM:SS otherwise. */
export function formatDuration(secs: number): string {
  secs = Math.max(0, Math.floor(secs));
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  const pad = (n: number) => n.toString().padStart(2, '0');
  if (h > 0) return `${h}:${pad(m)}:${pad(s)}`;
  return `${pad(m)}:${pad(s)}`;
}

/** Format seconds as `45m 12s` or `1h 04m 12s`. */
export function formatDurationLong(secs: number): string {
  secs = Math.max(0, Math.floor(secs));
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  const pad = (n: number) => n.toString().padStart(2, '0');
  if (h > 0) return `${h}h ${pad(m)}m ${pad(s)}s`;
  return `${m}m ${pad(s)}s`;
}

export function basename(path: string): string {
  return path.split('/').pop() ?? path;
}
