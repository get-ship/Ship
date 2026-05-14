# Ship Lint — VS Code Extension

Surfaces every Ship view-lint rule as inline diagnostics in VS Code,
backed by `ship lsp` (the Ship CLI's built-in language server).

## What you get

- Squiggles on every `*.html.j2` file under `.ship/views/`.
- Same rules, severities, and messages as `ship lint view` and the
  daemon's write-blocking lint pass — one source of truth.
- No setup beyond installing `ship` and this extension.

## Install

1. Install the Ship CLI (`ship --version` should work).
2. Install this extension. From source:
   ```bash
   cd apps/ship-views-j2-vscode
   bun install
   bun run compile
   code --install-extension .
   ```
   (Or package with `vsce package` and install the `.vsix`.)
3. Open any project with `.ship/views/<name>/index.html.j2`. The
   extension activates automatically.

## Configuration

| Setting          | Default | Description                                         |
| ---------------- | ------- | --------------------------------------------------- |
| `ship.lsp.path`  | `ship`  | Path to the `ship` binary. Override if not on PATH. |
| `ship.lsp.enable`| `true`  | Disable to silence diagnostics without uninstall.   |

## Disable temporarily

Set `"ship.lsp.enable": false` in your workspace settings, or disable
the extension via the VS Code Extensions panel for this workspace.

## How it works

The extension spawns `ship lsp` over stdio and speaks LSP. Diagnostics
are pushed on `didOpen`, `didChange`, and `didSave`, and cleared on
`didClose`. See `apps/docs/src/content/docs/reference/views/editor-integration.md`
for the rule list and editor-agnostic LSP setup.
