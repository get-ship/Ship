---
name: ship-sui
stable-id: ship-sui
description: SUI — Ship's `.sui` template format. Authoring guidance, the four-layer architecture (compiler / runtime / daemon view-host / authoring), and the supporting tooling (`sui-lsp`, `sui-build`, UI Studio, devtools, dispatch).
tags: [ship, sui, views, components, runtime, compiler]
authors: [ship]
audience: agent-only
---

# Ship SUI

SUI is the template format every Ship view and built-in component is authored in. A `.sui` file is a declarative template; an optional sibling `controller.ts` owns reactive state, lifecycle, and async I/O. The compiler in `crates/sui/compiler/` turns the pair into an ESM module that imports from `@ship/sui-runtime` (`packages/sui/runtime/`) — there is no parser at runtime. The view-host — `crates/deck/deck-core/` (transport-agnostic core), bound to HTTP by `crates/deck/deck-axum/` — resolves a route, serves the compiled module + a static SSR snapshot, and hydrates client-side by calling `mount(target, payload, { ship })`. That `mount` call injects a real `ship` SDK client into controller ctx (`crates/deck/deck-core/assets/view-host.html`), so daemon-served views get the same client the shell provides.

**The boundary (read this).** SUI : Deck : Ship is analogous to React : Next : your app. **SUI** (`crates/sui/*`, `packages/sui/*`) is the framework — compiler + runtime + reactivity — and must know nothing about Ship's daemon or its URLs. **Deck** (`crates/deck/*`, `packages/deck/*`) is the host/meta-framework that mounts SUI views, does SSR/routing, serves assets, and supplies the data-source client. **Ship** (`apps/shipd`, `crates/core`) is the app: the daemon, the domains, the data. Dependencies point one way only: deck → sui, ship → deck; the compiler depends on nothing downstream.

> The body below is the on-ramp — enough to author a first component / view in 30 minutes. For deep dives see `references/`.

## The four layers

| Layer | Owns | Entry |
|---|---|---|
| Compiler (SUI) | Parse → check → lower → emit (`.js`, `.css`, `.sui.d.ts`, `.sui.meta.json`, `.ssr.html`). Pure transform — no Ship/deck knowledge. | `crates/sui/compiler/src/lib.rs:185` (`compile(src, opts) -> CompileOutput`) |
| Runtime (SUI) | Reactive primitives, DOM bindings, control flow, transitions, error-boundary, portal, morph, `mount()` / `attach()` | `packages/sui/runtime/src/index.ts` |
| View-host (Deck) | Route → manifest → compiled JS + SSR HTML → CSS + assets → live data SSE; injects the `ship` client at mount | `crates/deck/deck-core/src/{view,view_host,resolver}.rs` (core) + `crates/deck/deck-axum/src/handlers.rs` (HTTP adapter) + `apps/shipd/src/views/sui_*.rs` (Ship wiring) |
| Authoring | `.sui` + optional `.controller.ts` + `style.css` + `manifest.jsonc` (views only) | `views/<name>/`, `packages/components/ui-components/src/<tag>/`, `.ship/views/<name>/` (project-local) |

Detail: `references/4-layer-overview.md`.

## The 2-file bet

Two files, paired by directory + filename stem:

```
views/<name>/
  page.sui            # template — declares scope + body, route entry
  controller.ts       # optional — state, derived, on:, mount, unmount
  style.css           # optional — page-scoped, auto-linked by the daemon
  manifest.jsonc      # required — { name, path, title, layout, auth, rail? }
  dist/               # compiler output (gitignored)
  _components/<tag>/  # view-private widgets; tag MUST prefix with view name
```

Components live under `packages/components/ui-components/src/<tag>/` (first-party) or are private to a view under `_components/`. Same shape: `<tag>.sui` + optional `<tag>.controller.ts` + optional `<tag>.css`. No `manifest.jsonc` for components.

**Templates declare. Controllers do.** A controller that emits markup is a bug — the `sui-controller-purity` lint rejects `el.innerHTML`, `outerHTML`, and `insertAdjacentHTML` in `*.controller.ts`. Use `bindHtml(el, () => string)` for HTML-blob cases (markdown, third-party SVG).

Scaffolds: `deck scaffold view <name>` and `deck scaffold component <tag>`.

## Minimal pair

```html
<!-- views/hello/page.sui -->
<data name="git" source="ship.git.overview()" live />

<div class="hello">
  <h1>Hello {who()}</h1>
  <p>Branch <strong>{branch()}</strong> · {clean() ? 'clean' : 'dirty'}</p>
  <button on:click={() => who.set('world')}>Wave</button>
</div>
```

```ts
// views/hello/controller.ts
import { controller, signal, unwrapOr } from '@ship/sui-runtime'
import type { GitOverview } from '@ship/sdk-client'
import type { Scope } from './page.sui'

const EMPTY: GitOverview = { workspace_id: null, branch: '', head_short: '',
  clean: true, total_changed: 0, staged: [], modified: [], untracked: [],
  commits: [], branches: [], ahead: 0, behind: 0 }

export default controller<Scope>({
  scope: () => ({ who: signal('there') }),
  derived: {
    branch: ({ git }) => unwrapOr(git, EMPTY).branch,
    clean:  ({ git }) => unwrapOr(git, EMPTY).clean,
  },
})
```

```jsonc
// views/hello/manifest.jsonc
{ "name": "hello", "path": "/hello", "title": "Hello", "layout": "default", "auth": "public" }
```

`Scope` is generated by the compiler from the template — both `<data>` names and the controller's `scope:` keys appear as typed fields. No casts, no defensive throws.

Worked exemplar: `views/home/page.sui` + `views/home/controller.ts`.

## Reactive primitives (cheat sheet)

```ts
import { signal, derived, effect, batch, untrack } from '@ship/sui-runtime'

const count = signal(0)              // Signal<T> — `count()` reads, `count.set(n)` writes
const doubled = derived(() => count() * 2)
effect(() => console.log(doubled())) // re-runs on tracked-dep change
batch(() => { count.set(1); count.set(2) }) // one flush
untrack(() => count())               // read without subscribing
```

Full surface (control flow, transitions, slots, refs, `<data live>`): `references/runtime-primitives.md`.

## Template surface (compact)

```html
<!-- top-level directives (declare scope) -->
<data name="agents" source="ship.agents.summaries()" live />
<prop name="tone" values="default|primary" default="default" reflect />
<use-shell as="shell" />

<!-- dynamic routes — read params from the shared route signal -->
<!-- (host must pre-register the pattern via defineRoutes([{pattern:'/x/:id'}])) -->
<!-- controller: const id = derived(() => route.params.id() ?? '') -->
<!-- controller: const detail = live(() => ship.x.detail({id: id()})) -->
<!-- template: {#if detail().kind === 'ok' && detail().value}{detail().value.name}{/if} -->

<!-- control flow — block syntax -->
{#if loading()}…{:else if error()}…{:else}…{/if}
{#for a of items() key={a.id}}<li>{a.name}</li>{/for}
{#match status()}{:case 'idle'}…{:case 'ok'}…{/match}
{#while-loading of agents}…{/while-loading}
{#while-error err of agents}{err.message}{/while-error}

<!-- bindings -->
<input bind:value={query} />
<button on:click={() => count.set(count() + 1)}>+</button>
<div class:active={isActive()} style:color={color()} attr:href={url()}>
  {count() === 1 ? '1 item' : count() + ' items'}
</div>

<!-- composing components — pass signals / objects via prop:* -->
<sui-grid prop:rows={rows} prop:columns={columns()}></sui-grid>

<!-- write-side — form mutation -->
<form mutation="ship.workspaces.activate">
  <input name="id" />
  <button type="submit">Switch</button>
</form>
```

The file IS the view / component — no `<component>` wrapper, no `<script>` block. Detail: `references/authoring-patterns.md`.

## Live data — `<data live>`

`<data name="x" source="ship.<domain>.<op>(args)" live />` opens an SSE channel at `/_sui/data?expr=…` against the daemon (`apps/shipd/src/views/sui_data.rs`). The signal flips `loading → ok → ok` as patches arrive; `bindWhileLoading` / `bindWhileError` / `{#if x().kind === 'ok'}` cover the three states. The SDK in `packages/deck/sdk-client/src/index.ts` is the only sanctioned place that constructs SSE — never hand-roll `EventSource` in a controller, and never reach the wire by raw expression string (the `liveExpr` escape hatch is being removed; use the typed `ship.<domain>.<op>()` client).

Worked end-to-end loop (wire path, registry, signal-backed sources, local verification): `references/runtime-primitives.md` § Live SSE data.

## SDK access — typed responses, hand-written client surface

`ship.<domain>.<op>(args)` calls go through `@ship/sdk-client` (`packages/deck/sdk-client/src/index.ts`). Two halves, and the distinction matters:

- **Response types** live in `packages/sui/runtime/src/generated.ts`, emitted by `cargo xtask gen-types` from Rust source — do not edit by hand.
- **The client method surface** (the `ship.<domain>.<op>` object) is a **hand-maintained object literal** in `packages/deck/sdk-client/src/index.ts`. It is NOT generated. Domains present today: `account`, `agents`, `artifacts`, `canvas`, `chat`, `collections`, `dashboard`, `events`, `git`, `settings`, `sessions`, `skills`, `workspaces`.

**Known gap (the source of `liveExpr`):** because the client surface is hand-written, daemon ops can exist on the wire while being absent from the client — `ship.pipelines.*` is served by the daemon but is **not** in the client, so the call form throws. The fix in flight is a generated client; until it lands, a missing surface is a real gap, not an invitation to hand-roll SSE.

**Known gap:** `ship.skills.invoke()` is wired in the SDK but the daemon route may reject with `{ code: 'not_implemented', op: 'skills.invoke' }`. Callers must handle the `not_implemented` code as a first-class state.

Mutations are HTTP POST/DELETE through the same client (e.g. `ship.sessions.start({...})`, `ship.workspaces.activate({...})`); use `<form mutation="…">` for the declarative path.

## Tooling

| Tool | What it does | Entry |
|---|---|---|
| `sui-compiler` (bin) | One-shot compile of a `.sui` file; runs the full lint pass and exits non-zero on errors. Used by `sui-build` and editor extensions. | `crates/sui/compiler/src/bin/sui-compiler.rs` (`cargo run -p sui-compiler -- build <file>`) |
| `sui-lint` (example bin) | Lint one file from the CLI — the fast per-edit check now that MCP is gone. Exits 1 on findings. | `cargo run -p sui-lint --example check -- <file>` |
| `@ship/sui-build` | CI / package-build bundler (NOT the daemon path) — discovers `.sui` sources, calls the compiler, runs Bun/esbuild adapter, optional `--watch`. The daemon + framework bake use the in-process Rolldown bundler (`cargo xtask build-assets`, #163). | `packages/deck/build/bin/sui-build.mjs` |
| `@ship/sui-lsp` | LSP server — diagnostics + hover for `.sui`. TS server `packages/deck/lsp/`; Rust core `crates/sui/lsp/`. | `packages/deck/lsp/src/` |
| Devtools panes | In-page panels: template source, components tree, signals, lint, schema. | `packages/components/ui-components/src/sui-devtools/` |
| `dispatch.sh` | Worktree + job-spec + agent compile + terminal-open script. | `.ship/skills/spawn-agent/scripts/dispatch.sh` |
| Drift detection | `GET /api/_meta` returns `{compilerVersion, daemonVersion, builtAt, git, dirty}`. Every compiled artifact carries a `_compilerVersion` marker (leading `/*! */` comment in JS, sidecar field in CEM/schema/meta). Shell compares its loaded bundle against the daemon to surface stale-bundle drift. | `crates/deck/deck-core/src/meta.rs` |

Detail: `references/tooling.md`.

## Hard rules

1. **Mutations in `on.*` handlers only.** `s.x = v` in a derived or template expression is a type error.
2. **Subscriptions in `<data>` elements.** Never call `ship.*` outside `<data>` (or inside an `on:` mutation form).
3. **Templates declare; controllers do.** Declarative → template; imperative → controller.
4. **No `<script>` blocks in templates.** Logic in paired `.controller.ts`.
5. **No `customElements.define` in view code.** The compiler emits it for components.
6. **Read signals by call:** `count()` reads, `count.set(n)` writes.
7. **Block syntax for control flow.** `{#if}/{#for}/{#match}/{#while-loading}/{#while-error}`.
8. **No `el.innerHTML / .outerHTML / .insertAdjacentHTML`** in `*.controller.ts`. Use `bindHtml()`.
9. **`{#for}` blocks declare `key={EXPR}`.** Without a stable key DOM identity is lost on reorder. `sui-for-key` lint warns when missing.
10. **`on:<event>` event names match the platform whitelist** (DOM events, `sui-*` / `ship-*` custom events, `tag:event` cross-component). `sui-event-name` lint warns on typos.
11. **Guard live-signal `.value` for null in detail/get ops.** `ship.<domain>.detail({id})` returns `null` on miss — `unwrap(sig)` resolves to `null`, so a bare `sig().value.name` crashes. Use `{#if sig().kind === 'ok' && sig().value}` or pre-compute via `derived(() => unwrap(sig)?.name ?? '')` in the controller.
12. **Dynamic routes need `defineRoutes` at shell startup.** Views under `views/<name>/[id]/` only see `route.params.id()` if the host registered the pattern via `defineRoutes([{pattern: '/<name>/:id'}])`. View-bundle imports of `@ship/sui-runtime/route` live in a different module instance from the shell's, so per-controller `defineRoutes` calls don't reach the shell's signal proxy.

## Before declaring an edit done

MCP is gone — the loop is the binary. Run it per edit, not at PR time:

```sh
# Fast per-edit lint (grammar, missing bindings, unstable keys, purity,
# css tokens, prop values). Exits 1 on findings — wire it into your loop:
cargo run -p sui-lint --example check -- views/hello/page.sui

# Full compile (lint folds into the diagnostic stream; aborts on errors):
cargo run -p sui-compiler -- build views/hello/page.sui

# Full build + visual regression suite (proxy for CI):
just smoke-deploy
```

For bundle-size + bench regressions: `just bench-check` (gates at >15% vs the latest committed baseline in `tests/perf/baselines/`). Current cumulative numbers and known regressions in `.artifacts/sui-arc/perf-status.md`.

## Authoring a view in one shot (AI agents)

If you're an AI agent producing a new view from a job spec — read
`references/agent-authoring.md`. It is the do-now checklist: the
30-second scaffold, the canonical 3-branch loading shape, the SDK
domain table, the rail-registration steps, the 10 common pitfalls,
and the verification commands. Paired with `references/cookbook.md`
(7 copy-paste worked examples for list views, mutations,
master-detail, tabs/sub-routes, conditional branches, skeleton/empty/
error states, and custom-element prop wiring) it covers the path
from empty `views/<name>/` to a registered, rendering view in one
session.

## References

Two tiers. The `references/*.md` files are concise agent-facing overviews; the `references/docs/*.md` files are the docs-site deep dives shown on the public/internal site (collect-skill-docs walks the `docs/` subdir). They complement each other — different consumers, different granularities.

### Agent-facing overviews (`references/`)

| When you need | Read |
|---|---|
| Author a new view in one shot (agent-first checklist) | `references/agent-authoring.md` |
| Copy-paste worked examples (lists, mutations, master-detail, tabs, branches, states, props) | `references/cookbook.md` |
| Compiler API, runtime exports, daemon routes, where each piece lives | `references/4-layer-overview.md` |
| Component vs view, controller shape, `<prop>`, scope typing, `bindHtml`, `bind:this` refs | `references/authoring-patterns.md` |
| Reactive primitives, control flow, `<data live>` end-to-end, transitions, morph, error-boundary, portal, route signals | `references/runtime-primitives.md` |
| sui-lsp, sui-build, devtools, `dispatch.sh`, scaffolds, the `sui-lint`/`sui-compiler` CLI loop | `references/tooling.md` |

### Docs-site deep dives (`references/docs/`)

| When you need | Read |
|---|---|
| The three-role split (template / controller / data) + MVVM mapping | `references/docs/index.md` |
| Full `.sui` directive surface — `<data>`, `<prop>`, control flow, bindings, slots | `references/docs/template.md` |
| Paired `.ts` shape — three tiers, six sections, scope rules, mutation discipline | `references/docs/controller.md` |
| `<data>` element semantics, live envelope, static `<data>`, mutation forms | `references/docs/data-subscriptions.md` |
| Template hooks (`<use-*>`) vs controller hooks (`use:`) — when + how | `references/docs/hooks.md` |
| Effect-scope cleanup ownership — `createScope`, dispose order, leak detection | `references/docs/effect-scope.md` |
| Three canonical examples — counter, agents, settings form (tier progression) | `references/docs/examples.md` |
| Rust IR render walker, attach-only client runtime, renderable expression subset | `references/docs/ssr-hydration.md` |
| End-to-end test harness via Playwright | `references/docs/e2e-testing.md` |
| Stability matrix — what's stable, what's active, what's not yet implemented | `references/docs/status.md` |