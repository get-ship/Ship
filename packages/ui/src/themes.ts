/**
 * Shared theme catalog for the SHIP app and marketing site.
 *
 * The palette definitions live alongside this module in `./themes.css`
 * (alternates) and `./tokens.css` (canonical light + dark). Consumers
 * only need this module to know the theme ids, human labels, and
 * whether each theme should carry the canonical `.dark` class.
 */

export type ThemeId =
  | 'ship-light'
  | 'ship-dark'
  | 'high-contrast'
  | 'warm'
  | 'gruvbox-dark'
  | 'gruvbox-light'
  | 'nord'
  | 'solarized-dark'

export interface Theme {
  id: ThemeId
  label: string
  /** Grouping hint for pickers — "Ship" for the canonical themes,
   *  "Classics" for the community alternates. */
  group: 'Ship' | 'Classics'
  /** Whether the theme needs the canonical `.dark` class applied so
   *  shadow/elevation tokens from @ship/ui/tokens.css flip too. */
  dark: boolean
  /** Preview swatches for picker UIs — [background, card, primary, accent].
   *  Mirrors the palette the theme actually paints with so cards can
   *  show at a glance what the theme looks like. */
  swatches: readonly [string, string, string, string]
}

export const THEMES: Theme[] = [
  {
    id: 'ship-light', label: 'SHIP Light', group: 'Ship', dark: false,
    swatches: [
      'oklch(0.99 0.001 106.423)', // background
      'oklch(1 0 0)',              // card
      'oklch(0.67 0.16 58)',       // primary — SHIP amber
      'oklch(0.61 0.23 303)',      // accent — purple
    ],
  },
  {
    id: 'ship-dark', label: 'SHIP Dark', group: 'Ship', dark: true,
    swatches: [
      'oklch(0.18 0.01 250)',      // background
      'oklch(0.18 0.01 50)',       // card
      'oklch(0.77 0.16 70)',       // primary — warm amber
      'oklch(0.3 0.05 300)',       // accent — muted purple
    ],
  },
  {
    id: 'high-contrast', label: 'High Contrast', group: 'Ship', dark: true,
    swatches: [
      'oklch(0.09 0 0)',
      'oklch(0.15 0 0)',
      'oklch(0.87 0.18 85)',       // amber primary = accent
      'oklch(0.99 0 0)',
    ],
  },
  {
    id: 'warm', label: 'Warm (paper)', group: 'Ship', dark: false,
    swatches: [
      'oklch(0.97 0.014 80)',
      'oklch(0.99 0.010 80)',
      'oklch(0.55 0.14 35)',
      'oklch(0.28 0.03 55)',
    ],
  },
  {
    id: 'gruvbox-dark', label: 'Gruvbox Dark', group: 'Classics', dark: true,
    swatches: [
      'oklch(0.28 0.014 80)',
      'oklch(0.33 0.016 80)',
      'oklch(0.82 0.16 80)',
      'oklch(0.72 0.03 85)',
    ],
  },
  {
    id: 'gruvbox-light', label: 'Gruvbox Light', group: 'Classics', dark: false,
    swatches: [
      'oklch(0.95 0.04 85)',
      'oklch(0.97 0.035 85)',
      'oklch(0.66 0.16 55)',
      'oklch(0.50 0.02 80)',
    ],
  },
  {
    id: 'nord', label: 'Nord', group: 'Classics', dark: true,
    swatches: [
      'oklch(0.29 0.02 255)',
      'oklch(0.34 0.02 255)',
      'oklch(0.77 0.06 220)',
      'oklch(0.95 0.01 250)',
    ],
  },
  {
    id: 'solarized-dark', label: 'Solarized Dark', group: 'Classics', dark: true,
    swatches: [
      'oklch(0.26 0.03 220)',
      'oklch(0.30 0.03 215)',
      'oklch(0.65 0.14 240)',
      'oklch(0.72 0.02 220)',
    ],
  },
]

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
)

export const DEFAULT_THEME: ThemeId = 'ship-light'
export const THEME_STORAGE_KEY = 'ship-site-theme'

export const isValidTheme = (value: unknown): value is ThemeId =>
  typeof value === 'string' && value in THEME_BY_ID

export const isDarkTheme = (id: ThemeId): boolean =>
  THEME_BY_ID[id]?.dark ?? false

/**
 * Apply a theme to the given root element (or documentElement by
 * default). Sets `data-theme` and toggles the canonical `.dark` class.
 * Returns the applied ThemeId.
 */
export function applyTheme(
  id: unknown,
  root: HTMLElement = document.documentElement,
): ThemeId {
  const next: ThemeId = isValidTheme(id) ? id : DEFAULT_THEME
  root.dataset.theme = next
  root.classList.toggle('dark', isDarkTheme(next))
  return next
}

/**
 * Read the persisted theme from localStorage, fall back to
 * DEFAULT_THEME. Safe to call server-side (returns default).
 */
export function readStoredTheme(): ThemeId {
  if (typeof localStorage === 'undefined') return DEFAULT_THEME
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY)
    return isValidTheme(raw) ? raw : DEFAULT_THEME
  } catch {
    return DEFAULT_THEME
  }
}

export function persistTheme(id: ThemeId): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id)
  } catch {}
}
