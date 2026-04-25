---
group: Design System
title: Token stability
section: reference
order: 4
description: The v1 stability contract for Ship's CSS custom properties — which token families are public API and which are internal implementation detail.
---

# Token stability

Ship's design tokens live in `packages/ship-set/src/tokens.css`. Each token is split into one of two contracts:

- **`@stable`** — public v1 API. Renames within v1.x ship a deprecation alias kept for one minor. New tokens may be added at any time. Themes, third-party components, and user CSS may rely on these names.
- **`@internal`** — implementation detail of the default themes. May be renamed, retyped, or removed at any minor without notice. Override at your own risk.

The split is annotated inline in `tokens.css` so contributors see it next to the values, and the four-layer model (surfaces → tone ramps → role tokens → primitives) described in [Foundations](./foundations) maps cleanly onto it: every layer is `@stable` except where called out below.

## Stable token families

These are guaranteed within v1.x.

### Brand palette
`--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--success`, `--success-foreground`, `--warning`, `--warning-foreground`, `--info`, `--info-foreground`, `--border`, `--input`, `--ring`. Both light (`:root`) and dark (`.dark`) blocks are contract.

### Semantic surfaces
`--surface-0`, `--surface-1`, `--surface-2`, `--surface-3`, `--surface-inverse`, `--backdrop`, `--scrim`, `--border-subtle`, `--border-strong`, `--divider`, `--hover-bg`, `--active-bg`, `--selected-bg`, `--selected-border`.

### Tone ramps
For each tone in `{default, muted, info, success, warning, destructive, accent, primary}`: `--tone-{name}-fg`, `--tone-{name}-bg`, `--tone-{name}-bg-solid`, `--tone-{name}-border`, `--tone-{name}-ring`. Forty tokens total. New tones may be added; existing names frozen.

### Typography roles + scale
Roles: `--heading-font`, `--heading-weight`, `--heading-tracking`, `--body-font`, `--mono-font`, `--label-font`, `--label-weight`, `--label-tracking`, plus the resolved stacks `--font-display`, `--font-body`, `--font-mono`. Scale: `--text-xs..4xl`, `--leading-{tight,normal,relaxed}`, `--font-weight-{normal,medium,semibold,bold}`, `--tracking-{tight,normal,wide,wider}`.

### Spacing, radii, z-index
`--space-1..8`, `--radius`, `--radius-{sm,md,lg,xl,full}`, `--z-{base,elevated,dropdown,sticky,popover,dialog,toast,max}`.

### Motion + focus
`--ease-{out,out-soft,in,in-out,spring}`, `--duration-{instant,fast,medium,slow,deliberate}`, `--ring-width`, `--ring-offset`, `--ring-opacity`. The `prefers-reduced-motion` collapse is part of the contract.

### Density + elevation
Density: `:root[data-density={compact,comfortable,spacious}]` and the `--density-row-py`, `--density-row-px`, `--density-row-gap`, `--density-text`, `--density-icon` tokens. Elevation: `--shadow-{xs,sm,md,lg,xl}`, `--shadow-hover`, `--shadow-sticky`, `--shadow-accent`.

## Internal categories

Names exist in `tokens.css` today but carry **no compatibility promise**. Themes should override the stable token they back, not the internal name itself.

- **Status / data-viz ramp** — `--status-{gray,blue,red,green,yellow,purple,orange,amber}`. Will migrate to a dedicated `--chart-*` set before v1.0.
- **Sidebar slots** — `--sidebar`, `--sidebar-foreground`, `--sidebar-primary`, `--sidebar-primary-foreground`, `--sidebar-accent`, `--sidebar-accent-foreground`, `--sidebar-border`, `--sidebar-ring`. Likely to fold into `--surface-*`.
- **Interaction tints** — `--hover-strength`, `--active-strength`, `--selected-strength`. Knobs that feed `--hover-bg` etc.; the surface tokens are the contract, the percentages are not.
- **Tooltip delays** — `--tooltip-show-delay`, `--tooltip-hide-delay`. Read by the tooltip primitive only.
- **Legacy tone-mix percentages** — `--tone-bg-mix`, `--tone-border-mix`, `--tone-hover-mix`. Will be removed when `util.toneVars()` callers migrate to the named ramps.
- **Font-stack aliases** — `--font-display-app`, `--font-body-app`, `--font-mono-app`. Wiring detail; the unsuffixed roles are the contract.
- **`--ring-color`** — alias of `--primary`; subject to removal.

## Stability policy

1. **Stable tokens never rename within v1.x.** A rename ships a deprecation alias kept for at least one minor release.
2. **Additions are non-breaking.** New stable tokens may land in any minor; existing names + intent stay frozen.
3. **Internal tokens carry no promise.** They may be renamed, retyped, or removed at any minor without notice.
4. **Theme overrides target stable names.** A custom theme that only sets stable tokens is forward-compatible across all v1.x releases.
5. **Promotions are explicit.** Moving an `@internal` token to `@stable` requires updating both the inline annotation in `tokens.css` and this page.

This mirrors the lint-rule stability policy: same shape, same cadence, same guarantee.
