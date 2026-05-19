---
title: "The LSP — diagnostics on tap"
description: "Three subcommands that cover 90% of agent feedback."
group: "Tutorial"
order: 10
slug: "10-lsp-cli"
prev: "09-agent-loop"
next: "11-sui-eval"
chapter: "agents"
---

`@ship/sui-lsp` is the same analyser the daemon and the
`mcp__ship__sui_*` tools wrap, but exposed as a CLI so it works from
anywhere a shell does. Three subcommands cover the entire authoring
feedback surface.

## diagnostics

```sh
bun packages/deck/lsp/src/cli.ts diagnostics views/tasks/page.sui
```

Returns one JSON line per issue:

```json
[
  { "line": 12, "col": 4, "severity": "error", "message": "unknown modifier \"enter\"", "source": "sui-compiler" },
  { "line": 18, "col": 9, "severity": "warning", "message": "<data> directive without `name` attr", "source": "sui-lint" }
]
```

Severities: `error` (compile-blocking), `warning` (compiles but
suspicious), `info` (style nudges). The `source` distinguishes
compiler errors from linter rules — useful when filtering or
mapping to your editor's diagnostic provider.

## hover

```sh
bun packages/deck/lsp/src/cli.ts hover views/tasks/page.sui --line 12 --col 8
```

Returns the type, signature, or directive doc under the cursor. Use
it when the agent is unsure whether a `<sui-button>` prop accepts a
union or a boolean, or what the difference between `bind:value` and
`prop:value` is at this exact callsite.

```json
{
  "kind": "prop",
  "name": "tone",
  "type": "default | primary | info | success | warning | destructive | accent | muted",
  "default": "default",
  "reflects": true,
  "doc": "Tone palette. Drives the active accent + focus ring colour."
}
```

## dependencies

```sh
bun packages/deck/lsp/src/cli.ts dependencies views/tasks/page.sui
```

What does this file actually reach into? Tags, signals, routes, data
sources. The agent reads this before edits to scope the blast radius.

```json
{
  "tags": ["sui-card", "sui-button", "sui-status-dot"],
  "signals": ["count", "rate", "alarming"],
  "routes": ["/api/tasks"],
  "imports": ["@ship/sui-runtime"]
}
```

## Try it

The example to the right is a healthy view — no compile errors. Try:

1. **Read the diagnostics**: it returns `[]`. Healthy.
2. **Hover on `<sui-button tone=…>`**: the LSP would return the tone
   union — open the controller pane and find a `tone` reference to
   imagine where you'd run hover.
3. **Break it**: edit the template to introduce a bad modifier (e.g.
   `on:click|nope`) and watch the inline squiggle appear — the editor
   uses the same diagnostic pipeline via the WASM compiler. The CLI
   would report it identically.

## Why this matters

The same diagnostic signal flows through three surfaces:
- The wasm compiler in this playground → inline squiggles
- The CLI → JSON for any shell-bound agent
- The MCP tool `sui_diagnostics` → for agents with the MCP server attached

One source of truth (`packages/deck/lsp/src/diagnostics.ts`), three
delivery channels. An agent writing in CI can use the CLI; an agent
inside Claude Desktop can use the MCP tool; a human typing in the
docs site sees the wasm version.
