---
title: Design System Roadmap
audience: agent-only
description: Honest punch-list of known gaps in Ship's design system. Agent-only; not published on the public site.
---

# Design System Roadmap

Known gaps in the design system today. Captured so no one re-invents a workaround. Each item: what's broken or missing, what fixing it looks like, and whether it blocks shipping.

## 1. Islands don't subscribe to `ship-theme-change` — priority

`ship-code-editor`, `ship-terminal`, and `ship-markdown-editor` each wrap a heavy third-party engine (Monaco, xterm, tiptap) with its own internal color palette. When the shell dispatches `ship-theme-change`, these islands do not re-read tokens or rebuild their engine themes — the inner palette stays stale until the view is remounted. **Fix:** have each island subscribe to the `ship-theme-change` event on `document` and call its engine's theme-rebuild API (Monaco `editor.defineTheme` + `editor.setTheme`, xterm `options.theme`, tiptap CSS token re-read). Priority because it's the most visible stale-state bug during a theme swap demo.

## 2. Density channel has no consumers — non-blocking

The `--density-*` token channel (`--density-row-py`, `--density-row-px`, `--density-row-gap`, `--density-text`, `--density-icon`) is structurally live: `tokens.css` exposes defaults and `:root[data-density='compact'|'spacious']` overrides them. But no shipped component reads from the channel yet, so toggling `data-density` on `<html>` currently has no visible effect. **Fix:** migrate `ship-list-tile` first — it's the canonical row primitive and every other row/tile in the library composes through it. Once the tile reads `--density-row-py/-px` and `--density-icon` instead of the base `--space-*` scale, density immediately propagates to every list in the app. Non-blocking; density is an opt-in power-user feature.

## 3. `ship-theme-toggle` is dead code — non-blocking

`ship-theme-toggle` (a simple light/dark flipper) predates the theme picker. The picker is now the canonical UI and covers all six themes flatly; a two-state toggle is both redundant and misleading (it can't represent `paper` vs `halo` vs `console`). **Fix:** delete the component, or repurpose it as a one-click shortcut that flips between the user's last light theme and last dark theme (a preset wrapper over the picker). Non-blocking — the component is not wired into the shell.

## 4. Theme cards nest awkwardly in the picker popover — non-blocking

`console` and `contrast` theme cards render inside the rounded, soft-shadowed picker popover. Their own aesthetics (sharp radii, flat or inset shadows) clash with the popover chrome — the cards look like mis-registered previews rather than representative swatches. **Fix:** either anchor theme-card chrome to each theme's own `--radius-*` / `--shadow-*` tokens (so the card visually previews the theme), or accept the current behavior (the card is a swatch, not a preview). Non-blocking; cosmetic.

## 5. Status bar paints from `--card` — non-blocking

The shell status bar reads `var(--card)` directly for its background. In warm themes (`paper`, `console`) the pure card color looks off against the rest of the bottom chrome — too bright, too flat. **Fix:** promote the status bar to read from `--surface-2` (the nested/raised surface token), or add a tiny `--status-bar-bg` role token and let each theme tune it. Non-blocking.

## 6. Theme persistence is per-browser — non-blocking

`persistTheme` writes `localStorage['ship-site-theme']`. There is no cross-device or daemon-side sync: opening Ship in a second browser always starts at `ship-light`. **Fix:** wire the theme through daemon settings (`GET /api/runtime/settings/theme`, `PUT …`) with localStorage as the offline fallback. Trivial now since theme is the only setting of its kind. Non-blocking until the multi-device story becomes a priority.
