---
name: ship-getting-started
stable-id: ship-getting-started
description: Use when helping users get started with Ship — installation, first project, first agent, compiling, and running pipelines.
tags: [ship, onboarding, guide]
authors: [ship]
---

# Getting Started with Ship

Ship compiles declarative agent configuration into provider-native files. You define agents, skills, and MCP servers once in `.ship/`, then compile to any supported provider (Claude, Gemini, Codex, Cursor).

## Install

Build from source:

```bash
git clone https://github.com/anthropics/ship.git
cd ship
cargo install --path apps/ship-cli
```

Or install directly:

```bash
cargo install ship-cli
```

Verify installation:

```bash
ship --version
```

## Initialize a Project

```bash
cd my-project
ship init
```

This scaffolds a `.ship/` directory with default configuration:

```
.ship/
  agents/         # agent profiles (.jsonc)
  skills/         # skill definitions
  pipelines/      # pipeline JSONC files
  ship.toml       # project manifest
```

For global user configuration (identity and defaults):

```bash
ship init --global
```

## Create an Agent

Agent profiles live in `.ship/agents/<name>.jsonc`. A minimal agent:

```jsonc
{
  "id": "my-agent",
  "name": "My Agent",
  "description": "A custom development agent",
  "providers": ["claude"],
  "skills": [],
  "mcp_servers": []
}
```

## Compile Provider Config

Activate an agent and compile its configuration:

```bash
ship use my-agent
```

This compiles the agent's skills, rules, and MCP servers into provider-native files (e.g., `CLAUDE.md`, `.mcp.json` for Claude). The output files are gitignored build artifacts.

To recompile after editing config:

```bash
ship compile
```

To preview without writing files:

```bash
ship compile --dry-run
```

## Check Status

```bash
ship status
```

Shows the active agent and compilation state for the current project.

## Validate Configuration

```bash
ship validate
```

Checks `.ship/` for errors: TOML syntax, skill references, MCP field validation, and permission consistency.

## Run a Pipeline

Pipelines define multi-step agent workflows. Ship includes built-in pipelines (`simple`, `standard`, `tdd`).

```bash
ship run tdd "implement user authentication" --var impl_agent=rust-lane
```

This loads `.ship/pipelines/tdd.jsonc`, interpolates variables, and dispatches each step sequentially. The `tdd` pipeline runs: `test-writer` -> `impl_agent` -> `reviewer`.

To preview the resolved pipeline without executing:

```bash
ship run tdd "my task" --dry-run
```

## Manage MCP Servers

Register an MCP server:

```bash
ship mcp add-stdio my-server npx my-mcp-tool
ship mcp list
```

## Manage Packages

Install dependencies declared in `.ship/ship.toml`:

```bash
ship install
```

Add a new dependency:

```bash
ship add github.com/owner/repo
```

## Start the Daemon

For multi-agent coordination, pipelines, and the event system, start the daemon:

```bash
ship daemon start
```

Check daemon status:

```bash
ship daemon status
```

## Next Steps

- **`ship docs topics`** — list available help topics
- **`ship docs <topic>`** — detailed help on agents, skills, providers, pipelines
- **`ship convert <source>`** — import existing provider configs into `.ship/` format
