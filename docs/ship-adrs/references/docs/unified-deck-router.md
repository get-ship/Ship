---
group: ADR
title: Unified Deck Router
audience: internal
status: accepted
---

# Unified Deck Router

**Status:** accepted
**Date:** 2026-05-24
**Related:** `sui-deck-crate-boundary`, `view-directory-convention`

## Context

Navigation is currently spread across six overlapping mechanisms with no single
owner:

1. Runtime `navigate()` (`packages/sui/runtime/src/route`) — `pushState` +
   `__shipSetRouteUpdate`. Updates the reactive `route.*` signals but **does not
   mount a view**.
2. Shell `mountRoute()` / `readRouteFromUrl()` — the only path that actually
   swaps views, driven by anchor-click interception and `popstate`.
3. `window.ship.navigate` → `chrome.navigate` → `emit('sui:navigate')`.
4. A `sui:navigate` custom event the shell listens for.
5. `notifySameShellRouteChange()` — dispatches a **synthetic `popstate`** to kick
   view-bundle route signals for same-view param changes.
6. Imperative per-view `defineRoutes([{ pattern }])`, called from each `[id]`
   controller, subject to a cross-bundle module-instance timing caveat.

Consequences of the sprawl: a controller calling the runtime `navigate('/x')`
silently fails to navigate (no mount), so authors hand-roll routers (e.g.
`sui-view-picker` combines `pushState` + the event + `window.ship.navigate`).
`SAME_SHELL_VIEWS` is a dead empty set. Routes are stringly-typed; a typo or a
missing `:param` is not caught. This is the "multiple ways to do one thing" the
framework is trying to eliminate, and it leaks host (Deck) concerns into view
authoring.

Routing is a Deck responsibility — the host runs views, the way Next runs React
components. It must be a designed feature, not an accreted set of side-channels.

## Decision

**1. One seam — `RouterPort`.** The runtime declares a host-agnostic
`interface RouterPort { navigate(to, opts?); back(); }`. The host registers one
(`setRouterPort`); the runtime `navigate()` delegates to it. This mirrors the
`SuiLoader` and `ShipHost` ports: the runtime owns the *mechanism* (reactive
`route.*` signals), the host owns the *policy* (history + match + mount). One
`navigate()`, callable from any bundle, that actually mounts.

**2. The router lives in Deck (the host).** It owns history (push/replace/
popstate), match (`path → { view, params }`), and the mount decision (different
view → mount; same view with changed params → update signals only). This
absorbs `mountRoute` + `readRouteFromUrl` + `SAME_SHELL_VIEWS` + the synthetic-
popstate kick into one coherent unit.

**3. Typed routes by codegen, not inference.** The bundler already discovers the
route tree (`views/<name>/page.sui` + `[id]` dirs + manifest `path`). A generated
`routes.gen.ts` emits a typed route table — the same "generate from a single
source of truth" move `feat/op-registry` uses for ops. Navigation becomes
`navigate('/agents/:id', { id })` with typed route ids and typed param **keys**
(values are `string`, the honest ceiling for URL segments). This **replaces
imperative `defineRoutes()`**: patterns come from the file tree, one source of
truth, no per-view calls, no cross-bundle timing issue.

**4. Host-pluggable for Tauri.** Because `RouterPort` is host-supplied, a browser
host uses the History API and a Tauri host supplies its own port — runtime and
views unchanged.

## Consequences

- A single `navigate()` works from any view bundle and mounts correctly; the
  `window.ship.navigate` side-channel, the `sui:navigate` event, and the
  synthetic-popstate kick collapse into the port.
- Authors stop hand-rolling routers; a lint can finally enforce "use the router"
  (allowing deliberate full-page `window.location` reloads, e.g. post-onboarding).
- Route definitions move from imperative scattered calls to one generated table;
  navigation is typed against real routes.
- Deck gains a first-class tooling surface (router), balancing SUI's compiler/LSP.

## Alternatives considered

- **Adopt a generic client router (Navigo, as before).** Rejected: bolts a
  generic router onto a model that already file-discovers routes; no typing; no
  integration with view mounting; perpetuates "an afterthought."
- **TanStack-style type-level inference.** Rejected: our routes are
  file-discovered and compiled, not expressed as a typed route tree in TS; we
  can't get inference without their complexity. Codegen reaches the same DX
  (typed paths + params) from the SSOT we already have.
- **Leave runtime `navigate()` to update signals only; keep host navigation
  separate.** Rejected: this *is* the current leak — it forces every author to
  learn that "navigate doesn't navigate."

## Rollout

1. `RouterPort` seam — runtime delegates `navigate()` to a registered port
   (falls back to in-place signal update when none); shell registers a port
   wrapping its existing mount logic. Additive, non-breaking.
2. Migrate callers (`sui-view-picker` hand-roll, SPA `window.location`) to
   `navigate()`; delete the side-channels.
3. Codegen `routes.gen.ts` from the view tree + manifests; router matches against
   it; retire imperative `defineRoutes`.
## Status (2026-06-05)
Step 1 (`RouterPort` seam) is shipped: the shell's `setRouterPort(...)` in
`apps/shell/src/main.ts` registers a thin wrapper around `navigate()` →
`mountRoute(location.pathname)`. Every entry point — runtime `navigate()`,
the anchor-click delegate, `popstate`, keymap Esc — flows through one
mount path. The `SAME_SHELL_VIEWS` no-remount hack, the manual `fetch +
runtime.mount(payload)` injection, the `payloadUrl()`/`readRoute()`/
`readRouteFromUrl()`/`notifySameShellRouteChange()` helpers, and the dead
`/artifacts/:id` + `/workspaces/:id` registrations are all removed.
Step 3 (codegen) is shipped: `crates/deck/deck-bundler/src/routes.rs`
walks the shell's full `src/` tree (every `[param]` dir, not just the
studio sub-tree) and emits a single `apps/shell/src/routes.gen.ts` with
`DYNAMIC_ROUTE_PATTERNS: RouteShape[]` plus namespaced path builders
(`routes.studioAgentsId({ id })`, etc.). The shell imports it and calls
`defineRoutes(DYNAMIC_ROUTE_PATTERNS)` once at boot. Imperative
`defineRoutes` from view controllers is gone (no longer supported by
the studio controllers — they import the typed builders instead).
Step 2 (migrate callers) is in progress. The shell's hand-rolled
`sui-view-picker`-style router is removed; the lint that bans raw
`pushState` from view controllers is a follow-up.
Memory-leak (issue #129/#130): `apps/shell/src/hmr.ts` switched from
`await import(url)` (which retains each compile in the ESM registry) to
`fetch + Blob URL + import(blob) + URL.revokeObjectURL`. The dev loop
no longer accumulates module-graph entries per rebuild.