---
group: SDK
title: Authoring loop
order: 4
description: The feedback loop for Ship view authoring. SDK discovery, scaffold, hot-reload, lint, dry-run, runtime errors, iterate.
---

Ship views hot-reload off disk. There is no build step, no bundler,
no framework compile. The loop below is the fastest way to build a
view, whether you're a human in an editor or an agent driving the
MCP.

## 1. Discover — what does the SDK expose?

The live SDK manifest is served by the daemon. Every macro, every
URL, every registered view and collection.

```bash
# Full manifest
curl -s http://localhost:3000/api/sdk | jq .

# Focused query across macros, URLs, views, collections
curl -s 'http://localhost:3000/api/sdk/query?q=collection' | jq .
```

The machine-readable spec is also checked into
`packages/sdk/src/generated/sdk-framework.json` and
`packages/sdk/src/generated/sdk-urls.json` for offline browsing.

## 2. Inspect a view's compile-time context

Before writing the template, look at what the daemon will hand your
Jinja context at render time — available signals, SSE topics, action
URLs, required packs, referenced collections.

```bash
curl -s http://localhost:3000/api/views/<name>/meta | jq .
```

## 3. Edit on disk — it hot-reloads

Views live at `.ship/views/<name>/`:

```
.ship/views/tasks/
  manifest.jsonc
  index.html.j2
  partials/content.html.j2     # optional — rendered on SSE re-render
  _components/*.j2             # optional — view-local Jinja partials
```

Save the file. Refresh the browser. The daemon re-reads from disk on
every request — no cache, no restart. In dev mode the shell also
renders inline lint findings as an overlay when the daemon reports
them.

## 4. Lint — AST + regex rules

Two entry points to the same lint engine:

**CLI:**

```bash
ship lint views
ship lint views .ship/views/tasks/index.html.j2
```

**Runtime HTTP (per-template, for agents mid-edit):**

```bash
curl -s -X POST http://localhost:3000/api/lint/view \
  -H 'content-type: application/json' \
  -d '{"label":"tasks","source":"…raw template…"}' | jq .
```

**MCP tool (preferred for agent loops):** `lint_view`. The MCP server
dispatches to the same engine; give it either a source string or a
view directory and receive structured findings with fix suggestions.

Lint enforces the hard rules (no inline script/style, no raw
`data-on:click` where a macro exists, missing `packs:`, stale packs,
typed-signal mismatches) and surfaces `{#- SDK gap: ... -#}` comments
as warnings.

## 5. Dry-run — render without data

Any view supports a query-string dry-run. `?__dry-run=1` alone echoes
the real Jinja context the daemon would hand the template.
`?__dry-run=1&__context=<urlencoded JSON>` renders the template with
your provided context instead of hitting collections / the kernel.

```bash
# Render with fabricated context, no DB dependency
curl -s 'http://localhost:3000/api/view/tasks?__dry-run=1&__context=%7B%22items%22%3A%5B%5D%7D'

# Inspect the real context without rendering
curl -s 'http://localhost:3000/api/view/tasks?__dry-run=1'
```

Use this to build views against context shapes that aren't fixtured
yet, and to debug "the template renders but the data looks wrong."

## 6. Runtime errors

Every uncaught template error, view render failure, or client-side
fetch error is buffered by the daemon.

```bash
# List recent errors
curl -s http://localhost:3000/api/view-errors | jq .

# Clear
curl -s -X DELETE http://localhost:3000/api/view-errors
```

**MCP tool (preferred for agent loops):** `get_view_errors`. Returns
the buffered errors as an agent-readable blob. The shell's
`error-reporter.ts` also POSTs client-side exceptions into this buffer
so runtime-only bugs surface without a browser devtools round-trip.

## 7. Iterate

The loop in practice:

1. `curl /api/sdk/query?q=<concept>` — find the macro.
2. Edit `.ship/views/<name>/index.html.j2`.
3. Refresh the browser (or dispatch an `@get` in the shell).
4. If the render fails visibly → `get_view_errors`.
5. If it renders but behaves wrong → `?__dry-run=1` to inspect context.
6. Before commit → `lint_view` (or `ship lint views`).

Keep the file under 300 lines. If it grows past that, split into
`_components/*.j2` partials or file a pack-component gap.
