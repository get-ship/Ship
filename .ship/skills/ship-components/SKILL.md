---
name: ship-components
stable-id: ship-components
description: Author Ship components. Two stacks live here — Lit `<ship-*>` (current production) and SUI `<sui-*>` (in-flight, partial parity). Self-contained skill; deep references are siblings under references/.
tags: [ui, components, web-components, lit, sui, design-system, frontend]
authors: [ship]
audience: agent
---

# ship-components

Two stacks coexist:

| Stack | Status | Source |
|---|---|---|
| **Lit `<ship-*>`** | Current production | `packages/ship-set/src/web-components/` |
| **SUI `<sui-*>`** | In-flight, partial parity (~19 primitives) | `packages/ui-components/src/<tag>/` |

Both ship as real custom elements; both are composable in views (j2 or SUI). The choice is about which authoring shape you're working in.

## References

| When you're doing | Read |
|---|---|
| Authoring a new `<ship-*>` Lit component | `references/lit-charter.md` |
| Authoring or porting a `<sui-*>` SUI component | `references/sui-components.md` |
| Component-pack manifest, distribution, on-demand loading | `references/packs.md` |
| Composing components in a view | the `ship-views` skill |

## Hard rules (apply across both stacks)

- **`tone` for semantic color, `variant` for visual style.** `variant="destructive"` is wrong — destructive is a tone. (Same applies to SUI `<sui-*>` ports.)
- **Composition over reimplementation.** Any list row / tile / card uses `<ship-list-tile>` (slots: `leading`, default, `meta`, `trailing`). Don't ship a thin wrapper whose only value is filling slots with view-specific content.
- **Distribution test.** An element earns a spot in the library only if it's a semantic primitive, a platform concept, or has its own brain. View-local compositions live in `partials/`, not as components. Full rule in `references/lit-charter.md` § "The distribution test" — applies identically to SUI.
- **Icon-only buttons need `aria-label`.**
- **`<ship-select>`** uses native `<option>` (no `<ship-option>`). Emits `ship-change`, not `ship-select-change`.

## Mixing rule

SUI views can compose Lit `<ship-*>` until the SUI library has parity. Pragmatic — don't block on porting. When a primitive is missing in SUI, use the Lit version; flag for strategic port.

## Custom Elements Manifest

The live catalog of every `<ship-*>` element + its API is at `/api/assets/custom-elements.json` (machine-readable, served by the daemon). Use it to discover what exists before reaching for raw HTML.
