/**
 * Shared helpers for Ship web components.
 *
 * Brand tokens are not defined here — they live in `@ship/set/tokens.css`
 * and are referenced inside component templates as plain `var(--foo)`.
 * The tokens stylesheet is guaranteed loaded by Studio and by the daemon
 * view host, so fallbacks are unnecessary.
 */

import { css, unsafeCSS } from 'lit'

export const MONO_STACK =
  "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

/** Use as `font-family: ${MONO};` inside a `css` tagged template. */
export const MONO = unsafeCSS(MONO_STACK)

/** Safe `customElements.define` — skips if already defined or in non-browser env. */
export function defineOnce(name: string, ctor: CustomElementConstructor): void {
  if (typeof customElements === 'undefined') return
  if (!customElements.get(name)) customElements.define(name, ctor)
}

/**
 * Fixed vocabulary — see Ship Component Charter §Composition model.
 * Components must pick from these sets; new values require charter amendment.
 */
export type Tone =
  | 'default'
  | 'primary'
  | 'info'
  | 'success'
  | 'warning'
  | 'destructive'
  | 'accent'
  | 'muted'

export type Variant = 'solid' | 'soft' | 'outline' | 'ghost'

export type Size = 'sm' | 'md' | 'lg'

/**
 * CSS fragment mapping a `tone` axis value onto `--tone-fg / --tone-bg /
 * --tone-border`. Components consume these via `var(--tone-*)` so the same
 * tone-attr selectors apply everywhere.
 *
 * Percentages chosen to match the existing pipelines/specs views' soft tints
 * (oklch at ~16–20% opacity over the background). Do not adjust without
 * updating the ground-truth views.
 */
export const toneVars = css`
  :host {
    --tone-fg: var(--tone-default-fg);
    --tone-bg: var(--tone-default-bg);
    --tone-border: var(--tone-default-border);
    --tone-ring: var(--tone-default-ring);
  }
  :host([tone='info'])        { --tone-fg: var(--tone-info-fg);        --tone-bg: var(--tone-info-bg);        --tone-border: var(--tone-info-border);        --tone-ring: var(--tone-info-ring); }
  :host([tone='success'])     { --tone-fg: var(--tone-success-fg);     --tone-bg: var(--tone-success-bg);     --tone-border: var(--tone-success-border);     --tone-ring: var(--tone-success-ring); }
  :host([tone='warning'])     { --tone-fg: var(--tone-warning-fg);     --tone-bg: var(--tone-warning-bg);     --tone-border: var(--tone-warning-border);     --tone-ring: var(--tone-warning-ring); }
  :host([tone='destructive']) { --tone-fg: var(--tone-destructive-fg); --tone-bg: var(--tone-destructive-bg); --tone-border: var(--tone-destructive-border); --tone-ring: var(--tone-destructive-ring); }
  :host([tone='accent'])      { --tone-fg: var(--tone-accent-fg);      --tone-bg: var(--tone-accent-bg);      --tone-border: var(--tone-accent-border);      --tone-ring: var(--tone-accent-ring); }
  :host([tone='muted'])       { --tone-fg: var(--tone-muted-fg);       --tone-bg: var(--tone-muted-bg);       --tone-border: var(--tone-muted-border);       --tone-ring: var(--tone-muted-ring); }
`

/**
 * Focus-ring recipe. Apply to an interactive element on `:focus-visible`
 * via `${focusRing}` template interpolation, or reference the CSS custom
 * properties directly. Includes a `forced-colors: active` CanvasText
 * outline fallback so the ring stays visible in high-contrast mode.
 *
 * Usage:
 *   static styles = [focusRingVars, css`
 *     button:focus-visible { ${focusRingStyle} }
 *     @media (forced-colors: active) {
 *       button:focus-visible { ${focusRingForcedColors} }
 *     }
 *   `]
 */
export const focusRingStyle = unsafeCSS(`
  outline: none;
  box-shadow:
    0 0 0 var(--ring-offset) var(--background),
    0 0 0 calc(var(--ring-offset) + var(--ring-width))
      color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
`)

export const focusRingForcedColors = unsafeCSS(`
  outline: 2px solid CanvasText;
  outline-offset: 2px;
`)

/**
 * Shadow-DOM-local `prefers-reduced-motion` override. Add as the first entry
 * in a component's `static styles = [reducedMotion, css\`…\`]` so user-level
 * reduced-motion collapses every transition/animation inside the shadow tree.
 * Outer custom-property durations are *also* respected via tokens.css, but
 * hard-coded ms values need this nuclear rule to be neutralized.
 */
export const reducedMotion = css`
  @media (prefers-reduced-motion: reduce) {
    :host, :host *, ::slotted(*) {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
      scroll-behavior: auto !important;
    }
  }
`

/**
 * Flush the test DOM between assertions. Use in tests as
 * `afterEach(cleanupTestDom)`.
 */
export function cleanupTestDom(): void {
  document.body.innerHTML = ''
}
