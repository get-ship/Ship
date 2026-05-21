# Job Spec: `<sui-command-palette>` primitive + shell wiring

**Branch:** `job/command-palette`
**Agent:** ship-designer
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/shell-mock.html` (the `⌘K`-open palette section, ~lines 380-510 — exact lines may differ; search for `palette` / `cmd-` in the file)

## Goal

The shell-chrome-refactor job emits `sui-open-command-palette` on `⌘K`. Nothing receives it. Build the receiver: a `<sui-command-palette>` primitive that opens as an overlay (centered, ~640px wide, modal backdrop) and provides keyboard-driven navigation across three groups: Open · Run · Settings. This is the **primary navigation surface** of the shell — empty canvas + ⌘K together replace what the rail used to do.

## File scope

```
packages/components/primitives/src/sui-command-palette/sui-command-palette.sui          # NEW primitive
packages/components/primitives/src/sui-command-palette/sui-command-palette.controller.ts # NEW state
packages/components/primitives/src/index.ts                                              # register
packages/components/primitives/test/sui-command-palette.test.ts                          # NEW tests
apps/shell/src/main.ts                                                                   # mount palette + handle event
apps/shell/test/components/shell-chrome.test.ts                                          # extend with palette flow
```

OPTIONAL:
- `crates/sui/compiler/tokens.json` — register any new tokens (verify `--backdrop`, `--shadow-xl`, `--radius-lg` etc. exist)

DO NOT touch:
- `apps/shell/src/components/sui-shell-*` — chrome primitives stay as the refactor agent built them
- `views/studio/**`, `views/agents/**` — separate parallel jobs

## What to change

### `<sui-command-palette>` primitive

Element exposes:
- `open` (bool, reflect) — whether the palette is visible
- `placeholder` (string, default "Search…") — input placeholder
- `items` (computed via slotted `<sui-command-group>` + `<sui-command-item>` children, OR via a `commands` prop holding an array)

Slots / structure:
- `<sui-command-palette>` host — fixed overlay, dimmer backdrop on `::backdrop` (use `<dialog>` element for native modal semantics + Esc-to-close)
- Inside: filter input at top, scrollable group list below
- Each group: small-caps header label + N items
- Each item: optional leading icon + label + optional hotkey kbd + optional trailing meta (description / type)
- Empty state when filter matches nothing

Keyboard:
- `↑` / `↓` move selection
- `Enter` activates selected item
- `Esc` closes
- Typing filters the items by fuzzy match (case-insensitive substring or simple fuzzy)

Events:
- `sui-command-select { detail: { id, value } }` — fires when user picks an item
- `sui-command-close` — fires on Esc / backdrop click

### Three default groups (commands)

From shell-mock palette:

**Open**
- Studio (`⌘.`) — opens `/studio`
- Sessions — opens `/sessions`
- Events — opens `/events`
- Git — opens `/git`

**Run**
- (dynamic) every agent in `ship.agents.list()` as a row "Run <agent-id>"
- (dynamic) every skill in `ship.skills.list()` as "Invoke <skill-id>"
- (dynamic) every pipeline definition as "Run pipeline <name>"

**Settings**
- Theme: switch theme (sub-pick of 7 packs)
- Open Settings

Loading the dynamic lists happens lazily when palette opens; show a spinner if data isn't ready.

### Shell wiring

In `apps/shell/src/main.ts`:
- On boot, mount `<sui-command-palette>` once (hidden, attached to body)
- Listen for `sui-open-command-palette` event → toggle palette's `open` attr to true
- Listen for `sui-command-select` event → navigate accordingly (push route, dispatch run event, etc.)
- Already-emitted hotkey ⌘K from the shell-chrome refactor → palette opens

### Visual fidelity

Match `.artifacts/studio-design/shell-mock.html`'s palette aesthetic:
- Centered overlay, ~640px wide, max-height ~70vh
- Backdrop: `var(--backdrop)` with subtle blur
- Border: `var(--border-subtle)`, `var(--radius-lg)`, `var(--shadow-xl)`
- Group headers: small-caps, `var(--text-xs)`, `var(--tracking-wide)`, `var(--muted-foreground)`
- Selected row: primary-tinted background + primary-colored leading icon
- Kbd shortcuts: mono, small, muted

## Architectural context

- Charter: L2 primitive (lives in `@ship/primitives`). L3 (shell) consumes it.
- File cap: 300 lines per `.sui` / `.ts`. Palette likely splits into helper modules — filter state, group rendering, kbd handling.
- Use existing primitives where possible: `<sui-icon>`, `<sui-kbd>`, `<sui-list-tile>` (for rows).
- Accessibility: trapping focus when open, restoring on close. `<dialog>` handles the inert background. Live region for filter result count.

## Acceptance criteria

1. `<sui-command-palette>` primitive defined, registered, importable from `@ship/primitives`.
2. Pressing `⌘K` in the shell opens the palette centered over the canvas.
3. Three groups render with the items listed above. Dynamic groups (Run) pull from SDK.
4. Typing filters items live; arrow keys move selection; Enter activates.
5. Esc closes; backdrop click closes; route activates the selected item.
6. All 7 themes apply correctly to the palette + backdrop.
7. Forced-colors mode: palette renders legibly with system colors.
8. `bun run test` passes in primitives + shell.
9. Tests:
    - `packages/components/primitives/test/sui-command-palette.test.ts` — open/close, filter, kbd nav, event firing
    - `apps/shell/test/components/shell-chrome.test.ts` — extend to assert ⌘K → palette opens
10. Visual QA artifact at `.artifacts/web-qa/command-palette/<timestamp>/`. Cover open state in ship-light, ship-dark, console themes.

## Out of scope

- Studio's OWN ⌘K palette (the accelerator inside Studio). Separate job; this is the shell-level palette.
- Marketplace integration. Future enhancement once marketplace exists.
- Recent/frecency ordering. First pass is stable alphabetical within groups.

---

> Execute this job yourself. Do NOT delegate to a subagent.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
