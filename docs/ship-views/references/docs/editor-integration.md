---
title: "Editor Integration (LSP)"
description: "Inline view-lint diagnostics in any LSP-capable editor — VS Code, Zed, Helix, Neovim, JetBrains. Driven by `ship lsp`."
sidebar:
  label: "Editor Integration"
  order: 5
---
The Ship CLI ships with a built-in language server (`ship lsp`) that
delivers the same diagnostics you get from `ship lint views` and the
daemon's write-blocking lint pass — but inline, on every keystroke.

## What surfaces

Every rule in [Lint Rule Catalog](./lint-rules.md) is rendered as an
editor diagnostic with:

| Finding field        | LSP field                       |
| -------------------- | ------------------------------- |
| `rule`               | `Diagnostic.code`               |
| `severity::Error`    | `DiagnosticSeverity.ERROR`      |
| `severity::Warn`     | `DiagnosticSeverity.WARNING`    |
| `message` + `suggest`| `Diagnostic.message`            |
| `(line, col)`        | `Diagnostic.range` (1-char span)|
| —                    | `Diagnostic.source = "ship-lint"`|

Files matched: `**/*.html.j2` under any `.ship/views/` directory in
the workspace. Pack-declaration rules
(`pkg-missing-pack-declaration`, `pkg-stale-pack-declaration`) walk
up to the nearest `manifest.{yaml,jsonc,json}` automatically.

## VS Code

Install the extension at `apps/ship-set-vscode/` (see its `README.md` for
local install steps; published Marketplace listing is forthcoming).
The extension activates on any workspace containing
`.ship/views/**/*.html.j2`.

To disable temporarily without uninstalling:

```jsonc
// .vscode/settings.json
{ "ship.lsp.enable": false }
```

To override the binary path (e.g. when `ship` is not on `PATH`):

```jsonc
{ "ship.lsp.path": "/usr/local/bin/ship" }
```

## Zed

```toml
# ~/.config/zed/settings.json
"lsp": {
  "ship-lsp": { "binary": { "path": "ship", "arguments": ["lsp"] } }
}
"languages": {
  "HTML": { "language_servers": ["ship-lsp"] }
}
```

## Helix

```toml
# ~/.config/helix/languages.toml
[language-server.ship-lsp]
command = "ship"
args = ["lsp"]

[[language]]
name = "html"
file-types = ["html.j2"]
language-servers = ["ship-lsp"]
```

## Neovim (nvim-lspconfig)

```lua
-- init.lua
local lspconfig = require("lspconfig.configs")
lspconfig.ship_lsp = {
  default_config = {
    cmd = { "ship", "lsp" },
    filetypes = { "jinja.html", "html" },
    root_dir = require("lspconfig.util").root_pattern(".ship"),
  },
}
require("lspconfig").ship_lsp.setup({})
```

## JetBrains / other editors

Any editor that speaks LSP can spawn `ship lsp` over stdio. Point its
generic LSP client at the binary, scope it to `*.html.j2` files
under `.ship/views/`, and you're done.

## Disabling

- **VS Code** — `"ship.lsp.enable": false` (see above).
- **Zed / Helix / Neovim** — comment out the language-server block
  in your config and reload.
- **Globally** — uninstall the editor extension; `ship lsp` only runs
  when an editor spawns it.

## Performance

`ship lsp` re-runs the lint engine on every `didChange`. Typical view
files lint in well under a millisecond, so no debounce is applied. If
you author an unusually large view and notice latency, file an issue
with the file size and observed latency — debouncing lives in the
LSP layer, not the lint engine.

## Future work

Today's server is diagnostics-only. Hover (macro signatures, op
signatures), completion (`ship.<TAB>`), go-to-definition (macro →
`_ship.html.j2`), and code actions (wrap-in-signal, add lint-escape
comment) are explicit follow-ups.
