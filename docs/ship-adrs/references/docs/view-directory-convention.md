---
title: "Route & View Directory Convention"
description: "ADR: a SUI unit is a DIRECTORY (page.sui + optional controller.ts + _components/), discovered by deck file-based routing. The same shape serves first-party app routes (no manifest) and user-applet views (manifest.jsonc, because a view is a publishable applet). Co-location, root-route, and large-component-decomposition rules included."
group: ADRs
order: 21
audience: internal
status: accepted
date: 2026-05-05
updated: 2026-06-22
supersedes: []
tags: [views, routes, sui, routing, conventions]
---

# Route & View Directory Convention

> **2026-06-22 refresh.** The directory shape below has held since 2026-05-05; this revision rescopes it to how deck actually implements routing today and folds in the co-location, root-route, and large-component rules. **Superseded since the original:** routing is **deck file-based discovery** (`crates/deck/deck-bundler/src/routes.rs`), NOT the retired `apps/shell` `defineRoutes()` / `ship-ui-framework` resolver; components live at `packages/components/{sui,ship}/<pack>/src/<tag>/` (the old `packages/components/ui-components/` is deleted).

## The unit: a directory, never loose files

SUI authoring is a **two-file unit** — a `.sui` template plus an optional `.controller.ts`. Because a unit is two files, **every routable/composable unit is a directory** that holds its pair. Never loose `foo.sui` + `foo.controller.ts` siblings sharing a level.

```
<unit>/
  page.sui            ← the entry. The dir IS the unit, so the template is always `page.sui`.
  controller.ts       ← optional pair. Named `controller.ts`, NOT `<stem>.controller.ts`
                          (no ambiguity — one template per dir).
  style.css           ← optional, page-scoped.
  _components/<tag>/   ← optional: unit-private widgets (see Co-location).
    <tag>.sui
    <tag>.controller.ts
```

deck already enforces this for routes: `routes.rs` discovers "a directory with a `page.sui` and (optionally) `controller.ts`." A `[param]` directory captures a path segment; `_layout.sui` at a level wraps everything beneath it.

## Two consumers, one shape

The same directory convention serves two distinct things:

1. **First-party app routes** (`apps/<app>/routes/`, e.g. the hub). File-based: the directory tree IS the URL tree (`routes/studio/page.sui` → `/studio`). **No `manifest.jsonc`** — these are part of a first-party app, not distributable units; their metadata is their position in the tree.

2. **Views — user applets** (the views feature). A view injects into a **space** application's route tree **at runtime**, using the *same* routing convention. A view **requires `manifest.jsonc`** — not because routes need metadata, but because **a view is a publishable applet**: its own distributable thing, downloadable from the Ship marketplace or shipped as a built-in. The manifest is the applet's descriptor (identity, the path it claims, title, the data/events it preloads) — effectively the `package.json` of a view.

So `manifest.jsonc` is an **applet** concern, not a route concern. App routes omit it; views require it.

## The root route

`/` has no path segment, so the root route's files (`page.sui`, optional `controller.ts`) live at the `routes/` directory root alongside `_layout.sui`. This is correct, not an accident: **`routes/` *is* the root route's directory.** Don't invent a synthetic `(home)/` wrapper to avoid "top-level" files — the root dir doubles as the `/` unit and the layout container by design.

## Co-location: where a widget lives

A widget's home is decided by **how many surfaces use it** — promote *up* this table only when a second consumer actually appears, never preemptively:

| Used by | Lives in |
|---|---|
| one route | `routes/<route>/_components/<tag>/` (private, scoped registration) |
| ≥2 routes of one app | `apps/<app>/src/components/<tag>/` |
| ≥2 ship apps | `@ship/shared` (internal, non-published) |
| a publishable feature | its domain / supplementary pack |
| generic + public-worthy | `sui` (on merit) |

`_components/` widgets are **view-private**: registered lazily on mount, unregistered on dispose, tag prefixed by the unit name (`home-hero`, not `hero`) so the registry stays collision-free across views. Putting a single-route widget in `src/components/` (or a pack) is the anti-pattern: it divorces the widget from its owner and re-creates "which bucket is this in?"

## Large components decompose into sub-unit directories

A component that outgrows one legible `.sui`+`.controller.ts` pair **splits into sub-component directories**, each its own pair, composed by the parent — exactly as a route uses `_components/`. The healthy model in-tree is the command bar split (`sui-command-bar` + `sui-command-chat` + `sui-command-palette`, three dirs). The anti-pattern is the 824-line `ship-command-bar.controller.ts` monolith. Decomposition keeps every file a glanceable unit and lets sub-parts compose `sui` primitives directly.

## Consequences

- **Filename encodes role; the dir encodes the unit.** `page.sui` says "I am the entry"; the directory says "this pair is one thing." Loose files at a level are the smell the convention removes.
- **One shape, two policies.** App routes and view-applets share discovery; only the manifest (applet descriptor) differs. No second routing convention to learn.
- **Co-location keeps ownership legible.** A widget sits with whatever owns its lifetime; promotion is demand-driven.
- **Cost:** views carry a required `manifest.jsonc` (small); the `_components/` tag-prefix collision rule is convention, enforced by the router at registration, not the compiler.

## Status of the original decision

The 2026-05-05 `manifest.jsonc` schema (`name`, `path`, `title`, `layout`, `auth`, `preload`, `events`) stands **for views**. App routes do not carry it. JSONC, not YAML. The richer path matcher and `auth`/`preload`/`events` enforcement remain downstream work.
