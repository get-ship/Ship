---
title: "SUI / Deck / Ship Responsibility Boundaries"
description: "ADR: define what each layer OWNS (not just where it lives) — SUI is the language (React), Deck is the metaframework that bundles+serves (Next.js), Ship is an app built on Deck. One implementation per concern. Deck owns the single build+serve pipeline; SUI emits structure not behavior and never names a URL; the shell dogfoods Deck's bundler. Resolves the bundling/serving clutter and the parallel-systems class of bugs."
group: ADRs
order: 41
audience: internal
status: proposed
date: 2026-05-26
supersedes: []
tags: [sui, deck, ship, metaframework, bundling, serving, boundaries, runtime]
---

# SUI / Deck / Ship Responsibility Boundaries

## Context

`sui-deck-crate-boundary` (accepted) split the workspace by *name and directory*:
`crates/sui` + `packages/sui` = the SUI framework, `crates/deck` + `packages/deck`
= the Deck metaframework. (That ADR predates the REST/MCP cull — its `deck-server`
crate is now `deck-core` + `deck-axum`; a leftover untracked `crates/deck/deck-server/`
dir remains and is tracked for cleanup.)

The directory split is done. The **responsibility** split is not. Nothing states
what each layer *owns*, so concerns smeared across layers and hacky fixes filtered
in. Concretely (from a 2026-05-26 audit):

- **SUI emits behavior, not just structure.** The compiler hand-rolls the
  component-mount state proxy (`__sp`), cleanup loops, and signal-detection inline
  into every compiled module. This is runtime behavior living in compile output —
  and it caused a shipped bug: the compiler's `__sp` lacked the `$signals` accessor
  the runtime proxy had, so every controller migrated to `state.$signals.x` threw at
  mount. tsc was green; only web-QA caught it (97 errors on the shell home).
- **Bundling has three owners.** Deck's bundler worker, `sui-build`, *and* vite
  (shell-only). The vite/bundler asymmetry around runtime externalization is the
  root of the split-brain runtime class.
- **Serving is scattered.** Three asset output dirs (`.deck/`, `deck-core/assets`,
  dead `deck-server/assets`); the `/_sui/` URL scheme hardcoded in ≥4 places + two
  import maps; the single-runtime-instance guarantee enforced for bundler targets
  but not for the shell.
- **Duplicated logic that drifts.** Two `MutableState` proxy builders; four
  signal-detection guards (one already divergent).

Every one of these is the same disease: **one concept, two implementations** — kept
in agreement by hand until they drift, then caught (if lucky) by QA.

## Decision

Define the layers by **what they own**, make each concern have **exactly one
implementation**, and give Deck undivided ownership of bundling + serving.

### 1. The responsibility contract

| Concern | Owner | Notes |
|---|---|---|
| The `.sui` grammar + language semantics | **SUI** | |
| Compile `.sui` → per-file `.js`/`.css`/`.d.ts`/`.ssr.html`/CEM/meta | **SUI** | one compiler; bin/wasm are thin shims over the lib |
| Reactive runtime (signals, bindings, control flow, mount/attach) | **SUI** | one runtime |
| The `MutableState` proxy (view-mount + component-mount + handlers) | **SUI** | **one** runtime-owned builder |
| Language services: LSP, lint | **SUI** | operate on the language only; like the TS language service belongs to TS |
| Bundling (a tree of files → app bundles + islands + views) | **Deck** | **the one** bundler; `sui-build` is Deck's project builder |
| Serving (routes, view-host HTML, import maps, asset endpoints) | **Deck** | |
| Routing | **Deck** | per `unified-deck-router` |
| Data/SSE wiring, HMR/live-reload | **Deck** | |
| The single-runtime-instance guarantee | **Deck** | externalization + import map, applied to **all** outputs |
| In-page devtools panels (signals/components/lint/schema inspectors) | **Deck** | they inspect a *running hosted app* |
| The app: shell, views, studio, domains, `.ship/` config | **Ship** | consumes SUI + Deck; zero privileged access |
| Component studio / playground | **Ship** | an app surface, like any other |

### 2. Three enforceable invariants

- **I1 — SUI emits structure, not behavior, and never a location.** Compiler output
  contains bare-specifier imports (`@ship/sui-runtime`) and DOM/structure wiring
  only. It must never emit a non-trivial proxy/loop/guard inline (call a runtime
  helper instead), and must never emit a URL, an import map, or an output path.
  *Grep gate:* no `new Proxy(` / hand-rolled signal checks in `emit/js/*.rs`; no
  `/_sui/` or `importmap` in compiler output.
- **I2 — Deck owns one build pipeline and one serve contract, and names no Ship
  concepts.** One bundler, one output dir, one route/URL source of truth (the import
  map is generated from it), one externalization contract applied to every output
  including the shell. Deck hardcodes no `apps/shell`/`views/`/`.ship/`/specific
  tags — it is *told* its roots. *Grep gate:* zero Ship paths/names in `crates/deck`
  + `packages/deck` (env-var contracts like `SHIP_FIRST_PARTY_VIEWS` are the host
  telling Deck a root — allowed, but Deck reads them generically).
- **I3 — one implementation per concern.** No parallel systems. If a second copy of
  a concept appears, it's a bug to be deleted, not maintained in parallel.

### 3. Bundling + serving: the one pipeline

- **Deck owns the build. Dogfood it.** The shell stops being a vite special-case and
  is built by Deck's bundler like any other Deck app — Ship dogfoods its own
  metaframework. This kills the externalization asymmetry and the split-brain class
  at the root. (Vite may remain a *thin dev-serve transform* during migration, but
  it must defer to Deck's externalization + URL + single-runtime contract and is not
  an authoritative build path. The destination is Deck-owned build.)
- **One output dir:** `.deck/` for the app build; Deck's crate-assets hold only
  Deck's own runtime (baked via `include_str!`). The dead `deck-server/assets` and
  every writer/reader pointed at it are removed.
- **One serve contract:** a single route/URL table is the SSOT; the import map and
  every `/_sui/*` reference are generated from it, not hand-copied.
- **One runtime instance, structurally:** the externalization contract is applied to
  *all* outputs incl. the shell. `globalThis.__suiRuntimeInstances` must be 1 in
  prod (the existing counter becomes the gate).
- **One compiler resolution:** in a repo/dev context the locally-built compiler wins;
  the deployed `~/.ship/runtime` copy is the fallback, not the default.

### 4. SUI standalone scope

SUI keeps a **clean internal boundary** (I1 fully enforced) but is **Deck-coupled in
practice** — we make no commitment to external, non-Deck consumers. So I1 is code
discipline + grep/lint gates, not an npm-publishing/packaging deliverable.

## Consequences

### Positive

- One mental model that matches the analogy: SUI = React (language + runtime), Deck =
  Next.js (build + serve + host), Ship = the app. A reader knows where anything lives.
- The parallel-systems bug class (the `$signals` failure) becomes structurally
  impossible: SUI emits no behavior, so there is no second proxy to drift.
- Bundling/serving has one owner and one pipeline — the clutter (3 dirs, 3 bundlers,
  4 hardcoded URL sites) collapses.
- Dogfooding Deck on the shell means Ship's most important surface exercises the
  metaframework's real path, so regressions surface in our own product first.

### Negative

- Moving the shell off vite onto Deck's bundler is a real migration with DX risk
  (HMR, dev-serve speed) that must be matched before vite is retired.
- Pushing compiler-emitted behavior into runtime helpers touches the compiler + the
  runtime together and requires recompiling all components to land.
- Several broad, mechanical edits (URL SSOT, output-dir consolidation) conflict with
  any branch touching bundling/serving while they land.

## Alternatives Considered

- **Keep vite for the shell, just make it obey.** Rejected as the destination
  (accepted only as a migration waypoint): it keeps two build paths and a special
  case Deck must forever keep in sync — the exact asymmetry that caused the
  split-brain. Dogfooding Deck is the elegant end state.
- **Make SUI fully standalone/publishable (React-without-Next).** Deferred: same code
  discipline (I1), but no commitment to external consumers now — avoids packaging
  work for a use case we don't have.
- **Leave the boundaries implicit, fix smells one by one.** Rejected: that is how the
  hacky fixes filtered in. Without a stated contract, every fix is local and the next
  parallel system reappears.

## Migration

Sequenced; each step is one job, validated by `cargo test` + shell typecheck +
web-QA (0 console errors). Existing tasks map in:

1. **#152** — unify the `MutableState` proxy into one runtime-owned builder; compiler
   emits a call to it (satisfies I1 + I3 for the proxy).
2. **#154** — one exported `isSignal()`; delete the three copies (I3).
3. **#153** — collapse to one asset output dir; delete dead `deck-server` + repoint
   all references (I2). Trace `apps/shipd` refs before deleting.
4. **#157** — one route/URL SSOT; generate the import map from it (I2).
5. **#155** — compiler resolution prefers the local build in dev (one pipeline).
6. **New — dogfood the shell on Deck's bundler**; apply the externalization contract
   to the shell; retire vite-as-authoritative-build (I2, the split-brain root).
7. **#156** — split the SUI god-files (hygiene; lowers the surface where I1 leaks).
8. **Enforcement** — add lint/CI grep gates for I1 + I2 so the boundaries can't
   silently erode again (mirrors the existing `sui-controller-*` lint approach).

## How to Measure Success

- One bundler, one asset output dir, one route/URL SSOT (grep proves it).
- `emit/js/*.rs` emits zero hand-rolled proxies/guards and zero URLs/import-maps.
- `crates/deck` + `packages/deck` name zero Ship concepts.
- `__suiRuntimeInstances === 1` in a production-shaped build; web-QA 0 console errors
  on shell + studio.
- The shell builds via Deck's bundler with no vite-specific runtime handling.
