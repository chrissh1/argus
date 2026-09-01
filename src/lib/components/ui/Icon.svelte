<script lang="ts">
  /** Lightweight Lucide-style line icons, inline SVG. */
  type Name =
    | 'home'
    | 'clock-history'
    | 'brain-circuit'
    | 'aperture'
    | 'book-open'
    | 'settings'
    | 'play'
    | 'pause'
    | 'stop'
    | 'chevron-down'
    | 'chevron-right'
    | 'arrow-up-right'
    | 'arrow-down-right'
    | 'plus'
    | 'x'
    | 'pencil'
    | 'check'
    | 'circle'
    | 'folder'
    | 'more-horizontal';

  interface Props {
    name: Name;
    size?: number;
    stroke?: number;
  }
  let { name, size = 16, stroke = 1.5 }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class="icon"
>
  {#if name === 'home'}
    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    <polyline points="9 22 9 12 15 12 15 22" />
  {:else if name === 'clock-history'}
    <circle cx="12" cy="12" r="9" />
    <polyline points="12,7 12,12 16,14" />
  {:else if name === 'brain-circuit'}
    <path d="M9 4a3 3 0 0 0-3 3v1a3 3 0 0 0-2 3v2a3 3 0 0 0 2 3v1a3 3 0 0 0 3 3" />
    <path d="M15 4a3 3 0 0 1 3 3v1a3 3 0 0 1 2 3v2a3 3 0 0 1-2 3v1a3 3 0 0 1-3 3" />
    <line x1="9" y1="9" x2="15" y2="9" />
    <line x1="9" y1="15" x2="15" y2="15" />
    <circle cx="12" cy="12" r="1.5" />
  {:else if name === 'aperture'}
    <circle cx="12" cy="12" r="9" />
    <line x1="14.31" y1="8" x2="20.05" y2="17.94" />
    <line x1="9.69" y1="8" x2="21.17" y2="8" />
    <line x1="7.38" y1="12" x2="13.12" y2="2.06" />
    <line x1="9.69" y1="16" x2="3.95" y2="6.06" />
    <line x1="14.31" y1="16" x2="2.83" y2="16" />
    <line x1="16.62" y1="12" x2="10.88" y2="21.94" />
  {:else if name === 'book-open'}
    <path d="M2 4h7a3 3 0 0 1 3 3v13a3 3 0 0 0-3-3H2z" />
    <path d="M22 4h-7a3 3 0 0 0-3 3v13a3 3 0 0 1 3-3h7z" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h0a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82h0a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
  {:else if name === 'play'}
    <polygon points="6,4 20,12 6,20" fill="currentColor" />
  {:else if name === 'pause'}
    <rect x="6" y="5" width="4" height="14" fill="currentColor" />
    <rect x="14" y="5" width="4" height="14" fill="currentColor" />
  {:else if name === 'stop'}
    <rect x="6" y="6" width="12" height="12" rx="1" fill="currentColor" />
  {:else if name === 'chevron-down'}
    <polyline points="6,9 12,15 18,9" />
  {:else if name === 'chevron-right'}
    <polyline points="9,6 15,12 9,18" />
  {:else if name === 'arrow-up-right'}
    <line x1="6" y1="18" x2="18" y2="6" />
    <polyline points="9,6 18,6 18,15" />
  {:else if name === 'arrow-down-right'}
    <line x1="7" y1="7" x2="17" y2="17" />
    <polyline points="17,11 17,17 11,17" />
  {:else if name === 'plus'}
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  {:else if name === 'x'}
    <line x1="6" y1="6" x2="18" y2="18" />
    <line x1="18" y1="6" x2="6" y2="18" />
  {:else if name === 'pencil'}
    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
  {:else if name === 'check'}
    <polyline points="20,6 9,17 4,12" />
  {:else if name === 'circle'}
    <circle cx="12" cy="12" r="9" />
  {:else if name === 'folder'}
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
  {:else if name === 'more-horizontal'}
    <circle cx="5" cy="12" r="1" fill="currentColor" />
    <circle cx="12" cy="12" r="1" fill="currentColor" />
    <circle cx="19" cy="12" r="1" fill="currentColor" />
  {/if}
</svg>

<style>
  .icon { flex-shrink: 0; vertical-align: middle; }
</style>
