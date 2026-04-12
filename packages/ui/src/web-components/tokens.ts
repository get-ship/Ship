/**
 * Ship design token references.
 * Components consume these via `var(--token, fallback)`. Dark mode is
 * inherited from a `.dark` class on an ancestor outside the shadow root.
 */

export const TOKENS = {
  background: 'var(--background, oklch(0.99 0.001 106.423))',
  foreground: 'var(--foreground, oklch(0.147 0.004 49.25))',
  card: 'var(--card, oklch(1 0 0))',
  cardForeground: 'var(--card-foreground, oklch(0.147 0.004 49.25))',
  primary: 'var(--primary, oklch(0.67 0.16 58))',
  primaryForeground: 'var(--primary-foreground, oklch(0.99 0.02 95))',
  accent: 'var(--accent, oklch(0.61 0.23 303))',
  accentForeground: 'var(--accent-foreground, oklch(0.98 0.02 301))',
  muted: 'var(--muted, oklch(0.95 0.003 106.424))',
  mutedForeground: 'var(--muted-foreground, oklch(0.40 0.02 58.071))',
  destructive: 'var(--destructive, oklch(0.577 0.245 27.325))',
  border: 'var(--border, oklch(0.147 0.004 49.25 / 25%))',
  radius: 'var(--radius, 0.625rem)',
  // Derived semantic colors with fallbacks
  success: 'oklch(0.65 0.17 145)',
  warning: 'oklch(0.78 0.15 80)',
  info: 'oklch(0.65 0.15 240)',
} as const

export const FONT_STACK =
  "system-ui, -apple-system, 'Segoe UI', Roboto, Ubuntu, sans-serif"

export const MONO_STACK =
  "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

export function defineOnce(name: string, ctor: CustomElementConstructor): void {
  if (typeof customElements === 'undefined') return
  if (!customElements.get(name)) customElements.define(name, ctor)
}
