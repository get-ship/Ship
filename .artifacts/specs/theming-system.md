# Spec: Ship Theming System

Status: Draft
Author: Commander
Date: 2026-04-18

## Summary

Ship views, components, and the shell share a unified design token layer via CSS custom properties. This spec adds user-customizable themes — both built-in presets (ship-dark, ship-light, gruvbox, retro, etc.) and fully custom themes authored as CSS files. Themes are selectable from the Settings view and persist per-workspace or globally.

## Goals

1. Users can switch between built-in themes with one click
2. Users can author custom themes as `.css` files
3. Community themes are installable (future: registry)
4. Theme changes are instant (no rebuild, no restart)
5. Code editor (CodeMirror), terminal (xterm), and markdown editor (TipTap) respect the active theme
6. The settings page has a live theme picker with preview

## Non-Goals

- Per-view theming (all views share one theme)
- Runtime theme generation (themes are static CSS files)
- Theme marketplace (future work, not this spec)

## Architecture

### Token Cascade

```
1. packages/ui/src/tokens.css          ← canonical defaults (ship-light)
2. .ship/themes/<name>.css             ← project theme overrides (optional)
3. ~/.ship/themes/<name>.css           ← global theme overrides (optional)
4. User inline overrides via settings  ← highest precedence
```

Each layer overrides `:root` and `.dark` custom properties. Later layers win via CSS cascade (loaded after earlier ones).

### Theme File Format

A theme is a CSS file containing `:root` and/or `.dark` blocks that override design tokens:

```css
/* .ship/themes/gruvbox.css */
:root {
  --background: oklch(0.98 0.005 90);
  --foreground: oklch(0.25 0.02 60);
  --primary: oklch(0.65 0.15 60);
  --primary-foreground: oklch(0.98 0 0);
  --accent: oklch(0.70 0.14 145);
  --muted: oklch(0.92 0.01 90);
  --muted-foreground: oklch(0.55 0.03 60);
  --border: oklch(0.85 0.02 80);
  --card: oklch(0.96 0.005 85);
  --destructive: oklch(0.55 0.2 25);
  --success: oklch(0.60 0.18 145);
  --warning: oklch(0.70 0.15 80);
  --info: oklch(0.55 0.12 240);
  --font-body: "IBM Plex Sans", system-ui, sans-serif;
  --font-display: "IBM Plex Sans", system-ui, sans-serif;
  --font-mono: "IBM Plex Mono", ui-monospace, monospace;
  --radius-md: 4px;
}
.dark {
  --background: oklch(0.18 0.02 60);
  --foreground: oklch(0.87 0.02 80);
  --primary: oklch(0.72 0.13 60);
  /* ... */
}
```

### Theme Metadata

Each theme has a companion JSON sidecar (optional, for UI display):

```json
{
  "name": "Gruvbox",
  "author": "community",
  "description": "Retro groove color scheme",
  "preview": {
    "light": { "bg": "#fbf1c7", "fg": "#3c3836", "accent": "#b8bb26" },
    "dark": { "bg": "#282828", "fg": "#ebdbb2", "accent": "#b8bb26" }
  },
  "tags": ["retro", "warm", "community"]
}
```

### Built-in Themes

Ship ships with these themes embedded in the daemon binary:

| Theme | Style | Description |
|-------|-------|-------------|
| `ship-light` | Light | Default. Warm neutral with orange primary. Current tokens. |
| `ship-dark` | Dark | Default dark. Current `.dark` tokens. |
| `gruvbox` | Both | Retro groove. Warm earthy tones. |
| `nord` | Both | Arctic, cool blue-gray palette. |
| `catppuccin` | Both | Soothing pastel. Mocha/Latte variants. |
| `rose-pine` | Both | Elegant, muted with rose accents. |
| `tokyo-night` | Dark | Neon on dark. VS Code-inspired. |
| `solarized` | Both | Ethan Schoonover's classic. |
| `dracula` | Dark | Purple-heavy dark theme. |
| `one-dark` | Dark | Atom-inspired dark theme. |
| `github` | Both | GitHub's color scheme. |
| `high-contrast` | Both | WCAG AAA. Maximum readability. |

### Storage

Active theme is stored in Ship's settings system:

```
ship config set theme.name "gruvbox"
ship config set theme.mode "dark"        # light | dark | system
ship config set theme.custom_css ""      # inline overrides
```

The shell reads `localStorage('ship-shell-theme-name')` for instant apply. The daemon serves the resolved theme CSS.

### Daemon Endpoints

```
GET /api/runtime/assets/theme.css        → resolved theme CSS (base + active overrides)
GET /api/runtime/themes                  → list available themes [{name, meta}]
GET /api/runtime/themes/:name            → single theme CSS
PUT /api/runtime/settings                → save active theme selection
```

### Shell Integration

The shell loads theme CSS after base tokens:

```js
// Dynamic theme loading (already in index.html pattern)
const link = document.createElement('link');
link.rel = 'stylesheet';
link.href = '/api/runtime/assets/theme.css';
document.head.appendChild(link);
```

Theme switching is instant — swap the `<link>` href or replace the stylesheet content.

### Island Theme Sync

Code editor, terminal, and markdown editor need to sync with the active theme:

- **CodeMirror**: `ship-code-editor` reads CSS variables at mount time and builds a CM theme. On theme change, it re-reads and applies.
- **xterm.js**: `ship-terminal` builds theme from CSS variables via `buildTheme()` in `ship-terminal.theme.ts`. Already works — just needs to re-apply on theme change.
- **TipTap**: Styles via CSS variables already (shadow DOM inherits from host).

A `ship-theme-changed` custom event on `document` signals islands to refresh their internal themes.

### Settings UI

Add a "Theme" section to the settings view (above Providers):

```
┌─────────────────────────────────────────────────┐
│ 🎨 Theme                                        │
│                                                  │
│ Active theme: [Gruvbox          ▾]               │
│ Mode:         [● Light ○ Dark ○ System]          │
│                                                  │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│ │ Ship │ │Gruv  │ │ Nord │ │Capp  │ │Rosé  │   │
│ │      │ │      │ │      │ │      │ │      │   │
│ │  ✓   │ │      │ │      │ │      │ │      │   │
│ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
│                                                  │
│ Custom CSS overrides                    [Edit]   │
│ ┌──────────────────────────────────────────────┐ │
│ │ :root { --primary: oklch(0.5 0.2 280); }    │ │
│ └──────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

Each preset card shows a mini preview (4-5 color swatches). Clicking applies instantly. The custom CSS textarea allows inline overrides on top of any preset.

## Implementation Plan

### Phase 1: Theme infrastructure (daemon)
- [ ] Create `apps/shipd/src/themes.rs` — theme discovery, loading, resolution
- [ ] Scan `.ship/themes/` and `~/.ship/themes/` for `.css` files
- [ ] Embed built-in themes via `include_str!` (start with 4: ship, gruvbox, nord, catppuccin)
- [ ] `GET /api/runtime/assets/theme.css` endpoint — returns resolved theme
- [ ] `GET /api/runtime/themes` endpoint — lists available themes with metadata
- [ ] `PUT /api/settings` — persist theme selection

### Phase 2: Shell integration
- [ ] Load `/api/runtime/assets/theme.css` after `view-tokens.css` in index.html
- [ ] Theme switcher function in main.ts — swaps stylesheet, dispatches `ship-theme-changed`
- [ ] Persist selection in localStorage for instant apply before daemon responds

### Phase 3: Settings UI
- [ ] Add Theme section to settings view (above Providers)
- [ ] Theme preset cards with color swatch previews
- [ ] Light/Dark/System mode radio
- [ ] Custom CSS override textarea (optional, collapsible)
- [ ] Live preview — changes apply instantly on hover/click

### Phase 4: Island sync
- [ ] CodeMirror theme refresh on `ship-theme-changed`
- [ ] xterm.js theme refresh on `ship-theme-changed`
- [ ] Verify TipTap inherits via CSS variables (should work already)

### Phase 5: Community themes (future)
- [ ] `ship install theme <name>` CLI command
- [ ] Theme registry at registry.getship.dev
- [ ] Theme export (`ship theme export`)
- [ ] Theme sharing (`.ship/themes/` committed to repo)

## Token Reference

Themes can override any of these tokens:

### Colors
`--background`, `--foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--accent`, `--accent-foreground`, `--muted`, `--muted-foreground`, `--card`, `--card-foreground`, `--border`, `--ring`, `--destructive`, `--success`, `--warning`, `--info`, `--sidebar`, `--sidebar-foreground`, `--sidebar-accent`

### Typography
`--font-body`, `--font-display`, `--font-mono`, `--text-xs` through `--text-3xl`, `--font-weight-normal` through `--font-weight-bold`, `--leading-tight` through `--leading-relaxed`

### Spacing
`--space-1` through `--space-12`

### Radii
`--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl`, `--radius-full`

### Shadows
`--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-xl`

### Motion
`--duration-fast`, `--duration-normal`, `--duration-slow`, `--ease-out`, `--ease-in-out`

## Acceptance Criteria

1. `ship config set theme.name gruvbox` changes the active theme
2. All 9 views render correctly with gruvbox theme (no broken colors)
3. Settings page shows theme picker with at least 4 presets
4. Clicking a preset applies instantly (no page reload)
5. Dark mode toggle works independently of theme selection
6. Code editor, terminal, and markdown editor match the active theme
7. Custom CSS overrides in settings textarea take effect immediately
8. Theme persists across page refreshes and daemon restarts
9. `.ship/themes/custom.css` in a project is auto-discovered
