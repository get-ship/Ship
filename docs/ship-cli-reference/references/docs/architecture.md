---
group: CLI
order: 4
title: CLI Internals
description: How ship-cli is structured — clap dispatch, transport split between local (compiler) and remote (daemon HTTP), path resolution.
audience: internal
---

# CLI Internals

`apps/ship-cli/`. A thin transport. Parses clap commands, resolves paths, and either calls the compiler directly (pure commands) or POSTs to `shipd` (stateful commands). Holds no domain logic itself.

## Layout

```
apps/ship-cli/src/
  main.rs              — entry; calls dispatch(command)
  cli.rs               — clap Cli + Commands enum (top-level)
  commands.rs          — subcommand enums (Hook/Events/Workspace/Mcp)
  paths.rs             — global_dir(), project_dir() helpers
  daemon/              — daemon lifecycle (pidfile, supervision, service registration)
  compile/             — local compile command
  <per-command>.rs     — one module per top-level command (init.rs, install.rs, ship_run.rs, ...)
```

Every top-level command has a module. The dispatch pattern:

```rust
fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        None => run_status(None),
        Some(cmd) => match cmd {
            Commands::Init { global, harness, force } => init::run(global, harness, force),
            Commands::Run { ... } => ship_run::run(opts),
            Commands::Compile { ... } => compile::run(...),
            // ~40 arms total
        }
    }
}
```

Each arm calls into its module's `run()` function (name varies — `run`, `execute`, etc.). The arm does argument adaptation; the module does the work.

## Clap structure

`cli.rs` defines:

```rust
#[derive(Parser, Debug)]
#[command(name = "ship")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init { global, harness, force },
    Use { agent, ... },
    Compile { ... },
    Run { pipeline, workspace, vars, dry_run, headless, ... },
    Workspace(WorkspaceCommands),
    Mcp(McpCommands),
    Hook(HookCommands),
    Events(EventsCommands),
    Config(ConfigCommands),
    Daemon(DaemonCommands),
    // ... ~40 commands
}
```

Subcommand-heavy groups (workspace, mcp, hook, events) nest their variants in `commands.rs`. `ConfigCommands` and `DaemonCommands` are defined alongside `Commands` in `cli.rs`. Either way, `ship workspace trash` / `ship mcp add-stdio` stay discoverable.

## Feature gating

Unstable commands live behind `#[cfg(feature = "unstable")]`. `main.rs` has:

```rust
#[cfg(feature = "unstable")]
mod top;
```

Stable builds don't ship unstable commands at all. `build_version()` tags the binary:

```rust
pub fn build_version() -> &'static str {
    if cfg!(feature = "unstable") {
        concat!(env!("CARGO_PKG_VERSION"), "+unstable.", env!("GIT_COMMIT_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}
```

## Transport split

Commands fall in three buckets:

### Pure (compiler-only, no daemon)

These commands load `.ship/`, call the compiler crate directly, write bytes. No daemon required.

- `ship compile` — calls `compiler::compile` and `compiler::resolve_library`.
- `ship use <agent>` — sets active agent, re-runs compile.
- `ship validate` — schema validation against `.ship/` files.
- `ship convert` — decompile pathway (`compiler::decompile`).
- `ship audit` — scans for Unicode hiding attacks.
- `ship init` — scaffolds `.ship/` + writes defaults.
- `ship install` / `ship add` — registry-backed, uses `ship_platform::registry` via direct function call (registry is sync, no daemon needed).
- `ship events list` — reads the project event log locally via `ship_substrate::events::read_events()`. No daemon required.

These commands emit events via the init-on-first-use `KernelRouter` — `try_init_kernel_router()` at startup best-efforts the router so audit events land in `platform.db` even without a running daemon. Silent fallback if the global dir is unavailable.

### Daemon-backed (HTTP POST)

These commands POST to `http://127.0.0.1:{port}` using the daemon token.

- `ship run <pipeline>` — POSTs to `/api/v1/agent_sessions/dispatch` (see [Architecture: ship run trace](/architecture/#ship-run-end-to-end-trace)).
- `ship procs` — Queries the ProcessRecord projection (use `--kind agent|pipeline-step|shell` to filter by session class). The REST `/api/v1/procs` endpoint was deprecated after the Connect migration; the CLI now accesses the projection via a direct read from the runtime.
- `ship job <slug>` — single-job operations. The only subcommand today is `cancel` (POSTs `/api/v1/agent_sessions/{slug}/cancel`).
- `ship workspace *` — some subcommands go through the daemon for pipeline coordination.
- `ship hook <trigger>` — emits hook events via daemon (for hook scripts invoked by harnesses).
- `ship tail` — SSE / long-poll for live events.

Token read: `~/.ship/daemon.token` via `auth::read_token`. Port read: `~/.ship/shipd.port` via `daemon::read_port_file`. Connection failures surface as "Try: ship daemon start."

`post_to_daemon(path, body)` is defined separately in each module that uses it (hook.rs, ship_run.rs) — small helper, not worth a shared module today. Each implementation reads token + port, constructs `http://127.0.0.1:{port}{path}`, sends JSON body with `Authorization: Bearer {token}` header, returns parsed JSON response.

### Long-running

These don't return — they're the process.

- `ship daemon run` — spawns the daemon (`shipd::run_network`). The CLI's own `daemon` subcommand is the supervisor entry point.
- `ship mcp serve` — starts an MCP stdio server in-process. Never returns to the shell.

## Daemon lifecycle

`apps/ship-cli/src/daemon/`. The CLI owns starting, stopping, and supervising the daemon process.

- `daemon/pidfile.rs` — reads / writes `~/.ship/shipd.pid`. Knows how to detect stale pidfiles (pid not alive).
- `daemon/supervision.rs` — the supervision strategy abstraction.
- `daemon/supervision_pidfile.rs` — pidfile-based supervision (default).
- `daemon/supervision_systemd.rs` — systemd-based supervision for Linux systems using `ship install-service`.
- `daemon/service_registration.rs` — generates and installs systemd/launchd service files.

`ship daemon start`, `ship daemon stop`, `ship daemon status`, `ship daemon restart` go through this module. `ship daemon run` is the process entry itself — doesn't supervise, just becomes the daemon.

## Path resolution

`paths.rs`. Simple helpers, no runtime dependencies.

- `global_dir()` — `~/.ship/`.
- `global_skills_dir()`, `global_mcp_dir()`, `global_cache_dir()` — children of global.
- `project_dir()` — `.ship/` (relative, per-invocation cwd).
- `agents_dir()`, `rules_dir()`, `skills_dir()`, `mcp_path()` — children of project.
- `project_ship_jsonc()`, `project_ship_toml()` — primary/legacy manifest paths.

For git-worktree-aware resolution (which is usually what you want — discover `.ship/` walking up from cwd, following worktree pointers):

```rust
pub fn project_ship_dir_required() -> Result<PathBuf> {
    ship_substrate::project::get_project_dir(None)
}
```

The heavy lifting is in `ship_substrate::project`; `paths.rs` is the CLI-only convenience layer on top.

## Logging

`logging.rs`. Initializes tracing at startup via `logging::init()`. Returns a guard so file sinks flush on drop. Log output goes to `~/.ship/logs/ship.log`. The error handler in `main()` prints the log path when a command fails:

```
Error: ...

  Logs:   ~/.ship/logs/ship.log
  Events: ship events list --since 5m
```

## Hook trigger scripts

`hook.rs` implements `ship hook <trigger>` — the endpoint hook scripts invoke. The `HookCommands` enum has four variants: `session-start`, `before-tool`, `after-tool`, `session-end`. Example: a harness's session-end hook fires `ship hook session-end < payload.json`. The CLI reads stdin, parses the hook payload, constructs the appropriate event (`session.ended`, `skill.completed`, etc.), and POSTs to the daemon.

Used primarily for events that originate inside harness execution loops and can't be captured any other way.

## Tests

Co-located per-module: `loader_tests.rs`, `convert/tests.rs`, `add/from_tests.rs`, etc. Each exercises the module's exported functions against synthetic inputs.

CLI integration tests that need a daemon spawn shipd in-process under `tokio::test` — this is heavier than unit tests and lives in `crates/` rather than `apps/ship-cli/`.

## Invariants

- `main.rs` stays thin. Dispatch is 40 arms; each arm is one-liner to a module.
- Modules don't reach for clap — they take parsed args or options structs. `cli.rs` is the only file importing `clap::Parser`.
- No module imports another command module. Dispatch is the only caller.
- Daemon-backed commands fail cleanly when the daemon isn't running. "Try: ship daemon start" is the canonical message.
- Pure commands work offline. `ship compile`, `ship validate`, `ship use` never talk to the network.

## Related

- [Commands](/cli/commands/) — public-facing per-command reference.
- [Daemon](/runtime/daemon/) — the other side of the HTTP boundary.
- [Architecture Overview](/architecture/) — where the CLI sits in the system.
