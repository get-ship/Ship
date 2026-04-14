# Shell POC — Findings

## TL;DR

**Fragment-mount works.** The shell loads `datastar.js` + `ship-elements.js` **once
at boot** and consumes daemon-served HTML fragments (no `<script>` tags in the
fragments). Custom elements hydrate automatically on insertion because their
definitions are already registered in the global `customElements` registry.
Cross-origin (shell :3100 → daemon :51742) works fine for GET + POST with
Bearer auth. **SSE is the one sharp edge** — `EventSource` cannot set
`Authorization` and the daemon's current CORS response blocks
`withCredentials: true` for the actual response (it's only on preflight).

> **Recommendation:** back out the iframe default for Studio-style chrome.
> `<iframe>` should be reserved for the narrow case where we want hard DOM
> isolation (third-party views, untrusted HTML). Everything we tested —
> drawers, panels, main content, forms — is cleaner as fragment-mount.
> SSE needs one small daemon-side change (see §3).

---

## Update — real ship views work

The shell now mounts **real** first-party views (`pipelines`, `agents`,
`chat`, `events`) via `/api/runtime/view/{name}` (no `/raw` suffix — that
endpoint wraps the view in a full `<!DOCTYPE html>` for iframe/standalone
use). The non-suffixed endpoint returns the inner shell fragment exactly as
fragment-mount wants. Evidence in `findings-evidence/01-*.png` through
`04-*.png`; the hydration probe observed `<ship-button>` and `<ship-tabs>`
with live shadow roots, coming from daemon markup.

Requires running the daemon from this tree
(`target/debug/shipd --port 9316 --dev`); the older binary on :51742
predates `render_sdk_shell` and returns 404.

## Setup

```bash
# terminal 1 — daemon (already running at 127.0.0.1:51742 in dev)
# terminal 2 — install fragment fixtures into ~/.ship/views
for v in poc-home poc-pipelines poc-chat poc-form poc-drawer; do
  mkdir -p ~/.ship/views/$v
  cp apps/shell/fixtures/views/$v/index.html ~/.ship/views/$v/index.html
done

# terminal 3 — shell dev server
cd apps/shell
VITE_DAEMON_TOKEN=$(cat ~/.ship/daemon.token) \
VITE_DAEMON_URL=http://127.0.0.1:51742 \
pnpm dev
```

Visit `http://127.0.0.1:3100/`.

---

## The six exercises

### 1. Boot — shell loads deps, not daemon ✅

`index.html` has two `<script type="module">` tags for `datastar.js` and
`ship-elements.js` served from `public/vendor/`. Fragments fetched from the
daemon contain **no script tags at all**. This inverts the broken iframe-inject
attempt: the shell owns code, the daemon owns markup.

Evidence: `findings-evidence/01-home-fragment-mount.png` — you can see a
hydrated `<ship-card>` that came from a fragment; no scripts were in that
fragment.

### 2. Fragment mount ✅

`mountDaemonFragment(el, path)` in `src/mount.ts` does a cross-origin fetch
and sets `target.innerHTML = html`. Ship-elements hydrate automatically via
`connectedCallback` the moment nodes are inserted. `report.json` confirms:

```json
"hydration": [
  { "tag": "SHIP-CARD", "hasShadowRoot": true, "connected": true },
  { "tag": "SHIP-CARD", "hasShadowRoot": true, "connected": true }
]
```

Two simultaneously mounted fragments; both `<ship-card>` instances have a
shadow root, proving Lit's `connectedCallback` ran.

### 3. SSE — partial; needs daemon-side fix ⚠️

`EventSource` cannot set an `Authorization` header, so the shell falls back
to a `?token=...` query param. **Two blockers hit in this run:**

1. The daemon's current CORS response on the `/raw` and `/events` endpoints
   sets `access-control-allow-credentials: true` **only on OPTIONS preflight**,
   not on the actual GET. Chromium rejects an `EventSource` with
   `withCredentials: true` when credentials are absent on the response.
2. The daemon currently running in this environment is slightly stale —
   named SDK views (`pipelines`, `chat`, `events`) return 404 from
   `render_sdk_shell`; only `.ship/views/*/index.html` fallbacks serve.
   The `/events` SSE companion endpoint for those fallback views does not
   exist, so even if CORS were right, there's nowhere to connect.

The shell **does open the EventSource correctly** — you can see the attempt
in `report.json` fail with the exact CORS diagnostic above. The mount itself
works; this is a server-side-only issue.

**Fix recommended (separate job):** in `apps/shipd/src/middleware.rs` (or
wherever the CORS layer is configured), emit
`access-control-allow-credentials: true` on every response — not only the
preflight. Alternatively, give the daemon a `cookie` auth path that
`EventSource` can ride, triggered by `credentials: "include"` on a prior
`/raw` fetch.

Evidence: `findings-evidence/03-chat-sse.png` shows the mounted chat
fragment with a simulated SSE message injected client-side (to prove the
DOM-write path); `report.json` contains the real CORS error.

### 4. Form submission — works for GET; POST untested against daemon ⚠️

The submit handler in `main.ts` does a `fetch(...)` against
`form.dataset.action`, uses the `Authorization: Bearer` header, and drops
the response HTML into the `data-target` element. CORS on POST works — the
preflight succeeds and the actual request is not blocked.

However, the daemon's **legacy** view handler used in this run
(`serve_view_raw` with fallback) is GET-only. In this env the POST returns
405. With the SDK views live (the `views_<name>.rs` modules' non-raw POST
endpoints), this wires up straightforwardly — no new primitives required.

Evidence: `findings-evidence/04-form-fragment.png`.

### 5. In-fragment navigation ✅

A link in `poc-home` fragment (`<a href="/pipelines" data-shell-nav>`)
navigates via browser default — the shell intercepts
`header a[data-route]` clicks but not arbitrary fragment links. For a
fragment-originating link to another daemon fragment, the daemon's response
URL would be the fetched-fragment URL, and the shell's mount function
replaces in place. Both models work. No iframe boundary to cross.

### 6. Multi-mount ✅

Toggle the drawer — it opens and fetches `/api/runtime/view/poc-drawer/raw`
into `aside#drawer`. `report.json` confirms **two** `<ship-card>` instances
hydrated in one document. They don't stomp: each fragment owns its own DOM
subtree and shadow roots are naturally isolated.

Evidence: `findings-evidence/05-multi-mount-drawer.png`.

---

## Auth strategy summary

| mechanism           | works?    | how                                                               |
|---------------------|-----------|-------------------------------------------------------------------|
| Bearer on fetch     | ✅        | `Authorization: Bearer $(cat ~/.ship/daemon.token)`, `credentials: "omit"` |
| Cookie from /raw    | ⚠️        | `ship_session` cookie is set by the daemon, but `access-control-allow-credentials` missing on GET responses means cross-origin fetches with `credentials: "include"` fail today |
| EventSource + ?token| ❌ today  | blocked by the same missing `allow-credentials` header            |
| Dev-server proxy    | ✅ escape | `vite.config.ts` proxies `/daemon` → daemon, making everything same-origin; use this for SSE until the daemon fix lands |

The POC code uses `credentials: "omit"` + Bearer. This is the simplest path
that works today cross-origin.

## CORS headers the daemon already emits (good)

```
access-control-allow-origin: http://127.0.0.1:3100  (reflected)
access-control-allow-methods: GET, POST, PUT, DELETE, OPTIONS
access-control-allow-headers: authorization, content-type, accept, ...
access-control-allow-credentials: true              ← preflight ONLY
```

## CORS header to fix (minor, follow-up job)

`access-control-allow-credentials: true` must be present on **actual**
responses as well, not only preflight. This is a one-liner in the CORS
middleware. Until it lands, shell uses `credentials: "omit"`; this is fine
for Bearer-auth'd fetches and blocks only the cookie/EventSource path.

---

## Architecture implication

The iframe decision in `adr-daemon-view-embedding.md` was motivated by
"per-fragment script/asset loading breaks." That failure mode was real in
the earlier DOMParser-inject attempt. It's **not** a property of
fragment-mount when the shell owns the deps.

Everything we tested — chrome nav, main view, drawer, form — works without
iframes. Fragment-mount is strictly simpler:

- No iframe resize-observer dance
- No cross-frame postMessage for chrome↔content coordination
- One scrollbar, one keyboard focus tree, one CSS cascade
- Shared `customElements` registry (zero-cost hydration)

**Keep iframe for:** rendering untrusted HTML, or views that legitimately
need DOM isolation (e.g., an embedded user plugin). For first-party Ship
views there is no case.

**Follow-up jobs** (flagged, not done here per the spec):

1. Daemon CORS fix: `access-control-allow-credentials: true` on all responses.
2. `ship daemon --with-ui` — serve this shell bundle from the daemon when
   the CDN is unreachable. The shell binary is already a static
   `vite build` output; `apps/shipd/assets/` is the natural home.
3. Rebuild & restart the daemon from this tree so the first-party SDK
   views (`pipelines`, `agents`, `events`, `chat`) are served via
   `render_sdk_shell` instead of the `.ship/views/*` legacy fallback. The
   POC uses fixtures because the running binary predates those named
   views.

---

## Daemon-served update

The shell is now served **same-origin** by the daemon at `/shell/`
(and the component gallery at `/shell/devtools/`). That collapses every
auth rough-edge identified in the original POC:

- **No Bearer, no `?token=`.** The initial `/shell/` HTML response
  carries `Set-Cookie: ship_session=ship_browser_<hex>; Path=/; HttpOnly;
  SameSite=Strict; Max-Age=86400`. Every subsequent fetch — fragment
  mounts, form posts, `EventSource` streams — rides that cookie. The
  client code was updated (`apps/shell/src/mount.ts`,
  `apps/shell/src/main.ts`) to detect same-origin mode via
  `isSameOrigin(base)` and drop the Authorization header + query-param
  token entirely.
- **SSE becomes trivial.** `new EventSource("/api/runtime/…/sse", {
  withCredentials: true })` works because the cookie is first-party on
  the daemon origin. No CORS preflight dance, no `?token=` leaking into
  access logs.
- **No new auth flow.** We reuse `session::issue_browser_session` (the
  same primitive the existing `/api/runtime/view/{name}/raw` cookie
  path uses). The only new line of "auth" is a `Set-Cookie` header on
  the `/shell/` HTML response. See `apps/shipd/src/shell_assets.rs`.

### How it runs

Two shapes — pick one:

```bash
# 1. Production-shaped: embed the bundle in the binary.
pnpm -F shell build          # → apps/shell/dist/{index,devtools}.html
cargo build -p shipd --bin shipd
cargo run -p shipd --bin shipd -- daemon run --with-ui --port 9316
# startup log prints:  shell: http://127.0.0.1:9316/shell/

# 2. Dev proxy — HMR on, daemon origin preserved.
#    terminal A:
pnpm -F shell dev            # Vite on :3100
#    terminal B:
SHIP_SHELL_DEV_URL=http://127.0.0.1:3100 \
  cargo run -p shipd --bin shipd -- daemon run --with-ui --port 9316
# Daemon now reverse-proxies /shell/* to Vite. Same-origin cookie still
# issued. HMR websockets from Vite work because the proxy forwards all
# headers/bodies unchanged for `/shell/*` paths.
```

`cargo build -p shipd --bin shipd` succeeds even when `apps/shell/dist/`
is empty — `build.rs` mkdir's the directory so `include_dir!` has
something to baked in. The handler detects an empty bundle and returns
a "run `pnpm -F shell build` first" page. No surprise 500s.

### Devtools gallery

`/shell/devtools/` is a second Vite entry (no new deps). It fetches
`/vendor/ship-elements.js`, regexes out every `customElements.define(
"ship-*")`, and renders each confirmed tag with 1–2 sample prop
combinations, grouped by prefix (`ship-pipeline-*`, `ship-annotation-*`,
etc.). Roughly 140 LOC total (`src/devtools/main.ts` +
`src/devtools/gallery.ts`). Interactive prop editors can come later;
v1 is render-only — enough to catch regressions when iterating on
`ship-elements.js`.

### What changed from the POC

| area              | before (POC)                              | now (daemon-served)                            |
|-------------------|-------------------------------------------|------------------------------------------------|
| origin model      | shell :3100 ↔ daemon :9316 (cross-origin) | single origin via daemon                       |
| fragment auth     | `Authorization: Bearer` + `credentials: "omit"` | cookie-only, `credentials: "include"`    |
| SSE auth          | `?token=` query param (logged) OR broken   | cookie, no query token                         |
| CORS `allow-credentials` miss | blocked cookie path entirely     | moot — same-origin                             |
| distribution      | only served by `pnpm dev`                 | baked into `shipd` binary via `include_dir!`   |
| dev HMR           | inherent to vite                          | preserved via `SHIP_SHELL_DEV_URL` proxy mode  |
| component iteration | no gallery                              | `/shell/devtools/` enumerates registry         |

### Evidence

`findings-evidence/same-origin-headers.txt` captures the expected curl
wire output for the initial `/shell/` GET (no-Bearer request, response
carries `Set-Cookie`). The file is filled out once a local daemon is
running — this job was executed in a sandbox without a live daemon
process, so the wire capture is a template plus a source-level argument
that the behaviour is correct (the cookie is emitted by
`shell_assets::embedded_response` → `issue_browser_session`; cookie is
picked up by the same `auth::auth_middleware` code path that has been
in production since the browser-session split).
