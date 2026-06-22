# Ship UI — component packs

Ship's UI is **SUI** — a `.sui` template + optional `.controller.ts`, compiled AOT to web
components. Components ship as **packs** under `packages/components/{sui,ship}/`. This README is
the canonical map of the pack taxonomy; the full rationale lives in the ADR
[`ship-component-architecture.md`](../../docs/ship-adrs/references/docs/ship-component-architecture.md).

## The two tiers

- **`sui-*`** — the framework's **reusable, atomic primitives**. A pack is `sui` **only if it is
  all of**: SDK-pure · atomic / single-concern · generic (useful in a stranger's app) · our own
  engine (not a wrapper) · published. The standard library.
- **`ship-*`** — **everything built from `sui`** (the default). A pack is `ship` if it trips
  **any** of: touches a Ship domain / the SDK · wraps an external engine (CodeMirror, Tiptap,
  pdf.js, PixiJS) · is a composed feature / vertical · is internal / unpublished · is shared
  across ≥2 `ship` packs.

> **The law:** `sui` has to earn it; `ship` is the default. **Runtime/SDK dependency ⇒ `ship`**
> is the one hard line. Pure-sui reused by ≥2 `ship` packs ⇒ promote to a `sui` element pack.

Tag prefix, directory (`sui/` vs `ship/`), and package name must agree. Everything is
**token-themed** (`var(--token)`, never hardcoded — compiler-enforced).

## What lives here vs. elsewhere

- **Packs** (this directory) are the **library you compose with** — and the primary frontend
  deliverable for a Ship feature (a feature = L0/L1 backend + an RPC surface + a pack). Our own
  views are *suggested use*; the pack is the deliverable. **A domain is just a pack.**
- **Apps** (`apps/hub`, `apps/spaces`) are first-party programs that **compose packs**. The hub
  is all-static first-party code; its routes (home, `studio/*`, settings, account) are **app
  code — not packs and not views.** Hub-only widgets are co-located in `apps/hub`, not exported.
- **Views** — user-defined applets that slot dynamically into a space's route tree — are a
  separate **feature** (their own docs), not a pack tier. The hub does not use the views feature.
- **Windows** are a Tauri host concern. `just shell` opens the native hub window.

## Inventory (status)

### `sui-*` — the standard library
| Pack | What |
|---|---|
| `sui/primitives` (~60 tags) | forms, overlays, layout, feedback, data-display, nav |
| `sui/primitives` (table/pagination, tree, sheet, calendar) | data-grid, hierarchy, spreadsheet (formula engine), calendar |
| `sui/charts` (~7) | area/bar/donut/gauge/heatmap/line/sparkline |
| `sui/primitives` (canvas/board/draw) | pan-zoom substrate · dagre node-edge · SVG whiteboard |
| `sui/chat-elements` (~6) | message-list (virtualized), composer, citation, tool-call |
| `sui/media` | image / audio-video / csv renderers + mime detection |
| `sui/viewer` | **`<sui-component-viewer>`** — live render, prop knobs, state matrix, source panel |

### `ship-*` — features / engine-wrapped / domain
| Pack | What |
|---|---|
| `ship/code` | `ship-code-editor` (CodeMirror), `ship-terminal` (xterm), `ship-git-diff` |
| `ship/markdown` | `ship-markdown-editor` (Tiptap) |
| `ship/pdf` | `ship-pdf` (pdf.js) — a viewer-router renderer |
| `ship/viewer` | `ship-viewer` — the mime **router** (dispatches to `sui/media` + `ship` renderers) |
| `ship/arcade` | game tags over a PixiJS engine |
| `ship/chrome` | app chrome: `ship-nav-top`, `ship-app-chrome`, project switcher, account menu |
| `ship/agents` · `ship/pipelines` · `ship/authority` · `ship/collections` · `ship/skills` · `ship/spaces` · `ship/artifacts` | **domain packs** — runtime-bound (RPC · events · verbs) |

## Viewing components
`<sui-component-viewer>` (`sui/viewer`) live-renders any component by tag — props panel, state
matrix, theme switch. The native hub window: `just shell` (Tauri).
