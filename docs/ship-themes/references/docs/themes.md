---
group: Design System
title: Themes
section: reference
order: 2
description: The six built-in Ship themes, the dimensions each one bends, and how to switch between them.
---

# Themes

Ship ships six themes. Every one is a full aesthetic — palette, surfaces, radii, shadows, motion, typography, and interaction strength. The picker is flat: no tiers, no palette-only tier. You pick one and the app feels different.

## The catalog

| Theme | Label | Feel | Good for |
|---|---|---|---|
| `ship-light` | SHIP Light | Canonical. Warm off-white, amber primary, soft shadows, display font for headings, balanced motion. | Default, day-to-day work. |
| `ship-dark` | SHIP Dark | Sibling to ship-light. Warm-navy background with the same amber warmth and identical motion/radii. | Same feel after sundown. |
| `paper` | Paper | Editorial cream + terracotta. Rounded radii (12–20px), slow calm motion (200/300/420ms, `ease-out-soft`), display font demoted to body, low tints (4–10%). | Long reading, writing, docs-heavy sessions. |
| `halo` | Halo | Glassy pale-blue with a primary-tinted nested surface, spring motion as the interaction default, snappy 100/150/200ms durations, primary-tinted accent glow. | Fast UI work, premium feel. |
| `console` | Console | 90s beige-box nostalgia. Tan/amber + teal accent, sharp radii (2/3/4/6px), inset-style shadows in lieu of blur, mono-forward labels, bold heading weight, chunky 12/18/22% tints. | Terminal-native users, tactile affection. |
| `contrast` | High Contrast | Accessibility-hero. Near-black background, amber fused as both primary and accent, pure white foreground, flat 1–2px border "shadows", blocky tiny radii, bold weights, amped 14/22/30% interaction tints. | Low-vision needs, presentation mode. |

## What a theme changes

Every theme may bend the dimensions below. This is why a theme swap is a different *feel*, not just a recolor.

| Dimension | What it controls | Example override |
|---|---|---|
| Palette | `--background`, `--foreground`, `--card`, `--primary`, `--accent`, `--muted`, `--border`, status bases (`--success`, `--warning`, `--info`, `--destructive`). | Every theme. |
| Surface depth | The `--surface-0..3` stack — how nested panels separate from the page. | `halo` adds a primary-tinted `--surface-2` for its glass layer. |
| Radii | `--radius-sm..xl`. | `console` goes sharp (2/3/4/6px); `paper` goes round (6/12/16/20px). |
| Motion | `--ease-out`, `--duration-fast..deliberate`. A theme can remap the default ease entirely. | `paper` maps `--ease-out` to `--ease-out-soft`; `halo` maps it to `--ease-spring`. |
| Shadows | The `--shadow-xs..xl` ramp plus semantic shadows (`--shadow-hover`, `--shadow-sticky`, `--shadow-accent`). | `console` replaces blurs with inset chrome; `contrast` uses 1–2px rings. |
| Typography roles | `--heading-font`, `--heading-weight`, `--label-font`, `--label-weight`, tracking. | `paper` demotes display to body; `console` moves labels to mono-bold. |
| Interaction strength | `--hover-strength`, `--active-strength`, `--selected-strength` — the tint percentages used across interactive states. | `paper` relaxes (4/7/10%); `contrast` amps (14/22/30%). |
| Tone ramps | Status tints rebuild from the base `--success`/`--warning`/`--info`/`--destructive` colors, so theme-appropriate chip/alert colors follow automatically. | Every theme overrides the bases. |

## Using themes

The canonical UI is the **theme picker** in the shell topbar.

- **Opening** — click the palette icon in the top-right of the shell chrome. The picker is a single flat grid of theme cards with live swatches (background, card, primary, accent) on each.
- **Selecting** — click a card. The theme applies immediately; `data-theme` on `<html>` swaps, the canonical `.dark` class toggles for dark-aesthetic themes, and a `ship-theme-change` event is dispatched so other UI (status bar, host surfaces) can follow.
- **Persistence** — the choice is saved to `localStorage` under `ship-site-theme` and re-applied on reload. There is no server-side sync yet: each browser remembers its own selection.
- **Scope** — per-browser, per-origin. Clearing site data resets to `ship-light` (the default).

If you are embedding Ship or wiring a custom host, drop a `<sui-theme-picker>` (or `<sui-theme-toggle>`) into the page chrome — the primitive owns load / persist / apply and dispatches `sui-theme-change` on the root so other UI can react.

> **TODO: SUI equivalent pending.** The previous `applyTheme` / `persistTheme` / `readStoredTheme` functions lived on the legacy ship-set themes module. A public TS surface for the same primitives on `@ship/ui-components` has not landed yet; until it does, embed via the components above and listen for `sui-theme-change` events. Track the gap before consuming this API in integrator code.
