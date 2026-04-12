---
name: datastar-views
stable-id: datastar-views
description: Build live views with Datastar SSE + axum. Server pushes HTML fragments, browser morphs DOM. No polling, no React needed. Hard-won gotchas included.
tags: [views, datastar, sse, axum, rust, frontend]
authors: [ship]
---

# Datastar Views in Ship

Datastar is a tiny (11KB gzipped) client that morphs the DOM from server-sent HTML fragments. Paired with Ship's axum daemon, it gives us push-based UIs with zero client-side framework — ideal for first-party views (pipeline monitor, artifact viewer) and AI-generated interfaces.

## When to use this

- **First-party Ship views** (pipeline monitor, artifact renderer, anything live)
- **Canvas components** that need real-time server state
- **AI-generated interfaces** where token efficiency matters — the server generates plain HTML, not React components
- **Any view where push > pull** (job events, session activity, build progress)

Don't use for: rich interactive editors (canvas games, video timelines, 3D) — those need full client-side frameworks.

## The stack

```
┌──────────── Browser ────────────┐
│  data-init="@get('/sse/...')"   │
│  Datastar morphs DOM on patches │
└────────────┬────────────────────┘
             │ SSE: datastar-patch-elements
┌────────────┴────────────────────┐
│  axum handler with stream_fn    │
│  PatchElements::new(html)       │
│  .write_as_axum_sse_event()     │
└─────────────────────────────────┘
```

## Dependencies

```toml
# Cargo.toml
datastar = { version = "0.3.1", features = ["axum"] }
axum = "0.8"
async-stream = "0.3"
asynk_strim = "0.1"
futures = "0.3"
```

Client-side: match versions. `datastar@1.0.0-RC.7` client pairs with Rust SDK `0.3.1`.

**Version mismatch breaks silently.** beta.11 emits `datastar-merge-fragments`. RC.7 emits `datastar-patch-elements`. If protocol names don't match, patches arrive and nothing happens.

Always vendor Datastar JS locally — CDN URLs with `@version` suffixes get URL-encoded badly and break.

## Server pattern

```rust
use axum::response::sse::{Event, KeepAlive, Sse};
use datastar::prelude::{PatchElements, PatchSignals};
use asynk_strim::{Yielder, stream_fn};
use std::convert::Infallible;

pub async fn my_sse_handler() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream_fn(|mut y: Yielder<Result<Event, Infallible>>| async move {
        // Initial state — fires immediately
        let html = render_current_state();
        let _ = y.yield_item(Ok(PatchElements::new(html).write_as_axum_sse_event())).await;

        // Stream pushes forever
        loop {
            wait_for_some_change().await;
            let html = render_current_state();
            let _ = y.yield_item(Ok(PatchElements::new(html).write_as_axum_sse_event())).await;
        }
    }))
}
```

Key points:
- **Return type must be `Sse<impl Stream<...>>`** — not `impl IntoResponse` (won't match handler trait)
- **`stream_fn` over `async_stream::stream!`** — cleaner lifetime handling for `PatchElements`
- **Loop forever** for push. Don't close unless the client disconnects.
- **HTML fragments need a top-level element with a matching `id`** — Datastar morphs by ID

## Client pattern

```html
<script type="module" src="/datastar.js"></script>

<div id="my-view" data-init="@get('/api/my-sse', {requestCancellation: 'disabled'})">
  Loading...
</div>
```

Key attributes:
- **`data-init`** — fires once when the element mounts. Use this for auto-load.
  - NOT `data-on-load` (that's window load event, may fire before element exists)
  - NOT `data-on-init` (wrong name)
- **`{requestCancellation: 'disabled'}`** — prevents multiple `data-init`s from killing each other's streams.
- **`data-on:click="@get(...)"`** — colon syntax for standard browser events
- **`data-on-interval__duration.500ms`** — dash syntax for plugin attributes like intervals
- **`data-on-intersect`** — fires when element enters viewport (intersection observer)

## Morphing rules

Datastar uses `id` attributes to match patches to DOM elements. The incoming HTML's top-level element's `id` finds the same-id element in the DOM and morphs.

```rust
// Top-level must have the same id as the target in the DOM
fn render_jobs(jobs: &[Job]) -> String {
    format!(r#"<ul id="jobs-list">{rows}</ul>"#, rows = ...)
}
```

To append instead of replace, use mode:

```rust
PatchElements::new(html)
    .selector("#feed")
    .mode(ElementPatchMode::After)
```

## Signals (reactive state)

```html
<div data-signals:counter="0">
  <span data-text="$counter">0</span>
  <button data-on:click="$counter++">+</button>
</div>
```

Server patches signals with `PatchSignals`:

```rust
let patch = PatchSignals::new(r#"{"counter": 42}"#);
yielder.yield_item(Ok(patch.write_as_axum_sse_event())).await;
```

## Auth for SSE streams

Browser `EventSource` / fetch streams can't set headers. Pass the auth token via query string:

```
/api/my-sse?token=...
```

Template the token into the HTML server-side when rendering the view:

```rust
let html = template_html.replace("{{SHIP_TOKEN}}", &caller_token);
```

Then in the view:

```html
data-init="@get('/api/my-sse?token={{SHIP_TOKEN}}', {requestCancellation: 'disabled'})"
```

Ship's auth middleware already accepts `?token=...` as a query fallback for Bearer.

## CORS

If the view is cross-origin (e.g., Studio at `:3000` talking to daemon at `:9315`), CORS headers must allow `datastar-request`:

```rust
"access-control-allow-headers": "content-type, authorization, accept, datastar-request, ..."
```

Missing this header → Datastar's fetch gets blocked by browser CORS preflight → "NetworkError" in console.

## Debugging

**Stream works in curl but not browser:**
- Check Datastar client version matches Rust SDK version
- Check browser console for "Error in input stream" (malformed SSE) or "NetworkError" (CORS/auth)
- Open Network tab → your SSE request → watch response chunks arrive

**`data-init` doesn't fire:**
- Verify the Datastar version supports `data-init` (RC.7+ does, beta.11 doesn't)
- Confirm the script loaded and ran (console: `window.datastar` or similar probe)
- Make sure element ID is unique

**Stream closes immediately:**
- `requestCancellation: 'auto'` (default) cancels previous requests to the same URL. Set to `'disabled'` for long-lived streams.
- Server handler returning too early — ensure `loop {}` or `pending().await` keeps it open

**Patches arrive but DOM doesn't update:**
- Element ID mismatch — Datastar morphs by `id`, the outer wrapper must match
- Wrong mode — default is `Morph`, use `.mode(ElementPatchMode::After)` for append

## Pattern: streaming from another SSE source

When the daemon's event stream needs to be relayed to a Datastar view:

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
        // Parse upstream event, render HTML, emit Datastar patch
    }
}
```

Disable compression (`.no_gzip().no_brotli().no_deflate()`) — some servers buffer compressed streams which breaks real-time delivery.

## Ship's reference implementation

- **`apps/shipd/src/views.rs`** — `pipeline_monitor_sse` shows the full pattern: subscribes to kernel events, renders HTML on each event, pushes via PatchElements
- **`.ship/views/pipeline-monitor/`** — the client view with data-init + requestCancellation
- **Token inlining** — see `serve_view` which templates `{{SHIP_TOKEN}}` into the HTML

## Future directions (not built yet)

1. **Rust macros for type-safe fetch**: codegen a Rust-side API client from the Datastar view manifest, so handlers and the view share types via specta
2. **Web component library paired with Datastar**: `<ship-diff>`, `<ship-spec-fields>`, `<ship-job-card>` — rich components that accept JSON-like attributes, server renders with `data-*` bindings
3. **AI-generated views**: LLM outputs HTML with Datastar attributes. Token-efficient because no JS framework syntax. Server provides the endpoints.
4. **Canvas integration**: ship-canvas nodes render as Datastar views, live-updating from server state
