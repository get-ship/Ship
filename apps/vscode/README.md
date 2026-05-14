# SUI (.sui) — VS Code extension

Syntax highlighting and language server support for SUI
template files (`.sui`, `.set`).

## What's in the box

- **Language registration**: files ending in `.set` or `.sui`
  are recognized as the `set` language.
- **Syntax highlighting**: a TextMate grammar covers
  - **Directive elements** — `<data>`, `<component>`, `<prop>`,
    `<while-loading>`, `<while-error>` are coloured distinctly from
    regular HTML so the runtime contract is visible at a glance.
  - **Binding attribute prefixes** — `bind:`, `on:`, `class:`,
    `style:`, `attr:`, `let:` are coloured separately from plain
    attributes; `on:click|preventDefault|stop` highlights the event
    name and modifiers.
  - **`{expr}` interpolation** — body is treated as embedded
    TypeScript so JS operators, strings, and template literals get
    native colouring.
  - **Control-flow blocks** — `{#if}` / `{#for}` / `{#match}` and
    their continuations (`{:else if}`, `{:else}`, `{:case}`,
    `{:default}`, `{/if}`, `{/for}`, `{/match}`).
- **Language client**: launches the `sui-lsp` binary and wires
  up push diagnostics, hover, and completion (with `<`, `{`, `.`,
  `:`, `=` as trigger characters).

The TextMate grammar is the floor; the tree-sitter grammar in
`packages/sui-grammar/` provides richer semantic highlighting
once the LSP picks it up.

## Requirements

You need a `sui-lsp` binary on your system. The extension does
not ship one — install it separately from the Ship monorepo.

### Building `sui-lsp`

From the repository root:

```bash
cargo build --release -p sui-lsp
```

The binary lands at `target/release/sui-lsp` (or
`target/debug/sui-lsp` for a debug build).

## Configuration

| Setting                 | Default        | Description                                                                                       |
| ----------------------- | -------------- | ------------------------------------------------------------------------------------------------- |
| `sui.serverPath`   | `sui-lsp` | Path to the `sui-lsp` executable. Use an absolute path, or a name resolved against `PATH`.   |
| `sui.trace.server` | `off`          | Trace LSP traffic between VS Code and `sui-lsp`. Set to `messages` or `verbose` to debug.    |

Add the binary to your PATH, or point `sui.serverPath` at it:

```jsonc
// .vscode/settings.json
{
  "sui.serverPath": "/abs/path/to/target/release/sui-lsp"
}
```

## Install

### From a packaged `.vsix`

```bash
cd apps/vscode
bun install
bun run compile
npx vsce package
code --install-extension ship-ui-0.1.0.vsix
```

### Development load

1. Open `apps/vscode/` in VS Code.
2. `bun install` then `bun run compile` (or run the `watch`
   script during development).
3. Press `F5` ("Run Extension") to launch an Extension Development
   Host with the extension loaded.
4. Open any `.sui` or `.set` file — highlighting and
   diagnostics should engage.

## Status / scope

- **v1 (now)**: TextMate grammar, language client wiring
  (diagnostics, hover, completion), language configuration.
- **v2 (planned)**: tree-sitter grammar via WebAssembly for richer
  semantic highlighting using the queries in
  `packages/sui-grammar/queries/highlights.scm`.
- The LSP server itself advertises `codeAction` and `definition`
  capabilities, but the handlers currently return empty results;
  upstream those land in `crates/sui-lsp`.
