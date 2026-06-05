<script lang="ts">
  import { visualFor, type EyeState } from './EyeStates';

  interface Props {
    state?: EyeState;
    size?: number;
    /** Hide chrome to render as a pure glyph for the tray. */
    glyph?: boolean;
  }
  let { state = 'idle', size = 64, glyph = false }: Props = $props();

  const v = $derived(visualFor(state));
  // Lid path: vesica piscis where openness controls top/bottom curve y-offset.
  // ViewBox 100x100, center (50,50). At openness=1 the lens spans y=20..80.
  const span = $derived(30 * v.openness);
  const topY = $derived(50 - span);
  const botY = $derived(50 + span);
  const irisR = $derived(Math.max(2.4, 9 * v.openness));
</script>

<svg
  class="argus-eye"
  class:pulse={v.pulse}
  class:breathe={v.breathe}
  class:glyph
  width={size}
  height={size}
  viewBox="0 0 100 100"
  role="img"
  aria-label={`Argus ${state}`}
>
  <!-- Outer brass ring (decorative; hidden in glyph mode) -->
  {#if !glyph}
    <circle
      cx="50" cy="50" r="46"
      fill="none"
      stroke="var(--color-border-default)"
      stroke-width="1"
      opacity="0.35"
    />
  {/if}

  <!-- Lid: vesica piscis -->
  <path
    class="lid"
    d={`M 14 50 Q 50 ${topY} 86 50 Q 50 ${botY} 14 50 Z`}
    fill="none"
    stroke={v.outline}
    stroke-width="1.4"
    stroke-linejoin="round"
  />

  <!-- Iris pulse ring -->
  {#if v.pulse}
    <circle class="iris-ring" cx="50" cy="50" r={irisR + 2} fill="none" stroke={v.iris} stroke-width="1" />
  {/if}

  <!-- Iris -->
  <g class="iris" class:rotate={v.rotate} style:transform-origin="50px 50px">
    <circle
      cx="50" cy="50" r={irisR}
      fill={v.iris}
      opacity={state === 'idle' ? 0.6 : 1}
    />
    {#if v.rotate}
      <!-- Subtle iris texture lines for synthesizing rotation. -->
      <g opacity="0.45">
        {#each [0, 45, 90, 135] as angle}
          <line
            x1="50" y1={50 - irisR}
            x2="50" y2={50 - irisR + 3}
            stroke="white" stroke-width="0.6"
            transform={`rotate(${angle} 50 50)`}
          />
        {/each}
      </g>
    {/if}
  </g>

  <!-- Crosshair when idle to suggest 'closed' rest. -->
  {#if state === 'idle'}
    <line x1="14" y1="50" x2="86" y2="50" stroke={v.outline} stroke-width="1" opacity="0.4" />
  {/if}
</svg>

<style>
  .argus-eye {
    display: inline-block;
    overflow: visible;
    transition: filter var(--duration-slow) var(--ease-default);
  }
  .pulse .iris-ring {
    animation: recording-pulse 3s ease-in-out infinite;
    transform-origin: 50px 50px;
  }
  .pulse {
    filter: drop-shadow(0 0 6px rgba(62, 200, 122, 0.35));
  }
  .breathe {
    animation: paused-breathe 4s ease-in-out infinite;
  }
  .rotate {
    animation: iris-rotate 20s linear infinite;
    transform-box: fill-box;
  }
  .glyph .lid {
    stroke-width: 1.8;
  }
  .lid {
    transition: d var(--duration-normal) var(--ease-default);
  }
</style>
