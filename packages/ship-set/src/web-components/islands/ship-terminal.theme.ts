/**
 * xterm.js theme derived from Ship design tokens.
 *
 * Since xterm.js themes accept concrete color strings (not CSS custom
 * properties), we read computed values from the host element at runtime.
 * The `buildTheme` function resolves token variables against a live DOM
 * element so the theme follows light/dark mode automatically.
 */

/** Fallback palette used when computed styles are unavailable (e.g. JSDOM). */
const FALLBACKS = {
  background: '#fafaf9',
  foreground: '#1c1917',
  cursor: '#c2410c',
  cursorAccent: '#fafaf9',
  selectionBackground: 'rgba(194, 65, 12, 0.3)',
  black: '#78716c',
  red: '#dc2626',
  green: '#16a34a',
  yellow: '#ca8a04',
  blue: '#2563eb',
  magenta: '#9333ea',
  cyan: '#0891b2',
  white: '#1c1917',
  brightBlack: '#a8a29e',
  brightRed: '#ef4444',
  brightGreen: '#22c55e',
  brightYellow: '#eab308',
  brightBlue: '#3b82f6',
  brightMagenta: '#a855f7',
  brightCyan: '#06b6d4',
  brightWhite: '#fafaf9',
} as const

/**
 * Token-to-CSS-variable mapping. Each key is an xterm ITheme property;
 * the value is the CSS custom property to resolve.
 */
const TOKEN_MAP: Record<string, string> = {
  background: '--background',
  foreground: '--foreground',
  cursor: '--primary',
  cursorAccent: '--background',
  black: '--muted-foreground',
  red: '--destructive',
  green: '--success',
  yellow: '--warning',
  blue: '--info',
  magenta: '--accent',
  white: '--foreground',
}

/** Resolve a single CSS variable from a live element. */
function resolve(el: Element, prop: string): string {
  return getComputedStyle(el).getPropertyValue(prop).trim()
}

/**
 * Build an xterm.js ITheme from Ship tokens resolved against `host`.
 * Call this in `connectedCallback` and whenever the theme attribute changes.
 */
export function buildTheme(host: Element): Record<string, string> {
  const theme: Record<string, string> = { ...FALLBACKS }

  for (const [key, cssVar] of Object.entries(TOKEN_MAP)) {
    const val = resolve(host, cssVar)
    if (val) theme[key] = val
  }

  // Selection background: primary at 30% opacity
  const primary = theme.cursor || FALLBACKS.cursor
  theme.selectionBackground = `color-mix(in oklch, ${primary} 30%, transparent)`

  // Cyan is not a token; use a fixed oklch value
  theme.cyan = 'oklch(0.7 0.15 200)'

  // Bright variants: slightly lighter/more saturated
  theme.brightBlack = theme.white
  theme.brightRed = theme.red
  theme.brightGreen = theme.green
  theme.brightYellow = theme.yellow
  theme.brightBlue = theme.blue
  theme.brightMagenta = theme.magenta
  theme.brightCyan = theme.cyan
  theme.brightWhite = theme.background

  return theme
}
