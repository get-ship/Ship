---
title: "View Directory Convention"
description: "ADR: views/<name>/ uses page.sui (route entry), controller.ts, style.css, manifest.jsonc, dist/ (generated), and _components/ (view-private widgets) — distinct from the component shape under packages/ui-components/src/sui-*."
group: ADRs
order: 21
audience: internal
status: accepted
date: 2026-05-05
supersedes: []
tags: [views, sui, routing, conventions]
---

# View Directory Convention

## Context

Today views and components share the same directory shape. A view at `views/counter/` has a `view.sui`, `view.controller.ts`, `view.css`, and `package.json`. A component at `packages/ui-components/src/sui-button/` has `sui-button.sui`, `sui-button.controller.ts`. The two layers are conceptually distinct — a view is a **page** (route entry, served at a URL, owns a viewport), a component is a **reusable widget** (custom element, composed by views, owns its own internal state) — but the on-disk shape doesn't say so.

Three follow-on problems:

1. **No hint about role.** Reading `view.sui` in isolation does not tell a future author or agent whether they're looking at a routed page or one of many reusable widgets that happens to live under `views/`. The filename stem is mute.
2. **No place for view-private widgets.** A page often has one or two bespoke widgets that are not worth promoting to `packages/ui-components/`. Today there's no convention for where those land — authors either inline the markup, hand-roll a one-off custom element somewhere ad hoc, or put them next to siblings, blurring "exported component" with "page-private fragment."
3. **No declarative metadata.** Route path, title, layout slot, auth requirement, preload data — all of it lives in code (the daemon resolver, the shell main, ad hoc per-view setup). A view's metadata cannot be read without reading the runtime that interprets it.

The component shape (`packages/ui-components/src/sui-foo/`) is settled and stays settled — see `ship-components` skill. This ADR settles the **view** shape.

## Decision

### 1. Canonical view directory layout

```
views/<name>/                  ← route entry
  page.sui                     ← required: template (was view.sui)
  controller.ts                ← optional: paired controller (was view.controller.ts)
  style.css                    ← optional: page-scoped styles (was view.css)
  manifest.jsonc               ← required: route metadata
  dist/                        ← generated artifacts (compiler emits here)
  _components/                 ← optional: view-private widgets (not exported)
    <tag>/
      <tag>.sui
      <tag>.controller.ts      ← optional
      dist/
```

Five rules cover authoring:

- **`page.sui` is the route entry.** Filename stem encodes role: "this is the page." A view directory that only owns one `.sui` file always names it `page.sui`.
- **`controller.ts`, not `<stem>.controller.ts`.** The view directory IS the view; there's no ambiguity about which template the controller is paired with. Inside `_components/<tag>/`, the existing `<tag>.controller.ts` rule applies — the directory holds many tags by convention.
- **`style.css`, not `view.css`.** Matches `page.sui`/`controller.ts` cadence: noun-only, role-clear. The compiler/daemon links it automatically.
- **`manifest.jsonc` is required.** Route metadata declared, not inferred from directory name. See schema below.
- **`_components/` for view-private widgets.** Leading underscore marks "do not export, do not register globally." Custom elements registered from a `_components/` tree are scoped to the view (registered lazily on view mount, opaque to other views) — see "Components, scoped" below.

Generated artifacts go in `dist/` (per the artifact-emit job). No bundles, no `.compiled.js`, no `.d.ts` checked into the source tree.

### 2. `manifest.jsonc` — the v1 schema

```jsonc
// views/<name>/manifest.jsonc
{
  // Required.
  "name": "home",                     // stable identity; matches directory
  "path": "/",                        // URL pattern this view answers to

  // Optional, all with sensible defaults.
  "title": "Home",                    // <title> when this view is mounted
  "layout": "default",                // layout slot key; "default" if omitted
  "auth": "public",                   // "public" | "user" | "admin"
  "rail": {                           // shell rail entry (optional)
    "label": "Home",
    "icon": "home"
  },
  "preload": [                        // SDK calls to warm before mount (optional)
    "ship.workspaces.list()"
  ],
  "events": [                         // SSE channels to subscribe to (optional)
    "config.changed"
  ]
}
```

JSONC, not YAML. Matches the rest of the codebase — `ship.jsonc`, `mcp.jsonc`, `permissions.jsonc`, collection schemas. The previous `manifest.yaml` recommendation in the `ship-views` skill is superseded.

The schema is intentionally minimal for v1. `auth`, `preload`, and `events` are stubs — the router records them but enforcement / wiring is downstream work and may shift the schema. `path` is a literal pattern for v1 (`/`, `/counter`, `/agents/[id]`); a richer matcher (regex, nested layouts, route groups) is out of scope.

### 3. Components, scoped

A view-private widget under `views/<name>/_components/<tag>/<tag>.sui` is a SUI custom element exactly like any other component (same `.sui` surface, same compiler mode), with one difference: **its custom-element registration is scoped to the view's lifetime**. The router/runtime registers the tag lazily when the view mounts and unregisters on dispose, so two views can each define a `<my-list>` widget without colliding on `customElements.define`.

This is enforced by convention, not by the compiler — `<tag>` in `_components/` MUST start with the view name as a prefix to keep the registry collision-free in practice (`home-hero`, not `hero`). The router checks the prefix at startup and refuses to register a violator.

### 4. Migration

Existing first-party views (`views/agents/`, `views/counter/`, `views/git/`) keep their current `view.sui` shape until separately migrated. The Rust resolver (`crates/ship-ui-framework/src/resolver.rs`) will gain awareness of the new convention in a follow-on job — this ADR locks in the shape that follow-on aims at. One proof-of-concept view (`views/home/`) ships in the new shape alongside this ADR; the router (`packages/ship-set/src/router/`) walks it and resolves routes against its `manifest.jsonc`.

Mass migration of the existing three views is its own job once the convention is validated end-to-end (router → daemon → shell mount). Not in scope here.

## Consequences

### Positive

- **Filename encodes role.** `page.sui` says "I am a page" — a glance at the file system separates pages from reusable widgets. New contributors and agents stop guessing.
- **View-private widgets have a home.** `_components/` settles the question authors keep asking ("where does this one-off widget live?") without polluting the global component registry.
- **Route metadata is declarative and tooling-friendly.** A static `manifest.jsonc` walker (the new router) can build the route table at startup or build time — no code execution required.
- **Internal consistency.** JSONC for manifests matches every other Ship config format. No more YAML-in-views, JSONC-everywhere-else.

### Negative

- **One more required file per view.** Every view now ships a `manifest.jsonc`. The cost is low (the file is small) and the upside is route metadata becoming first-class.
- **Two name conventions on disk during migration.** Until existing views migrate, the daemon's resolver has to recognise both `view.sui` and `page.sui` (or the existing views are migrated together with the resolver in the same follow-on job). The router added in this job only walks the new shape; it ignores legacy directories.
- **`_components/` registration is a runtime concern.** The collision-avoidance rule (tag prefix matches view name) is convention, not a compiler check. A lint rule or compile-time check is a possible follow-up.

## Alternatives Considered

### Keep `view.sui` and only add `manifest.jsonc`

Cheapest path. Rejected because it doesn't fix the role-confusion problem the ADR exists to fix — the filename stem stays mute, and views still look identical to components on disk. If we're going to touch the convention at all, we settle the name.

### Use `index.sui` (mirroring `index.html` / Next.js conventions)

Familiar to web devs. Rejected because `index` is a generic "this directory has a default" name; `page` says what the file IS. Next.js's own router moved from `pages/index.tsx` to `app/page.tsx` for the same reason — the role-named file wins.

### Components register globally with full pathnames (no view scoping)

Avoids `_components/` and lets any widget be reused. Rejected because we already have `packages/ui-components/` for shared widgets. The whole point of `_components/` is opting OUT of the global registry — once a widget is reusable, it gets promoted to the components package. Two registries with clear directionality (`_components/` → `packages/ui-components/`) beats one giant flat namespace.

### YAML manifests

Existing skill docs suggested YAML. Rejected for v1: the codebase has converged on JSONC, and JSONC parses with `serde_jsonc` / native JSON tooling without an extra dependency. The skill doc gets updated.

## How to Measure Success

- `views/home/` ships in the new convention and the router (`packages/ship-set/src/router/`) resolves `/` → `views/home/page.sui`.
- The router's happy-path test (vitest) loads a manifest fixture and resolves a URL.
- `scripts/scaffold-view.mjs` produces a working stub matching the convention.
- The `ship-views` skill documents the convention with the migrated example.
- A follow-on job migrates the remaining first-party views (`agents`, `counter`, `git`) and updates the daemon resolver. This ADR is the contract that job aims at.
