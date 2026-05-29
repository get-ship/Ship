---
name: ship-design-system
stable-id: ship-design-system
description: Charter for Ship's design-system layer — tokens, themes, surfaces, tone ramps, motion, density. Load this when authoring or reviewing changes to packages/components/ui-components/src/tokens/tokens.css or themes.css, adding a new theme, debugging why a component doesn't respond to a theme switch, or helping a user customize Ship's look beyond brand color.
tags: [design-system, tokens, themes, css, theming, frontend]
authors: [ship]
---

# Ship Design System

Ship ships a **full design system**, not a color palette.

A theme is not just a swap of `--primary`. A Ship theme can bend surface
depth, motion curves, radii, shadow vocabulary, typography role,
interaction strength, and tone ramps — and the component library
rides along without edit. The component charter (`ship-components`)
tells authors how to *consume* tokens; this document tells authors
how to *build* them.

Read this when you are:

- Adding or editing a theme in `packages/components/ui-components/src/tokens/themes.ts`
  or `themes.css`.
- Introducing a new token to `packages/components/ui-components/src/tokens/tokens.css`.
- Debugging a "why doesn't this component flip with the theme" bug.
- Helping a user customize Ship past the palette layer.

Do **not** read this to decide which token a component should consume.
That's `.ship/skills/ship-components/SKILL.md` → "Design tokens — read
the highest layer that fits."

## The four token layers

In order of abstraction (highest first). Components read the highest
layer that fits; themes override the highest layer they need to.

| Layer | Lives in | Examples | Override to change… |
|---|---|---|---|
| 1. Semantic surfaces & intents | tokens.css | `--surface-0..3`, `--backdrop`, `--hover-bg`, `--selected-bg`, `--divider` | Depth hierarchy, interaction feel |
| 2. Tone ramps | tokens.css (built from base) | `--tone-success-{fg,bg,bg-solid,border,ring}` for each of default/muted/info/success/warning/destructive/accent/primary | Semantic status color. Override base `--success`/`--warning`/etc. and the ramp rebuilds |
| 3. Role tokens | tokens.css | `--heading-font`, `--heading-weight`, `--body-font`, `--label-font`, `--ease-out`, `--density-row-py`, `--shadow-hover`, `--shadow-accent` | Typography commitment, motion character, density, semantic shadow intent |
| 4. Primitives | tokens.css | `--space-1..8`, `--radius-sm..xl`, `--text-xs..4xl`, `--duration-fast`, `--font-display` | Raw scale. Rarely the right override point. |

The canonical palette (`--background`, `--foreground`, `--card`,
`--primary`, `--accent`, `--muted`, `--border`, …) sits between layers 1
and 4. Themes almost always override it. Surfaces fall through to it
by default (`--surface-1: var(--card)`), so you can change a card color
once and lift it everywhere.

## The theme catalog

Ship ships six themes. Every one is a full aesthetic bundle — palette
+ surfaces + radii + shadows + motion + typography + interaction.
Comparable in scope to Material or Apple's HIG: a theme swap is a
different *feel*, not a different hue. There is no `palette`-only
tier, and the picker is flat.

- **`ship-light`** — canonical default. Warm off-white, SHIP amber,
  standard motion, soft shadows, display font for headings.
- **`ship-dark`** — sibling. Warm-navy background, warm amber
  primary. Same motion/radii/typography feel as ship-light.
- **`paper`** — editorial reading mode. Cream + terracotta, rounded
  radii (12–20px), slow calm motion (200/300/420ms, `ease-out-soft`),
  heading font demoted to body, low interaction strength (4–7%).
- **`halo`** — glassy atmospheric. Pale-blue surfaces with a
  primary-tinted nested layer, spring motion as the interaction
  default, snappy 100/150/200ms durations, primary-tinted
  `--shadow-accent` glow.
- **`console`** — 90s beige-box nostalgia. Tan/amber + teal, sharp
  radii (2/3/4/6), inset-style shadows with dark inner edges, mono
  label font, bold heading weight, chunky tints (12–18%).
- **`contrast`** — accessibility-hero. Near-black + fused amber
  primary/accent, pure white foreground, flat 1–2px border shadows,
  blocky tiny radii, bold weights, amped interaction (14/22/30%).

**When authoring a new theme, self-audit:** does it bend at minimum
the palette + one of (surfaces/motion/radii/shadows) + one of
(typography/interaction)? If not, it's a recolor pretending to be a
theme — bring it up to the quality bar or don't ship it.

## Authoring a theme

Two files. Always both.

### 1. Register in `themes.ts`

```ts
{
  id: 'midnight-lab',
  label: 'Midnight Lab',
  dark: true,               // applies .dark so shadow/elevation flip
  swatches: [
    'oklch(0.18 0.02 265)', // background
    'oklch(0.22 0.02 265)', // card
    'oklch(0.78 0.14 180)', // primary
    'oklch(0.68 0.20 320)', // accent
  ],
}
```

The `ThemeId` union must grow to include the new id. TS will complain
if you forget.

### 2. Add the CSS block in `themes.css`

Group the file the way it's already grouped: a comment block with the
theme's story, then a `:root[data-theme='…']` rule. Keep the order
*palette → surfaces → shadows → radii → motion → typography →
interaction* so diffs between themes are visually comparable.

**Palette skeleton.** Start here — the palette block every theme needs.
A real theme keeps going past this (surfaces, radii, motion, shadows,
typography, interaction) to earn its slot in the catalog:

```css
:root[data-theme='midnight-lab'] {
  --background: oklch(0.18 0.02 265);
  --foreground: oklch(0.96 0.01 260);
  --card: oklch(0.22 0.02 265);
  --card-foreground: oklch(0.96 0.01 260);
  --popover: oklch(0.22 0.02 265);
  --popover-foreground: oklch(0.96 0.01 260);
  --muted: oklch(0.26 0.02 265);
  --muted-foreground: oklch(0.76 0.02 260);
  --border: oklch(1 0 0 / 14%);
  --input: oklch(1 0 0 / 14%);
  --primary: oklch(0.78 0.14 180);
  --primary-foreground: oklch(0.15 0.02 265);
  --accent: oklch(0.68 0.20 320);
  --accent-foreground: oklch(0.98 0.01 320);
  --destructive: oklch(0.62 0.22 25);
  --success: oklch(0.72 0.16 150);
  --warning: oklch(0.78 0.15 80);
  --info: oklch(0.72 0.14 220);
  --ring: oklch(0.78 0.14 180 / 35%);
}
```

**Full example** — same palette, plus the bends that make it a theme:

```css
:root[data-theme='midnight-lab'] {
  /* palette — as above, omitted for brevity */

  /* Surfaces: a warmer nested panel instead of the default mix. */
  --surface-2: oklch(0.26 0.025 265);

  /* Shadows: cool-tinted, deeper blur for a laboratory-glass feel. */
  --shadow-sm: 0 1px 3px oklch(0.1 0.04 265 / 50%);
  --shadow-md: 0 6px 14px -4px oklch(0.1 0.04 265 / 55%);
  --shadow-lg: 0 14px 30px -6px oklch(0.1 0.04 265 / 60%);
  --shadow-accent: 0 8px 24px -6px oklch(0.78 0.14 180 / 40%);

  /* Radii: tighter, instrument-panel geometry. */
  --radius-sm: 3px;
  --radius-md: 4px;
  --radius-lg: 6px;
  --radius-xl: 10px;

  /* Motion: snappy, no spring. */
  --duration-fast: 100ms;
  --duration-medium: 150ms;
  --ease-out: cubic-bezier(0.2, 0.9, 0.3, 1);

  /* Typography: label tracking a touch wider for that schematic feel. */
  --label-tracking: 0.08em;

  /* Interaction: crisper selected state against deep blue. */
  --selected-strength: 20%;
}
```

### 3. Honour the `.dark` flag

Ship's elevation tokens (`--shadow-*`, `--backdrop`) have a dark-mode
pair under `.dark` in `tokens.css`. `applyTheme(id)` toggles that class
based on `Theme.dark`. If a theme is inherently dark but `dark: false`,
you get light-mode shadows on a dark background and everything looks
wrong. **Set `dark: true` for any theme whose `--background` is below
~0.4 L.**

Conversely, setting `dark: true` on a light theme will flip shadow
recipes you didn't want flipped. The flag is about the *elevation
story*, not a mood.

### 4. Override tone-ramp bases, not ramp entries

Tone ramps are built in `tokens.css` from `--success`, `--warning`,
`--destructive`, `--info`, `--accent`, `--primary`. Override those
base colors for your theme and the entire ramp rebuilds:

```css
/* In your theme block — rebuilds all five --tone-success-* entries. */
--success: oklch(0.72 0.16 150);
```

Do **not** write `--tone-success-bg: oklch(...)` in a theme. That
bypasses the ramp formula and will drift from `-border`, `-ring`,
`-fg`. The formula is canonical; override the input, not the output.

Special case: a theme whose background is so light the default 14% mix
produces a washed-out chip may override `--tone-bg-mix` (or the
interaction strengths). That's legitimate — tune the recipe, don't
bypass it.

## Motion curves and when to pick which

| Curve | Use for |
|---|---|
| `--ease-out` | Expressive default. Buttons, panels, most interactions. |
| `--ease-out-soft` | Surface fades, backdrop appearances, anything that should feel like a room lighting up. |
| `--ease-spring` | Tactile feedback: press-release on buttons, toggle flips, success confirmations. |
| `--ease-in` | User dismisses / removes. Rare. |
| `--ease-in-out` | Scroll-bound or utility motion where symmetry matters. |

A theme can remap: `paper` sets `--ease-out: var(--ease-out-soft)` to
globally soften motion character without touching individual
components. `halo` remaps it to `--ease-spring` for a tactile bounce.

Durations form a scale — `instant`/`fast`/`medium`/`slow`/`deliberate`.
`--duration-fast` is the default for hover/focus; `--duration-medium`
for layout changes; `--duration-slow`+ for deliberate, premium moments
(dialog open, toast slide). `prefers-reduced-motion` collapses all of
these to 0.01ms automatically.

## Density channel

Density is an independent axis from theme. Set on `<html>`:

```html
<html data-density="compact">     <!-- default is "comfortable" -->
<html data-density="spacious">
```

Density overrides the `--density-*` channel (`--density-row-py`,
`--density-row-px`, `--density-row-gap`, `--density-text`,
`--density-icon`). Every list-shaped component (`ship-list-tile`,
table rows, log lines) pulls padding and icon size from this channel
instead of the base `--space-*` scale. Result: one attribute on the
root element re-proportions every row in the app without a component
edit.

When authoring a new component that presents rows, **read from
`--density-*`, not `--space-3`.** If the size need is not row-shaped
(e.g. a button icon), read from the primitives.

## Consumer surfaces — applying a theme

The TS module at `packages/components/ui-components/src/tokens/themes.ts` is the canonical
entry point. Three functions:

```ts
import { applyTheme, persistTheme, readStoredTheme } from '@ship/set/themes'

// On load — read persisted choice and apply. Safe SSR (returns default).
const active = applyTheme(readStoredTheme())

// When user picks a new theme.
applyTheme('paper')
persistTheme('paper')
```

`applyTheme` sets `data-theme` on the root element and toggles the
canonical `.dark` class based on `Theme.dark`. It returns the effective
id (falls back to `DEFAULT_THEME` on invalid input) so you can sync
state.

The canonical UI pattern is the theme picker in the shell: a single
flat grid of theme cards, swatches visible on each. No tiers.

## Overriding individual tokens

End users and integrators don't have to fork a theme to tweak one
knob. Every Ship token is a CSS custom property at `:root`, so a
downstream stylesheet loaded *after* `themes.css` can override any
token:

```css
/* integrator-overrides.css */
:root {
  --radius-lg: 20px;
  --duration-fast: 220ms;
}
:root[data-theme='ship-light'] {
  --primary: oklch(0.68 0.18 40);
}
```

There's no formal override API yet — just the cascade. Keep this
minimal and unsurprising.

## Anti-smells

1. **Never** write `oklch(0 0 0 / 40%)` in a component. Use
   `var(--backdrop)` (or `--scrim` for a lighter dim).
2. **Never** write `color-mix(in oklch, var(--success) 14%, transparent)`
   in a component. Read `var(--tone-success-bg)`. The formula lives in
   tokens.css once.
3. **Never** write `rgba(0,0,0,0.08)` or hex anywhere in shipped CSS.
   Ship is OKLCH-native.
4. **Never** reach for `var(--card)` when `var(--surface-1)` captures
   intent. Cards consume the surface layer so a theme that raises card
   depth (a gradient background, a tinted nested panel) only touches
   `--surface-1` / `--surface-2`.
5. **Never** hardcode `"Inter"` or `"Syne"` in a component. Read the
   role token (`--heading-font`, `--body-font`, `--label-font`, `--mono-font`).
   A theme like `paper` demotes display typography by remapping
   `--heading-font: var(--font-body)` — components that hardcode the
   family defeat that.
6. **Never** ship a `--tone-foo-bg: …` theme override. Override the
   tone base (`--foo`) and let the ramp rebuild.
7. **Never** read a theme's `--primary` directly for a status-like
   intent. Use the tone ramps. `--primary` is a brand slot, not a
   status signal.
8. **Never** add `var(--foo, fallback)`. Tokens are guaranteed loaded
   (tokens.css is imported at the document root). Fallbacks create a
   parallel, stale system.

## Agent surfaces

Three MCP surfaces are relevant when working with tokens / themes:

| Tool | When |
|---|---|
| `TODO(#207-followup): verify replacement for mcp__ship__view_theme_tokens` | Read the **active** theme's resolved tokens. Use before you author CSS in a component so you stay token-locked instead of guessing. Returns `{ --background, --foreground, --card, --ring, --tone-*-bg, … }`. |
| `ship sui diagnostics <file>` | After editing a `.sui` template that reads tokens via `var(--…)`, run diagnostics to confirm no broken refs. |
| `ship://components` (resource) | Component catalog includes `cssProperties` per primitive — what CSS custom properties each component accepts as design-token override points. |

Full agent surface inventory: `docs/ship-sui/references/docs/agent-surfaces.md`.

## Relationship to `ship-components`

Two skills, one boundary:

- **`ship-design-system` (this)** — authoring tokens and themes.
  Token names, layer rules, override points, theme catalogue.
- **`ship-components`** — authoring `<ship-*>` elements that consume
  tokens. Component layout, distribution test, shadow DOM conventions,
  the "which token for which intent" table.

If you're editing `tokens.css`, `themes.css`, `themes.ts`, or a
consumer that applies a theme (shell theme picker, marketing site) →
this skill. If you're editing
`packages/components/ui-components/src/sui-*/**` → `ship-components`.

## Open gaps

Honest inventory of places the design system is not yet cohesive,
surfaced here so no one re-invents a workaround:

- **Motion is not themable at the curve level everywhere.** `--ease-out`
  is overridable, but individual components sometimes hardcode a
  specific cubic-bezier in their `css` templates. Grep for
  `cubic-bezier` under `packages/components/ui-components/src/sui-*/` to
  find the stragglers; they should all go through the role tokens.
- **`--tone-bg-mix` / `--tone-border-mix` / `--tone-hover-mix` are
  legacy.** They're still exposed for `util.toneVars()` but new code
  reads the ramp variables directly. The legacy names can retire once
  every consumer is migrated.
- **Sidebar palette is a parallel mini-system.** `--sidebar-*` tokens
  predate the surface layer and duplicate what `--surface-*` could
  express. Kept for Studio compat; a future pass folds them into the
  surface stack.
- **`--status-*` chart colors are not tone-ramped.** They expose only
  the base color; a theme that wants stronger data-viz contrast has to
  mix in component CSS. A `--status-{name}-{fg,bg,border}` ramp would
  close this.
- **Density only affects rows.** A density channel for controls
  (button height, input height) would let `compact` truly tighten a
  dashboard. Today `--density-icon` covers icons but not control
  heights.
- **No design-system linter.** The anti-smells above are enforced by
  review, not by a rule. A `stylelint` plugin that flags raw
  `rgba(`, `oklch(0 0 0 / `, and `cubic-bezier(` inside
  `packages/components/ui-components/src/sui-*/**` would turn this section
  into a CI check.

## Token schema (planned)

A machine-readable `tokens.json` exporter doesn't ship yet. The
**canonical source** today is the CSS in
`packages/components/ui-components/src/tokens/tokens.css` itself — every token
declared at `:root`, every override per `[data-theme='…']`, and the
dynamic `--tone-*` contract generated by `util.toneVars()` in the
themes module.

When the exporter lands it will:
- Emit one entry per token with its `layer`, `intent`, `type`,
  `default`, and `themable` flag.
- Drive Figma token sync + IDE autocomplete + design-asset exporters.
- Power a `bun run lint:tokens` drift checker (fails on hardcoded
  `rgba`/`oklch`/hex inside component CSS, on `var(--X)` references
  for unknown tokens, warns on `color-mix(... var(--tone) 14%, …)`
  formulae that should read the ramp tokens instead).

Until then, drift is caught visually via `ship-ui-studio` and the
Playwright baselines. If you add a token, eyeball every theme in the
catalog with the new token in play before committing.

Type vocabulary (today, encoded only in the CSS):

| type | example token | notes |
|---|---|---|
| `color` | `--primary`, `--tone-success-bg` | OKLCH or an alias. |
| `radius` | `--radius-lg` | Corner radius. |
| `space` | `--space-4`, `--density-row-py` | Padding/gap scalar. |
| `duration` | `--duration-fast` | Motion duration. |
| `easing` | `--ease-out` | Cubic-bezier role. |
| `font` | `--heading-font` | Font stack or alias. |
| `font-weight` | `--font-weight-semibold` | Numeric weight. |
| `letter-spacing` | `--tracking-wide` | Tracking value. |
| `shadow` | `--shadow-md`, `--shadow-accent` | Box-shadow composite. |
| `z-index` | `--z-dialog` | Layer index. |
| `size` | `--text-lg`, `--density-icon` | Length (px / rem). |
| `percent` | `--hover-strength` | Percentage value. |
| `number` | `--leading-normal` | Unitless number. |
