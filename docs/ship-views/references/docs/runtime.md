---
group: Views
order: 3
title: Views Internals
description: shipd's view serving stack — resolve, render, session registry, SSE topics, ship-elements bundling.
audience: internal
---

# Views Internals

`apps/shipd/src/views/`. The view subsystem inside shipd. Handles the full lifecycle: load `.ship/views/<name>/manifest.json`, render the template, wire signals, open an SSE stream, route events to subscribed fragments, GC idle sessions.

```
apps/shipd/src/views/
  mod.rs             — module tree
  handlers.rs        — HTTP handlers: serve_view, serve_view_asset, serve_view_manifest, serve_view_bundle, list_views
  resolve.rs         — view resolution (path → manifest + template)
  templates.rs       — Jinja engine setup + macro registration
  view_filters.rs    — Jinja filters for collection/event access
  view_filters_list.rs
  session.rs         — ViewSessionRegistry (canvas sessions)
  session_types.rs   — AgentId, ViewId, SessionError, TokenBucket
  stream.rs          — ViewStream + ViewSse (SSE delivery)
  generic_sse.rs     — generic SSE helper
  context.rs         — view render context construction
  state.rs           — per-session state
  setup.rs           — shipd-startup wiring
  error_overlay.rs   — render errors into the page rather than 500
  sanitize.rs        — HTML sanitization for agent-emitted content
  assets.rs          — static asset serving (assets/*)
  chat_state.rs, events_state.rs, agents_state.rs, skills_state.rs, specs_state.rs — per-domain state helpers
  templates_tests.rs, session_tests.rs, state_tests.rs, ...
```

## HTTP surface

Registered in `apps/shipd/src/router.rs` under `/views/*`:

| Route | Handler | Purpose |
|---|---|---|
| `GET /views` | `list_views` | JSON list of available view manifests. |
| `GET /views/<name>/` | `serve_view` | Render and serve the view HTML. |
| `GET /views/<name>/manifest` | `serve_view_manifest` | Raw manifest JSON. |
| `GET /views/<name>/assets/<file>` | `serve_view_asset` | Static asset from `.ship/views/<name>/assets/`. |
| `GET /views/<name>/bundle` | `serve_view_bundle` | Optional per-view JS bundle (rare; most views don't need one). |
| `GET /views/<name>/stream` | (SSE) | Event stream for this view. Subscribes to the manifest's `events` array. |

Plus canvas-specific routes under `/api/view-sessions/*` handled by `view_sessions_api.rs`.

## Resolve

`resolve.rs`. `resolve_view(name) -> Result<ResolvedView>`:

1. Look up `.ship/views/<name>/manifest.json`.
2. Parse the manifest. Fail with an actionable error if malformed.
3. Read `index.html.j2`.
4. Return a `ResolvedView { manifest, template_path, template_content }`.

Called on every request — no caching today. Cost is a file read + JSON parse; Jinja rendering dominates.

Workspaces and project-scoped view lookup are not implemented — views are always in the global `.ship/views/` dir. This may change if project-scoped views become a requirement.

## Templates engine

`templates.rs`. Uses `minijinja::Environment`. At startup, registers:

- **The shared SDK template** `_ship.html.j2` — all the `signal`, `bind`, `show`, etc. macros defined once; every view imports from it via `{% from '_ship.html.j2' import ... %}`.
- **Filters** from `view_filters.rs` — `ship_canvas_docs`, `ship_collection_items`, `ship_recent_events`, etc. These give templates programmatic access to runtime state.
- **Globals** — `view`, `workspace`, `session` injected into every render's context.

The environment is long-lived (one instance per daemon). Templates are looked up by path on each render via `render_named_str`.

### Jinja filters

Defined in `view_filters.rs` and `view_filters_list.rs`. Each filter is a Rust function that takes a value from the template and returns JSON. Examples:

- `{{ 'specs' | collection_items }}` — returns array of items in the specs collection.
- `{{ 'workspace.*' | recent_events(limit=20) }}` — returns recent events matching a prefix.
- `{{ agent_id | agent_details }}` — hydrate an agent id into full config.

Filters are the read-only bridge between Jinja and runtime state. No filter writes anything.

## Render context

`context.rs`. Per-request:

```rust
pub struct ViewContext {
    pub view: ViewMeta,
    pub workspace: Option<WorkspaceSummary>,
    pub session: Option<SessionSummary>,
    pub collections: HashMap<String, CollectionSnapshot>,  // declared in manifest.data.collections
    pub recent_events: Vec<EventEnvelope>,                 // matching manifest.events filters
}
```

Built by reading the manifest, querying `data.db` for each declared collection, and querying `platform.db` for recent events. Render happens against this snapshot — no live updates during render.

Live updates arrive after render via SSE.

## SSE stream

`stream.rs`. `ViewSse::new(view_name, subscriptions) -> (stream, subscriber_id)`:

1. Register with the kernel as a subscriber to each namespace in the view's `events` manifest field.
2. Open an SSE stream to the client.
3. When a matching event fires in the kernel, serialize it + optionally re-render the affected fragment (via Datastar's fragment API) and push to the client.

Ship's Datastar integration uses the Datastar-standard `datastar-merge-fragments` SSE event name; the client library reads them and swaps DOM.

Connection is long-lived. Typical lifecycle: page load → stream open → many event pushes → page unload → stream close → kernel unsubscribes.

## View sessions (canvas)

`session.rs`. Canvas views need something regular views don't: persistent agent-owned state across a single view's lifetime, token-bucket rate limits on agent emissions, a subscriber list for multiple humans viewing the same canvas.

`ViewSessionRegistry` tracks active canvas sessions. `SessionEntry`:

```rust
pub(crate) struct SessionEntry {
    pub(crate) owner_agent_id: AgentId,
    pub(crate) template: String,
    pub(crate) title: Option<String>,
    pub(crate) context: Value,
    pub(crate) subscribers: Vec<ViewStream>,
    pub(crate) last_disconnect_at: Option<Instant>,
    pub(crate) tokens: TokenBucket,
}
```

Lifecycle:

1. Agent calls MCP tool `view.open` → registry creates a session, returns `ViewId`.
2. Human navigates to `/views/canvas/?session=<id>` → browser's SSE subscribes to that session.
3. Agent emits fragments via `view.patch` / `view.append` / `view.signal` — tokens deducted from the bucket.
4. All subscribers receive the fragment.
5. On last human disconnect, `last_disconnect_at` starts. After `cfg.gc_timeout`, registry GCs the session.

Token bucket caps the rate of fragment emissions — prevents a runaway agent from flooding the client.

Session types in `session_types.rs`: `AgentId`, `ViewId`, `SessionError`, `RegistryConfig`, `SessionInfo`, `TokenBucket`.

The canvas-session contract is documented in `.artifacts/phase-1-agent-canvas-views-spec.md` (development artifact, not shipped).

## Error overlay

`error_overlay.rs`. When a view template errors (Jinja syntax, missing variable, filter error), shipd renders a styled error overlay INTO the page rather than returning a 500. The author sees the error in-browser with the line number and backtrace. This is the "edit, refresh, see result" authoring loop's forcing function for debuggability.

Test coverage in `error_overlay_tests.rs`. Errors in templates are visible; errors in filters surface with filter name + args.

## Sanitization

`sanitize.rs`. Agent-emitted HTML (via canvas views, `view.append`, etc.) passes through a sanitizer before rendering. Removes `<script>` tags, dangerous attributes (`on*=` handlers), limits iframe src to same-origin. Uses `ammonia` (standard Rust HTML sanitizer).

Author-written view templates (in `.ship/views/`) are NOT sanitized — the author controls them. Only agent-generated runtime HTML is. The distinction: templates are trusted, runtime agent output is untrusted.

## Ship-elements bundle

`assets.rs` + the `head()` Jinja macro load:

- Fonts — @fontsource-variable/syne, @fontsource-variable/dm-sans.
- Design tokens — `packages/ship-set/src/tokens.css` served as `/assets/ship-tokens.css`.
- Themes — `packages/ship-set/src/themes.css`.
- ship-elements — the web-component bundle built from `packages/ship-set/`. Served as `/assets/ship-elements.js`.
- Datastar — the reactive runtime, vendored under `apps/shipd/assets/datastar.js`.

All loaded from shipd's own assets dir, not per-view. View authors don't think about bundling.

## Setup

`setup.rs`. Called during shipd startup (`run_network`). Initializes the view environment:

1. Locate `.ship/views/` (fall back to not serving views if absent).
2. Build the Jinja environment.
3. Register filters.
4. Construct the `ViewSessionRegistry`.
5. Return handles to be stored on `ShipRuntime`.

Setup is fast — no view is rendered until first request.

## Feature-gating

The entire views subsystem is stable (not behind `unstable`). Canvas features (view sessions, MCP `view.*` tools) have been stable since the view-sessions-01 job landed.

Experimental tools around views (`view.open`, `view.patch`, `view.append`, `view.signal`) live in `apps/mcp/src/tools/view.rs` and are gated behind the `unstable` feature flag on the MCP server binary.

## Testing

- `templates_tests.rs` — macro rendering, filter correctness, error overlay behavior.
- `session_tests.rs` — canvas session lifecycle, GC, token bucket.
- `state_tests.rs` — state-helper modules (agents, skills, specs, chat, events).
- `view_filters_tests.rs` — filter signatures and edge cases.
- `sanitize_tests.rs` — HTML sanitizer coverage.

Integration-level testing happens via `apps/shipd/src/views/*_tests.rs` which spin up an in-process shipd and drive HTTP requests.

## Invariants

- Every view must have `manifest.json` and `index.html.j2` at minimum. Missing either = 404.
- Render errors produce the error overlay in-page, not HTTP 500.
- Agent-emitted HTML is sanitized; author-written templates are not.
- SSE streams hold kernel subscriptions; closing the stream unsubscribes.
- Canvas sessions GC after `cfg.gc_timeout` past last disconnect. Active sessions never auto-GC.
- Ship-elements bundle and tokens.css are the same bytes for every view — consistency is not optional.

## Related

- [Views Overview](/views/) — user-facing intro.
- [Manifest & SDK Reference](/views/) — public fields and macros.
- [Kernel](/architecture/kernel/) — SSE streams subscribe here.
- [Collections](/runtime/collections/) — the most common data source.
