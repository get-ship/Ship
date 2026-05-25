---
title: "The LSP — diagnostics on tap"
description: "One Rust grammar, three delivery channels — in-process MCP, the sui-lsp binary, and the in-browser WASM compiler."
group: "Tutorial"
order: 10
slug: "10-lsp-cli"
prev: "09-agent-loop"
next: "11-sui-eval"
chapter: "agents"
---

SUI's language intelligence is the Rust compiler — `sui_compiler::analyze`
(diagnostics + scope symbols + dependencies) and `check_all` (every diagnostic,
not just the first). It's the same grammar the build uses, exposed to agents
through three channels. For agents, the channel is the in-process MCP tools
(`mcp__ship__sui_*`) — no subprocess, no Node, no bun.

## diagnostics

```ts
mcp__ship__sui_diagnostics({ file: "views/tasks/page.sui" })
```

Returns every issue at once (the same set `cargo build` would emit):

```json
{
  "diagnostics": [
    { "line": 12, "col": 4, "end_line": 12, "end_col": 18, "severity": "error",
      "message": "`new` is not allowed in renderable expressions", "rule": "expr `new Date()`" },
    { "line": 18, "col": 9, "end_line": 18, "end_col": 9, "severity": "warning",
      "message": "<data> directive without `name` attr", "rule": "sui-data-name" }
  ]
}
```

`severity` is `error` (compile-blocking) or `warning` (compiles but suspicious).
`rule` carries the lint rule id or the offending expression — useful for
filtering or mapping to your editor's diagnostic provider. Positions are
1-based.

## hover

```ts
mcp__ship__sui_hover({ file: "views/tasks/page.sui", line: 12, col: 8 })
```

Resolves the symbol under the cursor — its binding kind and where it's declared
in the controller. Use it to confirm whether `items` is a signal you call or a
plain value, and to jump to its definition:

```json
{ "hover": { "name": "items", "kind": "signal", "source": "scope", "line": 7, "col": 11 } }
```

`kind` is the binding shape (`signal` / `ref` / `derived` / `action` / `data` /
`prop` / `value`); `source` is the section it came from (`scope` / `derived` /
`on` / `data` / `prop`); `line`/`col` point into the paired `.controller.ts`
(`0,0` when declared in the `.sui` itself).

## dependencies

```ts
mcp__ship__sui_dependencies({ file: "views/tasks/page.sui" })
```

What does this view reach into — which custom elements it instantiates, which
events it handles, and its paired controller. Read it before an edit to scope
the blast radius:

```json
{
  "tags": ["sui-button", "sui-card", "sui-status-dot"],
  "events": ["click", "keydown"],
  "controller": "views/tasks/controller.ts"
}
```

## Try it

The example to the right is a healthy view — no compile errors.

1. **Diagnostics**: `sui_diagnostics` returns `{ "diagnostics": [] }`. Healthy.
2. **Break it**: introduce a bad modifier (`on:click|nope`) or a non-renderable
   expression (`{new Date()}`) in the template and watch the inline squiggle
   appear — the playground editor runs the **WASM** build of the same compiler,
   so it reports exactly what `sui_diagnostics` and `cargo build` would.

## Why this matters

There is **one** source of truth — the Rust `sui_compiler` grammar — flowing
through three channels:

- **WASM compiler** in this playground → inline squiggles, and the
  `@ship/sui-lsp-browser` analyser behind the editor.
- **In-process MCP** (`sui_diagnostics` / `sui_hover` / `sui_dependencies`) →
  for agents; `apps/mcp` calls `sui_compiler` directly, no subprocess.
- **The `sui-lsp` binary** (`crates/sui/lsp/`, on PATH via `just install-dev`) →
  standard LSP over stdio for editors (VS Code, Claude Code via `.lsp.json`,
  opencode, any LSP client).

A human typing in the docs site, an agent inside Claude Code, and an editor on
your laptop all see the *same* diagnostics — because they're all the same
compiler. No second grammar to drift out of sync.
