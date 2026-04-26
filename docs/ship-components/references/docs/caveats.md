---
group: Components
title: Caveats and known gaps
description: What the v1 SFC format doesn't yet cover, and how to work around it.
order: 6
---

The `.ship` SFC format is the v1 contract. Several pieces are intentionally deferred to keep the surface tight. Each item below is real and documented; workarounds are linked.

## View-side `.ship` files don't execute scripts yet

Today only `.ship/components/*.ship` get the full SFC treatment (script + template + style + custom-element registration).

`.ship/views/<name>/index.html.j2` remains the canonical view path. The script-block IIFE for view-side files is not wired yet.

**Workaround:** keep authoring views as `.html.j2`. Compose `.ship` components inside them. This works for almost every case because views are usually markup + ship.* op interpolation + Datastar attributes — they don't need imperative setup. When a view DOES need scripty behavior, that behavior probably belongs in a component anyway.

**Cost:** none for now. The `.html.j2` extension is identical syntactically to a script-less `.ship` file.

## No `data:` mount-time fetch declaration

The eventual shorthand will let components declare data dependencies inline:

```js
// Future API — not implemented
const $agent = data(() => ship.agents.detail({ id: props.agentId }))
```

Until that lands, authors handle the fetch manually inside `onConnect` or an `effect`.

**Workaround:** see [Data fetching](./data-fetching.md) for the manual `fetch()` + `credentials: 'same-origin'` pattern. ~5 lines per component that needs server data.

**Cost:** boilerplate. Not capability.

## LSP doesn't lint inside `<script>` or `<style>` blocks

The Ship LSP catches Jinja issues in the template body — undefined signals, unknown macros, missing pack declarations. It does NOT yet:

- Parse the `<script>` block for Datastar-attribute-shaped errors (signal name collisions, unhandled prop renames, etc.).
- Lint the `<style>` block for theme-token compliance (bare hex colors, undocumented var names).
- Validate prop schemas at the call site (e.g. `<my-component count="abc">` when `count` is declared `number`).

**Workaround:** use your editor's stock JS and CSS language servers for inner-block diagnostics. Ship's LSP fills in the Jinja-aware piece. The cross-language pieces land in LSP v2.

**Cost:** authors get standard JS/CSS support, just not Ship-aware. Fine for most work; will feel limiting once you write enough components to want consistent-with-views auto-complete on signal names.

## Existing `ship-set` LitElement components stay LitElement

The 200+ pre-built `<ship-*>` components in `packages/ship-set/` are LitElement TypeScript classes. They will not be migrated to `.ship` SFC format until each one earns the conversion (post-launch incremental work).

**Workaround:** none needed. They already work. Compose them in views as before. The SFC format is for NEW components you author; the SDK primitives stay where they are.

**Cost:** mixed-vocabulary in the SDK (some primitives are TypeScript, some will be `.ship`). When reading the SDK to learn patterns, you encounter both shapes. Long-term we want consistency; short-term the cost is "two reading patterns instead of one".

## Component signals don't deep-watch nested structures

Signals are JSON-serializable values. Updates trigger by REPLACING the value:

```js
// Triggers update
$items.set([...$items.get(), newItem])

// Does NOT trigger update — mutation, not replacement
$items.get().push(newItem)
```

**Workaround:** always `set` with a new value (immutable update pattern). Spread operators are your friend for arrays and objects.

**Cost:** authors coming from MobX or Vue (which deep-watch) need to learn this. Authors coming from React already know it.

## No HMR for the `<script>` block

Editing the JavaScript in a `.ship` component requires a hard refresh in the browser. The daemon recompiles on the next request, but already-mounted instances keep their old behavior until the page reloads.

**Workaround:** Cmd-R / F5 after script-block edits. Template-only edits hot-reload via Datastar's morph behavior (the daemon serves the new HTML, Datastar diffs in place).

**Cost:** small. Components are usually small enough that the round-trip is negligible.

## `--features unstable` Rust build currently ICEs

A specific async-closure pattern in `runtime::services::orchestrator::pipeline::advance_pipeline_inner` triggers an internal compiler error in rustc 1.93.1 when built with `--features unstable`. The non-unstable build is clean.

**Workaround:** build with `cargo build -p shipd` (no `--features unstable`) until rustc 1.94 lands. Stable build covers all production code paths; unstable gates a few in-progress dev tools that aren't required for SFC work.

**Cost:** `just build` (which adds `--features unstable`) breaks. Use `cargo build -p shipd` directly.

## Status summary

| Capability | State |
|---|---|
| Component authoring | ✅ Full (`.ship/components/*.ship`) |
| Mount + lifecycle | ✅ |
| Per-instance signal scoping | ✅ |
| Style scoping | ✅ |
| Slots | ✅ |
| Events | ✅ |
| SDK macro globals | ✅ |
| Cookie-authed fetch from `onConnect` | ✅ |
| Hot-reload of template + style | ✅ |
| LSP recognizes `.ship` extension | ✅ |
| LSP lints Jinja content | ✅ |
| LSP lints inner JS/CSS | ❌ deferred |
| `data:` mount-time declaration | ❌ deferred |
| View-side `<script>` block execution | ❌ deferred |
| `script` block hot-reload | ❌ deferred |
| `ship-set` migration | ❌ post-launch |

None of the deferred items block real authoring. They're polish layers that compound across many components — worth landing as authors hit the friction.
