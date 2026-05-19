# Studio — path from where we are to where we want to go

The 3-pane Studio mock (`page.html` + `mockboard.html`) was a shape-finding
exercise. **It's not the deliverable.** Ship already has the building blocks
laid out in the canonical pattern: a per-domain route in the shell rail,
list view at `/<domain>`, detail view at `/<domain>/[id]`, optional deep
sub-routes (`/agents/[id]/wire`, `/agents/[id]/settings`).

"Studio" is the **design language** applied across those routes — not a new
top-level route. The shell chrome already provides the rail; per-domain
views provide the content.

## Current state

| Route | What exists today |
|---|---|
| `/agents` | List + filter + grid of cards (`views/agents/page.sui`) — refined aesthetic, 1024px wide, hue letters, provider blobs |
| `/agents/[id]` | Overview with `sui-view-chrome` + `sui-segmented` (Overview/Wire/Settings), big `sui-avatar`, stat row, `sui-collapsible` sections for Description / Skills / MCP / Rules / Permissions / Providers / Source |
| `/agents/[id]/wire` | Three-pane wire visualizer (palette · canvas · inspector) |
| `/agents/[id]/settings` | Edit form |
| `/agents/create` | 4-step wizard (Template / Identity / Wiring / Review) |
| `/skills` | List view (basic) |
| `/pipelines` | List view (basic) |
| `/theme-studio` | Token editor + live preview |
| `/ship-ui-studio` | Component catalog + preview |
| `/mcp` | **doesn't exist** |
| `/permissions` | **doesn't exist** |

## Where we want to go

Same routing topology, design-system applied uniformly:

| Route | Polish move |
|---|---|
| `/<domain>` (list) | Same shape as current `/agents` — refine spacing/typography, add wherever missing |
| `/<domain>/[id]` (detail) | Same shape as **the polished `/agents/[id]`** (see `agent-detail.html`) — single 880px column, hue tile + title + gradient underline, prose description, hairline-separated sections, no collapsibles |
| `/<domain>/[id]/<deepview>` | Type-specific sub-routes (wire, settings, tokens, body, graph, …) — keep what's already there |

## The two pieces I'm shipping in this pass

1. **`agent-detail.html`** — the canonical detail page. Single column,
   elegant typography, hairline sections, real sui components. This is
   what `/agents/[id]` should evolve into. Once locked, every other
   `/<domain>/[id]` mirrors it.

2. **`agents-list.html`** — a small polish on the existing `/agents` list.
   Mostly stays as-is; the few things to tighten are spacing rhythm and
   making the card less busy. The existing source is good; the diff is small.

## Order to actually do this

1. Land the polish on `/agents` and `/agents/[id]` upstream.
2. Mirror onto `/skills/[id]` (which doesn't have a detail page yet).
3. Mirror onto `/pipelines/[id]`.
4. Create `/mcp` + `/mcp/[id]`.
5. Create `/permissions` + `/permissions/[id]`.
6. Fold `/theme-studio` → `/themes/[id]` (the token editor becomes the
   deep sub-view, list view becomes the canonical list shape).
7. Fold `/ship-ui-studio` → `/components/[id]`.

Each step is a real PR against `madvib/Ship`. None of them are big.

## What's *not* in this design

- A `/studio` top-level route. The shell rail already gives us navigation.
- Runtime peek-ins (active counts, run history, status pulses). Out of
  scope per the conversation; goes to `/sessions`, `/events`,
  `/pipelines/history`, `/dashboard`.
- Speculative dev tools without backing SDK ops (eval suites, definition
  diff, MCP tool inspector). Land the SDK op first, then the surface.

## What `page.html` + `mockboard.html` are now

Reference / archive. They show the failed framing — useful to compare
against. Don't extend them.
