---
group: Schema Reference
title: Environment Variables
section: reference
order: 10
---

# Ship Environment Variables

Ship reads a small set of `SHIP_*` environment variables. They are an ephemeral, fourth-tier override on top of the layered config story (compile-time default → daemon KV → `.ship/ship.jsonc` → env var). Most users never set any of these directly; they exist for tooling, tests, and a handful of advanced configuration knobs.

The canonical source of truth is the `Var` enum in `crates/core/ship-substrate/src/env.rs`. Every `SHIP_*` value the runtime reads is registered there and accessed via `env::read(Var::…)`; a CI lint (`scripts/lint/check-raw-env-reads.sh`) flags raw `env::var("SHIP_*")` calls outside that module.

Variables are grouped by category — Configuration, Runtime identity, and Dev/debug — matching the taxonomy in `Var::category()`.

## Configuration

User-facing knobs that change how the daemon and CLI behave.

| Var | Read at | Purpose | Related setting / file |
|---|---|---|---|
| `SHIP_HOME` | Process startup (CLI + daemon) | Override the `~/.ship` root. Resolved by `ship_home()` in `ship-substrate/src/home/ship_home.rs:62`. Empty / unset uses `$HOME/.ship`. | Identity-shaped; no KV / project-file equivalent. |
| `SHIP_API_URL` | Lazy (CLI `convert/github`) | Base URL for the hosted ship-studio.com API. Read by `apps/ship-cli/src/convert/github.rs:89`. Default: `https://ship-studio.com`. | None — env-only override. |
| `SHIP_REGISTRY_URL` | Lazy (registry resolver, tracking, verify) | Base URL for the package registry. Read by `crates/core/ship-platform/src/registry/{resolver,tracking,verify}.rs`. Default: registry default URL. | None — env-only override. |
| `SHIP_DAEMON_MODE` | Daemon startup | `development` (default) or `production`. Fallback for shipd `--mode`. Production toggles immutable cache headers, disables HMR routes, enables SSR cache, redacts error detail. Read by `apps/shipd/src/main.rs:50`. Unknown values fall back to `development`. | CLI `--mode` flag wins. |
| `SHIP_LOG` | CLI startup | `tracing_subscriber::EnvFilter` directive. Read by `apps/ship-cli/src/logging.rs:20`. Default: `warn`. | None — env-only override. |

## Runtime identity

Values injected by the daemon into spawned agent processes so each child inherits the actor/workspace/session context it belongs to. Set by Ship; users do not set these directly.

| Var | Set by | Consumed by |
|---|---|---|
| `SHIP_ACTOR_ID` | Orchestrator + harness adapters when dispatching a job/lane (`dispatch.rs`, `pipeline.rs`, `fanout.rs`, every `harnesses/<provider>.rs`). | Hook helpers (`apps/ship-cli/src/hook.rs`), MCP gate (`apps/mcp/src/server/tool_gate.rs`), tests. |
| `SHIP_AGENT_ID` | Supervisor transport when launching an agent process (`ship-substrate/src/supervisor/transport.rs`). | Hook helpers in CLI. |
| `SHIP_WORKSPACE_ID` | Supervisor transport. | Hook helpers in CLI. |
| `SHIP_SESSION_ID` | Reserved for session-scoped subprocesses. | Reserved. |
| `SHIP_WORKTREE_DIR` | Daemon when launching processes pinned to a worktree. | Project-local config resolution (`ship-platform/src/config/local.rs:90`). |
| `SHIP_REPO_DIR` | Supervisor + bundler subprocess launches. | MCP `sui_lsp` to locate the repo (`apps/mcp/src/tools/sui_lsp.rs:23`), ship-supervisor helpers. |
| `SHIP_MESH_ID` | Reserved for cross-mesh identity propagation. | Reserved (declared in `Var`, no production read yet). |

## Dev / debug

Toggles for tests, debugging, and development workflows. Not intended for production use.

| Var | Read at | Purpose |
|---|---|---|
| `SHIP_DEV_PROJECT_DIR` | Per-call (project resolution, SUI server) | Override the project root returned by `get_project_dir(None)`. Used by tests and the dev SUI workflow to isolate parallel runs. Read by `crates/core/ship-substrate/src/project/discovery.rs:246`, `crates/ship/ship-sui-server/src/resolver.rs:38`, `apps/shipd/src/lib.rs:180`. Never set in production. |
| `SHIP_DEV_DB_DIR` | Per-call (db path resolver) | Override the directory for machine SQLite stores (`platform.db`, `kv.db`, etc.). Lets a dev worktree's daemon point at an isolated DB to iterate migrations without colliding with the canonical store. Read by `crates/core/ship-substrate/src/db/mod.rs:86`. Never set in production. |
| `SHIP_RESPECT_WORKTREE` | Per-call (project discovery) | When `=1`, project-root resolution honors a worktree's local `.ship/` instead of redirecting to the main checkout. Used by dev tooling that iterates per-worktree views/themes. Read by `crates/core/ship-substrate/src/project/discovery.rs:182`. |
| `SHIP_NO_BROWSER` | Daemon first-run | Suppresses the automatic browser open on `shipd` first run. Read by `apps/shipd/src/daemon/first_run.rs:18`. |
| `SHIP_DEFAULT_TERMINAL` | Per-call (terminal launcher) | Override the auto-detected terminal program (`tmux`, `kitty`, `wezterm`, `iterm2`, `terminal`). Read by `crates/core/ship-supervisor/src/terminal_launcher.rs:89`. Equivalent to the `terminal.program` KV setting; the env var wins when set. |
| `SHIPD_PATH` | CLI daemon launch | Override the path to the `shipd` binary the CLI spawns. Read by `apps/ship-cli/src/daemon/service_registration.rs:123`. |
| `SHIP_TMUX_TESTS` | Test gates | Opts a test into running the real tmux launcher. Read by `crates/core/ship-substrate/src/testutil.rs:156`. |
| `SHIP_TELEGRAM_TOKEN` | Lazy (human gateway) | Bot token for the Telegram human-gateway transport. Read by `crates/core/ship-substrate/src/services/human_gateway.rs:56` and `apps/shipd/src/services/registry/mesh.rs:196`. Pairs with `SHIP_TELEGRAM_CHAT_ID`. |
| `SHIP_TELEGRAM_CHAT_ID` | Lazy (human gateway) | Chat ID for the Telegram human-gateway transport. Read at the same sites as `SHIP_TELEGRAM_TOKEN`. |
| `SHIP_MANAGED_HOOKS` | Reserved | Declared in `Var`; no production read yet. Placeholder for the upcoming managed-hooks toggle. |
| `SHIP_HOOKS_BIN` | Reserved | Declared in `Var`; no production read yet. Placeholder for routing hooks through an explicit binary. |
| `SHIP_DEV` | Reserved | Declared in `Var`; no production `env::read` yet. Pairing dev-mode is driven by the in-process `DEV_MODE` flag rather than this var today (see `crates/core/ship-platform/src/auth/pairing.rs:40`). |

The remaining few directly-read `SHIP_*` names in the tree are test-only and not part of the public surface: `SHIP_TEST` (test-mode global-dir auto-detect), `SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR` (escape hatch for that auto-detect), and `SHIP_REPO_ROOT` / `SHIP_PROJECT` (passed into the bundler worker and into decompile fixtures). They are intentionally not in the `Var` registry.

## Adding a new env var

Per the home-layout-and-scope-convention ADR:

1. Add a variant to `Var` in `crates/core/ship-substrate/src/env.rs` with a doc comment describing purpose, read site, and override semantics.
2. Map it in `Var::as_str()` and `Var::category()`.
3. Add a row to this page in the matching section.
4. Read via `env::read(Var::…)` — never via `std::env::var("SHIP_…")` directly.
