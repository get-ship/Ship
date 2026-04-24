---
group: CLI
title: Command Reference
description: Complete reference for every Ship CLI command with flags, arguments, and examples.
section: reference
order: 2
---

# Command Reference

## Project Setup

### `ship init`

Scaffold `.ship/` in the current project, or configure `~/.ship/` globally.

| Flag | Description |
|------|-------------|
| `--global` | Configure `~/.ship/` instead of current project |
| `--provider <id>` | Default provider: `claude`, `gemini`, `codex`, `cursor` |
| `--force` | Overwrite existing `.ship/` |

```bash
ship init                        # scaffold with claude default
ship init --provider gemini      # scaffold with gemini default
ship init --global               # configure ~/.ship/ identity
```

### `ship use <agent-id>`

Activate an agent and compile immediately. Resolves skill references, MCP servers, and permissions, then writes provider-native config files. Flag: `--path <dir>` to bind to another path.

### `ship status`

Show active agent and compilation status. Flag: `--path <dir>`.

### `ship compile`

Compile the active agent to provider-native config files.

| Flag | Description |
|------|-------------|
| `--provider <id>` | Compile for a specific provider only |
| `--dry-run` | Preview without writing files |
| `--path <dir>` | Project root |

### `ship validate`

Validate `.ship/` config -- JSONC syntax, skill refs, MCP fields, permissions.

| Flag | Description |
|------|-------------|
| `--agent <id>` | Validate a single agent |
| `--json` | Emit errors as JSON |
| `--path <dir>` | Project root |

### `ship convert .`

Convert existing provider config files in the current directory into `.ship/` format.

## Agent Management

Agents are JSONC files in `.ship/agents/`. `ship init` scaffolds a `dev` agent automatically.

| Task | How |
|------|-----|
| Create an agent | Add a file at `.ship/agents/<id>.jsonc` |
| Activate | `ship use <id>` |
| Recompile | `ship compile` |
| Validate | `ship validate --agent <id>` |

To create additional agents, copy an existing one and change the `id` field.

## Skills

Skills are directories in `.ship/skills/` containing a `SKILL.md` file. Install from the registry or create locally.

| Task | How |
|------|-----|
| Install from registry | `ship add github.com/owner/skill-pack` |
| Create a local skill | `mkdir .ship/skills/my-skill` and add `SKILL.md` |
| Reference in an agent | Add the skill ID to `skills.refs` in the agent JSONC |

## MCP Servers

| Command | Description |
|---------|-------------|
| `ship mcp serve [--http] [--port <n>]` | Run the Ship MCP server (stdio default, or HTTP) |
| `ship mcp add <id> --url <url>` | Register an HTTP/SSE MCP server |
| `ship mcp add-stdio <id> <cmd> [args...]` | Register a stdio MCP server |
| `ship mcp list` | List configured MCP servers |
| `ship mcp remove <id>` | Remove a configured MCP server |

```bash
ship mcp add-stdio ship ship mcp serve
ship mcp add my-api --url http://localhost:8080/mcp
```

## Registry and Dependencies

### `ship install`

Resolve and install all dependencies from `.ship/ship.jsonc`, then compile.

| Flag | Description |
|------|-------------|
| `--frozen` | Fail if lockfile would change (CI-safe) |

### `ship add <package>`

Add a dependency to `.ship/ship.jsonc` and install it. Version defaults to `main` if omitted.

```bash
ship add some-package@0.2.0
```

### `ship publish`

Publish the current package to the Ship registry. Requires `ship login`.

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview without network |
| `--tag <tag>` | Dist-tag for pre-release |

## Authentication

### `ship login` / `ship logout` / `ship whoami`

Authenticate with getship.dev via PKCE OAuth, remove stored token, or show current identity.

```bash
ship login       # PKCE OAuth via browser
ship logout      # remove stored token
ship whoami      # show current identity
```

## Information and Diagnostics

### `ship events list`

Query the project event log.

| Flag | Description |
|------|-------------|
| `--since <time>` | Since timestamp or relative (`1h`, `24h`, `7d`) |
| `--actor <name>` | Filter by actor |
| `--entity <type>` | Filter by entity type |
| `--action <name>` | Filter by action |
| `--limit <n>` | Max events (default 50) |
| `--json` | Output as JSON |

### `ship docs [topic]`

Show detailed help. Topics: `agents`, `skills`, `mcp`, `providers`, `compile`, `workflow`.

```bash
ship docs topics    # list all topics
ship docs agents    # agent management help
```

