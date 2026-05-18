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

## Build a UI with Views

The same daemon that compiles agent configuration also hosts SUI. A view is a SUI template — a `.sui` file declaring markup, optionally paired with a `controller.ts` that owns reactive state and event handlers. The compiler emits a hydratable ESM module; the daemon serves an SSR snapshot, the browser hydrates it. The bundler scans two layouts under the workspace root:

```
views/<name>/page.sui          # top-level
.ship/views/<name>/index.sui   # colocated with .ship/ config (top-level wins on name collision)
```

`ship init` does not yet scaffold a `views/` directory — create one by hand.

A minimal view at `views/hello/page.sui`:

```html
<div class="hello">
  <h1>Hello from Ship</h1>
  <p>Clicked {count()} time{count() === 1 ? '' : 's'}.</p>
  <button on:click={() => count.set(count() + 1)}>Click</button>
</div>
```

And the paired `views/hello/controller.ts`:

```ts
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => ({ count: signal(0) }),
})
```

The bundler that compiles `.sui` files runs out of `~/.ship/runtime/` (Bun + the SUI compiler). Install it once per machine:

```bash
ship runtime install
```

Then start the daemon and read the bound URL:

```bash
ship daemon start
ship daemon status
```

`ship daemon start` prints a hardcoded `http://127.0.0.1:9315`, but the daemon auto-derives a per-worktree port — `ship daemon status` is what reports the actual `port:`. Visit `http://<host>:<port>/sui/hello` to mount the view directly through the view-host endpoint. The root path (`/`) redirects to `/setup` on a fresh install, or to `/shell/` once a workspace exists (`apps/shipd/src/views/setup.rs:83`).

## Live Edits

The daemon runs in `--mode development` by default (`apps/shipd/src/main.rs:50`), which exposes two SSE channels the browser subscribes to:

- **`/_sui/dev/hmr`** — forwarded from the bundler worker. Editing `views/<name>/page.sui` or its `controller.ts` re-emits the view bundle; if that view is currently mounted, the shell cache-busts the import and re-mounts it in place without a page reload. Editing a component, island, or the SUI runtime triggers a full reload (custom-element re-registration without reload is out of scope today).
- **`/_sui/hmr`** — file-watcher for `.set` files under `.ship/sui/` (ship-set component authoring). Always full reload.

Compile errors are streamed on the same `/_sui/dev/hmr` channel and logged to the browser console with `file:line`; the last good mount stays on screen until the next successful build.

## If Live Edits Don't Work

- **`~/.ship/runtime/` is missing.** No bundler, no rebuilds. Run `ship runtime install`.
- **Daemon is in production mode.** HMR routes are wired only when `daemon_mode == Development`; in production they return 404 by design. Since `ship daemon start` doesn't accept `--mode`, this only happens if `SHIP_DAEMON_MODE=production` is set in the environment (or `shipd` was launched directly with `--mode production`). Unset and restart.
- **The view directory was created after the daemon started.** The bundler computes its target list at startup, so brand-new top-level view directories aren't bundled until the daemon restarts.
- **The file isn't where the bundler looks.** Only `views/<name>/page.sui` and `.ship/views/<name>/index.sui` are scanned — other paths are invisible.

## Next Steps

- **`docs/ship-sui/SKILL.md`** — SUI deep dive: compiler, runtime, controllers, the four layers
- **`ship docs topics`** — list available help topics
- **`ship docs <topic>`** — detailed help on agents, skills, providers, pipelines
- **`ship convert <source>`** — import existing provider configs into `.ship/` format
