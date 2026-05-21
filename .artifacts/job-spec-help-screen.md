# Job Spec: `⌘?` help screen — keyboard shortcut reference

**Branch:** `job/help-screen`
**Agent:** ship-designer
**Mode:** autonomous

## Goal

Shell-chrome emits `sui-open-help` on `⌘?`. Build the receiver: a small overlay listing keyboard shortcuts. No backing data shape, no SDK calls — pure reference content.

Tiny job. Could share infrastructure with `<sui-command-palette>` (similar overlay shape) or stand alone. Implementer's call.

## File scope

```
apps/shell/src/components/sui-help-overlay/sui-help-overlay.sui        # NEW
apps/shell/src/components/sui-help-overlay/sui-help-overlay.controller.ts # NEW (minimal)
apps/shell/src/main.ts                                                  # mount + handle event
apps/shell/test/components/sui-help-overlay.test.ts                    # NEW
```

DO NOT touch:
- Other shell components.
- Anything outside `apps/shell/`.

## What to change

### `<sui-help-overlay>` element

- Overlay/dialog with backdrop. Esc closes. Click outside closes.
- Centered, ~520px wide.
- Header: "Keyboard shortcuts"
- Body: grouped tables of shortcuts.

### Content (groups + shortcuts)

**Global**
- `⌘K` Open command palette
- `⌘.` Open Studio
- `⌃\`` Toggle terminal
- `⌘?` Open this help
- `Esc` Return to canvas / close overlay

**Studio (when open)**
- `⌘K` Open Studio command palette (scoped)
- `⌘1..7` Jump to section (Agents, Skills, MCP, Pipelines, Components, Themes, Permissions)
- `Esc` or `×` Close Studio

**Editor / forms**
- (placeholder section — populate as features land)

Each shortcut row: kbd visualization on the left (use `<sui-kbd>` for each key, joined by `+`), description on the right.

Use `<sui-kbd>` for the kbd visualizations. Use `<sui-list-tile>` for the rows if it fits the style; otherwise table.

### Wiring

In `apps/shell/src/main.ts`:
- Mount `<sui-help-overlay>` once
- Listen for `sui-open-help` → toggle to open
- Listen for own close events → toggle to closed

## Architectural context

- L3 shell component. Doesn't need a primitive in `@ship/primitives` unless it's reusable (decide based on whether per-domain help is a future feature).
- File cap: 300 lines. Trivial.

## Acceptance criteria

1. `⌘?` opens the help overlay.
2. Esc + backdrop click close it.
3. All shortcuts listed match what the shell emits.
4. All 7 themes apply correctly.
5. `bun run test` in shell passes.
6. Smoke test: `sui-help-overlay` opens, contains expected sections + at least one row per section.

## Out of scope

- Context-aware help (showing only shortcuts active for the current view). Future enhancement.
- Search within help. Future.
- Per-domain help pages. Future.

---

> Execute. Tiny job. One commit reasonable.
>
> When done: complete_workspace + mesh_broadcast.
