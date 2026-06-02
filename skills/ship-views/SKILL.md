---
name: ship-views
stable-id: ship-views
description: USE BEFORE AUTHORING A VIEW. SUI templates compile AOT to ESM and hydrate server-rendered HTML. Pair `.sui` with optional `controller.ts` + `style.css` + required `manifest.jsonc`. Wrong shape and the resolver won't find it.
tags: [views, sui, sdk, extension-authoring, ship-elements]
authors: [ship]
audience: public
---

# ship-views

A Ship view is a `.sui` template with an optional paired `.controller.ts`. The compiler emits an ESM per template; the daemon SSRs it; `attach()` wires bindings on the client. Compose `<sui-*>` components inside.

## The rule

**Templates declare. Controllers do.** Dynamic markup goes in the `.sui` template via `{#if}/{#for}/{#match}` and the binding primitives. Controllers own state, refs, lifecycle, async I/O, and the public API the template binds against. A controller that produces markup is a bug — `sui-controller-purity` rejects it at compile and at edit.

Forbidden in `*.controller.ts`:
- `<el>.innerHTML = …` / `<el>.outerHTML = …` / `<el>.insertAdjacentHTML(…)`
- A sibling `render.ts` of HTML-string template-literal helpers
- Re-querying the shadow DOM after `mount` (use `bind:this` for refs)
- A `.sui` body that is just `<div bind:this={ref}></div>` — the template is the renderer

For HTML-blob cases (markdown rendering, third-party SVG generators, icon-pack swaps), use `bindHtml(el, () => string)` from `@ship/sui-runtime`.

## File layout

The canonical view directory shape (see ADR `view-directory-convention`):

```
views/<name>/                 # first-party view, baked into shipd
  page.sui                    # route entry — required
  controller.ts               # paired controller — optional
  style.css                   # page-scoped styles — optional, daemon auto-links
  manifest.jsonc              # route metadata — required
  dist/                       # generated artifacts (compiler emits here)
  _components/                # view-private widgets — optional
    <view>-<tag>/
      <view>-<tag>.sui
      <view>-<tag>.controller.ts
      dist/

.ship/views/<name>/           # extension-point view, project-local; same shape
  page.sui
  controller.ts
  manifest.jsonc
  partials/                   # reusable fragments
```

Five rules:

1. **`page.sui` is the route entry.** The filename stem says what the file IS — a page, not a reusable widget. Reusable widgets live under `packages/components/ui-components/src/sui-*/`.
2. **`controller.ts`, no stem prefix.** The directory IS the view; pairing is unambiguous. Inside `_components/<tag>/`, the `<tag>.controller.ts` rule from `ship-components` applies.
3. **`style.css` for page-scoped styles.** Linked automatically. Use `<style>` blocks inside `.sui` for component-scoped styles.
4. **`manifest.jsonc` is required.** JSONC, not YAML. See "Manifest" below.
5. **`_components/` for view-private widgets.** Tag MUST prefix with the view name (`home-hero` for `views/home/`) so two views can each ship their own `<my-list>` without colliding on `customElements.define`.

Plain `.ts` files (not `controller.ts`, not under `_components/`) are helpers, types, or tests.

Every first-party view in `views/` uses the `page.sui` + `controller.ts` + `manifest.jsonc` shape. The shell's route mounter (`apps/shell/src/main.ts::mountRoute`) walks only manifest-bearing directories — a missing `manifest.jsonc` means the view is invisible to the rail, regardless of which template form it ships.

### Worked example: `views/home/`

```
views/home/
  page.sui
  controller.ts
  style.css
  manifest.jsonc
  _components/
    home-hero/
      home-hero.sui
      home-hero.controller.ts
```

Manifest:

```jsonc
// views/home/manifest.jsonc
{
  "name": "home",
  "path": "/",
  "title": "Home",
  "layout": "default",
  "auth": "public",
  "rail": { "label": "Home", "icon": "home" }
}
```

Page composes the view-private widget. Reactive state lives in
`controller.ts`'s `scope:` factory — there is no `<state>` directive:

```html
<!-- views/home/page.sui -->
<div class="home">
  <home-hero prop:title={'Welcome to Ship'}></home-hero>
  <button on:click={() => seen.set(seen() + 1)}>Bump ({seen()})</button>
</div>
```

```ts
// views/home/controller.ts
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const seen = signal(0)
    return { seen }
  },
})
```

Scaffold a new view:

```sh
deck scaffold view settings
deck scaffold view settings --component settings-toolbar
```

## Template surface (compact)

```html
<!-- live data subscriptions (server-driven, SSE-backed) -->
<data name="agents" source="ship.agents.summaries()" live />
<data name="settings" source="ship.settings.read()" />

<!-- reactive args — re-subscribes when selectedId changes.
     `selectedId` is a signal declared in the controller's `scope:`
     factory (see Controller contract below). -->
<data name="agent" source="ship.agents.detail({id: $scope.selectedId()})" live />

<!-- props (component .sui files only) -->
<prop name="tone" values="default|primary" default="default" reflect />
<prop name="disabled" type="bool" default="false" reflect />

<!-- mutations (write-side complement to <data>) -->
<form mutation="ship.workspaces.activate">
  <input name="id" />
  <button type="submit" disabled={$mutation().kind === 'loading'}>Switch</button>
</form>

<!-- control flow — block syntax -->
{#if loading()}…{:else if error()}…{:else}…{/if}
{#for a of items() key={a.id}}…{/for}
{#match status()}{:case 'idle'}…{:default}…{/match}
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}

<!-- bindings -->
<button on:click={() => count.set(count() + 1)}>+</button>
<input bind:value={query} />
<input type="checkbox" bind:checked={enabled} />
<div class:active={isActive()} style:color={color()} attr:href={url()}>
  {count() === 1 ? '1 item' : count() + ' items'}
</div>

<!-- pass complex objects (signals, arrays, structured data) to a child component -->
<sui-grid prop:rows={rows} prop:columns={columns()} prop:total={count()}></sui-grid>

<!-- template hooks (browser-platform bridges) -->
<use-kbd><on key="/" action="search" /></use-kbd>
<use-click-outside bind:target={dialogEl} bind:action={close} />
<use-focus-trap bind:target={dialogEl} />
<use-clipboard />
<use-title source={title} />
<use-scroll-anchor />
<use-intersection callback={onIntersect} />

<!-- slots (components only) -->
<slot name="leading"></slot>
<slot></slot>
```

The file IS the view — no `<component>` wrapper. Top-level directives (`<data>`, `<prop>`, `<use-*>`) declare scope; the rest is the body. Reactive component state is declared in the paired controller's `scope:` factory, not the template.

For full examples see `references/template-surface.md`.

## Controller contract

```ts
// view.controller.ts
import { controller, signal, derived, shared, useShared, useFilteredList, unwrap } from '@ship/sui-runtime'
import type { Scope } from './view.sui'   // compiler-emitted .d.ts

export default controller<Scope>({
  scope: ({ agents }) => {
    const filter = signal('')
    const total  = derived(() => unwrap(agents)?.items.length ?? 0)
    const list   = useFilteredList(agents, (a, q) => a.name.includes(q))
    return { filter, total, list }
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
  mount:   ({ state }, el) => { /* runs after first DOM attach */ },
  unmount: ({ state })     => { /* runs on dispose */ },
})
```

Three blocks, each one job:

- **`scope:`** — factory that contributes typed signals, derived, hooks, shared providers and consumers. Runs first; entries land in template scope. Replaces the historical state/derived/use split.
- **`on:`** — multi-statement event handlers. The first arg is a writable proxy: `s.count = s.count + 1` mutates state.
- **`mount:` / `unmount:`** — lifecycle. Runs after first attach / on disconnect.

## Cross-component reactive context

For complex components that split into a public outer + internal inner pieces, share signals from parent to descendants without prop-drilling. The parent calls `shared(signal(…))` inside its `scope:`; descendants call `useShared<ParentScope>('parent-tag')` to resolve the same signal identity. `useShared` walks DOM ancestry across shadow boundaries; returns `null` when no provider is found.

Two controller tiers — smallest that fits: **no controller** (pure `.sui`, single-expression inline handlers only) or **paired controller** (state, derived, hooks, lifecycle, multi-statement handlers). A multi-statement `on:event` in a tier-1 file is a compile error.

Full contract, worked examples, lookup semantics: `references/shared-context.md`.

## Form-aware components

Component `.sui` files only — views don't participate in form submission. When a `<sui-*>` component is itself a form control, mark its value prop with `form-value` and (optionally) its disabled prop with `form-disabled`:

```html
<prop name="value" type="string" default="" form-value />
<prop name="disabled" type="bool" default="false" reflect form-disabled />
```

The compiler emits `static formAssociated = true` and the runtime auto-wires `setFormValue`, `formResetCallback`, `formStateRestoreCallback`, and `formDisabledCallback`. Drop any hand-rolled `ElementInternals` plumbing — the runtime owns it. The setup ctx still surfaces `internals?: ElementInternals` for `setValidity()` / `checkValidity()` paths.

Worked examples: `packages/components/ui-components/src/sui-input/sui-input.sui`, `packages/components/ui-components/src/sui-select/sui-select.sui`. Full contract and constraints: `references/form-association.md`.

## Live data envelope

```ts
type LiveState<T> = { kind: 'loading' } | { kind: 'error'; error: Error } | { kind: 'ok'; value: T }

unwrap(sig)               // T | undefined
unwrapOr(sig, fallback)   // T
```

```html
{#while-loading of agents}<p>Loading…</p>{/while-loading}
{#while-error err of agents}<p>{err.message}</p>{/while-error}
{#if agents().kind === 'ok'}
  {#for a of agents().value.items key={a.id}}…{/for}
{/if}
```

## Live SSE data

The full end-to-end loop for `<data live>` — what's on the wire, where each
piece lives, and how to confirm it locally.

### Wire path

```
.sui  <data live source="ship.git.overview()" live />
  ↓ sui-compiler
.js   $scope.git = live(() => ship.git.overview())
  ↓ @ship/sdk-client createClient()
HTTP  GET /_sui/data?expr=ship.git.overview()    [text/event-stream]
  ↓ apps/shipd/src/views/sui_data.rs
Rust  registry::call("git.overview", {}) → initial; subscribe `config` ns
  ↓ kernel event → re-resolve
SSE   event: patch \n data: {"op":"set","value":{…}}
  ↓ makeSseSource() in @ship/sdk-client
sig   LiveSignal<T> flips loading → ok → ok (per patch)
  ↓ bindWhileLoading / bindWhileError / bindIf
DOM   morphs in place — no re-render of the surrounding view
```

The compiler turns each `source=` attribute into a literal expression
inside `live(() => …)`. The expression must resolve to a `LiveSource<T>`
(has `.subscribe(sig)`) — that's the shape every SDK method returns, so
`ship.<domain>.<op>(args)` is the idiomatic form. Don't hand-roll an
`EventSource` in the controller; let the SDK own the wire.

### Worked example: `views/home/`

The home view ships the canonical demo. `<data live>` subscribes to
`ship.git.overview()`; the daemon's git_signal singleton refreshes on
filesystem events under `.ship/` and on git ops, so staging a file in
the worktree visibly flips the "clean / N changed" badge without a
page reload.

```html
<!-- views/home/page.sui -->
<data name="git" source="ship.git.overview()" live />

<section class="home-live" aria-live="polite">
  {#while-loading of git}
    <p>Connecting…</p>
  {/while-loading}
  {#while-error err of git}
    <p>SSE error: {err.message}</p>
  {/while-error}
  {#if git().kind === 'ok'}
    <p>
      Branch <strong>{branch()}</strong>
      · <span>{headShort()}</span>
      · <span>{clean() ? 'clean' : changedSummary()}</span>
    </p>
  {/if}
</section>
```

```ts
// views/home/controller.ts
import { controller, signal, unwrapOr } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'

const EMPTY: GitOverview = { branch: '', head_short: '', clean: true,
  total_changed: 0, staged: [], modified: [], untracked: [],
  commits: [], branches: [], ahead: 0, behind: 0, workspace_id: null }

const read = ({ git }: Scope) => unwrapOr(git, EMPTY)

export default controller<Scope>({
  scope: () => ({ seen: signal(0) }),
  derived: {
    branch:    (s) => read(s).branch,
    headShort: (s) => read(s).head_short,
    clean:     (s) => read(s).clean,
    changedSummary: (s) => `${read(s).total_changed} changed`,
  },
})
```

### Verify locally

```bash
just dev-restart            # rebuilds shipd + restarts the daemon
PORT=$(cat $(./target/debug/ship daemon dir)/port)

# 1. Confirm the SDK channel is live (Ctrl-C after a few `patch` events).
curl -N "http://localhost:$PORT/_sui/data?expr=$(printf 'ship.git.overview()' | jq -sRr @uri)"

# 2. Open the view, stage a file, watch the badge flip.
xdg-open "http://localhost:$PORT/"            # or open in your browser
touch CHANGE_ME && git add CHANGE_ME           # status flips to "1 changed"
git restore --staged CHANGE_ME && rm CHANGE_ME # back to "clean"
```

### When to add a new live source

A new `ship.<domain>.<op>(...)` live source needs three things:

1. **Registry entry** — `apps/shipd/src/services/registry/<domain>.rs` returns
   the current value synchronously. The dispatcher in `registry.rs` and the
   schema row in `services/schema.rs` keep the catalog honest.
2. **SDK method** — `packages/deck/sdk-client/src/index.ts` exposes the
   method returning `makeSseSource(...)` against `/_sui/data?expr=…`.
3. **(Optional) signal-backed source** — if the data has a tighter wake-up
   surface than the default `config` kernel namespace, mirror the pattern in
   `apps/shipd/src/views/git_signal.rs`: a `Signal<Value>` plus a singleton
   background task that updates it. `sui_sse.rs::is_signal_backed_expr` picks
   up the new registry name automatically.

The view-side surface (`<data live source="ship.X.Y()" live />` + controller
derived helpers) is the same regardless of which mechanism powers the
re-resolves on the daemon.

## Manifest

```jsonc
// views/<name>/manifest.jsonc — JSONC, the house format
{
  "name": "my-view",        // stable identity; matches directory
  "path": "/my-view",       // URL pattern — `/path` or `/path/[param]`
  "title": "My View",       // <title> when mounted
  "layout": "default",      // layout slot key
  "auth": "public",         // "public" | "user" | "admin"
  "rail": {                 // optional — shell rail entry
    "label": "My View",
    "icon": "sparkles"
  },
  "preload": [              // optional — SDK calls warmed before mount
    "ship.workspaces.list()"
  ],
  "events": [               // optional — SSE channels to subscribe to
    "config.changed"
  ]
}
```

The shell's route mounter (`apps/shell/src/main.ts::mountRoute`) walks `views/` at build/start time and registers each `manifest.jsonc`. URL → view module resolution and `[param]` capture happen there. For daemon route shapes, the SSR + attach pipeline, and the marker contract, see `references/manifests-routes.md`.

## Hard rules

1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.
2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>`.
3. **Templates declare; controllers do.** Declarative → template; imperative → controller.
4. **No `<script>` blocks in templates.** Logic in paired `.controller.ts`.
5. **No `customElements.define` in view code.**
6. **Read signals by call:** `count()` reads, `count.set(n)` writes.
7. **Block syntax for control flow.** `{#if}/{#for}/{#match}/{#while-loading}/{#while-error}`.
8. **No `el.innerHTML/.outerHTML/.insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()` for HTML-blob cases.
9. **`{#for}` blocks declare `key={EXPR}`.** Without a stable key the reconciler falls back to positional matching and DOM identity (focus, selection, in-place edits) is lost on reorder. `sui-for-key` lint warns when missing.
10. **`on:<event>` event names must match the platform whitelist:** DOM events (`click`, `input`, …), `sui-*` / `ship-*` custom-event families, and `tag:event`-prefixed cross-component events. `sui-event-name` lint warns on typos like `on:Click` with Levenshtein suggestions.
11. **Form-aware components use `form-value` / `form-disabled` on `<prop>`** — never hand-roll `attachInternals()` / `setFormValue` / `formResetCallback`. See "Form-aware components" above.

## Performance gates

Three recipes guard runtime + visual surface before a PR:

- `just bench` — capture a fresh js-framework-benchmark baseline; commit the
  resulting `tests/perf/baselines/<date>-baseline.json`.
- `just bench-check` — re-runs the bench suite against the current bundle and
  fails if any op regresses >15% vs the latest committed baseline. Override
  the threshold via `BENCH_REGRESSION_THRESHOLD`. Run before merging any
  runtime-affecting change.
- `just visual` — Playwright screenshots vs committed PNGs (daemon must be
  running). Pass `--update-snapshots` to refresh baselines after a
  deliberate visual change.
- `just smoke-deploy` — full build + daemon restart + every baselined view
  curled + Playwright visual suite. Proxy for what CI will see; aim for
  green before opening a PR.

## Agent surfaces (MCP)

Every agent-facing tool relevant to view authoring. Full catalog with
groupings + the LSP CLI + `@ship/sui-eval` scoring lives in
`docs/ship-sui/references/docs/agent-surfaces.md`.

### Scaffold + diagnose loop

| Tool | When |
|---|---|
| `ship ops call create.surface --json '{"kind":"view","name":"<name>"}'` | Emit a fresh `views/<name>/` skeleton. |
| `ship sui diagnostics <file>` | First thing after writing. Returns `[{ line, col, severity, message, source }]`. Same analyser as the VS Code LSP. |
| `ship sui hover <file> <line> <col>` | Type / directive / prop info at a cursor. Use when you're not sure what shape a binding wants. |
| `ship sui dependencies <file>` | What tags / signals / routes this file reaches into. Read before edits to scope blast radius. |
| `TODO(#207-followup): verify replacement for mcp__ship__view_compile_check` | Full lowering pipeline. Catches issues `sui_diagnostics` (parse + analyse only) misses. |
| `TODO(#207-followup): verify replacement for mcp__ship__get_view_errors` | Runtime errors from a mounted view session (post-mount, not compile). |

### Source I/O + snapshots

| Tool | When |
|---|---|
| Read tool on `views/<name>/` | Read one file or the full view dir. |
| Edit/Write tools on `views/<name>/` | Write — triggers recompile + SSE notify. |
| `TODO(#207-followup): verify replacement for mcp__ship__view_list_siblings` | Sibling `_components/<view>-<tag>/` widgets. |
| `TODO(#207-followup): verify replacement for mcp__ship__view_list_snapshots` | Timeline — every save creates one. |
| `TODO(#207-followup): verify replacement for mcp__ship__view_restore_snapshot` | Revert; non-destructive. |

### Live mount + poke

| Tool | When |
|---|---|
| `TODO(#207-followup): verify replacement for mcp__ship__open_view` | Mount in the agent's session shell; returns a `session_id`. |
| `ship ops call view.patch_session --json '{"selector":"...","html":"..."}'` | Morph an HTML subtree in place. |
| `ship ops call view.signal_session --json '{"name":"...","value":...}'` | Set a reactive signal as if the user did via UI. |
| `TODO(#207-followup): verify replacement for mcp__ship__view_preview_snapshot` | PNG + DOM dump — for evals, QA, before/after diffs. |
| `TODO(#207-followup): verify replacement for mcp__ship__close_view` | Release. |

### Theme + collections

| Tool | When |
|---|---|
| `TODO(#207-followup): verify replacement for mcp__ship__view_theme_tokens` | Active theme's tokens. Read before authoring CSS so you stay token-locked. |
| `ship ops call collections.list` | Schemas for project-defined `<data source="…">` collections. Use before writing a `<data>`. |
| `ship ops call annotation.post --json '{"..."}'` | Pin a TODO/comment to a DOM node; surfaces in the dev panel. |

### Verify before "done"

**The gate is the compiler, and it is always available. Run it.** Do not skip
this because an MCP tool is missing, and never substitute `tsc` — Ship does not
bundle TypeScript. Controller checking is the oxc-based lint pass *inside* the
compiler (`crates/sui/lint`), so the compiler is the one source of truth for
both template and controller correctness.

**Gate 0 — `sui-compiler build` (zero setup, no MCP, no daemon).** The compiler
ships on PATH (`~/.cargo/bin/sui-compiler`, installed by `just install-dev`).
It runs grammar + semantic checks **and the full lint pass** (the same rules the
LSP and `view_lint` surface, e.g. `sui-css-content-text`, `sui-controller-purity`).
A non-zero exit / printed `Error:` is a hard fail — fix and re-run until clean:

```sh
sui-compiler build views/tasks/page.sui     # one file
sui-compiler build views/tasks/             # whole view dir (template + siblings)
```

This is the gate you can *always* run — in any session, with or without the
Ship MCP server connected. If you only do one thing before declaring done, do this.

> **The MCP tool surface has been removed.** Use `sui-compiler build` as the
> durable gate. `@ship/sui-eval` (`runEval({ description, spec, catalog })`) remains
> the behavioural-scoring path when you authored against an interaction-script + asserts
> contract.

```sh
# The verify loop:
# 1. write template + controller
# 2. sui-compiler build views/tasks/   # the gate — fix every error, repeat until clean
# 3. web-qa skill                       # render + console + theme evidence
```

## LSP integration

The same analyser that backs `sui_diagnostics` is exposed as a standalone
language server.

| Surface | How |
|---|---|
| **CLI** (agents) | `ship sui diagnostics` / `hover` / `dependencies`. See above. |
| **VS Code** | `apps/vscode/` auto-launches the `sui-lsp` binary on `.sui` open. |
| **Any editor / Claude Code / opencode** | Point the LSP client at the `sui-lsp` binary (`crates/sui/lsp/`, on PATH via `just install-dev`) over stdio. Claude Code uses the repo `.lsp.json`. |
| **Browser** | `@ship/sui-lsp-browser` runs the WASM `analyze` (same Rust grammar) for diagnostics in the docs playground; hover/completion are static-dictionary for now. |

## Components

Views compose `<sui-*>` components from `packages/components/ui-components/`. See the `ship-components` skill for the component contract.

## Error handling

Wrap fault-prone subtrees in `<sui-error-boundary>`. Throws inside the
wrapped subtree (effects, controller mount, render expressions) are caught
and the fallback is rendered. Use `fallback-text` for a string fallback or
`<template slot="fallback">` for custom markup. Call `.reset()` on the host
element to retry.

```html
<sui-error-boundary fallback-text="Couldn't load feed">
  <sui-some-component prop={maybeThrows()}></sui-some-component>
</sui-error-boundary>
```

The boundary fires `sui-error-boundary:caught` (bubbling, composed) with
`{ error, source }` where `source` is `'effect' | 'mount' | 'render' |
'unknown'`. Nest boundaries to scope recovery — the innermost catches
first, mirroring React.

## Portal

Use `<sui-portal target="body">` to teleport its slotted content into a
different DOM location (typically `document.body`) for stacking-context
escape — modals, dropdowns, popovers, toasts. The portal element stays
the logical parent, so reactive bindings inside continue to fire and
events bubble through the original DOM ancestry, not the target's.

```html
<sui-popover-trigger>Open</sui-popover-trigger>
<sui-portal target="body">
  <sui-popover-panel>Floating content</sui-popover-panel>
</sui-portal>
```

`target` is a CSS selector resolved against `document`; the default is
`body`. Common alternatives: `#modal-root`, `[data-portal-root]`. A
missing target warns to the console and leaves children in place
(graceful degradation). Multiple portals targeting the same selector
append in registration order — z-index ordering remains the author's
job. Teleported nodes are removed from the target when the portal
itself unmounts.

## Transitions

Wrap elements with `transition:<name>` to animate enter/exit on `{#if}`
or `{#for}` mount/unmount. Built-ins: `fade`, `slide`, `scale`. Use
`in:<name>` + `out:<name>` for asymmetric directions. Custom transitions
export `{ in?, out? }` matching the `Transition` type.

```html
{#if visible()}
  <div transition:fade={{ duration: 200 }}>...</div>
{/if}

<!-- Asymmetric: slide in, fade out -->
{#if visible()}
  <div in:slide out:fade={{ duration: 150 }}>...</div>
{/if}
```

The compiler auto-imports built-in names from `@ship/sui-runtime`;
custom transitions must be on the controller's `scope:` factory.
Out-transitions defer DOM removal until the animation finishes; a
rapid toggle while an out-transition is in flight rescues the same
nodes so the in-transition picks up from current state without
re-rendering. `prefers-reduced-motion: reduce` collapses duration and
delay to `0` automatically. Transitions don't surface in SSR output —
they register on element construction (CSR + hydrate paths).

## When components use these — real-world exemplars

The portal + transition primitives are composed by three platform
components. Read them when you're authoring a new overlay-style
primitive:

| Primitive | Portal role | Transition |
|---|---|---|
| `sui-dialog` | `<sui-portal target="body">` teleports a `[data-sui-dialog-open]` marker so page-level styles can react to "any modal open" via `:has` selectors. The dialog element itself stays in shadow because native `showModal()` already escapes parent stacking context via top layer. | `transition:fade={{ duration: 150 }}` on the #if-mounted content wrapper drives open/close polish. |
| `sui-popover` | Same pattern — `[data-sui-popover-open]` marker portaled to body. Native `popover="auto"` handles top-layer + light dismiss; the marker is for page-level hooks. | `transition:scale={{ duration: 120 }}` on the #if-mounted `.surface`. |
| `sui-toast-container` | `<sui-portal target="body">` teleports the actual stack of toasts. Toast containers *genuinely* need to escape ancestor `transform` / `overflow` containing blocks — `position: fixed` alone isn't enough. The slot stays in shadow; the controller adopts slotted + `show()`-created toasts into the body-side stack. | Each `<sui-toast>` uses asymmetric `in:slide={{ axis: 'y' }}` + `out:fade` on its `#if visible()` panel. The controller flips `visible` to false on `close()` and tears the host down after the out-transition. |

The takeaway: where the native platform already provides top-layer
rendering (`<dialog>.showModal()`, `popover="auto"`), `<sui-portal>`
is best used for *document-level state markers* that page chrome can
target — not for re-portaling the panel itself, which would break
slot projection (slots only project inside their host's shadow tree;
once teleported they detach). Where the native platform does NOT (a
fixed-position toast stack inside a transformed ancestor),
`<sui-portal>` is the right tool to teleport the whole thing.

## Rail composition

`<sui-rail>` is the left chrome container; views are addressed via
slotted children. Author the rail in the shell HTML once and let the
controller handle features (project switcher, session-count badge,
launcher / create, active-view tracking, click-outside dismiss).

```html
<!-- apps/shell/index.html — single static instance -->
<sui-rail id="ship-rail">
  <sui-rail-item icon="layout-dashboard" label="Dashboard" href="/" data-view="dashboard"></sui-rail-item>
  <sui-rail-item icon="bot" label="Agents" href="/agents" data-view="agents"></sui-rail-item>
  <sui-rail-item icon="git-branch" label="Git" href="/git" data-view="git"></sui-rail-item>
  <sui-rail-item icon="sparkles" label="UI Studio" href="/ship-ui-studio" data-view="ship-ui-studio"></sui-rail-item>
</sui-rail>
```

Three composition rules:

1. **`data-view` is the active-view key.** The rail's controller
   listens for `sui:navigate` (and `popstate`) and reads
   `location.pathname.split('/').filter(Boolean)[0]`. The matching
   `<sui-rail-item[data-view]>` child gets the `active` attribute set;
   all others have it cleared. No manual wiring in the shell.
2. **Mission Control is baked in.** `<sui-rail>` renders its own
   `<sui-rail-item part="mission-control" data-view="mesh">` inside
   the nav after the slot. The controller polls `/api/v1/pty-sessions`
   every 15s and reflects the live count to the item's `badge`
   attribute (suppressed at 0). Don't re-slot a Mission Control item
   from the shell — it would collide.
3. **Footer features dispatch bubbled events.** Project badge,
   launcher, create — every footer action emits a composed bubbling
   event the shell can listen for at the document level:
   `sui-rail:toggle`, `ship:launcher:open`, `ship:create:open`,
   `ship:project:change`. The project popover is wrapped in
   `<sui-portal target="body">` with `transition:fade={{ duration: 150 }}`
   so it escapes the rail's `overflow: hidden` + stacking context
   cleanly.

The rail controller fetches `/api/v1/projects` for the switcher and
positions the portaled popover against the project-badge bounding
rect (updates on `resize` / capturing `scroll`). When you need the
selected project, listen on `document` for `ship:project:change`.

## Morph (in-place DOM patching)

Add the `morph` directive to any element whose contents are produced
by `bindRawHtml` (or another HTML-string-replacing path) to preserve
focus, scroll, selection, and event-listener identity across
re-renders:

```html
<div morph>
  {/* server-rendered HTML — bindRawHtml in the controller */}
</div>
```

Without the directive, `bindRawHtml` replaces `innerHTML` and destroys
any in-progress UI state on the children. With the directive,
`bindRawHtml` routes updates through `morph()` — same-id elements are
reused (even across reorders), attributes are swapped only when they
change, and only truly-different subtrees are touched.

`bindHtml` always morphs (no directive required); the directive only
matters for `bindRawHtml` and any future HTML-producing binding.

Limitations: doesn't preserve `<select>` value across morphs (browser
quirk), doesn't morph into shadow DOM, matches only by `id` and
tag-plus-position (no content-similarity heuristics).

## References

| When you need | Read |
|---|---|
| Worked template examples, controller tiers, hooks, `<prop>` attribute reference, manifest schema, daemon routes, SSR + attach pipeline | `references/template-surface.md` |
| Form-aware components — full contract, constraints, controller access to `internals` | `references/form-association.md` |
| Cross-component reactive context (`shared` / `useShared`), controller tiers | `references/shared-context.md` |
| Working on the SUI codebase itself (compiler, runtime, daemon SUI handlers) | `references/sui-internals.md` |

## Hot reload

In dev mode, `.sui` and `.controller.ts` edits trigger an in-place view
re-mount without a full page reload. shell's boot calls
`initHmr()` (apps/shell/src/hmr.ts), which uses
`subscribeHmr` from `@ship/sui-runtime/dev` to listen on shipd's
`/_sui/dev/hmr` SSE channel. When the Bun bundler emits
`module-changed` for `views/<name>/view` and `<name>` matches the
current route, shell cache-busts the rebuilt `view.js` and re-mounts
through the existing `mount()` lifecycle.

Other bundles (sui-components, sui-components/*, sui-runtime, sui-dom,
ship-elements) trigger a full reload — custom-element re-registration
without reload isn't yet supported. Compile errors are logged to the
browser console with `file:line`. Production builds inline
`import.meta.env.DEV` to `false`, so HMR is a no-op and no
EventSource opens.

Run `just dev` and edit any view file under `views/<name>/` to see it
apply in place.

## Routes

URL state is reactive. Import from `@ship/sui-runtime/route`:

```ts
import { controller } from '@ship/sui-runtime'
import { route, defineRoutes, navigate } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

defineRoutes([{ pattern: '/agents/:id' }])

export default controller<Scope>({
  scope: () => {
    const r = route()
    return {
      agentId: r.params.id,             // Signal<string | undefined>
      tab:     r.query.tab,             // Signal<string | undefined>
      goHome:  () => navigate('/'),
    }
  },
})
```

`route().params.<name>` and `route().query.<name>` are signals — call them
to read (and subscribe) inside `effect`, `derived`, or a `<data>` source
thunk. Param keys are lazy: reading `r.params.id` the first time creates
the signal and seeds it from the current pathname. `defineRoutes([...])`
registers patterns whose `:param` segments populate `params`; safe to
call any number of times. Mutate the URL via `navigate(path)` —
assigning to `params`/`query` throws. `popstate` and `navigate` both
fire the global updater so every attached `route()` instance re-evaluates.

## Controller scope is typed

The compiler emits `Scope` in `<file>.sui.d.ts` covering BOTH template
reactive identifiers AND the keys you add via `scope: () => ({...})`
factory. `mount(state, host)` sees direct, typed members:

```ts
mount(state, host) {
  state.caughtError.set(null)  // Signal<Error | null>
  state.reset()                 // () => void
}
```

No casts. No "scope signals missing" defensive throws. If a key is
mistyped or missing, the TypeScript compiler catches it.
