---
name: ship-canvas
stable-id: ship-canvas
description: Use when the conversation needs a visual surface — mockups, data visualizations, diagrams, design reviews — where the human wants to watch and guide your work from a single pane. You produce a live Ship view (not a static HTML file) that lets the human send structured feedback back to you via interactive widgets you design.
tags: [design, canvas, visualization, feedback, extension-authoring]
authors: [ship]
artifacts: [ship-view]
---

# Ship Canvas — interactive visual surface between agent and human

You're producing a **live, interactive Ship view** — not a static HTML file. The human loads it in Studio or the shell, interacts with widgets you design, and their input flows back to you as structured feedback on your next turn.

This is the single-pane loop: human watches, you build, human rates, you iterate.

**Prerequisite:** Read `ship-views` skill first. This skill assumes you know the Ship View SDK (macros from `_ship.html.j2`, ship-elements, Datastar patterns). This skill is the canvas-specific guidance on top.

## What to produce

Drop files into `.ship/views/canvas-<slug>/`:

- `manifest.yaml` — usually trivial (no data loader, optional events)
- `index.html.j2` — the Jinja template using the Ship View SDK
- (optional) `partials/content.html.j2` — if you want SSE-morphable sections

The daemon serves it live at `/canvas-<slug>`. Edit the file → human refreshes → sees changes. No build step, no restart.

## The feedback contract — you design the payload

The human's feedback doesn't have to be "drew a box, added a note." You decide what feedback shape makes sense for what you're showing. Wire each interactive widget to a payload you designed.

```jinja
{% from '_ship.html.j2' import feedback %}

{# You designed this payload shape. You'll read it on your next turn. #}
<ship-button {{ feedback("{option: 'A', rating: 5}") }}>5 stars for A</ship-button>

{# Compose signals into the payload #}
<ship-input {{ bind('note') }} placeholder="Quick thought..."></ship-input>
<ship-button {{ feedback("{kind: 'comment', text: $note}") }}>Send</ship-button>

{# Any JSON-serializable shape works #}
<ship-button {{ feedback("{action: 'approve', section: 'hero', timestamp: Date.now()}") }}>
  Approve hero
</ship-button>
```

Each click emits `studio.annotation.created` with your payload attached. You read it on your next turn and iterate.

**Design rule:** the payload should be structured enough that you can act on it in one prompt. Free-text requires you to parse — enums and numbers don't. Prefer:
- `{option: 'A', rating: 5}` over `"rating: A-5"`
- `{action: 'approve', section: 'hero'}` over `"approve hero"`
- `{kind: 'copy-direction', preference: 'bold'}` over `"make it bolder"`

The human still has free-text (via `bind` + `feedback`) when they want to say something unstructured. You're just giving them faster structured buttons for common responses.

## The single-pane loop

Typical session:

1. Human: "show me two directions for the hero"
2. You: write `.ship/views/canvas-hero/index.html.j2` with two mocks + rating buttons
3. Human: loads the view, clicks ratings, types a comment
4. You: receive `{option: 'A', rating: 4}`, `{option: 'B', rating: 2}`, `{kind: 'comment', text: '...'}`
5. You: edit the same file — now three directions, rating buttons preserved
6. Human: refreshes, rates again
7. Iterate until one is approved, then `{action: 'approve', section: 'hero'}`

**Save in the same file** for iteration — the daemon hot-reloads. Past versions live in git history, not as `v2.html` files. Clean filesystem.

**When the human approves**, produce a text reply summarizing what was learned and where the final artifact lives (`.ship/views/canvas-hero/index.html.j2`). The human can copy that into wherever it needs to go (Studio view, landing page, etc.).

## What to make

- **Mockups** — UI layouts, pages, components, design explorations
- **Data visualizations** — charts, graphs, dashboards (use `<ship-bar-chart>`, `<ship-line-chart>`, `<ship-gauge>`, `<ship-donut>`, `<ship-sparkline>`)
- **Diagrams** — architecture, relationships, flows
- **Design reviews** — side-by-side options, ratings, pick-one
- **Interactive reports** — "scroll through these findings, flag any that look wrong"

Not a fit: rich interactive editors (canvas games, video timelines), anything that needs frames per second. Those belong in a full client framework.

## Visual quality

You're a senior designer who codes. Render like it's production:

- **Use ship-elements** — don't hand-code buttons, cards, selects, toolbars. They exist in the library.
- **Use design tokens** — `var(--space-*)`, `var(--text-*)`, `var(--font-display)`, `var(--foreground)`, etc. No hardcoded px or hex. The `ship-views` skill has the full token list.
- **Use the component `tone × variant` model correctly** — `tone="accent" variant="solid"` for primary CTAs, `tone="destructive" variant="soft"` for destructive, `variant="ghost"` for secondary. See `ship-components` skill.
- **Realistic content** — no lorem ipsum, no "Chart Title", no "Label". Write the actual copy.
- **Clear hierarchy** — display font for titles, body font for copy, monospace for code/IDs, proper weight/size distinctions.
- **One concept per canvas** unless the human asked for a dashboard. Whitespace > density.

## Template starter

```jinja
{% from '_ship.html.j2' import head, signal, bind, feedback, styles %}
<!DOCTYPE html>
<html lang="en">
<head>{{ head('Canvas — <slug>') }}
  {{ styles() }}
  <style>
    .canvas-wrap { max-width: 64rem; margin: 0 auto; padding: var(--space-6) var(--space-5); }
    /* your layout styles — stay minimal, use tokens */
  </style>
</head>
<body>
<div class="ship-view" {{ signal('note', '') }}>
  <div class="canvas-wrap">

    {# your content here #}

    {# rating widget — agent designs the payload #}
    <ship-button {{ feedback("{option: 'A', rating: 5}") }}>Approve A</ship-button>

    {# free-text capture #}
    <ship-textarea {{ bind('note') }} placeholder="..."></ship-textarea>
    <ship-button tone="accent" variant="solid"
      {{ feedback("{kind: 'comment', text: $note}") }}>Send note</ship-button>

  </div>
</div>
</body>
</html>
```

## Reading feedback on your next turn

Feedback arrives as `studio.annotation.created` events in your session. Each event has:

```json
{
  "id": "01J...",
  "kind": "action",
  "target": { "file_path": "/canvas-hero" },
  "author_kind": "human",
  "created_at": "...",
  "payload": { /* your shape */ }
}
```

Read the events list, extract payloads, respond to the patterns. If the human clicked `{option: 'A', rating: 5}` and `{option: 'B', rating: 2}`, they prefer A — iterate on A, maybe generate A2/A3 variants.

## Live updates (SSE) — when your artifact is long-running

If your canvas is long-running (watching a job progress, live dashboard, collaborative review), use SSE so you can push updates without the human refreshing:

1. Add `events:` to the manifest listing kernel namespaces you'll emit to:
```yaml
events: [canvas.update]
```

2. Add a content partial at `partials/content.html.j2` with `id="<slug>-content"`.

3. Your long-running code emits events in that namespace; the generic SSE handler re-renders the partial and pushes.

For most one-shot design reviews you don't need this — just write the view and iterate by editing the file.

## Rules

- **Production quality.** Every canvas should look finished.
- **One file.** Iterate in place. Git is your history, not `-v2.html` suffixes.
- **No raw Datastar.** Use SDK macros. If you hit a gap, note it — don't work around it.
- **No inline hex/px.** Design tokens only.
- **Payload shapes are yours to design.** Choose ones you can act on in one prompt.
- **Ship-elements first.** Check the component catalog before writing CSS.

## Reference

- `ship-views` — primary SDK skill (required read)
- `ship-components` — component library charter
- `.ship/views/canvas-example/` — working reference view (hero-review pattern)
- `apps/shipd/templates/_ship.html.j2` — canonical macro source
