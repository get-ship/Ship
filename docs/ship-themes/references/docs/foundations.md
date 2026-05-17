---
group: Design System
title: Tokens
section: reference
order: 3
description: The four-layer token system that makes Ship themes coherent — surfaces, tone ramps, role tokens, primitives.
---

# Tokens

Every Ship token is a CSS custom property on `:root`, loaded from `packages/ui-components/src/tokens/tokens.css`. Themes override the same properties under `:root[data-theme='id']`. Components read `var(--foo)` with no fallback — the tokens are guaranteed present.


The token API is layered. A component or integrator reads the **highest layer that captures intent**; a theme overrides the **highest layer it needs to**.

## Layer 1 — Semantic surfaces and intents

For depth, interaction, and structure. Reach for these first.

| Intent | Token |
|---|---|
| Page background | `--surface-0` |
| Card surface | `--surface-1` |
| Nested panel / raised surface | `--surface-2` |
| Popover / overlay surface | `--surface-3` |
| Dialog / drawer backdrop | `--backdrop` |
| Light dim (under a drawer) | `--scrim` |
| Hover tint on an interactive row | `--hover-bg` |
| Pressed / active tint | `--active-bg` |
| Selected-row tint | `--selected-bg` |
| Selected-row accent border | `--selected-border` |
| Thin divider | `--divider` (alias of `--border-subtle`) |
| Hard boundary | `--border-strong` |

## Layer 2 — Tone ramps

For anything tinted by semantic intent. Eight tones × five ramp entries each:

- **Tones:** `default`, `muted`, `info`, `success`, `warning`, `destructive`, `accent`, `primary`.
- **Ramp entries per tone:** `-fg`, `-bg` (soft ~14%), `-bg-solid`, `-border`, `-ring`.

Example — a success chip reads four tokens, no custom color-mix:

```css
.chip-success {
  color: var(--tone-success-fg);
  background: var(--tone-success-bg);
  border: 1px solid var(--tone-success-border);
}
.chip-success:focus-visible {
  box-shadow: 0 0 0 3px var(--tone-success-ring);
}
```

Themes override the **base** (`--success`, `--warning`, `--info`, `--destructive`, `--accent`, `--primary`); the ramps rebuild automatically.

## Layer 3 — Role tokens

Theme-aware roles. These are the handles themes reach for when they want to reshape character without rewriting every component.

| Intent | Token |
|---|---|
| Heading / card header / dialog title family | `--heading-font` |
| Heading weight | `--heading-weight` |
| Heading tracking | `--heading-tracking` |
| Body text | `--body-font` |
| Monospace | `--mono-font` |
| Eyebrow label family | `--label-font` |
| Label weight / tracking | `--label-weight`, `--label-tracking` |
| Default expressive ease | `--ease-out` |
| Soft surface-change ease | `--ease-out-soft` |
| Tactile spring ease | `--ease-spring` |
| Row vertical padding (density-aware) | `--density-row-py` |
| Row horizontal padding | `--density-row-px` |
| Icon size (density-aware) | `--density-icon` |
| Resting card shadow | `--shadow-xs` |
| Interactive hover shadow | `--shadow-hover` |
| Popover / dropdown shadow | `--shadow-md` |
| Dialog shadow | `--shadow-lg` |
| Sticky header cue | `--shadow-sticky` |
| Primary-tinted focus glow | `--shadow-accent` |

**Density.** An independent axis. Setting `data-density="compact"` or `"spacious"` on `<html>` re-proportions every row-shaped component. The default is `comfortable`.

## Layer 4 — Primitives

Raw scales — reach only when the layers above don't fit.

| Scale | Tokens |
|---|---|
| Spacing (4px base) | `--space-1..8` |
| Radii | `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl`, `--radius-full` |
| Typography scale | `--text-xs..4xl` |
| Line height | `--leading-tight`, `--leading-normal`, `--leading-relaxed` |
| Font weight | `--font-weight-normal..bold` |
| Letter spacing | `--tracking-tight`, `--tracking-normal`, `--tracking-wide`, `--tracking-wider` |
| Duration | `--duration-instant`, `--duration-fast`, `--duration-medium`, `--duration-slow`, `--duration-deliberate` |
| Z-index | `--z-base`, `--z-elevated`, `--z-dropdown`, `--z-sticky`, `--z-popover`, `--z-dialog`, `--z-toast`, `--z-max` |

## Reduced motion

`@media (prefers-reduced-motion: reduce)` collapses every duration token to `0.01ms` and forces transitions to resolve immediately. Components that read the duration tokens inherit this for free.

## Where to go next

- Override individual tokens or author a full theme: [Customizing](./customizing).
- Component-author guidance — which token for which intent — lives in the impl skill at `.ship/skills/ship-components/SKILL.md`.
