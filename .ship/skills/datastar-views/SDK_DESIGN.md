# Ship Views SDK — Design

**Status:** Design proposal. Not yet implemented. This document names the shape, flags open questions, and sequences the dogfooding migration so Ship's own views become the reference consumers.

**See also:** `SKILL.md` (this directory) for Datastar day-to-day patterns. This doc describes the authoring surface *above* those patterns.

## 1. Problem

Today, a Ship view is a split system:

- `.ship/views/<name>/index.html` — an iframe shell that does a single `@get('/api/runtime/view/<name>/sse')`.
- `apps/shipd/templates/<name>/*.j2` — server-side Jinja fragments rendered by a hand-written Rust handler.
- `apps/shipd/src/views_<name>.rs` — the bespoke SSE handler that knows about that view's query params, subscribes to the right kernel events, and pushes HTML fragments back.

Three consequences:

1. **Every new view needs Rust code.** Third-party view authors can't ship a view without a Ship rebuild.
2. **No typing across the seam.** Signals, event payloads, action URLs, and fragment contexts are stringly-typed Jinja interpolations. A typo surfaces as a 500 at render time or a silent no-op in the browser.
3. **Anti-patterns get baked in.** See `apps/shipd/templates/pipelines/shell.html.j2` — state lives in URL query strings because there's no higher abstraction pushing authors toward signals.

**Goal:** authors drop `{manifest.yaml, index.html, partials/*.j2}` into `.ship/views/<name>/` and Ship renders a reactive app from them, with zero per-view Rust. Ship's own views (`pipelines`, `specs`, `git`, `settings`) become the reference consumers — their per-view handlers are *deleted* as part of dogfooding.

## 2. Core principle: no custom handlers

**A view is manifest + templates + Ship's existing data surface, nothing more.**

- Actions don't ship user-written Rust. They reference operations Ship already provides (`jobs.list`, `jobs.create`, `events.emit`, `workspace.active`, `files.read`, …) — the same namespace `.ship/views/ship-sdk.js` already exposes to iframes.
- Topics reference Ship's existing event streams (kernel events, git events, runtime events), not author-defined payload types.
- Fragments are Jinja templates rendered by a shared runtime, not per-view Rust renderers.

If a view needs an operation Ship doesn't expose, **that's a signal to extend Ship's core data surface** — not to add a bespoke handler to the view. The SDK's value comes from the promise that "author a view" never reduces to "write Rust."

Corollary: a well-maintained catalog of Ship ops (`jobs.*`, `events.*`, …) is the *only* extensibility surface. See §8, open question #4.

## 3. Authoring surface

Extend `.ship/views/<name>/manifest.yaml` from the current `{name, type, claims}` to a richer shape. Existing views continue to load under the minimal schema; new fields are optional.

```yaml
name: pipelines
type: html
claims: []

# Client-held UI state. Auto-injected as data-signals="{…}" on the view root.
# Reads into a generated PipelinesSignals struct on the Rust side.
signals:
  tab:      { type: string,  default: "active" }
  agent:    { type: string?, default: null }
  _creating: { type: bool,   default: false }   # _-prefix = local, not sent to server

# Server event streams this view subscribes to. Must be known Ship topics.
# See the op catalog for the valid set.
topics:
  - ship.jobs.changed
  - ship.git.branch-stats.changed

# Bindings to Ship ops. No custom handlers — `op:` must reference a built-in.
# args: maps signal refs / literals to the op's parameters.
actions:
  createJob:
    op: jobs.create
    args: { agent: "$agent" }
  endJob:
    op: jobs.end
    args: { job_id: "$_focusedJob" }

# Jinja fragments with typed contexts. The generated Rust exposes a strongly-typed
# render function per fragment; the template gets a fragment() macro.
fragments:
  pipelines-content:
    template: partials/content.html.j2
    context:
      runs: "array<Run>"
      stats: "Stats"
      agents: "array<string>"
  run-card:
    template: partials/run_card.html.j2
    context: { run: "Run" }
```

Types in `context:` reference a shared schema (derived from Ship's existing Rust types — `Run`, `Stats`, etc. — via `serde` + a schema bridge). Authors don't invent new shapes.

## 4. Primitives

### 4.1 Typed signals

Manifest `signals:` block →

- Generated Rust `#[derive(Serialize, Deserialize, Default)] struct PipelinesSignals { tab: String, agent: Option<String> }`.
- Generated TS `type PipelinesSignals = { tab: string; agent: string | null }`.
- The view root element gets `data-signals="{…}"` auto-injected from defaults; the author doesn't hand-write it.
- The generic SSE handler (§5) decodes signals via `datastar::axum::read_signals::<PipelinesSignals>(&req)` at the request boundary, then reads them as normal Rust fields.

### 4.2 Typed topics

Manifest `topics:` block →

- Generated Rust enum `PipelinesTopic { JobsChanged, GitBranchStatsChanged }` with `From<KernelEvent>` / filter-predicate impls.
- The SDK subscribes on the view's long-lived stream; on each topic tick, the SDK re-renders the fragments whose declared dependencies overlap the event kind.
- Topic payloads are Ship's existing event types, not author-defined structs. "Typed" here means "typed reference to a Ship-native event."

### 4.3 Bound actions — no custom handlers

Manifest `actions:` block →

- **TS side:** generated method on a per-view client module. `ship.pipelines.createJob({agent: 'cursor'})` delegates to the existing postMessage bridge's `_request('jobs.create', args)` path.
- **Template side:** Datastar helper `@ship.action('createJob')` (injected via generated `_ship.html.j2`) resolves to the right `@post('/api/runtime/ops/jobs.create', {body: {agent: $agent}})` call — no URL string building, no `JSON.stringify` boilerplate.
- **Rust side:** zero. The action routes to the existing op handler; no per-view Rust file exists.

If a declared `op:` isn't in the catalog, build fails — the manifest is the contract.

### 4.4 Typed fragments

Manifest `fragments:` block →

- Generated `Fragment<T>` per entry with a strongly-typed context struct.
- `fragment!(run_card, run)` in Rust returns HTML via the existing minijinja environment, but with compile-time checking that `run` matches the declared `Run` shape.
- `{{ fragment('run-card', {run: run}) }}` in Jinja goes through the same path, catching typos at template load time (minijinja `strict_undefined` + a schema check).

Replaces today's stringly-typed `templates::render("pipelines/partials/run_card.html.j2", context!{run})`.

### 4.5 Long-lived read stream

- One SSE endpoint per view (`/api/runtime/view/<name>/sse`), auto-wired by the generic handler.
- Reads signals from the request once, then subscribes to declared topics.
- Topic events trigger fragment re-renders (patched as `PatchElements`) or signal patches (`PatchSignals`) according to per-fragment metadata.
- No per-view SSE handler code to write.

## 5. Runtime architecture

```
.ship/views/<name>/
├── manifest.yaml     ← authoring surface
├── index.html        ← Jinja shell
└── partials/
    └── *.html.j2

         ↓ build-time

apps/shipd/
├── build.rs          ← extended: reads every .ship/views/*/manifest.yaml
│                       emits:
│                         views_generated.rs      (PipelinesSignals, PipelinesTopic, …)
│                         views_generated.ts      (ship.pipelines.createJob, …)
│                         _ship.html.j2           (signal(), fragment(), @ship.action)
└── src/
    └── views/
        └── generic.rs ← single SSE handler, dispatches by view name
                         looks up manifest → subscribes to topics
                         → renders fragments → patches via ViewStream

         ↓ runtime

Browser iframe loads index.html
 → evaluates data-signals (from defaults)
 → data-init="@get('/api/runtime/view/<name>/sse')"
 → long-lived stream pushes PatchElements + PatchSignals
 → Datastar morphs DOM + merges signals
```

**Key property: there is one SSE handler in Rust, shared across every view.** It looks up the manifest, decodes signals, subscribes to topics, renders fragments. A new view requires zero new Rust.

**ship-sdk.js stays untouched.** The per-view typed TS clients are generated as separate modules (`import { pipelines } from '/api/runtime/view/pipelines/client.js'`) and call into the bridge at runtime. `window.ship` remains the generic RPC surface. This keeps concerns separate — the bridge doesn't grow per-view methods, and the per-view clients stay tree-shakeable.

**Transport default:** one long-lived SSE stream per view, with topics multiplexed server-side (the Tao CQRS example). **This is explicitly provisional** — see §8.

## 6. Template-side abstractions

Inside `.ship/views/<name>/index.html` and `partials/*.html.j2`:

```jinja
{% from '_ship.html.j2' import signal, fragment, action %}

<div id="pipelines-root" {{ signal('tab') }} {{ signal('agent') }}>

  {# Action helper — resolves to the right @post/@get/etc with correct body shape. #}
  <button data-on:click="{{ action('createJob') }}" data-indicator:_creating>
    + Run <span data-show="$_creating">…</span>
  </button>

  {# Selected-state is reactive — no {% if %} branches per chip. #}
  {% for a in agents %}
    <ship-chip data-attr:selected="$agent == '{{ a }}'"
               data-on:click="$agent = '{{ a }}'">{{ a }}</ship-chip>
  {% endfor %}

  {# Long-lived read stream. No query string — signals are auto-sent. #}
  <div id="pipelines-content"
       data-init="@get('/api/runtime/view/pipelines/sse', {requestCancellation: 'disabled'})">
    {{ fragment('pipelines-content', {runs: runs, stats: stats, agents: agents}) }}
  </div>
</div>
```

What the macros expand to:

- `signal('tab')` → `data-signals:tab="'active'"` (using the default from the manifest; the macro looks it up).
- `action('createJob')` → `@post('/api/runtime/ops/jobs.create', {contentType: 'json', body: {agent: $agent}})`.
- `fragment('pipelines-content', …)` → validates the context shape against the manifest and calls minijinja with it.

## 7. Dogfooding migration

**End state:** `apps/shipd/src/views_pipelines.rs`, `views_specs.rs`, `views_git.rs`, `views_settings.rs` are **deleted**. Every Ship view is authored as `.ship/views/<name>/{manifest.yaml, index.html, partials/}` and rendered by the generic SDK handler. `apps/shipd/templates/` holds only the shared partials (`_page_shell`, `_breadcrumb`, `_toolbar`, `_toast`, etc.).

Order of migration, simplest first so the SDK hardens against easy cases before the hardest one:

1. **settings** — static shell with a few toggles. Proves: manifest loading, typed signals from defaults, `action()` helper, fragment codegen.
2. **specs** — list view with card fragments. Proves: typed fragment contexts on repeated items, `PatchElements` via fragment re-renders.
3. **git** — diff viewer with complex nested state. Proves: multiple signals, topic subscriptions (branch-stats events), `PatchSignals` for scalar counters.
4. **pipelines** — the full stack. Signals, multi-topic subscription, multiple actions, nested fragments (`run-card` inside `content`). Migrated last once the SDK is stable.

Each migration lands in its own PR, deleting the old `views_<name>.rs` and moving `apps/shipd/templates/<name>/*` into `.ship/views/<name>/partials/`.

## 8. Open questions

These are explicitly unresolved. The plan is to ship the SDK with pragmatic defaults and revisit after the pipelines migration — that's the stress test that'll reveal the real answers.

1. **Topic transport.** Topics will be arbitrarily complex (job events, git events, runtime events, multi-workspace fanout, possibly per-entity scopes). Start with one-SSE-stream-per-view + server-side topic multiplexing. Alternatives to weigh when the shape breaks down: per-topic SSE endpoints (more connections, more granular reconnect), a shared fan-out bus with subscription frames on the initial handshake, or a WebSocket channel for bidirectional subscribe/unsubscribe. Revisit after pipelines dogfoods the multi-topic case.
2. **Hot-reload vs type-safety for third-party views.** Ship's own views go through codegen. Third-party authors who drop a view in `~/.ship/views/<name>/` shouldn't need a Ship rebuild. Likely answer: dev-mode manifest loader that produces `serde_json::Value` signals at runtime for unfamiliar views, trading off compile-time typing for instant iteration. Ship-internal manifests still codegen at build time.
3. **Action arg-binding DSL.** `createJob: { op: "jobs.create", args: { agent: "$agent" } }` — how expressive does the mapping need to be? v1: literal signal refs and constants only (`"$x"`, `"y"`). If demand appears: ternaries, computed refs, body-builder functions. **Do not** add a full expression language; if the mapping gets hairy, the op signature is probably wrong.
4. **Op catalog ownership.** The SDK validates every `op:` against the list of built-in Ship operations. Today that list is implicit in `ship-sdk.js` (`jobs.*`, `events.*`, `workspace.*`, `files.*`, `theme.*`) and in the bridge's daemon-side routes. The SDK needs a single source of truth — likely a JSON schema under `packages/sdk/schemas/ops.json` that both the bridge and the manifest validator consume, regenerated from Rust ops via `specta` or equivalent.

## 9. Non-goals

- **Not replacing Datastar.** Every SDK helper compiles down to standard `data-*` attributes and SSE events. An author who needs a Datastar feature the SDK doesn't wrap can always reach for raw attributes.
- **Not replacing minijinja.** Templates stay in Jinja; the SDK adds typed macros and manifest-driven codegen around them. See the "Assess Jinja usage" discussion — Jinja is doing legitimate work and should stay.
- **Not a component framework.** `<ship-*>` web components (`<ship-status-dot>`, `<ship-button>`, `<ship-icon>`, …) already exist for rich interactions. The SDK is about the view shell, data flow, and handler plumbing — not about replacing the component layer.
- **Not a client-side router.** Datastar's "page navigation hasn't changed in 30 years" principle applies. Views are loaded by Studio's iframe host, not by an SPA router.

## 10. Locked-in decisions (from the planning conversation)

- **No custom handlers.** Views declare intent via manifest; Ship's data surface provides execution. If a view needs something Ship can't do, extend Ship's ops, not the view.
- **One generic SSE handler.** Parameterized by manifest, shared across all views. Not one handler per view.
- **Separate SDK namespaces.** `window.ship` stays the generic postMessage bridge. Per-view typed TS clients are generated as separate modules, imported by the view, not merged into the root namespace.
- **Dogfooding is the success metric.** The SDK is done when the four per-view Rust handlers are deleted.

## Critical files

- `.ship/views/pipelines/{index.html, manifest.yaml}` — current minimal author surface.
- `.ship/views/ship-sdk.js` — generic postMessage RPC bridge (stays as-is).
- `.ship/skills/datastar-views/SKILL.md` — day-to-day Datastar reference; the "Ship view layout" and "Further reading" sections link here.
- `apps/shipd/src/views.rs:87-98` — view resolver (project → global fallback). Extend to load the expanded manifest.
- `apps/shipd/src/views/stream.rs:49-137` — `ViewStream` public API (`patch`, `patch_html`, `signal`, `heartbeat_every`) that the generic handler wraps.
- `apps/shipd/src/views/templates.rs` — minijinja environment that the typed fragment primitive builds on.
- `apps/shipd/build.rs` — extend with manifest → types codegen.
- `apps/shipd/src/routes.rs` — annotation/registration pattern that the ops catalog will reuse.
- `apps/shipd/templates/<name>/*.j2` — content that migrates into `.ship/views/<name>/partials/` per view.
- `apps/shipd/src/views_{pipelines,specs,git,settings}.rs` — the hand-written SSE handlers the codegen replaces. End state: deleted.

## Appendix: before / after

**Before (today):**

```jinja
{# pipelines/shell.html.j2 — URL-templated state, per-chip server-rendered selected, SSE URL rebuilt on every click #}
<ship-chip {% if agent_filter == a %}selected{% endif %}
  data-on:click="@get('{{ view_pipelines() }}?tab={{ active_tab }}&agent={{ a }}')">
  {{ a }}
</ship-chip>
<div id="pipelines-content"
     data-init="@get('{{ view_pipelines_sse() }}?tab={{ active_tab }}{% if agent_filter %}&agent={{ agent_filter }}{% endif %}')">
  {% include 'pipelines/content.html.j2' %}
</div>
```

```rust
// views_pipelines.rs — Query extractor, manual mailbox loop, per-view handler
pub async fn pipelines_sse(
    State(state): State<ApiState>,
    Query(params): Query<PipelinesParams>,
) -> ViewSse { … }  // 50+ lines of bespoke plumbing per view
```

**After:**

```yaml
# .ship/views/pipelines/manifest.yaml
name: pipelines
signals: { tab: {default: "active"}, agent: {default: null} }
topics:  [ship.jobs.changed]
actions:
  createJob: { op: jobs.create, args: { agent: "$agent" } }
fragments:
  pipelines-content: { template: partials/content.html.j2, context: { runs: "array<Run>" } }
```

```jinja
{# .ship/views/pipelines/index.html #}
{% from '_ship.html.j2' import signal, action %}
<div id="pipelines-root" {{ signal('tab') }} {{ signal('agent') }}>
  <ship-chip data-attr:selected="$agent == '{{ a }}'"
             data-on:click="$agent = '{{ a }}'">{{ a }}</ship-chip>
  <div id="pipelines-content"
       data-init="@get('/api/runtime/view/pipelines/sse', {requestCancellation: 'disabled'})">
    {{ fragment('pipelines-content', {runs: runs}) }}
  </div>
</div>
```

```rust
// No views_pipelines.rs. The generic handler renders from the manifest.
```
