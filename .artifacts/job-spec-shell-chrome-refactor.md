# Job Spec: shell chrome refactor — thin top + thin bottom + empty canvas

**Branch:** `job/shell-chrome-refactor`
**Agent:** web-lane
**Mode:** autonomous
**Design source:** `.artifacts/studio-design/shell-mock.html`

## Goal

Replace the current shell's left-rail layout with a minimal three-zone chrome: a 36px top strip, a 28px bottom strip, and an empty full-canvas content area between them. The chrome is intentionally sparse — most surfaces (Studio, Sessions, Events, etc.) live in the canvas as views accessed via ⌘K command palette, not as persistent rail items. First-launch state is an explicit empty canvas with hint pills (⌘K, ⌘., ⌃`, ⌘?). The user's pinned views and widgets fill the canvas as they create them; Ship's chrome stays out of the way.

This implements the architectural decision documented in `.artifacts/studio-design/PATH.md` (and the subsequent design iteration in `shell-mock.html`) — Studio is not a peer of the chrome, it's a view that paints into the canvas via its own chrome.

## File scope

```
apps/shell/src/main.ts                                  # entry — replace rail mount with new shell layout
apps/shell/src/chrome/shell.html                        # if exists; otherwise create
apps/shell/src/chrome/layout.css                        # the new grid
apps/shell/src/components/sui-shell-top/                # NEW — 36px top strip primitive
apps/shell/src/components/sui-shell-bottom/             # NEW — 28px bottom strip primitive
apps/shell/src/components/sui-blank-canvas/             # NEW — empty-state surface with hint pills
apps/shell/test/setup.ts                                # update test imports if rail mount moves
apps/shell/sui.config.ts                                # may need to register new components
```

DO NOT touch:
- `apps/shell/src/components/sui-rail/` — leave intact for now; the rail components stay registered for any code path that still uses them (Studio's internal rail uses the same primitive). Remove from the shell mount only.
- `views/**` — view content is unchanged by this job.
- `crates/**` — no Rust changes.
- `packages/**` — no primitive changes.

## What to change

### `apps/shell/src/main.ts` (or equivalent entry)

Replace the current rail-based layout with the three-row grid:

```
┌────────────────────────────────────────┐  36px
│ <sui-shell-top>                        │
├────────────────────────────────────────┤
│                                        │  flex (canvas)
│  <sui-blank-canvas> (default)          │
│  OR the active view's content          │
│                                        │
├────────────────────────────────────────┤
│ <sui-shell-bottom>                     │  28px
└────────────────────────────────────────┘
```

### `<sui-shell-top>` (~36px)

Reference: `.artifacts/studio-design/shell-mock.html` lines 31–131.

Slots / content:
- **Brand** (left): 22×22 hue tile with `S` glyph, "Ship" wordmark in heading font, optional workspace name in mono (e.g. `ship-monorepo`) separated by a hairline.
- **⌘K command pill** (centered, flex: 1, min-width 320px, max-width 480px): a pulse dot on the left, "Search agents, skills, runs…" placeholder text, `⌘K` kbd on the right. Clicking opens the command palette (stub: emit `sui-open-command-palette` CustomEvent for now — palette wiring is a separate job).
- **Right cluster**: 26×26 icon buttons for theme, notifications, plus a 22×22 gradient avatar.

Use the existing token system. Background uses `surface-1` at 88% opacity with `backdrop-filter: blur(10px)` to read as a glassy strip. Border-bottom: 1px `border-subtle`.

### `<sui-shell-bottom>` (~28px)

Reference: `shell-mock.html` lines 235–330 approximately (the `.bot` styles).

Slots / content (left to right):
- Status: connected dot (primary, pulse), branch name in mono ("main"), workspace name in mono ("ship-monorepo")
- Spacer (flex: 1)
- Tool icons (24×24): terminal, devtools panel, annotate
- Theme indicator (e.g. "ship-light")
- Version label ("0.2.0")
- Mode chip (tone: warning for "development", success for "release", muted for "production")

### `<sui-blank-canvas>` (default content)

Reference: `shell-mock.html` lines 145–225 (the `.empty` styles).

Default rendered into the canvas when no view is open. Structure:
- Dotted-grid background: `radial-gradient(circle at 1px 1px, var(--border-subtle) 1px, transparent 0) 0 0 / 32px 32px, var(--background)`
- Centered content column:
  - 64×64 hue tile with `S` glyph (matches the brand mark but larger)
  - "Blank canvas" h1 (32px, heading font)
  - 96×4px gradient underline (linear-gradient(90deg, var(--primary), transparent))
  - Sub-copy: "Open <code>⌘K</code> to begin, or press <code>⌘.</code> to configure Ship."
  - Four hint pills (5×12px padding, border-subtle, mono kbd inside):
    - `⌘K` palette
    - `⌘.` Studio
    - `⌃\`` terminal
    - `⌘?` help

### Hotkey handling

Implement in `apps/shell/src/main.ts` (or a new `apps/shell/src/keymap.ts`):

| Combo | Effect |
|---|---|
| `⌘K` / `Ctrl+K` | Emit `sui-open-command-palette` on document; palette wiring is a separate job |
| `⌘.` / `Ctrl+.` | Navigate to `/studio` (Studio job builds the surface; for now, route push is enough) |
| `⌃\`` | Toggle terminal panel (emit `sui-toggle-terminal`; stub) |
| `⌘?` / `Ctrl+?` | Emit `sui-open-help` (stub) |
| `Esc` from any view | Return to blank canvas (clear `<active-view>` from the canvas slot) |

Use `event.metaKey || event.ctrlKey` for cross-platform.

### Cleanup

- Remove the old left-rail mount from `apps/shell/src/main.ts` (the existing `<sui-rail>` instance in the shell — the component definition stays).
- Audit `apps/shell/src/components/sui-rail/` for shell-only logic that's no longer reachable; flag for removal but DO NOT delete this job. The rail primitive itself stays for Studio to use.
- Ensure `apps/shell/test/setup.ts` still wires through the new components.

## Architectural context

- Charter rules (see `docs/ship-architecture/references/docs/charter.md`):
  - L0/L1/L2/L3 separation — the shell is L3 app code, no privileged access to L0 (`crate::services::*` etc.)
  - L2 has no privileged access; the shell consumes the same SDK as any L3 app
  - Hard file cap: 300 lines per .ts/.sui, 400 for unusual cases
- Per CLAUDE.md: prefer existing primitives over inventing new ones. The three new primitives here are necessary because none of them exist today.
- Per `.artifacts/studio-design/PATH.md`: Studio is design language, NOT a top-level rail concept. The chrome explicitly removes the rail to enforce this.

## Acceptance criteria

1. Shell renders with the new three-zone layout (top 36px, bottom 28px, canvas in between, no rail).
2. `<sui-shell-top>`, `<sui-shell-bottom>`, `<sui-blank-canvas>` all defined as custom elements, registered, and used by `main.ts`.
3. First-load state in a fresh browser instance shows the blank-canvas with all four hint pills, dotted grid, centered content.
4. ⌘K hotkey emits the expected event (verified by attaching a test listener — see test below).
5. ⌘. navigates to `/studio` (404 is fine for now — Studio job builds the surface).
6. Esc from any opened view (route-pushed) returns to the blank canvas state.
7. Cross-platform: hotkeys work on macOS (`metaKey`) and Linux/Windows (`ctrlKey`).
8. Forced-colors mode: chrome still renders with system colors; hint pills still legible.
9. No console errors on first load. No 404s in the network panel beyond the expected `/studio` route.
10. Tests:
    - `apps/shell/test/shell-chrome.test.ts` (new) — at minimum: layout structure, hotkey dispatch, blank-canvas content presence.
    - `bun run test` in `apps/shell` passes all existing tests too.
11. Visual QA: run web-qa skill on `apps/shell` post-change. Cover at minimum `ship-light`, `ship-dark`, `console` themes at `1280x720` viewport. Artifact at `.artifacts/web-qa/shell-chrome-refactor/<timestamp>/`.

## Pin

Match the **visual fidelity** of `shell-mock.html` — when both are open side-by-side, the implementation should be indistinguishable from the mock at `ship-light` theme. Spacing, typography, colors, hairlines, the brand mark's inner-highlight + shadow, the pulse dot on the command pill — all match.

The mock uses inline SVG (`<symbol>` defs + `<use href>`) for icons because `<sui-icon>` doesn't render reliably in static mock contexts. In the production implementation, use real `<sui-icon name="terminal">` etc. — the icon registry works end-to-end in the live shell.

## Out of scope

- Command palette interior (`⌘K` opens; what's inside is a separate job).
- Studio surface itself (separate job — running in parallel).
- Per-project themes (separate job — pending project metadata schema).
- Terminal panel implementation (just emit the toggle event for now).

---

> Execute this job yourself. Do NOT delegate to a subagent — subagents are for read-only research, not implementation.
>
> Commit incrementally: one commit per primitive (sui-shell-top, sui-shell-bottom, sui-blank-canvas), one for the main.ts integration, one for tests + QA.
>
> When all acceptance criteria are met:
> 1. Call `mcp__ship__complete_workspace` with a one-paragraph summary.
> 2. Call `mcp__ship__mesh_broadcast` with `{ topic: 'job.completed', workspace_id: '<id>', summary: '<same>' }`.
