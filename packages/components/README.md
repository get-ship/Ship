# Ship UI — component packs (design review)

> Orienting doc for a **design review** of Ship's component library. Read this,
> then the ask at the bottom. Capture scope for this pass: **the component packs**
> (not the product views or marketing site).

## What this is

Ship has its own UI framework, **SUI** — a `.sui` template + optional
`.controller.ts`, compiled AOT to web components. Think "our own React + a design
system." Components ship as **packs** under `packages/components/{sui,ship}/`.

Two tiers (full rationale: `docs/ship-adrs/.../ship-component-architecture.md`):

- **`sui-*`** — the framework's **reusable, atomic primitives**. SDK-pure, generic,
  one concern each, our own engines. The "standard library" any app would want.
- **`ship-*`** — **features/verticals built from `sui`**: domain packs, or surfaces
  wrapping a heavyweight engine (CodeMirror, Tiptap, pdf.js, PixiJS), or composed
  product surfaces. Over time the larger surface.

> **The law:** `sui` has to earn it (SDK-pure · atomic · generic · own-engine ·
> published); everything else is `ship`.

Everything is **token-themed** — colors/spacing/radii/typography come from design
tokens (`var(--token)`), never hardcoded. Themes: `ship-light`, `ship-dark`,
`console`, plus `paper`/`halo`/`contrast`/`glass`. A component that hardcodes a hex
is a bug (compiler-enforced).

## Inventory (honest status)

### `sui-*` — primitives (the standard library)
| Pack / tag | What | Status |
|---|---|---|
| `@ship/sui-primitives` (~60 tags) | forms, overlays, layout, feedback, data-display, nav | **solid**; 1 hardening pass applied (HIGH token findings); ~50 medium/low findings open |
| `sui-table` + `sui-pagination` | data-grid: sort/filter/group/paginate/select/resize, virtualized | **done** — the reference engine |
| `sui-tree` / `sui-tree-grid` | hierarchy + tree×grid | **done** |
| `sui-sheet` | spreadsheet: A1, **formula engine**, range/edit/fill/copy-paste/keyboard | engine + interactivity **just landed** |
| `sui-calendar` | month/week/day, events, **range-select + drag-create/move**, keyboard grid | **done** (range/drag just added) |
| `sui-charts` (~7) | area/bar/donut/gauge/heatmap/line/sparkline | exists; light harden |
| `sui-canvas` / `sui-board` / `sui-draw` | pan/zoom substrate · node-edge diagrams (**dagre auto-layout**) · SVG whiteboard | **done** (Wave-3) |
| `sui-chat-elements` (~6) | message-list (**virtualized**), composer, citation, tool-call, thinking-dots | done (just virtualized) |
| `sui-media` | image / audio-video / csv renderers + mime detection | image/av/csv done; **pdf/map are `ship` renderers** |
| `sui-viewer` tooling | the component-viewer itself: live render, prop-knobs, prop-matrix, source-panel | **done** (this is how you'll view the rest) |

### `ship-*` — features / engine-wrapped surfaces
| Pack / tag | Engine | Status |
|---|---|---|
| `@ship/code` | `ship-code-editor` (CodeMirror), `ship-terminal` (xterm), `ship-git-diff` | done |
| `@ship/markdown` | `ship-markdown-editor` (Tiptap WYSIWYG) | done |
| `@ship/pdf` | `ship-pdf` (pdf.js) | done; wired into the viewer router |
| `@ship/viewer` | `ship-viewer` — the mime **router** (dispatches to sui-media + ship renderers) | done |
| `@ship/arcade` | 8 game tags over a PixiJS engine (powers the marketing-site games) | done + tested |
| `@ship/agents` · `@ship/pipelines` · `@ship/authority` | Ship-domain packs | exist |

**Known gaps (not yet built):** map viewer (`ship-map`); secondary viewers
(html/yaml/notebook/hex/archive); `ship-code-diff` (merge view); the full
primitives-hardening pass (only HIGH token findings applied so far).

## How to view them

The intended surface is **`<sui-component-viewer>`** (pack #16) — it live-renders
any component by tag, with a prop-knobs panel, a variant/state matrix, and a theme
switcher. Note: a desktop **Tauri shell** hosts these in production, but it can't
run in this dev environment (WSL's GL stack is broken) — so review is via the
component-viewer surface in a browser / headless screenshots, not the native shell.

## The ask (design review)

Looking for design judgment on the **`sui-*` primitive library** first (it's the
foundation everything composes from):

1. **Consistency** — do the packs feel like one system? Spacing rhythm, density,
   radii, motion, focus/hover/active states, empty/loading/error states.
2. **Token & theme fidelity** — do they hold up across `ship-light`/`ship-dark`/
   `console`? Any contrast failures, near-invisible text, color-mix unwinding wrong?
   (Several packs were just token-hardened — sanity-check them.)
3. **A11y** — keyboard operability + ARIA on the interactive ones (table, tree,
   calendar grid, sheet, board, chat). Flag focus traps / unreachable controls.
4. **Gaps & priorities** — for a credible launch component library, what's *missing*
   or *underbaked*? Which of the "known gaps" above are launch-critical vs later?
5. **The composed `ship-*` surfaces** — do the editor/viewer/canvas surfaces read as
   polished product, or as primitives bolted together?

Concrete, prioritized, brutally honest > exhaustive. Point at specific tags.
