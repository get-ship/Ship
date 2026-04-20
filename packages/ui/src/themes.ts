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
}

export const THEMES: Theme[] = [
  { id: 'ship-light',     label: 'SHIP Light',     group: 'Ship',     dark: false },
  { id: 'ship-dark',      label: 'SHIP Dark',      group: 'Ship',     dark: true  },
  { id: 'high-contrast',  label: 'High Contrast',  group: 'Ship',     dark: true  },
  { id: 'warm',           label: 'Warm (paper)',   group: 'Ship',     dark: false },
  { id: 'gruvbox-dark',   label: 'Gruvbox Dark',   group: 'Classics', dark: true  },
  { id: 'gruvbox-light',  label: 'Gruvbox Light',  group: 'Classics', dark: false },
  { id: 'nord',           label: 'Nord',           group: 'Classics', dark: true  },
  { id: 'solarized-dark', label: 'Solarized Dark', group: 'Classics', dark: true  },
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
