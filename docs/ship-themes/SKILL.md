---
name: ship-themes
stable-id: ship-themes
description: Reference library for Ship's design system — themes, tokens, and customization. Ship ships a full design system per theme (palette, surfaces, radii, shadows, motion, typography, interaction), not just a color palette. Load this when researching how theming works, which tokens exist, or how to customize Ship's look.
tags: [design-system, themes, tokens, ui, customization]
authors: [ship]
---

# Ship Design System

Ship themes are full aesthetic bundles, comparable in depth to Material or Apple HIG. Each theme bends surfaces, radii, shadows, motion, typography, and interaction strength — not just hue. Six themes ship today, selected from a flat picker in the shell topbar.

This doc-skill is the published reference. Pages:

- **[Design System](./references/docs/index.md)** — positioning and tour.
- **[Themes](./references/docs/themes.md)** — the six-theme catalog, what a theme changes, how to switch.
- **[Tokens](./references/docs/tokens.md)** — four-layer token system at an integrator level.
- **[Customizing](./references/docs/customizing.md)** — override individual tokens or author a full custom theme.
- **[Roadmap](./references/docs/roadmap.md)** — agent-only punch list of known gaps.

## See also

- `skills/ship-design-system/SKILL.md` — impl skill for authoring tokens/themes inside `packages/ship-set/` (anti-smells, tone-ramp rules, theme quality bar).
- `skills/ship-components/SKILL.md` — component-author charter; the "which token for which intent" table lives there.
