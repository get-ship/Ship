/**
 * Shared theme catalog for the SHIP app and marketing site.
 *
 * Every theme is a full aesthetic bundle — palette, surfaces, radii,
 * shadows, motion, typography, and interaction strength. The picker is
 * flat: no tiers, no split. Pick one and the app *feels* different,
 * not just looks different.
 *
 * Palette definitions live alongside this module in `./themes.css`
 * (per-theme overrides) and `./tokens.css` (canonical ship-light +
 * ship-dark base). Consumers only need this module to know the theme
 * ids, human labels, and whether each theme should carry the canonical
 * `.dark` class.
 */

export type ThemeId =
  | 'ship-light'
  | 'ship-dark'
  | 'paper'
  | 'halo'
  | 'console'
  | 'contrast'

export interface Theme {
  id: ThemeId
  label: string
  /** Whether the theme needs the canonical `.dark` class applied so
   *  shadow/elevation tokens from @ship/set/tokens.css flip too. */
  dark: boolean
  /** Preview swatches for picker UIs — [background, card, primary, accent].
   *  Mirrors the palette the theme actually paints with so cards can
   *  show at a glance what the theme looks like. */
  swatches: readonly [string, string, string, string]
}

export const THEMES: Theme[] = [
  {
    id: 'ship-light', label: 'SHIP Light', dark: false,
    swatches: [
      'oklch(0.99 0.001 106.423)', // background
      'oklch(1 0 0)',              // card
      'oklch(0.67 0.16 58)',       // primary — SHIP amber
      'oklch(0.61 0.23 303)',      // accent — purple
    ],
  },
  {
    id: 'ship-dark', label: 'SHIP Dark', dark: true,
    swatches: [
      'oklch(0.18 0.01 250)',      // background
      'oklch(0.18 0.01 50)',       // card
      'oklch(0.77 0.16 70)',       // primary — warm amber
      'oklch(0.3 0.05 300)',       // accent — muted purple
    ],
  },
  {
    id: 'paper', label: 'Paper', dark: false,
    swatches: [
      'oklch(0.97 0.014 80)',      // warm cream
      'oklch(0.99 0.010 80)',      // card
      'oklch(0.55 0.14 35)',       // terracotta primary
      'oklch(0.48 0.09 55)',       // muted accent
    ],
  },
  {
    id: 'halo', label: 'Halo (Soft)', dark: false,
    swatches: [
      'oklch(0.92 0.008 230)',     // soft mid-grey-blue surface
      'oklch(0.92 0.008 230)',     // same — depth from shadow, not hue
      'oklch(0.62 0.14 50)',       // muted orange primary
      'oklch(0.55 0.05 230)',      // accent
    ],
  },
  {
    id: 'console', label: 'Console', dark: true,
    swatches: [
      'oklch(0.13 0.008 220)',     // near-black ground
      'oklch(0.16 0.010 220)',     // tinted card
      'oklch(0.78 0.20 145)',      // phosphor green
      'oklch(0.78 0.16 195)',      // teal accent
    ],
  },
  {
    id: 'contrast', label: 'High Contrast', dark: true,
    swatches: [
      'oklch(0.09 0 0)',           // near-black
      'oklch(0.15 0 0)',           // card
      'oklch(0.87 0.18 85)',       // amber primary = accent
      'oklch(0.99 0 0)',           // pure white fg
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
  const dark = isDarkTheme(next)
  root.dataset.theme = next
  root.classList.toggle('dark', dark)
  // Always announce theme application so chrome (status bar, devtools,
  // etc.) can repaint deterministically without a MutationObserver.
  if (typeof document !== 'undefined' && typeof CustomEvent !== 'undefined') {
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: next, dark } }),
    )
  }
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

/**
 * Persist the current theme. localStorage is the tier-4 cache that
 * the pre-paint boot script reads (must be synchronous); `/api/settings`
 * is the tier-2 authoritative store so the choice is queryable by the
 * settings view and future cross-device sync. Write-through is
 * fire-and-forget — if the daemon is unreachable the local cache is
 * still correct and `syncThemeFromSettings()` reconciles on next load.
 */
export function persistTheme(id: ThemeId): void {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id)
  } catch {}
  // Tier-2 sync-through. Ignore failures — cache is source of next read.
  try {
    fetch('/api/settings', {
      method: 'PUT',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key: 'shell.theme', value: id }),
    }).catch(() => {})
  } catch {}
}

/**
 * Fetch the authoritative theme from `/api/settings` and update the
 * local cache to match. Call on shell mount so another device's change
 * propagates. If the daemon is unreachable, localStorage stays as-is.
 */
export async function syncThemeFromSettings(): Promise<ThemeId | null> {
  try {
    const resp = await fetch('/api/settings', { credentials: 'same-origin' })
    if (!resp.ok) return null
    const body = (await resp.json()) as {
      data?: Record<string, unknown>
    }
    const raw = body.data?.['shell.theme']
    if (!isValidTheme(raw)) return null
    try {
      localStorage.setItem(THEME_STORAGE_KEY, raw)
    } catch {}
    return raw
  } catch {
    return null
  }
}
