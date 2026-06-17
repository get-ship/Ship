# design/components/ship/

Design work for the **`ship-*`** tier — features and engine-wrapped surfaces built
on top of `sui` primitives. Mirrors `packages/components/ship` in the repo.

**Scaffolding started.** `@ship/code` is live (`code/`); the rest of the tier is
designed *after* the `sui` primitives it composes are settled (see `../sui/`).
Tracking + format notes in [`INDEX.md`](INDEX.md).

## Candidates (from the repo's `ship-*` packs)

| Pack | Tag(s) | Engine |
|---|---|---|
| `@ship/code` | `ship-code-editor` · `ship-terminal` · `ship-git-diff` | CodeMirror · xterm |
| `@ship/markdown` | `ship-markdown-editor` | Tiptap |
| `@ship/pdf` | `ship-pdf` | pdf.js |
| `@ship/viewer` | `ship-viewer` (mime router → sui-media + ship renderers) | — |
| `@ship/arcade` | 8 game tags | PixiJS |

## To scaffold a pack here

Copy the three-surface pattern from `../sui/forms/`:
`components.html` (overview) · `<component>.html` (iteration canvas) · `handoff.md`.
Catalog tooling: `/tools/catalog/`. Tokens: `/sui/ship.css`.
