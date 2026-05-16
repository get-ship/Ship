/**
 * Local theme catalog for the docs site.
 *
 * Mirrors the canonical Ship theme catalog (id, label, dark flag, preview
 * swatches) and provides apply/persist/read helpers driven by
 * `data-theme` on `<html>`. Tier-2 sync to /api/settings is intentionally
 * omitted — the docs site is a static Astro build with no daemon.
 *
 * Tokens and per-theme overrides live in `../styles/tokens.css` and
 * `../styles/themes.css` (loaded via astro.config.mjs `customCss`).
 */

export type ThemeId =
  | 'ship-light'
  | 'ship-dark'
  | 'paper'
  | 'halo'
  | 'console'
  | 'contrast'
  | 'glass';

export interface Theme {
  id: ThemeId;
  label: string;
  /** Themes that should carry the `.dark` class so shadow/elevation
   *  tokens (which live in tokens.css under `.dark`) flip too. */
  dark: boolean;
  /** Preview swatches — [background, card, primary, accent]. */
  swatches: readonly [string, string, string, string];
}

export const THEMES: Theme[] = [
  {
    id: 'ship-light', label: 'SHIP Light', dark: false,
    swatches: [
      'oklch(0.99 0.001 106.423)',
      'oklch(1 0 0)',
      'oklch(0.67 0.16 58)',
      'oklch(0.61 0.23 303)',
    ],
  },
  {
    id: 'ship-dark', label: 'SHIP Dark', dark: true,
    swatches: [
      'oklch(0.18 0.01 250)',
      'oklch(0.18 0.01 50)',
      'oklch(0.77 0.16 70)',
      'oklch(0.3 0.05 300)',
    ],
  },
  {
    id: 'paper', label: 'Paper', dark: false,
    swatches: [
      'oklch(0.97 0.014 80)',
      'oklch(0.99 0.010 80)',
      'oklch(0.55 0.14 35)',
      'oklch(0.48 0.09 55)',
    ],
  },
  {
    id: 'halo', label: 'Halo (Soft)', dark: false,
    swatches: [
      'oklch(0.92 0.008 230)',
      'oklch(0.92 0.008 230)',
      'oklch(0.62 0.14 50)',
      'oklch(0.55 0.05 230)',
    ],
  },
  {
    id: 'console', label: 'Console', dark: true,
    swatches: [
      'oklch(0.13 0.008 220)',
      'oklch(0.16 0.010 220)',
      'oklch(0.78 0.20 145)',
      'oklch(0.78 0.16 195)',
    ],
  },
  {
    id: 'contrast', label: 'High Contrast', dark: true,
    swatches: [
      'oklch(0.09 0 0)',
      'oklch(0.15 0 0)',
      'oklch(0.87 0.18 85)',
      'oklch(0.99 0 0)',
    ],
  },
  {
    id: 'glass', label: 'Glass', dark: false,
    swatches: [
      'oklch(0.95 0.04 230)',
      'oklch(0.99 0.005 240)',
      'oklch(0.62 0.16 50)',
      'oklch(0.62 0.14 280)',
    ],
  },
];

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
);

export const DEFAULT_THEME: ThemeId = 'ship-light';
export const THEME_STORAGE_KEY = 'ship-site-theme';

export const isValidTheme = (value: unknown): value is ThemeId =>
  typeof value === 'string' && value in THEME_BY_ID;

export const isDarkTheme = (id: ThemeId): boolean =>
  THEME_BY_ID[id]?.dark ?? false;

export function applyTheme(
  id: unknown,
  root: HTMLElement = document.documentElement,
): ThemeId {
  const next: ThemeId = isValidTheme(id) ? id : DEFAULT_THEME;
  const dark = isDarkTheme(next);
  root.dataset.theme = next;
  root.classList.toggle('dark', dark);
  if (typeof document !== 'undefined' && typeof CustomEvent !== 'undefined') {
    document.dispatchEvent(
      new CustomEvent('ship:theme:change', { detail: { id: next, dark } }),
    );
  }
  return next;
}

export function readStoredTheme(): ThemeId {
  if (typeof localStorage === 'undefined') return DEFAULT_THEME;
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    return isValidTheme(raw) ? raw : DEFAULT_THEME;
  } catch {
    return DEFAULT_THEME;
  }
}

export function persistTheme(id: ThemeId): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id);
  } catch {}
}
