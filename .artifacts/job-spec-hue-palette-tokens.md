# Job Spec: hue palette tokens — register `--hue-*` + propagate to design system

**Branch:** `job/hue-palette-tokens`
**Agent:** ship-designer
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/agent-detail.html` + `agents-list.html` (consume `--hue-<name>` tokens)

## Goal

Register a canonical 16-hue palette as `--hue-<name>` tokens in `tokens.css` + `tokens.json`. The agent-detail keystone and agents-list polish both consume per-agent hue tiles (red, orange, amber, yellow, lime, green, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose). Today these aren't centralized — each view would re-define them, drifting over time. Establish them once and forever.

Tiny mechanical job. Single commit reasonable. Precursor for the polish jobs to consume cleanly without inventing local hue palettes.

## File scope

```
packages/components/ui-components/src/tokens/tokens.css     # add the 16 --hue-* declarations
crates/sui/compiler/tokens.json                              # register names in the lint registry
```

DO NOT touch:
- Any view, app, or primitive.
- `tokens/themes.css` (themes can override these later if needed, but the base palette is theme-neutral).

## What to change

### Palette

OKLCH values for each hue. Tune for: medium-saturation, medium-lightness, distinct from neighbors. These are the **identity colors** — for agent hue tiles, project hue dots, profile-color affordances. They render on white (`var(--background)` in ship-light) AND on dark backgrounds. The white text + drop-shadow recipe in `agent-detail.html`'s `.detail-tile` is the visual reference.

Suggested values (tune freely — the constraint is "16 distinct identifiable hues, all readable with white text"):

```css
--hue-red:      oklch(0.62 0.21 25);
--hue-orange:   oklch(0.68 0.18 50);
--hue-amber:    oklch(0.74 0.16 75);
--hue-yellow:   oklch(0.82 0.16 95);
--hue-lime:     oklch(0.78 0.18 130);
--hue-green:    oklch(0.62 0.16 150);
--hue-teal:     oklch(0.62 0.13 185);
--hue-cyan:     oklch(0.68 0.13 220);
--hue-sky:      oklch(0.65 0.16 235);
--hue-blue:     oklch(0.55 0.20 260);
--hue-indigo:   oklch(0.50 0.20 280);
--hue-violet:   oklch(0.55 0.22 295);
--hue-purple:   oklch(0.55 0.22 310);
--hue-fuchsia:  oklch(0.60 0.25 330);
--hue-pink:     oklch(0.68 0.20 345);
--hue-rose:     oklch(0.62 0.21 5);
```

Verify by eye: open `packages/components/ui-components/src/tokens/tokens.css` in a browser preview if possible, render swatches, check yellow isn't too pale + indigo isn't too dark + adjacent hues are distinguishable.

### Token registry

Add to `crates/sui/compiler/tokens.json`. Likely a new `"hues"` category or extend the existing `"colors"` category — match the existing structure. The lint rule needs to recognize `var(--hue-red)` etc. as valid tokens.

### Theme overrides (optional, follow-up)

Themes that want a tuned palette (e.g., `console` for the cyan-monochrome aesthetic) can override individual hues. Out of scope for this job — just register the base palette.

## Architectural context

- Token system: established. New tokens follow existing patterns.
- Lint: `sui-css-token` rule enforces all `var(--X)` references resolve to known tokens. Adding to `tokens.json` lets the polish jobs consume them without lint errors.
- File cap: trivial. Just additions.

## Acceptance criteria

1. 16 `--hue-*` declarations in `tokens.css` `:root` block (or under the appropriate scope).
2. 16 hue names registered in `crates/sui/compiler/tokens.json`.
3. `cargo build -p sui-compiler` succeeds.
4. `cargo test -p sui-lint` passes (token registration is enforced by tests).
5. `bun packages/deck/bundler/index.ts --all` succeeds — no token-lint errors.
6. Visual swatch verification: a quick smoke test view (or `.html` file in `.artifacts/`) rendering 16 swatches with their names, so the team can eyeball them. Drop this file in `.artifacts/hue-palette-swatches.html` for inspection — not a permanent surface.

## Out of scope

- Per-theme hue overrides.
- Consumer changes — agent-detail-keystone and agents-list-polish will update on their own to consume `var(--hue-*)`.
- A picker UI for choosing hues (lives in project-creation flow, separate later).

---

> Execute this job yourself. Tiny, mechanical. Probably one commit.
>
> When done: complete_workspace + mesh_broadcast.
