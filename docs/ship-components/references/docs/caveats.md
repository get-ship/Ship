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

## ~~No external style import~~ — fixed

Previously the `<style>` block had to be inline, which capped components at ~250 lines of CSS before bumping the file-size cap. Now:

```html
<style src="./shared.css">
  /* optional inline rules — appended after the external content */
  .local-override { padding: 8px; }
</style>
```

The daemon resolves the path relative to the `.ship` file. External + inline can coexist (external loads first, inline appends). Both go through the scoping pass unless `global` is set.

Reference: `.ship/components/ship-tabs.ship` uses this pattern with a sibling `ship-tabs.css`.

## ~~Auto prop signals — `$<propName>` works directly~~ — fixed

Every declared prop is now a per-instance signal automatically:

```js
const { placeholder = 'Search' } = props
// $placeholder is auto-created with the decoded value, scoped per instance.
```

```html
<input data-attr:placeholder="$placeholder" />
```

The signal stays in sync via `attributeChangedCallback` — external `setAttribute('placeholder', '...')` triggers the signal update. Templates and effects react automatically.

`{{ props.X }}` in the template still evaluates at compile time (Jinja, not signal-aware). Use `$X` in Datastar attribute expressions for runtime values.

## ~~`emit()` doesn't set `composed: true`~~ — fixed

Custom events dispatched via `emit()` now include `composed: true` so they cross shadow-DOM boundaries when the component is consumed inside a shadow root.

## ~~No `data:` mount-time fetch declaration~~ — fixed

`data(fn)` is now an injectable in the script block. Same shape as `signal()`:

```js
const $agent = data(({ signal }) =>
  fetch('/api/v1/agents/' + props.agentId, { signal, credentials: 'same-origin' })
    .then(r => r.json())
)
```

The signal initializes to `null`, then resolves to the fetch result. `signal` is an `AbortSignal` wired to the component's `disconnectedCallback` — pass it to `fetch` so in-flight requests cancel cleanly when the component unmounts. Errors are logged and dispatched as a `data-error` CustomEvent from the host (composed + bubbles).

In the template, reference the signal directly:

```html
<div data-show="$agent">
  <h3 data-text="$agent.name"></h3>
</div>
```

For dep-aware refetch (signal A changes → re-fetch from URL derived from A), use `effect()` explicitly:

```js
const $agent = signal(null)
effect($agentId, async () => {
  const r = await fetch('/api/v1/agents/' + $agentId.get(), { credentials: 'same-origin' })
  $agent.set(await r.json())
})
```

`data()` is the simpler one-shot path; `effect() + signal()` is the reactive-refetch path.

## ~~Reflecting state to host attributes is manual~~ — fixed

`reflectAttr(name, $signal)` is an injectable in the script block. Subscribes to the signal and mirrors changes to the host attribute:

```js
const $active = signal('home')
reflectAttr('active', $active)
```

Boolean `true` → empty attribute; `null` / `false` → attribute removed; everything else → `String(value)`. No infinite-loop risk — the attribute change fires `attributeChangedCallback` which calls `signal.set()`, but the signal short-circuits on unchanged values.

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

## ~~Jinja comments leak into served output~~ — fixed

`{# ... #}` Jinja comments now strip cleanly at compile time. Previously a fast-path in the template renderer bypassed Jinja for sources that contained no `{%` or `{{`, which inadvertently let `{#` survive verbatim into the served HTML / SFC template literal. Fast-path now also gates on `{#` absence.

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
| Auto prop signals (`$<propName>` works directly) | ✅ |
| `data(fn)` mount-time async fetch | ✅ |
| `reflectAttr(name, sig)` for host-attr mirroring | ✅ |
| Style scoping | ✅ |
| External style import (`<style src="...">`) | ✅ |
| `:host`-aware scoping (no double-prefixing) | ✅ |
| Jinja comment stripping | ✅ |
| Slots | ✅ |
| Events (with `composed: true`) | ✅ |
| SDK macro globals | ✅ |
| Cookie-authed fetch from `onConnect` | ✅ |
| Hot-reload of template + style | ✅ |
| LSP recognizes `.ship` extension | ✅ |
| LSP lints Jinja content | ✅ |
| LSP lints inner JS/CSS | ❌ deferred (v2) |
| View-side `<script>` block execution | ❌ deferred |
| `script` block hot-reload | ❌ deferred |
| `ship-set` migration | ⏳ in progress (`ship-tabs` ported) |

None of the deferred items block real authoring. They're polish layers that compound across many components — worth landing as authors hit the friction.
