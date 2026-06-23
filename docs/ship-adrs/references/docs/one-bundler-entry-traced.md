---
title: "One Bundler, Entry-Traced"
description: "ADR: deck-bundler (rolldown) is the only bundler; component membership is traced per entry, not declared in hardcoded lists; sui-build is retired; dev serves un-bundled ESM, prod emits traced chunks."
group: ADRs
order: 31
audience: internal
status: accepted
date: 2026-05-30
supersedes: []
tags: [bundling, deck, rolldown, sui-build, islands]
---

# One Bundler, Entry-Traced

## Context

SUI components are bundled four overlapping ways today, by two different tools:

- **deck-bundler** (rolldown, Rust, `/home/madvib/dev/sui/crates/deck/deck-bundler`) — the canonical
  one. Produces the framework singletons (`sui-runtime`/`sui-dom`/`sui-sdk-client`),
  the `sui-components` **umbrella** (all of `ui-components/src`), the
  `sui-marketing` aggregate (a hardcoded `MARKETING_SOURCES` list), `sui-chrome`,
  per-component **islands** (`<bundle separate />`), and per-**view** bundles.
  - **`@ship/sui-build`** (esbuild, `/home/madvib/dev/sui//home/madvib/dev/sui/packages/deck/build` post-carve) — a *parallel* bundler.
  Every component package runs `build: sui-build` → `dist/sui-<pack>.js`
  (consumed by apps via `import '@ship/<pack>/bundle'`) + `dist/custom-elements.json`
  + the `.ship/sui-out` test mirror.

Views are already entry-traced (rolldown follows imports; the runtime is
external). The mess is everything else: a second bundler, an "import everything"
umbrella, a hand-curated marketing list, and a **hardcoded island table in the
Rust compiler** (`SUI_ISLANDS` in `/home/madvib/dev/sui/crates/sui/compiler/src/islands.rs`) that has
already drifted from what the bundler produces (e.g. `sui-canvas` builds an island
but isn't in the table, so views can't lazy-load it — a live bug).

## Decision

**One bundler (deck/rolldown). Component membership is traced per entry, never
declared. Two modes: dev serves un-bundled ESM; prod emits traced chunks.**

1. **deck-bundler is the only bundler.** `@ship/sui-build` is retired. Its three
   jobs move:
   - test mirror (`.ship/sui-out/<tag>.compiled.js`) → `sui-compiler` directly
     (the vitest factory already compiles on demand);
   - per-pack `dist/sui-<pack>.js` → apps stop importing `@ship/<pack>/bundle`
     and *use* the tags; deck traces them into the consuming entry's chunk;
   - CEM (`custom-elements.json`) → emitted by deck (or the compiler) where docs/
     studio need it.

2. **Membership is traced, not declared.** Each entry — a view, an app, a docs/
   marketing page — is bundled by following its imports/template tags. This
   deletes:
   - the hardcoded `MARKETING_SOURCES` list (trace apps/site's actual `<sui-*>`
     usage instead);
   - the `SUI_ISLANDS` table in the compiler (island membership comes from the
     same `<bundle separate />` marker the bundler already scans, surfaced as a
     generated manifest both sides read — the compiler stops owning the list).

3. **Two modes, wired to `DaemonMode`** (the vite lesson):
   - **Development** → serve compiled `.sui` as un-bundled native ESM modules;
     HMR re-compiles only the edited component. No rebake per edit.
   - **Production** → rolldown emits per-entry traced chunks, `sui-runtime`/
     `sui-dom` as the shared external chunk, immutable-cached.

4. **The umbrella survives as ONE explicit convenience, not the app path.**
   `sui-components.js` (all components, one script tag) is a deliberate CDN-style
   install affordance the docs publish (`<script src="/api/assets/sui-components.js">`).
   Apps never load it — they trace. Keep it as an opt-in "everything" bundle,
   clearly labelled as such; it is not produced for, or loaded by, any first-party
   app surface.

## Slice plan (execution order, each independently shippable + verifiable)

1. **`SUI_ISLANDS` → generated manifest.** Bundler emits an islands manifest from
   the `<bundle separate />` scan; compiler reads it instead of the hardcoded
   table. Fixes the `sui-canvas` lazy-load bug. Contained to islands.rs + bundler.
2. **`MARKETING_SOURCES` → traced.** Replace the hardcoded list with a scan of
   apps/site's `<sui-*>` usage; verify the marketing bundle + site still render.
3. **Retire `sui-build` — test mirror.** Drop `pretest: sui-build`; the vitest
   factory's on-demand compile (or a single compiler precompile) feeds the mirror.
4. **Retire `sui-build` — pack bundles.** Apps drop `import '@ship/<pack>/bundle'`;
   deck traces the tags into each consuming entry. Remove `build: sui-build` and
   the `./bundle` exports. `@ship/sui-build` deleted.
5. **Dev = serve un-bundled ESM.** Wire DaemonMode::Development to serve compiled
   `.sui` modules directly (no rebake); prod stays rolldown-traced.

## Consequences

- One bundler, one mental model. No "which list does this component go in."
- Moving a component between `components/sui` and `components/ship` can never
  desync a bundle list again — there are no lists.
- Per-entry chunks are smaller than the umbrella; the marketing site ships only
  the tags it uses, traced, not a curated guess.
- Bigger blast radius than any cleanup so far — hence the slice order: the two
  pure-deletion-of-a-list slices (1, 2) land first and prove the tracing before
  `sui-build` is removed.

## Status of execution

Tracked as #220 (retire sui-build) + #221 (entry-traced / kill the lists).
Not started as of this ADR — this is the spec they execute against.
