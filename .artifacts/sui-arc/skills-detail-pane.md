# Skills detail pane — render content on selection

## Problem

After `skills-tree-interactive.md` re-wired the rail to set `skillId`,
selecting a skill flipped the right pane from `No file open` to a bare
`<sui-markdown-editor>` of the SKILL.md body plus a metadata sidebar.
Functional but thin: no header, no kind framing, no description, no
discoverable invocation surface, no history of past runs. The job
spec for `job/skills-detail-pane` calls out four affordances:

1. Header — name, kind chip, description.
2. Tabs — SKILL.md (rendered) / Metadata (frontmatter) / Runs (recent
   invocations).
3. Top-right Invoke button with inline result (run_id + session_id).
4. Source link — toggle to the raw SKILL.md.

## Layout

The right pane is now a vertical stack inside `.skills-editor`:

```
┌─ .sd-header ───────────────────────────────────────────────┐
│ h1.sd-name   <sui-chip kind>   USER SKILLS    [Source][Invoke] │
│ p.sd-description                                           │
│ <sui-alert>  (inline invoke result, dismissible)           │
├─ .sd-tabs-row ─────────────────────────────────────────────┤
│ <sui-tabs underline> SKILL.md · Metadata · Runs (count)    │
├─ .sd-tab-body ─────────────────────────────────────────────┤
│   SKILL.md      → sui-markdown-editor (or raw <pre>)       │
│   Metadata      → dl + chips + variable / file lists       │
│   Runs          → <ul> of recent skill.started events      │
└────────────────────────────────────────────────────────────┘
```

The old fixed-width `<aside class="skills-info">` metadata sidebar is
gone — its content moves into the Metadata tab so the detail pane
gets the full width of the editor column and the layout stays a
familiar two-pane explorer + content shell.

## Data wiring

All sources route through `/_sui/data?expr=…` because the view-host
mounts views without an SDK client in ctx.

| Signal       | Expression                                                                 | Triggered by               |
|--------------|----------------------------------------------------------------------------|----------------------------|
| `rail`       | `ship.skills.summaries({q, kind})`                                         | search / kind filter       |
| `skill`      | `ship.skills.detail({id})`                                                 | rail row click             |
| `runs`       | `ship.events.list({event_type:"skill.started", entity:id, limit:20})`      | rail row click             |

`runs` short-circuits when `skillId` is empty — the source emits
`{rows:[]}` synchronously instead of opening an EventSource against
`__none__`, which keeps the Runs tab empty-state visible before any
selection.

The Runs tab flattens the registry's `{groups:[{rows}]}` shape into
`{rows}` inside `runsSource` so the template only deals with a flat
list. With `entity:id` as the only filter, there's always at most one
group, so the flatten is a no-op or a single concat.

## Invoke

`POST /api/v1/skills/{id}/invoke` already returns
`{ok, run_id, skill_id, session_id, workspace_id, mode, reused_session}`
(see `apps/shipd/src/services/skills/mod.rs:415`). The controller
parses the response into a discriminated `InvokeResult` and writes it
to the `invokeResult` signal, which the header renders as a
`<sui-alert>` showing `Invoked · run <id> · session <id>` on success
or the error message on failure. The alert is dismissible. The button
maps to the active skill (`skill().value.id`), not the rail row, so
clicking Invoke from the detail pane always invokes the skill the
user is looking at.

## Source toggle (deviation from spec)

The spec called for "Source link: opens the raw SKILL.md in dock's
Source tab." That isn't deliverable inside the file scope — the dock
Source tab is implemented as `<ship-devtools-source>` in
`apps/shell/src/chrome/devtools/source-tab.ts`, which hard-binds to
the active view's template path via `/api/v1/view-source/{path}`. It
has no codepath for displaying a skill's SKILL.md, and extending it
would mean editing `apps/shell/`, which is excluded by the job
spec's file scope (`Forbidden: ANY change to … apps/shipd/, apps/`).

Pragmatic substitute: the Source button toggles an in-pane raw view
of `skill.value.body` rendered as `<pre><code>` instead of the
rich-markdown editor. The toggle is in the header next to Invoke so
both affordances live in the same row, and the button reflects state
with `aria-pressed` plus a `.is-active` style. Following up the dock
integration belongs on a job that owns `apps/shell` changes.

## File budgets

| File                                    | Lines | Cap   |
|-----------------------------------------|------:|------:|
| `views/skills/page.sui`                 |   284 |  300* |
| `views/skills/controller.ts`            |   175 |   300 |
| `views/skills/style.css`                |   462 |  none |

\* SUI templates aren't subject to the engineering 300-line cap the
same way TS/Rust files are, but the page is split-ready: the Metadata
and Runs panels are self-contained `{#if tab()==='…'}` branches and
could move into separate components without restructuring the rest.

## Acceptance map

1. **Selecting a skill renders meaningful content in the right pane.**
   The detail pane now renders the header + tabs as soon as
   `skill().kind === 'ok'`. Empty-state copy is updated to mention
   invocation.
2. **Invoke button calls real endpoint.** `controller.ts` posts to
   `/api/v1/skills/{id}/invoke` and parses `run_id + session_id` from
   the response into the inline banner.
3. **0 console errors.** All signals fall back gracefully —
   `runs` short-circuits before EventSource creation when there's no
   skill selected, `skill` queries `__none__` (404 → error state,
   not a thrown exception), and the invoke result signal never holds
   undefined values that the template would deref.
