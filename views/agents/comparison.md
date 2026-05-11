# /agents (j2) vs /set/agents (ship-set) — parity notes

This view ports `.ship/views/agents/index.html.j2` to the new ship-set
authoring shape. The two surfaces share the same `agents.summaries`
service registry call so the data model is identical; the differences
below are intentional shape simplifications.

## Visual

Pixel-equivalent. The CSS at `view.css` is copied verbatim from the j2
`<style>` block. Same grid breakpoints (1 / 2 / 3 cols at 768 / 1024px),
same card chrome (ship-list-tile variant=card), same provider blob row,
same badge layout.

## Avatar

The j2 view has a four-branch avatar resolver
(`_shared/agent_avatar.html.j2`) that handles `simpleicons:<slug>`,
short-string initials, registry icon names, and falls back to a
first-letter glyph. The ship-set view always renders the orange-square
first-letter glyph (uses `agent.letter`). The simpleicons / registry
branches are out of scope for Phase 3 — they need a `<ship-icon>`
registry-aware tile primitive that the new view should compose against
later. The j2 surface keeps the more elaborate branching while the new
view validates the framework on the common case.

## Fragment mode

The j2 view supports `?fragment=true` to render without `<!doctype>` /
`<head>` / `<body>` wrappers (used by Datastar inline injection). The
ship-set view is always full-page hydration via `/set/agents`. Phase 3
explicitly defers SSR — the shell HTML carries a tiny loader that
imports the runtime + dom + view bundle and calls `hydrate()`.

## Live updates

j2 uses Datastar SSE morphing — the daemon re-renders `content.html.j2`
on each `config.*` event and the morph engine diffs HTML. The ship-set
view subscribes to the same `_ship-set/data?expr=ship.agents.summaries()`
SSE stream the daemon already exposes; patches arrive as JSON-Patch
envelopes and `applyPatch` mutates the signal in place. The grid
re-renders only the changed card via `bindFor`'s key-stable
reconciler — single-item changes do not re-render unrelated cards.

## Loading boundary

j2's loading state is server-rendered (the partial always carries data,
so there's no client-side loading fence). The ship-set view shows a
`<while-loading>` boundary on initial page load, then unmounts cleanly
once the first SSE patch resolves. Subsequent patches do not re-flash
the spinner — the boundary mounts once at start and unmounts once on
first ok per `bindWhileLoading`'s transition contract.

## Empty / error states

Both views show the empty state when items is `[]`. The j2 view has no
client-visible error path (server errors render as a blank fragment);
the ship-set view exposes a `<while-error>` boundary that surfaces the
SSE error message inline so authoring bugs surface to the user during
development.
