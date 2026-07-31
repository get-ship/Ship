# Changelog

## Unreleased — stack marker rename (job/marker-rename)

The stack manifest filename flips from `ship.config.ts` to
`stack.config.ts`. The unit is the STACK; the previous name carried
the product family. The resolver recognizes only `stack.config.ts`
post-cut; the working file at the repo root remains `ship.config.ts`
until the integrator's coordinated cutover rolls.

- **Stack marker constant flipped.**
  `ship_paths::project::STACK_CONFIG_FILENAME` and the
  resolver-side `SHIP_CONFIG_FILENAME` mirror both carry
  `"stack.config.ts"`. The discovery walker, the space loader's
  `read_ship_config_ts` (now resolves the new name internally), and
  the per-space `service.register_stack` marker check all read
  through the constant — the rename is one literal each, the
  callers roll forward automatically.
- **Compile fixture renamed.** `apps/helmd/tests/compile_fixture/ship.config.ts`
  → `stack.config.ts` via `git mv`; the `compile.rs` test that
  copies it into the temp repo follows.
- **All hard-coded `ship.config.ts` literals replaced.** Doc
  comments, error messages, watcher filters, event payload paths,
  fixture writes, and the `Path::new("ship.config.ts")` example in
  `agent-config-eval` — every constant/string carrying the literal
  flips to `stack.config.ts` (60 files). Gate 1: zero hits in
  `crates/` + `apps/` `*.rs` files.
- **Tests depending on the working file's name carry
  `#[ignore]`.** Three tests that walk the repo root for
  `stack.config.ts` are marked pending the integrator's cutover:
  `space-resolver::tests::real_manifest::repo_manifest_rolls_up_with_all_nouns_present`,
  `space-resolver::tests::fm_b2a_helm_manifest::fm_b2a_real_manifest_no_longer_declares_working_in_helm`,
  `ship-triggers::schema_tests::trigger_schema_is_valid_json_schema` and
  `view_manifest_schema_is_valid_json_schema`,
  `helmd::tests::parity_capture::capture_full_compile_output_against_worktree_root`.
  The ignore messages cite this commit; removing the annotation
  is part of the integrator's cutover.

## Unreleased — heal-tooling (job/heal-tooling)

Claim c-2026-07-11T01-30-00 (destruction-drill). Close the four
integrity-recovery leaks the drill surfaced: PRAGMA integrity_check
doesn't catch missing projection tables; the static allowlist still
named the retired `projects` table; there was no `ship daemon
rebuild` to reach the rebuild path; `QueryItems` against an unknown
collection returned a raw storage error instead of a typed
`NotFound`.

- **`db::integrity` detects MISSING projection tables, not just
  PRAGMA corruption.** The boot path now walks each scope db's event
  log alongside `sqlite_master`; a registered projection whose
  table is absent while events for it exist is drift, classified as
  `ProjectionsCorrupt` (recoverable by rebuild). Includes the static
  per-scope list (new `scope_schema::static_tables`) and the dynamic
  `collection_<id>` / `_schema_registry` / `collection_artifacts`
  family the per-scope DDL intentionally leaves to event-sourced
  self-provisioning. PRAGMA is still the storage-corruption gate;
  every detail row is classified the same way.
- **Stale `projects` table name in the `PRAGMA integrity_check`
  projection allowlist is replaced with `spaces`** (the 2026-07-09
  space cutover renamed the projection; the spaces projection writes
  to the runtime-scoped `spaces` table).
- **`ship daemon rebuild [--dry-run]` is the reachable rebuild path.**
  Resolves the XDG data root via `ShipHome::from_env().data_dir()`,
  runs `check_platform_integrity`, and on `ProjectionsCorrupt` calls
  `db::rebuild::rebuild_projections`. Refuses on `EventsCorrupt`.
  Prints the rebuilt report (`events_replayed`, `projections_rebuilt`,
  `duration_ms`) and re-checks integrity so the operator sees the
  post-rebuild state. Wired into the `ship daemon` subcommand tree
  (matches `Cleanup`'s pattern).
- **`helmd` self-heals drift at boot.** `helmd` mounts collection,
  artifact, and agent wires over per-scope event dbs; a drift there
  means the wires serve empty / broken state. `boot_heal` runs the
  same check + rebuild before any wire mounts and exits 2 on
  uncorrectable corruption (operator must restore from backup).
- **`CollectionService::query_items_*` returns a typed `NotFound`
  on unknown collection, not a raw storage error.** The previous path
  stripped the `collection_` prefix and ran the SQL layer; an
  unknown collection surfaced as `Internal` ("no such table"). Now
  the service resolves the collection id (validating `is_safe_ident`)
  and looks it up in `SchemaRegistry` before issuing the query.
  Rejecting the query without the `collection_` prefix is a typed
  `Validation` — a programming bug, not a storage fault.
- **New destruction-drill test** (`tests_integrity.rs`): drops the
  static Runtime projection table `actors` after writing an event
  for it, proves the integrity check classifies the drop as
  `ProjectionsCorrupt`, runs `rebuild_projections`, and asserts the
  actor row materializes from the log. Also pins the
  PROJECTS→SPACES rename (no `"projects"` in `PROJECTION_TABLE_TAGS`;
  `"spaces"` present) and the empty-log-is-OK case.

## Unreleased — seam-leak fix (job/seam-leakfix)

Ship becomes honestly git/agent-free by default. The eight seam
leaks identified by the seam + workspace censuses are closed: three
harness-driver methods that shell-out to `git` in ungated code,
two L0 mechanism primitives that squared away in `bridge`, two
`dispatch_resume` / `baseline_sha` callers that survived without a
harness-driver path, and the silent-skip dispatch failure mode that
edits into a typed `session.failed`.

### What's in this slice

- **L0 leaf consolidation (`ship_runner::dedup`, `ship_runner::router`).
  **`EventDedup`**, **`terminal_guard`**, **`Admit`**, and **`DISPATCH_GATE`**
  move from `ship_code::runner` back to L0 `ship-runner` so both
  `ship-agents` (`service.agents` actor) and `pipeline-defs`
  (orchestrator) depend on them without a cargo cycle. The closure
  type that `terminal_guard` accepts is now an `Fn(&str) -> Pin<Box<dyn
  Future<Output=bool> + Send>` (per census §4.3 — port-typed to `bool`),
  freeing the gate from a `ProjectionReader` trait import.
  `ship_code::runner` keeps only `exit_handler` / `teardown` /
  `spawn_env` / `invocation` (harness-driver concerns). **Spec target
  diverged from pipeline-defs → ship-runner** because the
  `ship-agents ↔ pipeline-defs` cycle blocks any other L1
  home; the R13 keystone work breaks that cycle. Documented in the
  ship-agents / pipeline-defs README and the runner module
  docs.
- **Domain-free ANSI / git-evidence utilities leave `bridge`.**
  `runner::observe::strip_ansi` → `ship_substrate::text::strip_ansi`
  (new L0 leaf; `strip_ansi_bytes` for byte-buffer callers; the
  `runner/mod.rs::compute_step_invocation` and `log_stream::strip_ansi`
  near-duplicates route through the leaf). `scan_git_evidence`,
  `resolve_head_sha`, and `scan_agent_log_tail` → `pipeline_defs
  ::loader::git_helpers`. New L1 helper `count_commits_ahead` on the
  ship-git leaf replaces the raw `git rev-list --count` shell-out.
- **ship-code has zero direct `git` calls.** `helpers::remove_worktree`
  and `helpers::delete_branch` (the last two `Command::new("git")`
  shell-outs in `bridge`) now delegate to `ship_git::write
  ::{remove_worktree, delete_branch}` (already on the L1 leaf, with
  the job-branch prefix guard preserving the dispatch-path contract).
  `ship-code` gains a direct `ship-git` dep.
- **`AgentDispatchPort::worktrees_dir() -> Option<PathBuf>`.** Harness
  impls return `Some(...)`; the dispatch-only paths (resume-only)
  consume it. The dispatch-only branch + `resolve_resume` + the
  `dispatch_resume` gated behind `bridge`;
  shipd's NoopDispatch / flag-off impls return `None`. shipd's
  `apps/shipd/src/ports/local_paths.rs::worktrees_dir` is the single
  surviving definition (helmd's parallel `helpers::worktrees_dir`
  reads through `ship_settings::worktrees::worktrees_dir` — same answer,
  same env).
- **`resolve_project_head_sha` gates + floki = `baseline_sha: None`.
  The function is `#[cfg(feature = "bridge")]`-gated; the
  `DispatchRequestedPayload.baseline_sha` field reads `None` when
  the flag is off — the intended design (ship-standard is git-free
  by ruling, decisions-2026-07-09 §"Git surface"), NOT a regression.
- **Loud refusal, never silent drop.** `service.agents::actor
  ::handle_dispatch_requested` now emits a typed canonical
  `session.failed` event in two previously-silent cases: (a) a
  floki `dispatch.requested` arrives with `floki_run_fn == None`,
  (b) a coding-agent `dispatch.requested` arrives with
  `bridge` OFF. Both paths feed the canonical_sessions
  projection (Failed status), produce an audit row, and return
  without touching the executor. The boot-time posture announcement
  in `daemon::lifecycle::start_agent_stack` logs an `info` line at
  startup declaring whether harness-driver is wired.
- **shipd default = ship-standard (`default = []`).** shipd's
  default build no longer drags in the harness driver stack —
  `cargo build -p shipd` produces the floki-only ship-standard
  build. Coding-agent dispatch requires `--features bridge`.
  The Justfile recipes (`build`, `build-dev`, `build-release`,
  `install-dev`, `install`, `check-gates`) and `.github/workflows
  /ci.yml` (the `Build shipd` arms) now pass `--features bridge`
  explicitly. `helmd`'s `Cargo.toml` is unchanged (default still
  includes `bridge` per the helix testbed role).

### STOPs carried into the next slice

- **Pre-existing flake:** `orchestrator::tests_terminal_guard
  ::step_observed_ignored_when_job_cancelled` fails on `cargo test
  -p pipeline-defs` today on the parent commit; this slice
  does not regress it (confirmed by `git stash && cargo test
  …` + `cargo test … -- --skip`). The test seeds a terminal event
  and asserts the projection reflects it; a stale global DB /
  cross-test state is the suspected root cause. Tracked outside
  this commit.
- **R13 keystone work** (separate commit) breaks the
  `ship-agents ↔ pipeline-defs` cycle so the L0 mechanism
  primitives can live in the L1 runner crate as the spec
  requested. Today's stop note in ship-runner's `lib.rs` and
  pipeline-defs's README and `orchestrator/router.rs`
  documents the divergence.



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

Two related changes land in this slice. Together they make the `bridge`
feature gate drop the entire third-party-harness driver stack from the
daemon build AND keep the floki-harness path routable. The floki path is
now a feature-independent daemon concern — it lives or dies based on
its own runtime, not the harness-driver feature flag.

### (b) — floki is feature-independent (LANDED)

`shipd` previously built the floki-harness dispatch closure inside a
`#[cfg(feature = "bridge")]` block, so `--no-default-features` left
floki with no run_fn wired (the agent-session actor started but every
floki payload was silently no-op'd). The block sat inside
`run_dispatch::subscribe_job_events`, whose struct literal also swept
the harness-port fields into the same cfg arm.

Fix:
- `crates/core/ship-platform/crates/ship-agents/src/ports.rs`: added
  `AgentPorts::floki_only(run_fn)` constructor, gated to
  `#[cfg(not(feature = "bridge"))]`. The struct's `floki_run_fn`
  field is already ungated; this constructor is the flag-off entry point
  that fills ONLY the floki field and leaves the harness-side fields
  omitted (the literal doesn't name them).
- `apps/shipd/src/lib.rs`: the floki-harness dispatch closure build is
  moved OUTSIDE the `#[cfg(feature = "bridge")]` block. The closure
  captures `Arc<ShipdGateway>` and the kernel, both floki-/L1-clean.
  In the flag-on build, the closure hands to the existing
  `RunManager::init → subscribe_job_events → start_agents` path (the
  actor is started once, with the full port bundle). In the flag-off
  build, a small `start_agents` invocation with a floki-only
  `AgentPorts` keeps the agent-session actor alive and the floki
  branch in `handle_agent_session_started` routable. Non-floki
  sessions in flag-off become no-ops (the harness-driver stack is
  cfg-omitted from `handle_agent_session_started`, by design).
- New unit tests under `cfg(not(feature = "bridge"))` guard
  the constructor: `floki_only_round_trips_closure`,
  `floki_only_none_is_noop`. The existing floki-routing behavior tests
  in `crates/core/ship-platform/crates/ship-agents/tests/behavior.rs`
  (gated to `bridge`) still pass — they assert the harness
  executor is skipped when `floki_run_fn` is wired, which the floki
  branch in `handle_agent_session_started` does regardless of feature
  state.

### (a) — pipeline-defs gate (DEFERRED to R13 dissolution)

`crates/core/ship-substrate/crates/pipeline-defs/Cargo.toml:25` unconditionally pulls
`bridge` (no `optional`, no feature gate). The runtime is the L1
orchestrator that USES `ship_code` as its port surface — `ship_code::dispatch_ports::*`
(SessionContext, LaunchProfile, StepInfo, AgentPlatform, IsolationStrategy,
ProcessRegistryPort, ProjectionReader, SessionExecutor, SpawnPolicy,
ExitSignal, ProcessTreeProbe, TreeSample, ExitCause, Authority, ExecutorHandle)
and `ship_code::runner::*` (build_spawn_env, resolve_step_invocation,
dedup, exit_handler, router::{extract_slug, terminal_guard, Admit},
stop_by_slug, stop_handle, observe::{scan_agent_log_tail, ...}) are
called from ~30 sites across `crates/core/ship-substrate/crates/pipeline-defs/src/orchestrator/`
and the step_monitor / fanout / tests_heartbeat subtrees. Gating
`ship_code` to be optional would require port-typing every call site
and propagating feature gates through the orchestrator to shipd —
the same shape of work that R13's pipeline-runtime dissolution is
already chartered to do. We do not duplicate the R13-shaped work in
this slice.

Net result: `cargo tree -p shipd --no-default-features` still shows
`bridge`, but only via `pipeline-defs`. Once R13 dissolves
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

## Unreleased — XDG cutover (job/xdg-for-real)

The XDG split is now the only production on-disk layout. The
`home_dir().join(".ship")` legacy fallback in `ShipHome::from_env` is
DEAD. Fresh installs on a normal machine (no XDG vars set) land on
`$HOME/.config/ship` / `…/share/ship` / `…/state/ship` /
`~/.cache/ship`; the operator's data moves by hand at land time.

- **`ShipHome::from_env` is now exactly the spec.** Two branches,
  exhaustive: `SHIP_HOME` set → collapsed base; unset → XDG
  resolution. The "only consult XDG when a var is set" condition
  and the `home_dir().join(".ship")` fallback are gone. No third
  branch.
- **`resolve_xdg_roots` is now a thin env wrapper over a pure
  resolver.** `resolve_xdg_roots_from(config, data, state, cache,
  runtime, home)` is the testable surface — every per-root
  env-set / env-unset combination is exercised as a pure-function
  test. The process-env wrapper is one test.
- **`ShipHome::root()` is deleted.** The legacy `~/.ship` analogue
  has no live callers any more. The 5 production call sites
  (helmd agents, helmd collections, ship-cli daemon paths, ship-cli
  macos service registration, ship-cli supervision tests) flip to
  `data_dir()`. All five access the data root for DB openers and
  kernel-router init; that's the correct typed accessor.
- **Tests are behavior tests.** XDG per-root coverage lives in
  pure-function form (no env mutation). The `resolve_xdg_roots()`
  process-env wrapper has one thin test. The isolation-guard
  tests (no SHIP_HOME / no disable, explicit SHIP_HOME pinned to
  real production path) still exercise `from_env` directly — they
  cover the wrapper, not the resolver. A new
  `db_files_resolve_under_data_root` test pins the spec's
  root-mapping contract.
- **`scripts/lint/check-raw-ship-paths.sh` patterns** cover the
  four XDG roots plus the `~/.ship` fossil-defense. The XDG
  cutover retired the legacy branch; the lint keeps the patterns
  to prevent re-introduction.