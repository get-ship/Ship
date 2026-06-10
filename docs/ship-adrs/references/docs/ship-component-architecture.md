---
title: "Ship Component Architecture — the sui / ship tiers"
description: "ADR: component packs split into two tiers. sui-* is the framework's reusable, dependency-pure, atomic primitive library; ship-* is everything built from it — features, verticals, external-engine wrappers, internal/unpublished, and domain-coupled packs. A pack is sui only if it passes every sui gate; ship is the default. External engines are declared deps inlined at build (never committed as vendored source)."
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

1. **SDK-pure** — no `@ship/sdk-client`, no `ship.*`, no daemon/deck/`/_deck` coupling.
2. **Atomic / single-concern** — one composable building block, not a bundle.
3. **Generic / reusable** — useful outside Ship, in a stranger's app.
4. **Our own engine** — does not *wrap* a heavyweight external engine (see below).
5. **Published** — part of a published pack, not an internal/private surface.

**`ship` — everything built from `sui`.** The default tier. A pack is `ship` if
it trips **any** of these:

- **Domain coupling** — touches a Ship domain / the SDK (`agents`, `pipelines`, `authority`).
- **External engine** — *wraps* an external engine that is fundamentally the
  component (`code` ← CodeMirror, `markdown` ← Tiptap, `terminal` ← xterm,
  `arcade` ← PixiJS, a PDF viewer ← pdf.js, a map viewer ← maplibre). The engine
  is a declared dependency inlined at build, not committed source (see below).
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

### Engine vs helper (the external-dependency nuance)

An external dependency pushes to `ship` only when it **is** the component's
engine — the component is a thin SUI shell around it (xterm/CodeMirror/Tiptap/
pixi/pdf.js). Depending on a **pure helper** as one internal ingredient, while
the engine is ours, keeps a pack `sui`: `sui-draw` (perfect-freehand/roughjs
stroke math), `sui-sheet` (a formula evaluator), `sui-board` (dagre layout) —
our engines call third-party algorithms, but the architecture and identity are
ours. The test: *"is this a shell around someone else's engine, or our engine
that calls a third-party function?"*

> **How external engines are carried (no vendored bytes).** We do **not** commit
> vendored third-party source. Each external engine/helper is a *declared
> dependency* in the pack's `package.json` and is **inlined at build** by the
> Rolldown bundler. The two tiers differ only in `depKind`:
>
> - **ship-tier** → `"dependencies"`. A `ship` pack may be consumed as a
>   published package whose own dep graph carries the engine.
> - **sui-tier** → `"devDependencies"`. The published `sui-*` pack stays
>   dependency-pure (gate #1); the bundler inlines the helper at build, so the
>   shipped artifact carries it without a runtime dep edge. This is the proven
>   `@ship/code` model applied to the `sui` purity gate.
>
> **Pinning** is the lockfile (`bun.lock`), not a committed copy. **License
> control** is the CI audit gate (`just license-check` → `scripts/check-licenses.mjs`),
> which walks the resolved tree and fails on any copyleft license (GPL/AGPL/
> LGPL/EPL/MPL/CDDL…) — this replaces the old hand-copied vendor `LICENSE`
> files. A `packages/**/vendor/` `.gitignore` rule guarantees no third-party
> source is ever re-committed.

### Worked classification

| `sui` (standard library) | `ship` (features / verticals) | Why `ship` |
|---|---|---|
| primitives, table, tree, calendar, charts | agents · pipelines · authority | domain |
| canvas / board / draw (own engines; helper *deps*) | code (CodeMirror) · markdown (Tiptap) · terminal (xterm) | external engine |
| chat-elements; media renderers (image/av/csv); json-view, diff | arcade (PixiJS + a game vertical) | engine + feature |
| component-viewer tooling | viewer (the mime *router* — a capability); pdf (pdf.js), map (maplibre) renderers | feature / external engine |

The viewer family makes the split concrete: the **router** (`ship-viewer`) is a
`ship` capability; **own-rendering leaves** (image/av/csv) are `sui-media`;
**external-engine leaves** (pdf, map) are `ship` renderers the router dispatches to.

## Consequences

### Positive
- One unambiguous test; the tier stops drifting.
- The boundary lint's purity rule is correctly understood as *necessary, not
  sufficient* for `sui`.
- `ship` absorbs the messy reality (external engines, verticals, internal,
  domain) so `sui` stays a clean, publishable, atomic library.

### Negative
- A generic-but-engine-backed primitive (a bare terminal/code editor) is `ship`,
  not `sui`, even though a stranger might want it standalone. Accepted: we keep
  one home per component; if a non-feature consumer ever needs it alone, it
  becomes a shared `ship` component, not a `sui` promotion.
- Name/dir/tag must be kept in agreement by hand until the lint below lands.

## Migration
- Editors lifted from `apps/shell` → `@ship/code` (+ terminal, git-diff) and
  `@ship/markdown`; tags retagged `sui-*`→`ship-*`.
- Viewers split: `@ship/sui-media` (own-rendering leaves) + `@ship/viewer` (router).
- `arcade` stays `@ship/arcade` (correct tier); its tags retag `sui-*`→`ship-*`.
- **De-vendoring (this ADR's update).** Previously, several engines/helpers were
  carried as *committed vendored source* under `src/vendor/` with a hand-copied
  `LICENSE` next to the bytes. That is reversed: each is now a **declared
  dependency** inlined at build, the committed `vendor/` dirs are `git rm`'d, and
  a `packages/**/vendor/` `.gitignore` rule keeps third-party bytes out of the
  tree. Packs and `depKind`:
  - `@ship/pdf` (ship) → `pdfjs-dist` as a **`dependency`** (Apache-2.0).
  - `@ship/arcade-engine` (ship) → `pixi.js` as a **`dependency`** (MIT).
  - `@ship/sui-primitives` (sui, sui-sheet) → `fast-formula-parser` as a
    **`devDependency`** (MIT); bundler inlines it.
  - `@ship/sui-runtime` (sui) → `@tanstack/virtual-core` as a **`devDependency`** (MIT).
  - `@ship/sui-canvas` (sui) → `@dagrejs/dagre`, `perfect-freehand`, `roughjs` as
    **`devDependencies`** (all MIT); bundler inlines them.
  Pinning is `bun.lock`; license control is the CI audit gate
  (`scripts/check-licenses.mjs`, wired into `.github/workflows/ci.yml`).
- Follow-up: a lint that flags a `ship-*` pack shipping `sui-*` tags (and vice-versa).

## Alternatives Considered
- **Purity-only (pure ⟹ sui).** Rejected: makes `arcade` and external-engine
  wrappers `sui`, which is wrong — they're features/shells, not atomic primitives.
- **External-engine-only (any external dep ⟹ ship).** Rejected: would force
  `sui-draw`/`sui-sheet`/`sui-board` to `ship` for depending on a pure helper.
  The engine-vs-helper line preserves them as `sui`.
- **Committing vendored source (the prior practice).** Rejected: duplicates the
  registry into the tree, makes upgrades a manual byte-copy, and pushes license
  audit onto hand-copied `LICENSE` files. Replaced by declared deps + lockfile
  pinning + the CI license gate.
- **A single flat tier.** Rejected: loses the publishable-framework boundary that
  lets `sui-*` ship as a standalone component library.
