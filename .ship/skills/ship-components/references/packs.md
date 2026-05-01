# Component packs

Domain-specific component bundles loaded on demand by a view's manifest `packs:` key. The shell eager-loads only two JS assets on boot (`datastar.js`, `ship-elements.js`); heavy island bundles (TipTap, CodeMirror, xterm) are ~5 MB combined and stay off the wire unless a view declares it needs them.

## Available packs

| Pack | Manifest key | Bundles | Elements |
|---|---|---|---|
| Editor | `editor` | `ship-code-editor.js`, `ship-markdown-editor.js` | `<ship-code-editor>`, `<ship-markdown-editor>` |
| Terminal | `terminal` | `ship-terminal.js` | `<ship-terminal>` |
| Git | `git` | `ship-git.js` | `<ship-git-view>`, `<ship-file-row>`, `<ship-branch-row>`, `<ship-diff-pane>` |

## Declaring in a view manifest

```yaml
name: my-view
type: html
packs:
  - editor
events:
  - config.changed
```

Daemon reads the list at render time, injects `<script type="module" src=...>` tags into the response. `mount.ts` rehydrates injected scripts after fragment navigation so they actually execute.

## Authoring a pack

Pack components live at `packages/ship-set/src/packs/<name>/`. Each pack ships its own bundle (so it stays out of the core `ship-elements.js`) and its own CEM fragment (so the manifest catalog stays complete).

When a view needs domain-specific UI that doesn't exist yet, the right move is to create or extend a pack component — not inline raw CSS/HTML in the template. File an `{#- SDK gap: pack/<name>: ... -#}` comment and escalate.

## Lint enforcement

`ship lint views` enforces both directions:

| Diagnostic | Severity | When |
|---|---|---|
| `missing-pack-declaration` | error | Template uses a heavy-island tag without the matching pack declared |
| `stale-pack-declaration` | warn | Declared pack has no corresponding tag in the view |

Unknown pack names are ignored by the daemon but not enforced by the linter (reserved for future additions like `viz`).

## When to create a pack

A pack is justified when:

- The component(s) wrap a non-trivial third-party engine (Monaco, xterm, tiptap), AND
- They're large enough that always-loading them hurts boot time, AND
- They have a coherent domain (git, editor, terminal) — packs aren't a junk drawer.

If a single component would fit in `core/` or `domain/` without a heavy bundle, it doesn't need a pack — just ship it in `ship-elements.js`.

## Roadmap

Current packs are listed above. New packs land when a domain accumulates 3+ heavy components that consistently load together. Reserved names (no pack yet): `viz` (charts using shared canvas/SVG plumbing).

## Detail reference

For richer discussion of pack manifest format, distribution, when-to-author tradeoffs, testing patterns, and roadmap: see `docs/ship-components/references/docs/packs/{authoring, manifest-and-distribution, when-to-author, testing, roadmap}.md` — those pages render on the docs site and cover the full surface.
