# @sail/ship-set — parallel ports

Selected ship-set components reimplemented as `.sail` files for direct
contrast with the Lit originals. Same visual + behavioral contract,
different framework.

## Why this exists

The pitch for Sail rests on agent-token efficiency. Words don't prove
that — fixed corpora compiled head-to-head do. This package is the
side-by-side version: each `.sail` here mirrors a real ship-set
component in `packages/ship-set/src/web-components/core/`, and the
tests exercise both ports against the same expectations.

## Components ported (so far)

| Component | Lit (TS+styles) | Sail (.sail) | Notes |
|-----------|-----------------|--------------|-------|
| `ship-badge` | 75 + 237 = **312 LOC** across 2 files | **141 LOC** in 1 file | -55% lines, single file |
| `ship-progress` | **96 LOC** in 1 file (CSS inline via `css\`\`\``) | **55 LOC** in 1 file | -43% lines, derived bindings replace render-method math |

Counts include doc comments. Both ports keep visual + ARIA parity with
the original; tests assert behavior, not internal structure.

## What sails-shaped felt like

**Wins:**
- One file. The Lit `ship-badge.ts` + `ship-badge.styles.ts` split is a
  bookkeeping artifact — Sail merges them.
- No `static styles = [reducedMotion, css\`...\`]` ceremony. Just
  `<style>...</style>`.
- No `static properties = { tone: { type: String, reflect: true } }`
  duplicated against `tone: Tone`. Just `prop tone: 'a' | 'b' = 'a'`.
- `derived pct = ...` is a one-liner. The Lit version computes `pct`
  inside `render()` every call; Sail memoizes.
- ARIA attribute interpolation (`aria-valuenow={ariaNow}`) is just an
  expression — no `${this.value != null ? this.value : nothing}`.
- Style scoping mostly works: `:host([tone='success'])` rewrites to
  `[data-component="ship-badge"][tone='success']`. No shadow root.

**Honest costs:**
- `prop value: number | null` doesn't work cleanly. The number decoder
  emits `NaN` for missing attributes, which authors must guard with
  `Number.isNaN(value())`. The Lit version uses `null` natively.
- `@keyframes indet { ... }` got its prefix prepended (`[data-component]
  @keyframes indet`) — the style scoper doesn't recognize `@keyframes`
  as a top-level at-rule. Real bug. CSS still works (browsers ignore
  the malformed rule and the inner `@keyframes` registers anyway), but
  it's a blocker for production. **Filed below as a known gap.**
- `:host(:has(::slotted(*)))` from the Lit badge styles isn't
  reproducible in light DOM — Sail components don't have `::slotted`.
  We dropped that selector; a future fix could detect filled slots via
  `:has(:not(:empty))` on the wrapper element.
- `ElementInternals` (form association on `ship-checkbox`) isn't
  reachable from the script block. Not implemented yet.
- Multi-line expressions in script declarations needed a parser fix:
  `derived pct = a\n? b\n: c` was being truncated at the first newline
  before this work. Caught by the progress port. Fixed.
- The reactive-identifier auto-call rule (`{count}` → `count()`) is
  load-bearing. When wrong, every interpolation silently shows
  `function ...` text instead of the value. Caught earlier in the
  counter integration test.

## Known gaps surfaced by this exercise

1. **`@keyframes` over-prefixed** in the style scoper. Should be passed
   through verbatim (it's a top-level at-rule, not a regular rule).
2. **`::slotted(...)` selectors** have no light-DOM equivalent in Sail.
3. **Form-associated custom elements** (form participation, validation)
   aren't reachable from `.sail` script blocks.
4. **Nullable prop types** decode awkwardly (`number | null` → `NaN`
   sentinel by default). Worth a dedicated `Nullable<T>` decoder.

## How to read the contrast

The Lit + Sail ports compile to functionally equivalent custom elements.
The Sail version is the OSS-portable, agent-tuned authoring path; the
Lit version is the existing production path. The shrinkage isn't
gratuitous — it's removing ceremony that doesn't carry meaning. The
costs above are real and the gaps are tracked.

When more ports land here, the next one to attempt is `ship-checkbox`
(form association) once Sail grows the `ElementInternals` hook, or
`ship-tabs` (multi-component coordination) once Sail's component
composition story is stress-tested.
