---
title: "Ship Component Architecture — the sui / ship tiers"
description: "ADR: component packs split into two tiers. sui-* is the framework's reusable, dependency-pure, atomic primitive library; ship-* is everything built from it — features, verticals, vendored-engine wrappers, internal/unpublished, and domain-coupled packs. A pack is sui only if it passes every sui gate; ship is the default."
group: ADRs
order: 50
audience: internal
status: proposed
date: 2026-06-02
tags: [components, sui, ship, packs, boundary, architecture]
---

# Ship Component Architecture — the `sui` / `ship` tiers

## Context

Ship ships UI as component packs under `packages/components/{sui,ship}/`, each a
set of `.sui` + optional `.controller.ts` web components. The split between the
two tiers had been applied by feel, and drifted: editors that wrap CodeMirror /
Tiptap sat unpackaged in `apps/shell`; a dependency-pure game pack (`arcade`)
was alternately argued into both tiers; renderers and their dispatcher lived in
one pack; the `@ship/sui-*` naming made the `@ship/` npm scope look like the
tier marker when it is just the org scope.

The recurring confusion was **conflating two independent questions**: *is it
dependency-pure?* and *is it a reusable primitive or a feature?* Purity is the
only thing the boundary lint (`enforce SUI/Deck boundary invariants`) checks,
and treating "pure ⟹ sui" is wrong — `arcade` is pure and is not `sui`.

## Decision

Component packs occupy one of two tiers. The tier is encoded three ways, which
must agree: the **package name** (`@ship/sui-*` = framework, `@ship/*` without
`sui-` = feature), the **directory** (`packages/components/sui/` vs
`packages/components/ship/`), and the **tag prefix** (`sui-*` vs `ship-*`). The
`@ship/` npm scope is the org and carries no tier meaning.

**`sui` — the framework's standard library.** Generic, atomic, domain-agnostic
primitives any third-party SUI app would reuse verbatim. The atomic units of
composition. A pack is `sui` **only if it is all of**:

1. **SDK-pure** — no `@ship/sdk-client`, no `ship.*`, no daemon/deck/`/_sui` coupling.
2. **Atomic / single-concern** — one composable building block, not a bundle.
3. **Generic / reusable** — useful outside Ship, in a stranger's app.
4. **Our own engine** — does not *wrap* a heavyweight external engine (see below).
5. **Published** — part of a published pack, not an internal/private surface.

**`ship` — everything built from `sui`.** The default tier. A pack is `ship` if
it trips **any** of these:

- **Domain coupling** — touches a Ship domain / the SDK (`agents`, `pipelines`, `authority`).
- **Vendored engine** — *wraps* an external engine that is fundamentally the
  component (`code` ← CodeMirror, `markdown` ← Tiptap, `terminal` ← xterm,
  `arcade` ← PixiJS, a PDF viewer ← pdf.js, a map viewer ← maplibre).
- **Feature / vertical** — a composed surface with a product identity, even if
  pure (`arcade` the game vertical, the `code` dev workspace, the `viewer` router).
- **Internal / unpublished** — a Ship-internal surface not shipped as a pack.
- **Shared-across-`ship`** — a component several `ship` packs reuse is pulled up
  into a shared `ship` component (not promoted to `sui` — if it wraps a vendor or
  isn't atomic it can't be `sui` anyway).

> **The law:** `sui` has to earn it; `ship` is the default. `sui-*` are the
> atomic units of composition; `ship-*` is everything assembled from them — and
> over time `ship-*` is the larger surface.

### The decision test (for authors)

1. Imports `ship.*`/SDK or models a Ship domain? → **ship** (only hard signal).
2. *Wraps* an external engine that *is* the component? → **ship**.
3. A composed surface / feature / vertical with a product identity? → **ship**.
4. Internal / unpublished? → **ship**.
5. Otherwise — one atomic, generic, own-engine, published primitive? → **sui**.

### Engine vs helper (the vendoring nuance)

Vendoring pushes to `ship` only when the external lib **is** the component's
engine — the component is a thin SUI shell around it (xterm/CodeMirror/Tiptap/
pixi/pdf.js). Vendoring a **pure helper** as one internal ingredient, while the
engine is ours, keeps a pack `sui`: `sui-draw` (perfect-freehand/roughjs stroke
math), `sui-sheet` (a formula evaluator), `sui-board` (dagre layout) — our
engines call vendored algorithms, but the architecture and identity are ours.
The test: *"is this a shell around someone else's engine, or our engine that
calls a vendored function?"*

### Worked classification

| `sui` (standard library) | `ship` (features / verticals) | Why `ship` |
|---|---|---|
| primitives, table, tree, calendar, charts | agents · pipelines · authority | domain |
| canvas / board / draw (own engines; vendor *helpers*) | code (CodeMirror) · markdown (Tiptap) · terminal (xterm) | vendored engine |
| chat-elements; media renderers (image/av/csv); json-view, diff | arcade (PixiJS + a game vertical) | engine + feature |
| component-viewer tooling | viewer (the mime *router* — a capability); pdf (pdf.js), map (maplibre) renderers | feature / vendored engine |

The viewer family makes the split concrete: the **router** (`ship-viewer`) is a
`ship` capability; **own-rendering leaves** (image/av/csv) are `sui-media`;
**vendored-engine leaves** (pdf, map) are `ship` renderers the router dispatches to.

## Consequences

### Positive
- One unambiguous test; the tier stops drifting.
- The boundary lint's purity rule is correctly understood as *necessary, not
  sufficient* for `sui`.
- `ship` absorbs the messy reality (vendors, verticals, internal, domain) so
  `sui` stays a clean, publishable, atomic library.

### Negative
- A generic-but-vendored primitive (a bare terminal/code editor) is `ship`, not
  `sui`, even though a stranger might want it standalone. Accepted: we keep one
  home per component; if a non-feature consumer ever needs it alone, it becomes a
  shared `ship` component, not a `sui` promotion.
- Name/dir/tag must be kept in agreement by hand until the lint below lands.

## Migration
- Editors lifted from `apps/shell` → `@ship/code` (+ terminal, git-diff) and
  `@ship/markdown`; tags retagged `sui-*`→`ship-*`.
- Viewers split: `@ship/sui-media` (own-rendering leaves) + `@ship/viewer` (router).
- `arcade` stays `@ship/arcade` (correct tier); its tags retag `sui-*`→`ship-*`.
- Follow-up: a lint that flags a `ship-*` pack shipping `sui-*` tags (and vice-versa).

## Alternatives Considered
- **Purity-only (pure ⟹ sui).** Rejected: makes `arcade` and vendored-engine
  wrappers `sui`, which is wrong — they're features/shells, not atomic primitives.
- **Vendoring-only (any vendor ⟹ ship).** Rejected: would force `sui-draw`/
  `sui-sheet`/`sui-board` to `ship` for calling a pure helper. The engine-vs-helper
  line preserves them as `sui`.
- **A single flat tier.** Rejected: loses the publishable-framework boundary that
  lets `sui-*` ship as a standalone component library.
