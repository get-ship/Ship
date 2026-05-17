---
name: ship-cli-reference
stable-id: ship-cli-reference
description: Use when working with Ship CLI commands — init, use, compile, run, status, validate, mcp, workspace, daemon, and package management. Complete command reference with flags and examples.
tags: [ship, cli, reference]
authors: [ship]
---

# Ship CLI Reference

Ship compiles declarative agent configuration (`.ship/`) into harness-native files. The core workflow: `init` a project, `use` an agent, `compile` to emit outputs, `run` to execute pipelines.

## Core Workflow

```
ship init                         # scaffold .ship/ in current project
ship init --global                # configure ~/.ship/ identity and defaults
ship use <agent-id>               # activate agent and compile
ship use <agent-id> --with <skill># inject extra skills at compile time
ship compile                      # recompile after config changes
ship compile --dry-run            # preview output without writing files
ship compile --harness claude    # compile for a specific harness only
ship status                       # show active agent and compilation state
ship validate                     # check config for errors
ship validate --agent <name>      # validate a single agent
ship validate --json              # emit errors as JSON array
```

## Pipelines

```
ship run <pipeline> [spec]        # run a named pipeline
ship run tdd "my task"            # run tdd pipeline with inline spec
ship run standard "task" --var impl_agent=rust-lane
                                  # pass pipeline variables
ship run tdd "task" --spec path/to/spec.md
                                  # use a spec file instead of inline text
ship run tdd "task" --workspace my-branch
                                  # run in a specific workspace
ship run tdd "task" --dry-run     # show resolved pipeline without executing
```

Pipelines are loaded from `.ship/pipelines/<name>.jsonc`. Built-in pipelines: `simple`, `standard`, `tdd`.

## Workspace

```
ship workspace trash              # list trashed files for current workspace
ship workspace trash --purge      # force purge all trash (ignores 24h retention)
ship workspace restore <filename> # restore most recent trashed version
ship workspace restore <f> --dest <path>
                                  # restore to a custom path
```

## MCP Servers

```
ship mcp serve                    # run Ship MCP server (stdio)
ship mcp serve --http --port 3000 # run over HTTP
ship mcp add <id> --url <url>     # register an HTTP/SSE MCP server
ship mcp add-stdio <id> <cmd> [args...]
                                  # register a stdio MCP server
ship mcp list                     # list configured MCP servers
ship mcp remove <id>              # remove an MCP server
```

## Daemon

```
ship daemon start                 # start shipd in the background
ship daemon start --host 0.0.0.0  # bind to a specific address
ship daemon stop                  # stop the running daemon
ship daemon status                # show daemon status
ship daemon cleanup               # remove orphaned worktrees for merged/failed jobs
```

## Package Management

```
ship install                      # install all deps from .ship/ship.toml
ship install --frozen             # fail if lockfile would change
ship install --offline            # skip install tracking (no network)
ship add <package>[@version]      # add and install a dependency
ship add --from <url>             # import from a Studio share link
ship publish                      # publish to Ship registry
ship publish --dry-run            # preview without publishing
ship publish --tag beta           # publish with a dist-tag
ship audit                        # scan for hidden Unicode (security)
ship audit --json                 # emit findings as JSON
```

## Authentication

```
ship login                        # authenticate with getship.dev
ship logout                       # sign out
ship whoami                       # show current identity
```

## Configuration

```
ship config get <key>             # get a config value
ship config set <key> <value>     # set a config value
ship config list                  # list all set config values
ship config path                  # show config file path
```

## Other Commands

```
ship convert <source>             # convert harness configs to .ship/ format
ship docs topics                  # list help topics
ship docs <topic>                 # detailed help for a topic
ship hook session-start           # harness hook: session start
ship hook before-tool             # harness hook: before tool
ship hook after-tool              # harness hook: after tool
ship hook session-end             # harness hook: session end
```

## Harness Outputs

All outputs are gitignored build artifacts produced by `ship compile` / `ship use`.

| Harness | Context file | MCP config | Skills dir |
|----------|-------------|------------|------------|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/` |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` | `.gemini/skills/` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/` |
| `cursor` | `.cursor/rules/*.mdc` | `.cursor/mcp.json` | `.cursor/skills/` |
