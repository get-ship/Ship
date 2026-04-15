---
name: datastar-views
stable-id: datastar-views
description: Build live views with Datastar + axum. Signals hold client state, SSE pushes HTML morphs and signal patches. No polling, no React, no custom handler per view. Hard-won gotchas and the Tao included.
tags: [views, datastar, sse, axum, rust, frontend]
authors: [ship]
---

# Datastar Views in Ship

Datastar is a tiny (~14KB) hypermedia client that morphs the DOM from server-sent HTML, patches reactive signals from server JSON, and lets templates express interactivity via `data-*` attributes — no framework, no npm. Paired with Ship's axum daemon and minijinja templates, it gives us push-based UIs with zero client-side code for most views.

**Scope note.** This skill is the day-to-day reference. For the author-facing SDK shape (manifest-driven signals/topics/actions, codegen, dogfooding) see `.artifacts/specs/view-sdk-design.md`.

## When to use this

- **First-party Ship views** (pipelines, specs, git, settings — all live)
- **Canvas components** that need real-time server state
- **AI-generated interfaces** where token efficiency matters — the server generates plain HTML, not React components
- **Any view where push > pull** (job events, session activity, build progress)

Don't use for: rich interactive editors (canvas games, video timelines, 3D). Those need full client-side frameworks.

## The Tao (non-negotiable)

Datastar has a published "Tao" — read it before you start. The rules that matter most for Ship:

1. **State in the backend.** The server is the source of truth. The frontend holds only signals that represent user interactions (tab selection, input state, toggled menus) or state being staged for the backend (form buffers).
2. **Use signals sparingly.** Don't mirror server state into signals "just in case" — fetch fresh state. Signals are for UI intent, not caching.
3. **Morph big chunks.** Datastar's morph algorithm only touches what changed, so send fat HTML fragments rather than trying to produce minimal diffs yourself. IDs and `data-preserve-attr` handle state preservation.
4. **CQRS by default.** One long-lived `@get` that streams reads; short-lived `@post`/`@put`/`@patch`/`@delete` for writes. Writes don't need to render anything — the long-lived read stream will patch when the underlying state changes.
5. **Loading indicators, not optimistic updates.** Use `data-indicator` to show work in flight and let the server confirm the truth. Lying to the user is a bug.
6. **Backend templating is encouraged.** Jinja (minijinja, in our case) is the right tool for fragments. Use it.
7. **Start with defaults.** Don't customize `retry`, `requestCancellation`, `openWhenHidden`, etc. unless a real problem demands it.

## The stack

```
┌──────────────── Browser ─────────────────┐
│  data-signals="{tab:'active'}"           │   signals = client state
│  data-on:click="$tab='history'"          │   writes flip signals
│  data-init="@get('/view/pipelines/sse')" │   one long-lived read stream
│  Datastar morphs DOM on PatchElements    │
│  Datastar merges signals on PatchSignals │
└──────────────┬───────────────────────────┘
               │ SSE: datastar-patch-elements + datastar-patch-signals
┌──────────────┴───────────────────────────┐
│  axum handler → ViewStream → datastar    │
│    vs.patch("#id", html)    // fragments │
│    vs.signal(r#"{counts:…}"#) // deltas  │
└──────────────────────────────────────────┘
```

## Dependencies

```toml
# Cargo.toml
datastar = { version = "0.3", features = ["axum"] }
axum     = "0.8"
tokio    = { version = "1", features = ["full"] }
```

Client-side: vendor the matching Datastar JS. `datastar@1.0.0-RC.8` pairs with Rust SDK `0.3.x`.

**Version mismatch breaks silently.** Older betas emit `datastar-merge-fragments`; RC.7+ emits `datastar-patch-elements`. If protocol event names don't match, patches arrive and nothing happens. Always vendor Datastar JS locally — CDN URLs with `@version` suffixes get URL-encoded badly and break.

## Signals-first patterns

This is the biggest gotcha in Ship's existing views: **don't template URL query strings for state**. If you catch yourself writing `@get('/foo?tab={{ active_tab }}&agent={{ agent_filter }}')`, stop. Signals are auto-sent on every backend action — the server reads them from the request body (or `?datastar=...` query param on GET).

### Anti-pattern (what we did in pipelines/shell.html.j2)

```jinja
{# Every click rebuilds the URL, tears down the SSE, triggers a full shell re-render #}
<ship-chip {% if agent_filter == a %}selected{% endif %}
  data-on:click="@get('{{ view_pipelines() }}?tab={{ active_tab }}&agent={{ a }}')">
  {{ a }}
</ship-chip>
```

### Correct pattern

```jinja
{# Signals at the root, auto-sent on every request. #}
<div id="pipelines-root"
     data-signals="{tab: '{{ active_tab }}', agent: {{ agent_filter|tojson }}}">

  {# Server-free selected-state flip. #}
  <ship-chip data-attr:selected="$agent == '{{ a }}'"
             data-on:click="$agent = '{{ a }}'">
    {{ a }}
  </ship-chip>

  {# One long-lived stream. No query string; server reads signals from the body. #}
  <div id="pipelines-content"
       data-init="@get('/view/pipelines/sse', {requestCancellation: 'disabled'})">
    Loading…
  </div>
</div>
```

Rust handler reads signals via the Datastar SDK's `ReadSignals`:

```rust
#[derive(serde::Deserialize)]
struct PipelinesSignals { tab: String, agent: Option<String> }

pub async fn pipelines_sse(req: axum::extract::Request) -> ViewSse {
    let signals: PipelinesSignals = datastar::axum::read_signals(&req).await.unwrap_or_default();
    // ... subscribe to topics, render fragments using signals
}
```

### Attributes you should be using

| Attribute | Use when |
|---|---|
| `data-signals:name="default"` | Declare a client-held piece of UI state. Use camelCase or the `__case` modifier. |
| `data-bind:name` | Two-way bind a `<input>`/`<select>`/`<textarea>`/custom-element value to a signal. |
| `data-attr:selected="$x == 'y'"` | Reactive attribute (selected, disabled, aria-*). Replaces `{% if %}…{% endif %}` for per-element state. |
| `data-class:is-active="$open"` | Reactive class toggle. Replaces `{% if %}active{% endif %}` in `class=""`. |
| `data-show="$open"` | Reactive visibility. Add `style="display:none"` to the element to avoid flicker. |
| `data-text="$x"` | Reactive text content. Use for counts/status strings you patch via signals. |
| `data-computed:total="$items.length"` | Derived read-only signal. |
| `data-indicator:creating` | Binary signal that flips `true` during an in-flight fetch. Pair with `data-show="$creating"` for spinners. |
| `data-preserve-attr="open"` | Keep an attribute (e.g., `<details open>`) across morphs. |
| `data-on:click__debounce.300ms` | Debounce rapid clicks / typeahead input. |

Signal names cannot contain `__` (that's the modifier delimiter). Signals prefixed with `_` are **local** (not sent to the server) — use them for UI-only state like `$_creating`, `$_menuOpen`.

## CQRS pattern

The Tao's recommended shape:

```jinja
<div id="pipelines-root"
     data-signals="{tab: 'active'}"
     data-init="@get('/view/pipelines/sse')">

  <button data-on:click="@post('/api/jobs')"
          data-indicator:_creating>
    + Run <span data-show="$_creating">…</span>
  </button>

  <div id="pipelines-content">…</div>
</div>
```

- **One long-lived read stream** (`data-init="@get('/sse')"`) per view. Subscribes to topics, pushes both fragment and signal patches.
- **Short-lived writes** (`@post`/`@put`/`@delete`) just fire and forget. They don't need to render anything — the read stream will patch when the underlying state changes.
- **Indicators (`data-indicator:_creating`)** give the user feedback while the round-trip happens.

Ship helper: `apps/shipd/src/views/stream.rs:85-104` exposes `ViewStream::patch(selector, html)` / `patch_html(html)` / `signal(json)` — use all three. The `signal` helper is currently under-used; reach for it whenever you're about to re-render HTML just to change a number.

## PatchSignals vs PatchElements — decision rubric

| Change kind | Event | Why |
|---|---|---|
| Counters, totals, status flags (`stats.active = 3`) | `PatchSignals` | Tiny JSON delta; the DOM binds via `data-text="$stats.active"` and updates client-side. |
| Toast / error text | `PatchSignals` | Same reasoning. Set `$toast = {level:'error', msg:'…'}` and template the toast with `data-show="$toast"`. |
| A row updated in a list | `PatchElements` with selector `#run-{{id}}` | Need to morph the actual DOM to reflect structural changes. |
| New row added / removed | `PatchElements` with mode `append` / `remove` | Structural. |
| Progress bar percentage | `PatchSignals` | Bind `data-style:width="$pct + '%'"`. |
| Full view refresh | `PatchElements` on the content region | Fall back when many things changed at once. |

Wire-format reminder (you rarely hand-write these — use `ViewStream`):

```
event: datastar-patch-signals
data: signals {"stats": {"active": 3, "history": 17}}

event: datastar-patch-elements
data: elements <article id="run-42" class="ship-run-card">…</article>
```

## Request lifecycle + indicators

Every backend action triggers `datastar-fetch` events you can hook:

```html
<button data-on:click="@post('/api/jobs')"
        data-indicator:_creating
        data-on:datastar-fetch="evt.detail.type === 'error' && ($toast = 'Failed')">
  + Run
</button>
<span data-show="$_creating">creating…</span>
<div data-show="$toast" data-text="$toast" class="ship-toast"></div>
```

For rapid input (search boxes, sliders), always pair with a modifier:

```html
<input data-bind:query
       data-on:input__debounce.250ms="@get('/view/search')">
```

## Ship view layout

Views live in `.ship/views/<name>/` (project) or `~/.ship/views/<name>/` (global). Resolution: see `apps/shipd/src/views.rs:87-98`.

```
.ship/views/pipelines/
├── manifest.yaml         # name, type, claims (see SDK_DESIGN.md for the proposed expanded schema)
├── index.html            # the view shell — what the iframe loads
└── partials/             # Jinja fragments used by the SSE handler (future: declared in manifest)
    ├── content.html.j2
    └── run_card.html.j2
```

Today the authoring surface is minimal (`name`, `type`, `claims`) and Ship hand-writes a Rust handler per view in `apps/shipd/src/views_<name>.rs`. The SDK design doc proposes extending the manifest to declare `signals`, `topics`, `actions`, and `fragments`, after which each view becomes template-only and Ship's per-view handlers disappear. See `SDK_DESIGN.md` for the shape.

## Auth for SSE streams

Browser `EventSource` / fetch streams can't set headers reliably. Pass the auth token via query string:

```
/api/runtime/view/pipelines/sse?token=…
```

Template the token into the HTML server-side:

```rust
let html = template_html.replace("{{SHIP_TOKEN}}", &caller_token);
```

Ship's auth middleware accepts `?token=…` as a Bearer fallback. See `serve_view` in `apps/shipd/src/views.rs`.

## CORS

If the view is cross-origin (Studio at `:3000` calling daemon at `:9315`), CORS headers must allow `datastar-request`:

```
access-control-allow-headers: content-type, authorization, accept, datastar-request, …
```

Missing this → Datastar's fetch gets blocked by CORS preflight → "NetworkError" in the console.

## Debugging

**Stream works in curl but not browser:**
- Client/Rust SDK version mismatch (RC.7+ required for `data-init` and `datastar-patch-elements`).
- Browser console for "Error in input stream" (malformed SSE) or "NetworkError" (CORS/auth).
- Network tab → SSE request → confirm chunks arriving.

**Signals don't patch:**
- Check `event: datastar-patch-signals` spelling in the raw SSE (use devtools Network → Response).
- Confirm the JSON parses — `{hal:'hi'}` is JS object notation, not strict JSON; Datastar accepts both but your templating might produce malformed strings.
- Signal name has `__` in it (illegal) — rename.

**`data-init` doesn't fire:**
- Not `data-on-load`; not `data-on-init`. Only `data-init`.
- Element ID must be unique in the DOM.
- Datastar script loaded before the element? Usually fine, but check.

**Stream closes immediately:**
- `requestCancellation: 'auto'` (default) cancels the previous request on the same element. For `data-init`-driven streams, set `{requestCancellation: 'disabled'}`.
- Handler returning too early — `ViewStream` requires the producer task to live across the whole stream.

**Patches arrive but DOM doesn't update:**
- Element ID mismatch. Datastar morphs by outer `id`; wrapper must match.
- Mode wrong. Default is morph/outer; for prepend/append use `datastar-mode` response header or the SSE `mode` line.

**`data-attr:selected` doesn't reflect:**
- You're also server-rendering `selected` as a literal attribute. Remove it — `data-attr` is authoritative.

## Pattern: relaying another SSE source

When daemon events need to be fanned into a Datastar view:

```rust
use futures::StreamExt;

let client = reqwest::Client::builder()
    .no_gzip().no_brotli().no_deflate()
    .build()?;
let resp = client.get(upstream_url)
    .bearer_auth(&token)
    .header("accept", "text/event-stream")
    .send().await?;

let mut body = resp.bytes_stream();
let mut buf = String::new();

while let Some(chunk) = body.next().await {
    buf.push_str(&String::from_utf8_lossy(&chunk?));
    while let Some(pos) = buf.find("\n\n") {
        let event = buf[..pos].to_string();
        buf = buf[pos + 2..].to_string();
        // Parse upstream event → render HTML → vs.patch(...) or vs.signal(...)
    }
}
```

Disable compression (`.no_gzip().no_brotli().no_deflate()`) — some servers buffer compressed streams and break real-time delivery.

## Ship's reference implementation (current state, pre-SDK)

- `apps/shipd/src/views.rs` — view resolver (project → global fallback), asset serving, token inlining.
- `apps/shipd/src/views/stream.rs` — `ViewStream` helper: `patch`, `patch_html`, `signal`, `heartbeat_every`.
- `apps/shipd/src/views_pipelines.rs:339-388` — `pipelines_sse` as the read-stream pattern. **Known redesign target:** today it receives state via `Query<PipelinesParams>` (URL query strings) and patches HTML fragments only. Post-SDK it should read signals from the request and emit `PatchSignals` for counts + `PatchElements` for content.
- `.ship/views/pipelines/` — the iframe shell (`index.html`) + manifest; content partials currently live in `apps/shipd/templates/pipelines/` and will move into the view dir per the SDK design.
- `.ship/views/ship-sdk.js` — generic postMessage RPC bridge (`ship.jobs.list()`, `ship.events.emit()`, `ship.workspace.active()`, `ship.files.read()`, `ship.theme.*`). Stays as-is; the SDK design keeps typed per-view clients separate from this namespace.

## Further reading

- `.artifacts/specs/view-sdk-design.md` — proposed manifest-driven author surface, typed signals/topics/actions, dogfooding migration.
- [data-star.dev/docs](https://data-star.dev/docs) — full attribute and action reference.
- Tao of Datastar (on the docs site) — re-read whenever you're tempted to reach for client-side state.
