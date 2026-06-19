# Changelog

## v0.1.0 — 2026-03-19

First release of the Ship CLI and MCP server.

### What's included

**Compiler**
- `ship use <preset>` — activate a preset: compiles and writes provider config files for Claude, Gemini, Codex, and Cursor simultaneously
- `ship compile` — compile without activating
- `ship validate` — validate `.ship/` config without compiling
- Multi-provider output: `CLAUDE.md`, `GEMINI.md`, `AGENTS.md`, `.mcp.json`, `.cursor/rules/`, `.codex/config.toml`
- Skill files compiled to `.claude/skills/`, `.agents/skills/`, `.gemini/agents/`
- Agent profiles compiled to provider-native agent definition files

**Package manager**
- `ship install` — install preset dependencies declared in `ship.toml`
- Git-native registry: dependencies are git repositories, no central blob store
- Lockfile (`ship.lock`), versioning, transitive dependency resolution
- `ship use @org/preset` — install and activate from a registry

**MCP server (`ship mcp serve`)**
- Project intelligence over MCP: workspaces, sessions, jobs, capabilities, targets, skills, notes, ADRs
- Attach to Claude Code or any MCP-compatible client
- Session lifecycle: `start_session`, `log_progress`, `end_session`
- Job queue: `create_job`, `list_jobs`, `update_job`, `append_job_log`

**CLI**
- `ship init` — initialize a project
- `ship status` — show active workspace and session
- `ship workspace` — workspace management
- `ship agent` — agent profile management
- `ship skill` — skill management
- `ship mcp` — MCP server config management
- `ship job` — job queue (create, list, update)
- `ship auth` — authentication

### Platforms
- macOS (arm64, x86_64)
- Linux (x86_64)
- Windows (x86_64)

### Install

```bash
# macOS / Linux — see release assets for your platform
curl -fsSL <release-url>/ship-<platform>.tar.gz | tar -xz
sudo mv ship /usr/local/bin/
```

The `ship` binary includes the MCP server. Run `ship mcp serve` to start it.

## Unreleased — R17 flag-off quarantine (#24)

Two related changes land in this slice. Together they make the `ship-code`
feature gate drop the entire third-party-harness driver stack from the
daemon build AND keep the floki-harness path routable. The floki path is
now a feature-independent daemon concern — it lives or dies based on
its own runtime, not the harness-driver feature flag.

### (b) — floki is feature-independent (LANDED)

`shipd` previously built the floki-harness dispatch closure inside a
`#[cfg(feature = "ship-code")]` block, so `--no-default-features` left
floki with no run_fn wired (the agent-session actor started but every
floki payload was silently no-op'd). The block sat inside
`run_dispatch::subscribe_job_events`, whose struct literal also swept
the harness-port fields into the same cfg arm.

Fix:
- `crates/core/ship-platform/crates/ship-agents/src/ports.rs`: added
  `AgentPorts::floki_only(run_fn)` constructor, gated to
  `#[cfg(not(feature = "harness-drivers"))]`. The struct's `floki_run_fn`
  field is already ungated; this constructor is the flag-off entry point
  that fills ONLY the floki field and leaves the harness-side fields
  omitted (the literal doesn't name them).
- `apps/shipd/src/lib.rs`: the floki-harness dispatch closure build is
  moved OUTSIDE the `#[cfg(feature = "ship-code")]` block. The closure
  captures `Arc<ShipdGateway>` and the kernel, both floki-/L1-clean.
  In the flag-on build, the closure hands to the existing
  `RunManager::init → subscribe_job_events → start_agents` path (the
  actor is started once, with the full port bundle). In the flag-off
  build, a small `start_agents` invocation with a floki-only
  `AgentPorts` keeps the agent-session actor alive and the floki
  branch in `handle_agent_session_started` routable. Non-floki
  sessions in flag-off become no-ops (the harness-driver stack is
  cfg-omitted from `handle_agent_session_started`, by design).
- New unit tests under `cfg(not(feature = "harness-drivers"))` guard
  the constructor: `floki_only_round_trips_closure`,
  `floki_only_none_is_noop`. The existing floki-routing behavior tests
  in `crates/core/ship-platform/crates/ship-agents/tests/behavior.rs`
  (gated to `harness-drivers`) still pass — they assert the harness
  executor is skipped when `floki_run_fn` is wired, which the floki
  branch in `handle_agent_session_started` does regardless of feature
  state.

### (a) — ship-pipeline-runtime gate (DEFERRED to R13 dissolution)

`crates/ship-pipeline-runtime/Cargo.toml:25` unconditionally pulls
`ship-code` (no `optional`, no feature gate). The runtime is the L1
orchestrator that USES `ship_code` as its port surface — `ship_code::dispatch_ports::*`
(SessionContext, LaunchProfile, StepInfo, AgentPlatform, IsolationStrategy,
ProcessRegistryPort, ProjectionReader, SessionExecutor, SpawnPolicy,
ExitSignal, ProcessTreeProbe, TreeSample, ExitCause, Authority, ExecutorHandle)
and `ship_code::runner::*` (build_spawn_env, resolve_step_invocation,
dedup, exit_handler, router::{extract_slug, terminal_guard, Admit},
stop_by_slug, stop_handle, observe::{scan_agent_log_tail, ...}) are
called from ~30 sites across `crates/ship-pipeline-runtime/src/orchestrator/`
and the step_monitor / fanout / tests_heartbeat subtrees. Gating
`ship_code` to be optional would require port-typing every call site
and propagating feature gates through the orchestrator to shipd —
the same shape of work that R13's pipeline-runtime dissolution is
already chartered to do. We do not duplicate the R13-shaped work in
this slice.

Net result: `cargo tree -p shipd --no-default-features` still shows
`ship-code`, but only via `ship-pipeline-runtime`. Once R13 dissolves
the orchestrator, the same cfg-gate pattern used here for the agent
domain will clean this up.

## Unreleased — storage cleanup (job/storage-cleanup)

Two bounded storage fixes:

- **Remove the dead `platform.db` subsystem.** It was legacy cruft, never
  created on fresh installs, with no live readers or writers. Pre-launch
  = no data to preserve, so the file, its one-shot legacy drain, the
  `Db::Platform` variant, the `ensure_db_at`/`platform_db_path` resolvers,
  the `db::repair` migration-drift diagnostic, the
  `migrate_skill_vars_from_platform_db` drain, the `ship daemon
  repair-migrations` CLI, the `0002_drop_kv_state.sql` platform-only
  migration, and the `db::repair` module are all deleted. The per-scope
  event dbs (`runtime.db` / `account.db` / `spaces/<id>/space.db`) and
  the shared tolerant migration runner are unchanged. Skill vars
  continue to read/write `kv.db` (the canonical key-value store) — no
  behavior change.
- **Triggers → `space.db`.** Trigger collection actions were bound to
  `data.db` (the dead single-content-store); they now bind to the
  per-scope `spaces/<id>/space.db` via the canonical
  `SpaceContext::for_space` construction. This greens the two
  `triggers_engine_dispatch_*` integration tests that previously read
  the inserted item back from a different file.