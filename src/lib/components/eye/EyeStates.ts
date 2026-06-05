export type EyeState = 'idle' | 'active' | 'paused' | 'synthesizing' | 'interrupted';

export interface EyeVisual {
  /** Lid open-amount: 0 (closed line) → 1 (full vesica) */
  openness: number;
  /** Iris color token. */
  iris: string;
  /** Outer color token (lid). */
  outline: string;
  /** Whether to apply the pulsing ring animation. */
  pulse: boolean;
  /** Whether to apply the slow breathe animation. */
  breathe: boolean;
  /** Whether to apply the synthesizing slow rotation. */
  rotate: boolean;
}

export function visualFor(state: EyeState): EyeVisual {
  switch (state) {
    case 'idle':
      return {
        openness: 0.05,
        iris: 'var(--color-text-tertiary)',
        outline: 'var(--color-text-tertiary)',
        pulse: false,
        breathe: false,
        rotate: false,
      };
    case 'active':
      return {
        openness: 1,
        iris: 'var(--color-active)',
        outline: 'var(--color-text-secondary)',
        pulse: true,
        breathe: false,
        rotate: false,
      };
    case 'paused':
      return {
        openness: 0.55,
        iris: 'var(--color-paused)',
        outline: 'var(--color-brass-dim)',
        pulse: false,
        breathe: true,
        rotate: false,
      };
    case 'synthesizing':
      return {
        openness: 1,
        iris: 'var(--color-synthesizing)',
        outline: 'var(--color-text-secondary)',
        pulse: false,
        breathe: false,
        rotate: true,
      };
    case 'interrupted':
      return {
        openness: 0.2,
        iris: 'var(--color-interrupted)',
        outline: 'var(--color-interrupted)',
        pulse: false,
        breathe: false,
        rotate: false,
      };
  }
}
