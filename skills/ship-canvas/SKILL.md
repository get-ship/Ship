---
name: ship-canvas
stable-id: ship-canvas
description: Use when the conversation needs a visual surface — mockups, data visualizations, diagrams, design reviews — where the human wants to watch and guide your work from a single pane. You produce a live Ship view (not a static HTML file) that lets the human send structured feedback back to you via interactive widgets you design.
tags: [design, canvas, visualization, feedback, extension-authoring]
authors: [ship]
artifacts: [html]
---

# Ship Canvas — interactive visual surface between agent and human

You're producing a **live, interactive Ship view** — not a static HTML
file. The human loads it in Studio or the shell, interacts with widgets
you design, and their input flows back to you as structured feedback on
your next turn.

This is the single-pane loop: human watches, you build, human rates,
you iterate.

**Prerequisite:** Read `ship-views` first. This skill assumes you know
the `.sui` template format, the paired `controller.ts` scope, and the
three-file shape (`page.sui` + optional `controller.ts` + optional
`style.css`). This is the canvas-specific guidance on top.

## What to produce

Drop files into `views/canvas-<slug>/`:

- `manifest.jsonc` — short metadata; usually no data subscriptions.
- `page.sui` — template using `<sui-*>` primitives.
- `controller.ts` — signals + the `sendFeedback` helper that POSTs to
  `/api/v1/feedback` with your payload.
- (optional) `style.css` — only if a layout primitive can't express it.

The daemon serves it live at `/canvas-<slug>`. Edit any file → the
shell hot-reloads → human sees the update. No build step, no restart.

## The feedback contract — you design the payload

The human's feedback doesn't have to be "drew a box, added a note."
You decide what feedback shape makes sense for what you're showing.
Each interactive control wires to a payload you authored.

Wire-format the daemon accepts (POST `/api/v1/feedback`):

```ts
{
  id: string,                   // ULID — generate per click
  kind: 'action' | 'point' | 'region',
  target: { view_id?: string, file_path?: string, /* … */ },
  payload?: unknown,            // ← YOUR shape
  note?: string,                // free-text companion, optional
  created_at: string,
  author_kind: 'human'
}
```

`payload` is opaque to the daemon — the receiving prompt reads it.

```html
<!-- page.sui -->
<sui-cluster gap="3" align="center">
  <sui-button tone="primary" variant="solid"
    on:click={() => sendFeedback({ option: 'A', rating: 5 })}>
    5 stars for A
  </sui-button>
  <sui-button variant="outline"
    on:click={() => sendFeedback({ option: 'A', rating: 3 })}>
    3 stars for A
  </sui-button>
</sui-cluster>

<sui-textarea bind:value={note} placeholder="Quick thought…"></sui-textarea>
<sui-button tone="accent" variant="solid"
  on:click={() => sendFeedback({ kind: 'comment', text: note() })}>
  Send
</sui-button>
```

```ts
// controller.ts
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({
    note: signal(''),
    sendFeedback: async (payload: unknown, extra?: { note?: string }) => {
      await fetch('/api/v1/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: crypto.randomUUID(),
          kind: 'action',
          target: { file_path: location.pathname },
          payload,
          note: extra?.note,
          created_at: new Date().toISOString(),
          author_kind: 'human',
        }),
      })
    },
  }),
})
```

Each `sendFeedback(...)` call emits `studio.feedback.created` through
the kernel; you read it on your next turn via the events API.

**Design rule:** the payload should be structured enough that you can
act on it in one prompt. Free-text requires you to parse — enums and
numbers don't. Prefer:

- `{option: 'A', rating: 5}` over `"rating: A-5"`
- `{action: 'approve', section: 'hero'}` over `"approve hero"`
- `{kind: 'copy-direction', preference: 'bold'}` over `"make it bolder"`

The human still has free-text capture (via `bind:value` + a `note`
field on the payload) when they want to say something unstructured.
You're giving them faster structured buttons for the common responses.

## The single-pane loop

Typical session:

1. Human: "show me two directions for the hero"
2. You: write `views/canvas-hero/{page.sui,controller.ts,manifest.jsonc}` with two mocks + rating buttons
3. Human: loads `/canvas-hero` in the shell, clicks ratings, types a comment
4. You: receive `{option: 'A', rating: 4}`, `{option: 'B', rating: 2}`, `{kind: 'comment', text: '…'}`
5. You: edit the same files — now three directions, rating buttons preserved
6. Human: shell hot-reloads, rates again
7. Iterate until one is approved, then `{action: 'approve', section: 'hero'}`

**Iterate in place.** Past versions live in git history, not as
`page-v2.sui` files.

**When the human approves**, produce a text reply summarizing what was
learned and where the final artifact lives. The human can copy that
into wherever it needs to go (a real product view, a landing page,
etc.).

## What to make

- **Mockups** — UI layouts, pages, components, design explorations
- **Data visualizations** — use the `@ship/charts` primitives
- **Diagrams** — architecture, relationships, flows
- **Design reviews** — side-by-side options, ratings, pick-one
- **Interactive reports** — "scroll through these findings, flag any that look wrong"

Not a fit: rich interactive editors (canvas games, video timelines),
anything that needs frames per second. Those belong in a full client
framework.

## Visual quality

You're a senior designer who codes. Render like it's production:

- **Use the primitives** — never hand-code buttons, cards, selects, toolbars. `<sui-*>` covers all of them. See the `ship-components` skill for the catalog.
- **Use layout primitives** — `<sui-stack>` / `<sui-cluster>` / `<sui-grid>` / `<sui-center>` instead of bespoke `display:flex` CSS. See the `ship-views` skill.
- **Use design tokens** — `var(--space-N)`, `var(--text-base)`, `var(--font-display)`, `var(--foreground)`, etc. No hardcoded px or hex. The compiler enforces this via `sui-css-literal` — raw colors are a hard error.
- **Use the canonical `tone` × `variant` vocabulary** — `tone="primary" variant="solid"` for primary CTAs, `tone="destructive" variant="soft"` for destructive, `variant="ghost"` for tertiary. The compiler enforces this via `sui-prop-value` — unknown values are a hard error.
- **Realistic content** — no lorem ipsum, no "Chart Title", no "Label". Write the actual copy.
- **Clear hierarchy** — display font for titles, body font for copy, monospace for code/IDs.
- **One concept per canvas** unless the human asked for a dashboard. Whitespace > density.

## Template starter

```html
<!-- views/canvas-hero/page.sui -->
<sui-center max="wide" padded>
  <sui-stack gap="6">

    <header>
      <h1>Hero exploration</h1>
      <p>Two directions for the hero section. Rate each, drop a note if it helps.</p>
    </header>

    {#for opt of options() key={opt.id}}
      <sui-card>
        <sui-stack gap="3">
          <h2>{opt.label}</h2>
          <div class="mock">{@html opt.html}</div>
          <sui-cluster gap="2" justify="end">
            {#for n of [1, 2, 3, 4, 5] key={n}}
              <sui-button size="sm" variant="ghost"
                on:click={() => sendFeedback({ option: opt.id, rating: n })}>
                {n}
              </sui-button>
            {/for}
          </sui-cluster>
        </sui-stack>
      </sui-card>
    {/for}

    <sui-stack gap="2">
      <sui-textarea bind:value={note} placeholder="Free-text reaction…"></sui-textarea>
      <sui-button tone="primary" variant="solid"
        on:click={() => sendFeedback({ kind: 'comment', text: note() })}>
        Send note
      </sui-button>
    </sui-stack>

  </sui-stack>
</sui-center>
```

## Reading feedback on your next turn

Feedback arrives as `studio.feedback.created` events in the kernel
stream. Use the events API to pull them:

```bash
ship events list --filter 'type=studio.feedback.created' --since 1h
```

Each event payload matches the `FeedbackEvent` wire shape — pull
`payload` out and act on it. If the human clicked
`{option: 'A', rating: 5}` and `{option: 'B', rating: 2}`, they prefer
A — iterate on A, maybe generate A2/A3 variants.

## Live updates — when your artifact is long-running

If your canvas is long-running (watching a job progress, live
dashboard, collaborative review), wire a `<data>` subscription to a
live source (e.g. `ship.events.tail()` or a domain endpoint). The
runtime keeps the signal current via SSE — no extra plumbing needed.

```html
<data name="run" source="ship.pipeline.run({slug})" live />
```

For most one-shot design reviews you don't need this — just write the
view and iterate by editing the files.

## Rules

- **Production quality.** Every canvas should look finished.
- **Iterate in place.** Same directory, same files. Git is your history.
- **Primitives first.** Check the `<sui-*>` catalog before writing CSS.
- **Tokens only.** The compiler will reject raw hex / rgb / px.
- **Canonical enum values.** The compiler will reject unknown `tone` / `variant` / `size` strings.
- **Payload shapes are yours to design.** Pick ones you can act on in one prompt.

## Reference

- `ship-views` — primary authoring skill (required read)
- `ship-components` — component catalog
- `ship-design-system` — tokens, themes, surfaces
- `apps/docs/src/content/docs/views/examples.md` — worked examples of the three-file shape
- `crates/core/ship-substrate/src/events/feedback.rs` — `FeedbackEvent` source of truth
