/**
 * Shared helpers for Ship web components.
 *
 * Brand tokens are not defined here — they live in `@ship/ui/tokens.css`
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
    --tone-fg: var(--foreground);
    --tone-bg: var(--muted);
    --tone-border: var(--border);
  }
  :host([tone='info']) {
    --tone-fg: var(--info);
    --tone-bg: color-mix(in oklch, var(--info) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--info) 35%, transparent);
  }
  :host([tone='success']) {
    --tone-fg: var(--success);
    --tone-bg: color-mix(in oklch, var(--success) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--success) 35%, transparent);
  }
  :host([tone='warning']) {
    --tone-fg: var(--warning);
    --tone-bg: color-mix(in oklch, var(--warning) 18%, transparent);
    --tone-border: color-mix(in oklch, var(--warning) 40%, transparent);
  }
  :host([tone='destructive']) {
    --tone-fg: var(--destructive);
    --tone-bg: color-mix(in oklch, var(--destructive) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--destructive) 35%, transparent);
  }
  :host([tone='accent']) {
    --tone-fg: var(--accent);
    --tone-bg: color-mix(in oklch, var(--accent) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--accent) 35%, transparent);
  }
  :host([tone='muted']) {
    --tone-fg: var(--muted-foreground);
    --tone-bg: var(--muted);
    --tone-border: var(--border);
  }
`

/**
 * Flush the test DOM between assertions. Use in tests as
 * `afterEach(cleanupTestDom)`.
 */
export function cleanupTestDom(): void {
  document.body.innerHTML = ''
}
