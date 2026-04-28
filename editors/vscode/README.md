# Sail (.set) — VS Code extension

Syntax highlighting and language-server integration for Sail component
files (`.set`).

## What's in the box

- **Language registration**: files ending in `.set` are recognized as
  the `set` language.
- **Syntax highlighting**: a TextMate grammar covers the three
  top-level blocks (`<script>`, `<template>`, `<style>`), framework
  declaration keywords (`prop`, `state`, `derived`, `data`, `effect`,
  `event`, `mount`, `unmount`, `function`, `component`, `on form_*`),
  template control flow (`{#if}`, `{#for}`, `{#await}`, `{:else}`,
  `{:then}`, `{:catch}`, `{:empty}`), interpolation `{...}`, raw HTML
  `{@html ...}`, and binding directives (`bind:`, `on:`, `use:`,
  `class:`, `style:`).
- **Language client**: launches the `ship-set-lsp` binary and wires up
  push diagnostics, hover, and completion (with `<`, `{`, `.`, `:`,
  `=` as trigger characters).

## Requirements

You need a `ship-set-lsp` binary on your system. The extension does not
ship one — install it separately from the Ship monorepo.

### Building `ship-set-lsp`

From the repository root:

```bash
cargo build --release -p ship-set-lsp
```

The binary lands at `target/release/ship-set-lsp` (or `target/debug/ship-set-lsp`
for a debug build).

## Configuration

| Setting              | Default     | Description                                                                                       |
| -------------------- | ----------- | ------------------------------------------------------------------------------------------------- |
| `ship-set.serverPath`    | `ship-set-lsp`  | Path to the `ship-set-lsp` executable. Use an absolute path, or a name resolved against `PATH`.       |
| `ship-set.trace.server`  | `off`       | Trace LSP traffic between VS Code and `ship-set-lsp`. Set to `messages` or `verbose` while debugging. |

Add the binary to your PATH, or point `ship-set.serverPath` at it:

```jsonc
// .vscode/settings.json
{
  "ship-set.serverPath": "/abs/path/to/target/release/ship-set-lsp"
}
```

## Install

### From a packaged `.vsix`

```bash
cd editors/vscode
pnpm install
pnpm run compile
npx vsce package
code --install-extension ship-set-0.1.0.vsix
```

### Development load

1. Open `editors/vscode/` in VS Code.
2. `pnpm install` then `pnpm run compile` (or run the `watch`
   script during development).
3. Press `F5` ("Run Extension") to launch an Extension Development
   Host with the extension loaded.
4. Open any `.set` file — highlighting and diagnostics should engage.

## Status / scope

- **v1 (now)**: TextMate grammar, language client wiring (diagnostics,
  hover, completion), language configuration.
- **v2 (planned)**: tree-sitter grammar via WebAssembly for richer
  semantic highlighting using the queries in
  `grammars/tree-sitter-set/queries/highlights.scm`.
- The LSP server itself advertises `codeAction` and `definition`
  capabilities, but the handlers currently return empty results;
  upstream those land in `crates/ship-set-lsp`.
