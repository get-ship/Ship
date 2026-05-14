---
group: Runtime
order: 5
title: Home Layout
description: ~/.ship/ structure, ShipHome typed accessors, project root resolution, env vars, init flow.
audience: internal
---

# Home Layout

Operational reference for everything under `~/.ship/` and project `.ship/`. The decisions live in the [home-layout-and-scope-convention ADR](../../../ship-adrs/references/docs/home-layout-and-scope-convention.md); this page documents how to use them.

## Two buckets at user scope

```
~/.ship/
  library/                  ← portable: follows the user across machines
    agents/  skills/  collections/  views/  pipelines/
    triggers/  rules/  permissions/  themes/
    mcp.jsonc
    settings.jsonc
  machine/                  ← machine-local: never portable
    db/
      platform.db (+ -shm + -wal)
      data.db (+ -shm + -wal)
    daemon/                 ← IPC handshake
      port  pid  token  socket  shipd.pid
    logs/                   ← shipd.log, ship.log
    cache/  sessions/  actors/  projects/  crashes/
    local.jsonc             ← machine-specific overrides (worktrees.dir, etc.)
```

The split answers *"what moves with the user"* vs *"what stays on this machine"*. Sync, backup, and portability operate on `library/`; runtime state lives in `machine/`.

## Project scope

Project `.ship/` mirrors the `library/` shape — no `library/` / `machine/` split, since everything project-side is project-local.

```
<project>/.ship/
  agents/  skills/  collections/  views/  pipelines/
  triggers/  rules/  permissions/  themes/
  mcp.jsonc
  ship.jsonc                ← project manifest
  ship.lock
  hooks/                    ← reactive triggers (.ship/hooks/*.jsonc — rename to triggers/ pending)
```

**Precedence (lowest → highest):** built-in defaults → user (`~/.ship/library/`) → project (`<project>/.ship/`) → project-local overrides (gitignored).

## ShipHome — the only path accessor

`crates/core/ship-substrate/src/home/ship_home.rs`. Every path under `~/.ship/` resolves through one of these. Raw `home_dir().join(".ship")` is forbidden — `scripts/lint/check-raw-ship-paths.sh` hard-fails CI on regression.

```rust
use ship_substrate::{ShipHome, DaemonFile, Db, Scope, Vector};

let home = ShipHome::from_env();           // production: reads SHIP_HOME or falls back to ~/.ship
let test = ShipHome::for_test(tmp.path()); // test helper: <tmp>/.ship
let h    = ShipHome::at(some_path);        // explicit root

home.root()                                          // ~/.ship/
home.library_dir()                                   // ~/.ship/library/
home.machine_dir()                                   // ~/.ship/machine/
home.settings_path()                                 // ~/.ship/library/settings.jsonc
home.local_path()                                    // ~/.ship/machine/local.jsonc
home.db_path(Db::Platform)                           // ~/.ship/machine/db/platform.db
home.db_path(Db::Data)                               // ~/.ship/machine/db/data.db
home.daemon_file(DaemonFile::Port)                   // ~/.ship/machine/daemon/port
home.daemon_file(DaemonFile::Token)                  // ~/.ship/machine/daemon/token
home.daemon_file(DaemonFile::Pid)                    // ~/.ship/machine/daemon/pid
home.daemon_file(DaemonFile::Socket)                 // ~/.ship/machine/daemon/socket
home.vector_dir(Scope::User, Vector::Agents, None)   // ~/.ship/library/agents/
home.vector_dir(Scope::Project, Vector::Views, Some(&proj))  // <proj>/.ship/views/

ShipHome::project_ship_dir(&proj)   // <proj>/.ship/
ShipHome::ship_dir_name()           // ".ship"
```

Two helpers in `ship_substrate::db`:

```rust
ship_substrate::db::platform_db_path(ship_dir)   // <ship_dir>/machine/db/platform.db
ship_substrate::db::data_db_path(ship_dir)       // <ship_dir>/machine/db/data.db
```

## Vector enum

Nine **directory** vectors, mirrored at user and project scope:

```rust
pub enum Vector {
    Agents, Skills, Collections, Views, Pipelines,
    Triggers, Rules, Permissions, Themes,
}
```

Adding a new directory-vector is a one-line enum change (plus matching `as_str()`/`all()` updates) — guarantees uniform treatment across scopes, loaders, lint rules, and docs.

**`mcp` is intentionally not a directory vector.** At every scope MCP config is a single JSONC file:
- User: `~/.ship/library/mcp.jsonc`
- Project: `<project>/.ship/mcp.jsonc`

## Project root resolution

`ship_substrate::project::resolve_project_ship_dir(start_dir)` is the canonical resolver. Returns `Option<PathBuf>` pointing at the `.ship/` directory.

Behavior:

1. Walk up from `start_dir` looking for `.ship/`.
2. If the found `.ship/` lives inside a **linked git worktree**, redirect to the **main checkout's** `.ship/` so worktrees never register as separate projects in `~/.ship/machine/projects/`.
3. Worktree dev opt-out: when `SHIP_RESPECT_WORKTREE=1` is set, the worktree's local `.ship/` wins (no redirect). Used by `scripts/dev-daemon.sh` and other dev tooling iterating per-worktree views/themes/agents.

Higher-level wrapper `ship_substrate::project::get_project_dir(start: Option<PathBuf>)`:

- If `SHIP_DEV_PROJECT_DIR` is set, returns it directly (test/dev override).
- Else delegates to `resolve_project_ship_dir`.
- Errors with "Project tracking not initialized" if no `.ship/` is reachable.

`apps/shipd/src/main.rs::discover_workspace()` is a thin wrapper that returns the project root (parent of `.ship/`) for use as the daemon's workspace dir. Both shipd and ship-cli go through this resolver — no parallel implementations.

## Env vars

`crates/core/ship-substrate/src/env.rs` is the single source of truth. Every `SHIP_*` env var the runtime reads has a `Var` variant; reads go through `env::read(Var::*)`. Raw `std::env::var("SHIP_*")` outside this file fails the `check-raw-env-reads.sh` lint guard.

| Var | Env name | Category | Purpose |
|---|---|---|---|
| `Var::Home` | `SHIP_HOME` | Configuration | Override `~/.ship/` location (single root knob) |
| `Var::ApiUrl` | `SHIP_API_URL` | Configuration | Cloud gateway URL |
| `Var::RegistryUrl` | `SHIP_REGISTRY_URL` | Configuration | Package registry URL |
| `Var::ActorId`, `WorkspaceId`, `SessionId`, `AgentId`, `MeshId`, `WorktreeDir`, `RepoDir` | various | RuntimeIdentity | Propagated to spawned processes |
| `Var::Dev` | `SHIP_DEV` | DevDebug | Dev-mode flag (relaxed auth, etc.) |
| `Var::DevProjectDir` | `SHIP_DEV_PROJECT_DIR` | DevDebug | Project-root override (tests + dev tooling) |
| `Var::RespectWorktree` | `SHIP_RESPECT_WORKTREE` | DevDebug | When `=1`, project-root resolver honors worktree's local `.ship/` |
| `Var::Executor`, `TmuxTests`, `DefaultTerminal`, `ShellDevUrl`, `DaemonPath`, `ManagedHooks`, `HooksBin` | various | DevDebug | Dev-loop knobs |
| `Var::TelegramToken`, `TelegramChatId` | various | DevDebug | Notification integrations |

`SHIP_HOME` is the **only** ship-home override. There are no per-bucket `SHIP_LIBRARY_DIR` / `SHIP_MACHINE_DIR` variables.

`SHIP_DEV_PROJECT_DIR` is for **tests and dev tooling only** — production callers never set it. Tests use it for parallel-safe project-root isolation; setting `cwd` per-test races in parallel test runs.

## Initialization

**On `shipd` start** (`apps/shipd/src/ship_runtime.rs`):

```rust
ship_substrate::home::ensure_layout(&ShipHome::at(global_dir))?;
```

`ensure_layout` `mkdir -p`'s the canonical bucket tree (`library/`, `machine/{db,daemon,logs,cache,sessions,actors,projects,crashes}/`) idempotently. No detection, no migration, no version marker — fresh installs and existing installs converge on the same shape.

**`ship init --global`** writes `~/.ship/library/settings.jsonc` if missing. Stub content:

```jsonc
// Ship user settings (portable across machines).
{
  "identity": { "name": "" }
}
```

**`ship init`** (project-scope) creates `<cwd>/.ship/`, scaffolds `ship.jsonc` (or imports existing provider configs via auto-detection). Writes JSONC; no TOML branch.

## Lint guards

| Guard | What it catches | Hard-fail? |
|---|---|---|
| `scripts/lint/check-raw-ship-paths.sh` | Raw `.join(".ship")` or `home_dir().join(".ship/…")` outside `crates/core/ship-substrate/src/home/` | Yes (via `just check-raw`) |
| `scripts/lint/check-raw-env-reads.sh` | `env::var("SHIP_*")` outside `crates/core/ship-substrate/src/env.rs` | Yes (via `just check-raw`) |

Both lints exit non-zero on violations and run in CI. Both currently report **zero violations** across the workspace.

Tests that previously set raw env vars (`std::env::set_var("SHIP_DEV_PROJECT_DIR", ...)`) should switch to typed accessors:

```rust
unsafe { ship_substrate::env::set(ship_substrate::env::Var::DevProjectDir, path) };
unsafe { ship_substrate::env::remove(ship_substrate::env::Var::DevProjectDir) };
```

## Regression tests

- `apps/shipd/src/daemon.rs::path_layout_tests` — pid/port/token/log paths land at the typed-accessor locations; no legacy root files.
- `apps/shipd/tests/smoke.rs::fresh_daemon_writes_canonical_home_layout` — full e2e: spawn shipd against a fresh `SHIP_HOME`, assert the canonical bucket tree exists and zero legacy paths.
- `crates/core/ship-substrate/src/home/ensure.rs::tests` — `ensure_layout` is idempotent and preserves existing content.

## See also

- ADR: [home-layout-and-scope-convention](../../../ship-adrs/references/docs/home-layout-and-scope-convention.md) — decisions and alternatives considered.
- [Config](./config.md) — `ProjectConfig` shape, merge order, `get_effective_config`.
- [Daemon](./daemon.md) — daemon lifecycle and IPC.
- [Data DB](./data-db.md) — `data.db` collections layer.
- [Platform DB](./platform-db.md) — `platform.db` event log + projections.
