---
group: Design System
title: Customizing
section: reference
order: 4
description: Two ways to customize Ship's look — override individual CSS tokens, or author a full custom theme.
---

# Customizing

There are two paths, in increasing scope:

1. **Override individual tokens** — a downstream stylesheet that tweaks specific CSS custom properties. Zero-API, works today.
2. **Author a full custom theme** — register a new entry in the theme catalog and write a `:root[data-theme='id']` block.

## Path 1 — Override individual tokens

Every Ship token is a CSS custom property at `:root`. A stylesheet loaded *after* `themes.css` can override any token without touching the design system source.

Global override (applies under every theme):

```css
/* integrator-overrides.css — loaded after @ship/ui-components styles */
:root {
  --radius-lg: 20px;
  --duration-fast: 220ms;
}
```

Theme-scoped override (applies only when that theme is active):

```css
:root[data-theme='ship-light'] {
  --primary: oklch(0.68 0.18 40);
}
```

Use this for small corrections — a sharper brand hue, rounder cards, a slower hover — without forking a theme. Keep these minimal; the more tokens you override, the more you are really authoring a theme.

**Rule:** override the **input** to a ramp, not the ramp entries themselves. Override `--success`, not `--tone-success-bg`. The ramp formula is canonical; bypassing it makes `-border`, `-ring`, `-fg` drift.

## Path 2 — Author a full custom theme

Five steps. Two files.

### 1. Pick a palette in OKLCH

Ship is OKLCH-native. No hex, no rgba. At minimum define `--background`, `--foreground`, `--card`, `--primary`, `--accent`, `--muted`, `--border`, and the status bases (`--success`, `--warning`, `--info`, `--destructive`).

### 2. Register the theme in `packages/ui-components/src/tokens/themes.ts`

Add an entry to the `THEMES` array and widen the `ThemeId` union. Swatches are the four squares the picker card shows — background, card, primary, accent.

```ts
{
  id: 'midnight-lab',
  label: 'Midnight Lab',
  dark: true,
  swatches: [
    'oklch(0.18 0.02 265)',
    'oklch(0.22 0.02 265)',
    'oklch(0.78 0.14 180)',
    'oklch(0.68 0.20 320)',
  ],
}
```

Set `dark: true` for any theme whose `--background` sits below ~0.4 L. The flag toggles the canonical `.dark` class, which flips elevation and backdrop recipes. Wrong flag → wrong shadows.

### 3. Add a `:root[data-theme='id']` block in `themes.css`

Group the block in the order the rest of the file uses: **palette → surfaces → shadows → radii → motion → typography → interaction**. That makes diffs between themes visually comparable.

```css
:root[data-theme='midnight-lab'] {
  /* palette */
  --background: oklch(0.18 0.02 265);
  --foreground: oklch(0.96 0.01 260);
  --card: oklch(0.22 0.02 265);
  --primary: oklch(0.78 0.14 180);
  --accent: oklch(0.68 0.20 320);
  --success: oklch(0.72 0.16 150);
  --warning: oklch(0.78 0.15 80);
  --info: oklch(0.72 0.14 220);
  --destructive: oklch(0.62 0.22 25);
  --ring: oklch(0.78 0.14 180 / 35%);

  /* surfaces */
  --surface-2: oklch(0.26 0.025 265);

  /* shadows — cool-tinted, instrument-glass feel */
  --shadow-sm: 0 1px 3px oklch(0.1 0.04 265 / 50%);
  --shadow-md: 0 6px 14px -4px oklch(0.1 0.04 265 / 55%);
  --shadow-accent: 0 8px 24px -6px oklch(0.78 0.14 180 / 40%);

  /* radii — tighter, panel geometry */
  --radius-md: 4px;
  --radius-lg: 6px;

  /* motion — snappy, no spring */
  --duration-fast: 100ms;
  --duration-medium: 150ms;

  /* interaction — crisper against deep blue */
  --selected-strength: 20%;
}
```

### 4. Override the layers that make the theme a theme

A theme that only bends the palette is a recolor. To earn a slot in the catalog, bend **at minimum the palette + one of (surfaces/motion/radii/shadows) + one of (typography/interaction)**. The built-in themes each do more.

### 5. Rebuild and verify

Rebuild the `@ship/ui-components` package so the `ThemeId` union and theme export refresh, then reload the shell. The new theme appears in the picker automatically — the UI is driven off the `THEMES` array.

## Going deeper

The authoring rules in full — tone-ramp mechanics, motion curve guidance, density conventions, anti-smells — live in the impl skill at `.ship/skills/ship-design-system/SKILL.md`. Read it before submitting a theme for inclusion.
