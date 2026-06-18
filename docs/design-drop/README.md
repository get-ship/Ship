# design-drop — archival reference bundle

This is the original design bundle from 2026-06-10. The bundle's `before/after`
pairs are exploratory sketches — the canonical implementations live in
`packages/components/sui/primitives/` and `packages/components/ship/*`. The
linter (`crates/sui/lint/`) is the source of truth for the design system
contract; see the `ship-design-system` skill and
`crates/sui/compiler/tokens.json`. This directory is reference material only;
it is not a planning document and is not the source of design intent.

> **Note:** The 2026-06-10 last-modified date on these files is from the
> original drop, not the current source. The local SUI and ship component
> packs in `packages/components/` have diverged significantly since this drop
> was created — anything stamped 2026-06-10 in this directory is historical,
> not current. Do not use it as a planning input. If you need the current
> component contract, read the source under `packages/components/sui/` and
> `packages/components/ship/`, or run the SUI compiler's lint rules.
