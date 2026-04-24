---
group: Design System
title: Design System
section: reference
order: 1
description: Ship's themes are full aesthetic bundles — palette, surfaces, radii, shadows, motion, typography, and interaction — not recolors.
---

# Design System

Ship ships a **full design system**, not a color palette. A theme here is not a swap of `--primary`; it is a coordinated change to surface depth, corner geometry, motion curves, shadow vocabulary, typography role, and interaction strength. The component library rides along without edit.

The scope is comparable to Material or Apple's Human Interface Guidelines: switching themes changes how the app *feels* within a second, not only how it looks.

## What this reference covers

- **[Themes](./themes)** — the six built-in themes, the dimensions a theme changes, and how users pick one.
- **[Tokens](./tokens)** — the four-layer token system that makes theming cohesive: semantic surfaces, tone ramps, role tokens, primitives.
- **[Customizing](./customizing)** — two paths: override individual CSS custom properties, or author a full custom theme.

If you are building `<ship-*>` components or editing `tokens.css` / `themes.css` directly, also read the impl skill at `.ship/skills/ship-design-system/SKILL.md` for the authoring rules and anti-smells.
