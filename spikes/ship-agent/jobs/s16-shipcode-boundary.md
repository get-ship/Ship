# Job S16 boundary analysis — `ship-code` (spawned-harness) vs shared agent domain

> Status: **read-only map.** No code moved, no renames. This is the
> S16.0 gap-analysis pass that `spikes/ship-agent/jobs/s16-graduate.md` §3
> calls for. Goal: draw the box around `ship-code` so the spike's
> embedded in-process runtime (rig + deno_core) can land beside the
> existing spawned-harness dispatch in `ship-agents`, sharing one
> domain layer.
>
> Spike baseline confirmed: `cargo test --manifest-path spikes/ship-agent/Cargo.toml` → **130 passed** (clean build, 12 dead-code warnings, unrelated).

## 0. North star (one paragraph)

`ship-agents` is the **L1 agent domain** — one crate, one set of types
(`AgentSpec`, `AgentProfile`, `Permissions`, `Rule`, `Skill`,
`McpServerConfig`, `Session`), one disk door (`FilesService`), and one
projection table (`agent_sessions`). It already hosts **two execution
models**: the legacy *spawned-harness* one (drives claude/codex/cursor
through `runtime::executors`) and the bare *agent-session* dispatch
(also via `runtime::executors`). The spike adds a third — the
*embedded* in-process runtime (rig + deno_core + scoped SDK). All three
share the domain; **none** of them is the boundary. The boundary is
**ship-code = the spawned-harness executor stack**: everything
`runtime::executors::{ProcessExecutor, ServerExecutor, TmuxWindowExecutor,
RoutingExecutor, pty_actor_handle, pty_executor_handle_for_test}` plus
its L1 lifters (`runtime::{adapters, agent_lifecycle, helpers,
run_dispatch}`) plus `ports::AgentPorts` and the actor that drives it
(`actor::start_agents`/`handle_agents_event`, `dispatch`,
`cancel`). Domain — def CRUD, sessions, skills, permissions, rules,
mcp-import, hooks, compile, agent config, view_state — stays in the
shared box.

## 1. Classification table — `crates/core/ship-platform/crates/ship-agents/src/`

Every module/file under that tree. Paths are relative to the crate root.
`★` = unambiguous ship-code. `◇` = unambiguous shared-domain. `?` = ambiguous, see §6.

### 1.1 Top-level (`src/`)

| Path | Classification | Why |
|---|---|---|
| `src/lib.rs` | mixed (re-exports) | Re-exports `start_agents`, `handle_agent_session_started`, `handle_agent_cancelled`, `AgentPorts`. The re-exports name the ship-code public surface; the module list exposes the domain too. |
| `src/active_agent.rs` | ◇ shared-domain | Reads `project_compile_state` projection + `ship_config::project::AgentProfile`. Pure domain lookup. |
| `src/actor.rs` | ★ ship-code | `service.agents` actor — sole handler of `agent.session.{started,cancelled}`. Drives the dispatch atom. |
| `src/cancel.rs` | ★ ship-code | `agent.session.cancelled` → stop handle + teardown worktree + emit `process.killed`. |
| `src/dispatch.rs` | ★ ship-code | The bare agent-session dispatch primitive (`handle_agent_session_started`). Skips pipeline-composed sessions; spawns the harness through `AgentPorts`. |
| `src/ports.rs` | ★ ship-code | `AgentPorts` — the port bundle the agent-session actor injects (isolation / executor / platform / registry / spawn-policy / projection-reader / event_dedup). |
| `src/testutil.rs` | shared (test-only) | Inert helper for integration tests. |
| `src/agent_domain/` | ◇ shared-domain (entire subtree) | `config`, `harnesses`, `permissions`, `rule`, `skill`, `skill_paths`, `skill_vars`, `import/`, `tests/`. Pure config + CRUD. The `harnesses::HARNESSES` table is the L1 mirror of `compiler::compile::harness::HARNESSES` — they coexist intentionally; `ship-agents::agent_domain::harnesses` is consumed by `services/compile` (loader), `services/skills` (catalog), and the runtime-vars resolver. **Not** spawn-time. |
| `src/hooks/` | ◇ shared-domain | Hooks engine + tool gate (Layer 2). Pure logic. Lives on the harness-side hook invocation (`ship hook`); *not* the spawned-process API. |
| `src/runtime/` | ★ ship-code (entire subtree) | The agent-run assembly. See §1.2. |
| `src/services/` | ◇ shared-domain (entire subtree) | Domain services. See §1.3. |
| `src/sessions/` | ◇ shared-domain (entire subtree) | Runtime-scoped session records, NOT the spawned session itself. See §1.4. |

### 1.2 `src/runtime/`

| Path | Classification | Why |
|---|---|---|
| `src/runtime/mod.rs` | ★ ship-code | Lifts executor assembly out of L0; owns the `runtime::executors` surface. |
| `src/runtime/helpers.rs` | ★ ship-code | Worktree / tmux / branch / `compile_agent_config` subprocess helpers. Process-shaped. |
| `src/runtime/agent_lifecycle.rs` | ★ ship-code | `graceful_agent_exit` — sends harness exit cmd via tmux, watches for prompt. Process-shaped. |
| `src/runtime/adapters.rs` | ★ ship-code | `GitWorktreeIsolation` — git worktree prep, `write_phase_spec_from_context` (pipeline-runtime call), `compile_agent_config`. The composes-pipeline-runtime dep is the cycle wall (see §7). |
| `src/runtime/run_dispatch.rs` | ✱ **NOT in this crate** | Lives at `apps/shipd/src/dispatch/run_dispatch.rs` — composition root that needs `ship-pipeline-runtime`, which depends on `ship-agents`. Kept in `apps/shipd` to break the cycle. |
| `src/runtime/executors/mod.rs` | ★ ship-code | Module wiring + `pty_actor_handle`, `pty_executor_handle_for_test` re-exports. |
| `src/runtime/executors/process.rs` | ★ ship-code | `ProcessExecutor` — spawns harnesses as PTY-backed child processes via `ship_substrate::supervisor::spawn_actor`. The default spawned-harness path. |
| `src/runtime/executors/tmux_window.rs` | ★ ship-code | `TmuxWindowExecutor` — interactive per-pipeline-step tmux windows. |
| `src/runtime/executors/server.rs` | ★ ship-code | `ServerExecutor` — delegates to codex app-server / opencode serve (scaffolding; per-harness bindings land later). |
| `src/runtime/executors/routing.rs` | ★ ship-code | `RoutingExecutor` — per-job selection (Server / Tmux / Process) by `HarnessCapabilities` + `headless` flag. |
| `src/runtime/executors/tests_server.rs` | ★ ship-code | Tests for `ServerExecutor` translate_events etc. |
| `src/runtime/executors/mint_token_tests.rs` | ★ ship-code | Tests for per-process scoped token mint on the spawn path. |
| `src/runtime/executors/tmux_window_tests.rs` | ★ ship-code | Tests for `TmuxWindowExecutor` tmux lifecycle. |

### 1.3 `src/services/`

| Path | Classification | Why |
|---|---|---|
| `src/services/mod.rs` | ◇ shared-domain | `data_op_examples` fixture aggregator. |
| `src/services/agents/` | ◇ shared-domain | Profile CRUD + view reads + orchestration RPC wiring + watch. See `§1.3.1`. |
| `src/services/compile/` | ◇ shared-domain | The L1 native compile wrapper around the `compiler` crate. See `§1.3.2`. |
| `src/services/mcp/` | ◇ shared-domain | MCP server-entry CRUD over `mcp.jsonc`. |
| `src/services/permissions/` | ◇ shared-domain | Permission preset CRUD + tool-pattern catalog. |
| `src/services/rules/` | ◇ shared-domain | Rule CRUD. |
| `src/services/skills/` | ◇ shared-domain | Skill CRUD + frontmatter parse + view discovery. |
| `src/services/sessions/` | ◇ shared-domain | **PTY session registry + supervisor-primitive consumers.** The name is misleading — it is the harness-session sidecar around `ActorHandle`. See `§1.3.3`. |
| `src/services/*/wire.rs` | ◇ shared-domain | ConnectRPC wire layer for each domain. Pure marshalling. |
| `src/services/*/wire_types.rs` | ◇ shared-domain | Named wire shapes (replaces anonymous `json!({...})`). |
| `src/services/*/state.rs` | ◇ shared-domain | Pure file-tree state layer. |
| `src/services/*/service.rs` | ◇ shared-domain | Method-oriented domain trait + impl. |
| `src/services/*/view_state.rs` | ◇ shared-domain | View-shaped read projections. |
| `src/services/*/fixtures.rs` | shared (test-only) | Fixture data for the L1 service surface. |

#### 1.3.1 `src/services/agents/`

| Path | Classification | Why |
|---|---|---|
| `service.rs` | ◇ | `AgentsService` trait + `AgentsServiceImpl` (the L1 native impl). Domain CRUD. |
| `state.rs` | ◇ | Pure agent-def tree state via `FilesService`. |
| `create.rs` | ◇ | Templates + presets + harnesses for the create-wizard. |
| `view_state.rs` | ◇ | Agent discovery helpers (project + user scope). |
| `settings_view.rs` | ◇ | `agents.settings(id)` panel payload. |
| `wire.rs` | ◇ | `agents.wire(id)` canvas payload. |
| `wire_types.rs` | ◇ | Typed wire shapes. |
| `watch.rs` | ◇ | Watch prefilter + scope keys (reactive ALL/AGENT). |
| `annotation.rs` | ◇ | Annotation domain (validates, persists, emits). |
| `feedback.rs` | ◇ | Feedback domain (validates, rate-limits, emits). |
| `dispatch_request.rs` | ? | `DispatchRequest` wire type + `validate_axes` + `billing_warning_for_axes`. The wire type is **domain** (request shape); the validators cross the line because they encode "print mode × claude is billable at API rates." The handler that calls `dispatch_agent_session_core` lives in `apps/shipd/src/routes/runtime_api/dispatch/`, and `dispatch_agent_session_core` is in `dispatch_core.rs` (★). **Recommendation: keep `dispatch_request.rs` in shared-domain; the request shape is part of the surface that BOTH the spawned-harness path AND the embedded path will accept.** |
| `dispatch_resume.rs` | ? | `resolve_resume` + `load_and_validate_session_file` — reads `ship_harnesses::harnesses::session_file::SessionFile` and verifies the harness's `build_resume_args`. The "harness-specific resume id" notion is **spawned-harness** by definition (deno_core scripts don't have a resume CLI shape). **Recommendation: ship-code.** It currently lives in `services/agents/dispatch_resume.rs` purely as a co-location convenience. |
| `dispatch_core.rs` | ★ | `dispatch_agent_session_core` + `AgentDispatchPort` — emits `agent.session.started` envelope that the spawned-harness path consumes. Pure message construction; the actual spawn is `src/dispatch.rs`. **The split is clean: this is "produce the message"; `src/dispatch.rs` is "act on it."** Embedding this in shared-domain would force the embedded runtime to skip the message. **Recommendation: keep `dispatch_core.rs` in shared-domain; it's the "one canonical dispatch" entry both models call. The cross-boundary is at `src/dispatch.rs` and `src/runtime/executors::*`.** |
| `rpc_wire.rs` | ◇ | ConnectRPC wire for the canonical 23-method `AgentsService`. Includes all of: reads, writes, watch, dispatch/stop/steer/grant (closures injected by shipd), post-feedback, post-annotation. The wire layer is purely marshalling. **The closure types it injects (`DispatchFn`/`StopFn`/`SteerFn`/`GrantFn`) are the seam — the closures hold the ship-code behavior; the wire holds the domain.** |

#### 1.3.2 `src/services/compile/`

| Path | Classification | Why |
|---|---|---|
| `mod.rs` | ◇ | Native compile pipeline entry (`compile_project`). Pure transform over `compiler` crate. |
| `loader.rs` | ◇ | `.ship/` → `compiler::ProjectLibrary` loader. |
| `skill_parser.rs` | ◇ | SKILL.md frontmatter parse + embedded `skills/sdk/` resolution. |
| `mcp_allowlist.rs` | ◇ | Per-harness MCP pre-approval (`mcp__ship__*` to claude/cursor allowlist). |
| `permissions.rs` | ◇ | Agent permission merge (floor + preset + per-agent). |
| `projection.rs` | ◇ | `project_compile_state` projection (materializes `compile.completed`). |
| `agent.rs` | ◇ | Agent profile → library merge. |
| `tests.rs` | shared (test-only) | |

#### 1.3.3 `src/services/sessions/` — name trap

The module is called "sessions" but it is **not** the lifecycle the spike's
S9 built. It is the L1 PTY-session registry wrapping the L0
`ship_substrate::supervisor::ActorHandle` (the unified spawn primitive
after the supervisor split). It is the harness-session sidecar, **used
by both spawned-harness AND embedded** — the embedded path also needs
to expose a shell attach (even if just for `tools/attach` later). The
shape: `actor_id -> (ActorHandle, Arc<Mutex<SessionMeta>>)`.

| Path | Classification | Why |
|---|---|---|
| `mod.rs` | ◇ | Module wiring. |
| `types.rs` | ◇ | `SessionId`, `SessionKind`, `SessionStatus`, `PtySession`, `SessionMeta`, `SpawnRequest`. The L0 supervisor primitive plus a session-flavored sidecar. **The data type is shared; the spawn path (submodule) is the seam.** |
| `registry.rs` | ◇ | Legacy in-memory `SessionRegistry` for per-turn harnesses. Being retired. |
| `actor_registry.rs` | ◇ | Destination `ActorRegistry` keyed on `ActorHandle::actor_id`. |
| `projection.rs` | ◇ | `agent_sessions` SQL projection over `agent.session.*` events. **RUNTIME-scoped; both execution models project here.** |
| `lifecycle.rs` | ? | `spawn_agent` / `send_agent_message` / `emit_actor_created_for` / `sweep_idle` / `stop` / `load_agent_rules`. The `spawn_agent` body uses `ship_substrate::supervisor::spawn_actor` to bring up a long-lived harness session. The shape is **shared for the session sidecar**; the *spawn* of the harness is the ship-code call. **Recommendation: keep in shared-domain but tag the spawn-through-`spawn_actor` block with a comment naming the ship-code seam.** The path exists to support any execution model; only the leaf fn calls the supervisor. |
| `agent_actor.rs` | ? | `spawn_or_attach` (long-lived harness) + `run_headless_turn` (per-turn harness through Headless transport) + the auto-dismiss-claude-menu reader middleware. **Spawned-harness by construction — every `spawn_actor` here is a PTY child process.** **Recommendation: ship-code.** This is the clearest "embedded would never call this" boundary: there is no `ActorHandle` to mint for an in-process JS turn. |
| `shell_actor.rs` | ◇ | `spawn_shell` — wraps `spawn_actor` with `ActorKind::Shell`. **Used by embedded too** (the embedded runtime's `bash(...)` op would not go through this; but a studio-style "open a shell pane" feature might). Keep shared. |

### 1.4 `src/sessions/` (the OTHER sessions module)

This is the C8b-folded S9 session module — RUNTIME-scoped session
records (analytics over actor activity, not the harness session
itself). Confusingly coexists with `services/sessions/`.

| Path | Classification | Why |
|---|---|---|
| `mod.rs` | ◇ | Module wiring. |
| `session.rs` | ◇ | Hydration + list/get/progress + artifacts dir. Pure data. |
| `types_session.rs` | ◇ | `Session`, `SessionLiveness`, `SessionStatus`, `EndSessionRequest`. |
| `helpers.rs` | ◇ | Date/branch/space-id parsing. |
| `session_lifecycle.rs` | ◇ | `start_session` / `end_session` — calls `compile_session_context` (compile service) and persists `SessionStarted`/`Ended` to the sessions DB. |
| `compile.rs` | ◇ | `resolve_session_agent_config` / `resolve_session_harnesses` / `resolve_session_context_root` / `compile_session_context` — bridges to `agent_domain::config::resolve_harness_settings_with_agent_override` and the compile service. |
| `tests*` | shared (test-only) | |

## 2. Cross-boundary dependency edges

The box only matters where it gets drawn cleanly. Below: the exact
types/functions/modules whose names or values cross the
`ship-code` ↔ `shared-domain` line in *both* directions. Each is
followed by a "what it actually means" gloss.

### 2.1 shared-domain → ship-code (domain reaches into spawn machinery)

| Cross | Where | What it means |
|---|---|---|
| `services::compile::agent::apply_agent_to_library` indirectly reaches `services::compile::permissions::FLOOR_DENIES` and pulls `ProfilePermissions` | `services/compile/agent.rs:32-33` | Domain pure, no ship-code. (Listed for completeness — this is fine.) |
| `services::sessions::lifecycle::spawn_agent` calls `ship_substrate::supervisor::spawn_actor` | `services/sessions/lifecycle.rs:152` | Spawned-harness. The fn is named `spawn_agent` and the body uses `ActorKind::Harness` / `TransportKind::Pty`. **This is the seam.** |
| `services::sessions::agent_actor::spawn_or_attach` calls `spawn_actor` with `Harness` kind | `services/sessions/agent_actor.rs:44-134` | Spawned-harness. |
| `services::sessions::agent_actor::run_headless_turn` calls `spawn_actor` with `Harness::Headless` kind | `services/sessions/agent_actor.rs:265-377` | Spawned-harness (per-turn child). |
| `services::sessions::shell_actor::spawn_shell` calls `spawn_actor` with `Shell` kind | `services/sessions/shell_actor.rs:44` | Shared — shell sessions are useful for both. |
| `services::agents::dispatch_resume::resolve_resume` reads `ship_harnesses::harnesses::session_file::SessionFile` + calls `adapter.build_resume_args` | `services/agents/dispatch_resume.rs:31-60` | Spawned-harness only — "resume" is a CLI shape. |
| `services::agents::dispatch_request::billing_warning_for_axes` keys on `claude -p` billing | `services/agents/dispatch_request.rs:144-159` | Billed-by-CLI; embedded doesn't bill per-CLI. Spawned-harness. |
| `services::agents::rpc_wire::GrantFn` closure type | `services/agents/rpc_wire.rs:71-75` | The grant semantics (live-session upgrade + baseline persist) are domain. The closure is injected by shipd. **Domain signature, ship-code caller.** |

### 2.2 ship-code → shared-domain (spawn code reaches into domain)

| Cross | Where | What it means |
|---|---|---|
| `dispatch::handle_agent_session_started` reads `agent_session.started` payload (an `AgentSessionStartedPayload` whose `agent` field names an agent) | `dispatch.rs:44` | The domain `agent` is just a string id. Fine. |
| `runtime::adapters::GitWorktreeIsolation::prepare` calls `ship_agents::runtime::helpers::compile_agent_config` and (via shipd) `ship_pipeline_runtime::orchestrator::isolation::write_phase_spec_from_context` | `runtime/adapters.rs:24-48` | The isolation impl reaches back into the L1 helpers (cycle, see §7) and the pipeline-runtime isolation helper. **Both are L1+; the box stays above.** |
| `runtime::executors::process::build_headless_with_default_prompt` calls `adapter_for_or_default` (from `ship_harnesses`) | `runtime/executors/process.rs:35, 47-67` | Pulls harness metadata. Domain coupling but spawn-time. **Stays ship-code.** |
| `runtime::executors::process::resolve_srt_entry` reads `@anthropic-ai/sandbox-runtime` install path | `runtime/executors/process.rs:145-150` | Hard-codes the Anthropic-srt backend path. Spawned-harness specific. |
| `runtime::executors::routing::RoutingExecutor::spawn` resolves `adapter_for(harness)` + `caps().server_mode` | `runtime/executors/routing.rs` (whole file) | The whole routing decision is about harnesses. **Stays ship-code.** |
| `runtime::agent_lifecycle::graceful_agent_exit` calls `adapter_for_or_default(harness).exit_command()` | `runtime/agent_lifecycle.rs:7-12` | Send-exit-via-tmux. Spawned-harness. |
| `runtime::helpers::compile_agent_config` shells out to invoke the compile CLI | `runtime/helpers.rs:63-95` | Compiles agent config into a worktree. Used by both dispatch paths — the compile output is *content* the harness reads, so it's domain, but the subprocess invocation is ship-code. **Subtle: the OUTPUT of the compile service is domain; calling it from a subprocess is ship-code.** This is the right home for now. |
| `cancel::handle_agent_cancelled` calls `ship_runner::stop_by_slug` and `ports.executor.stop` | `cancel.rs:51-74` | The L0 teardown core. Spawned-harness. |
| `runtime::run_dispatch.rs` (in `apps/shipd/src/dispatch/`) constructs `Arc<RoutingExecutor>` + `Arc<ProcessExecutor>` + `Arc<TmuxWindowExecutor>` + `Arc<ServerExecutor>` and feeds them to `OrchestratorContext` and `AgentPorts` | `apps/shipd/src/dispatch/run_dispatch.rs:46-132` | Composition root. **This is the place the box leaves `ship-agents` for `apps/shipd` — and is where the embedded runtime would enter the same composition.** |

### 2.3 Domain → runtime — hard rule currently in the code

`runtime::adapters` and `runtime::run_dispatch` (in shipd) are the
*only* places the L1 helpers are bound to the L0 ports. The compile
service calls into the `compiler` crate, but the **executor assembly**
is gated at `apps/shipd/src/dispatch/{adapters,platform,run_dispatch,
orphan_cleanup}.rs`. `runtime::adapters` is the L1 self-loop that
exists because the runner needs `IsolationStrategy` impls visible at
L1; if a submodule split (see §7) moves the spawned-harness executor
stack into `ship_code::runtime`, the `adapters` impl can move with
it without an L1↔L0 cycle.

## 3. Crates fully in the `ship-code` box

### 3.1 `crates/core/ship-platform/crates/ship-harnesses/` — ★ fully in box

- `Cargo.toml` depends on `ship_runner` (L0) + `ship_substrate` (L0). No platform dep.
- `src/lib.rs`: re-exports `HarnessAdapter` / `HarnessCapabilities` / `HarnessEvent` / `PushMechanism` / `ResumeError` from `ship_runner::harness_adapter` (where the trait lives). The trait, the registry, and the concrete adapters (`claude`, `codex`, `codex_session`, `cursor`, `opencode`, `opencode_session`, `opencode_session_sse`, `pi`, `session_file`) all describe **spawned-harness CLIs** — argv construction, stream-json parsing, resume CLI shape. None of this is meaningful for an embedded model.
- Consumers: `ship-agents::runtime::executors::process`, `ship-agents::runtime::executors::routing`, `ship-agents::runtime::agent_lifecycle`, `ship-agents::services::agents::dispatch_resume`, `apps/shipd/src/sessions/api/messages.rs`, `apps/shipd/src/supervisor/tests_executor.rs`, `apps/shipd/src/routes/runtime_api/dispatch/tests.rs`. **All ship-code consumers.** (Domain consumers that read the *list* of supported harness ids go through `ship-agents::agent_domain::harnesses::supported_harness_ids`, which re-derives from the registry — keeping the registry itself a ship-code concern.)

### 3.2 `crates/core/ship-isolation/` — ★ fully in box

- `Cargo.toml` depends on `ship_substrate` only. No platform dep.
- `src/lib.rs` re-exports the L0 substrate isolation abstraction (`Isolator`, `SandboxHandle`, `SandboxPolicy`, `NetworkPolicy`, `FsMount`, `MountMode`, `ResourceLimits`) and exposes the concrete backends (`backends/`: `Docker`, `Podman`, `Youki`, `Sidecar`, `Firecracker`, `Lima`, `AppleContainer`), the sidecar binary, the network proxy, the capability probe, the tier/strategy/recommender, and the runtime (`runtime::SidecarActorRuntime`).
- The whole crate is the isolation decision + sandbox backends. The only **shared-domain consumer** is `runtime::executors::process::write_srt_settings` (`runtime/executors/process.rs:158-175`), which calls `ship_isolation::backends::render_settings` to render the JSON schema the Anthropic srt wraps. That call is itself inside a spawned-harness path (the harness is wrapped in srt before launch). **No domain calls.**
- Consumers across the tree: `ship-agents::runtime::executors::process::*` (heavy: `PreparedSandbox`, `resolve_sidecar_bin`, `write_srt_settings`, srt setting render), `apps/ship-cli/src/isolation_cmd.rs` (operational CLI), `apps/shipd/src/routes/sandbox_api_tests.rs` (sandbox API surface), `apps/shipd/src/dispatch/adapters.rs` (no — that imports `ship_pipeline_runtime::orchestrator::isolation::write_phase_spec_from_context`, not `ship_isolation` directly; the isolation is wired through the port). `ship-isolation` is also the home of the `ship-sidecar` binary (`crates/core/ship-isolation/bin/ship-sidecar/`), which is a spawned-PID by definition.

### 3.3 `crates/core/ship-substrate/crates/ship-supervisor/` — ★ fully in box

- `Cargo.toml` depends on `ship_substrate` only. No platform dep.
- `src/lib.rs`: after the supervisor split, this crate owns ONLY the OS-level pieces — `process_events` (re-exports of process event payloads) and `terminal_launcher` (spawn a new macOS Terminal tab — interactive attach for the user's tmux session). It re-exports `ship_substrate::services::process_registry::graceful_shutdown`.
- The supervisor primitive itself (`spawn_actor`, `ActorHandle`, `ActorKind`, `ActorProfile`, `TransportKind`, `Pty*`, signals, wait) lives one level down in `ship_substrate::supervisor`. The L0 `ship-substrate` crate is **shared**, not ship-code — it owns the OS-level fork/exec/PTY/signal/wait mechanics that any process-shaped execution model can use.
- Consumers: `apps/shipd` (re-exports as `crate::supervisor` via `lib.rs:65`), `ship-agents::runtime::executors::process` (heavy: `spawn_actor`, `ActorHandle`, `ActorKind`, `ActorProfile`, `TransportKind`), `ship-agents::services::sessions::{agent_actor, shell_actor, lifecycle}`. All ship-code consumers. The re-export through `apps/shipd::supervisor` is a facadeless convenience for shipd tests (`apps/shipd/src/supervisor/tests_*`).

### 3.4 What shared-domain depends on from those crates

The "what shared-domain depends on" question is the dangerous one. Verified:

- `ship-agents::hooks::*` — pure logic. Imports `ship_substrate` (port traits, events), `ship_substrate::trash`, no L1 stuff.
- `ship-agents::agent_domain::*` — `ship_config::project`, `ship_harnesses::harness_adapter` (only for `supported_harness_ids` / `is_supported_harness` / `adapter_for` in `import/mcp.rs`, `import/skills.rs`, `import/permissions.rs`, `harnesses::require_harness`).
- `ship-agents::services::*` — `ship_files` (L1), `ship_ops_rpc` (L1 generated stubs), `connectrpc`, `async-trait`, `serde`. No ship-isolation / ship-harnesses / ship-supervisor imports in `services/`.
- `ship-agents::sessions::*` — `ship_substrate::db`, `ship_substrate::events`, `ship_config::project`. No L1 ship-code imports.
- `ship-agents::services::sessions::lifecycle::spawn_agent` and `services::sessions::agent_actor::spawn_or_attach` and `services::sessions::shell_actor::spawn_shell` are the **only** domain-tree files that import `ship_substrate::supervisor`. They sit inside `services/sessions/` and break the rule that domain ≠ process. **These are the seam candidates for moving into `ship-code::sessions_runtime` (§7).**

**Net:** the three crates (`ship-harnesses`, `ship-isolation`, `ship-supervisor`) ARE spawned-only. Shared-domain only touches them via the explicit seams named in §2.1.

## 4. Spawned RPC surface

`proto/ship/ops/v1/agents.proto` is the **canonical agent surface** (collapsed from historic split per agents-arc E4/E6, see the proto file's leading comment). All twenty-three `AgentsService` methods below come from that single file (`agents.proto:337-492`); the `RuntimeService` is from `proto/ship/ops/v1/runtime.proto`.

### 4.1 `AgentsService` (23 methods)

| Method | Tier | Capability | Classification | Notes |
|---|---|---|---|---|
| `List` / `Summaries` / `Detail` / `Catalogs` / `Get` / `Wire` / `Settings` / `Watch` / `Templates` / `PermissionPresets` / `Harnesses` / `CreateCtx` | C0 | `ship.agents.read` | ◇ domain | Agent-DEFS reads. |
| `Sessions` / `SessionsLive` | C0 | `ship.sessions.read` | ◇ domain | RUNTIME-scoped session reads (the C8b fold; bare AND composed project here). |
| `Create` / `Put` / `Patch` / `Delete` / `Rename` / `Duplicate` / `MoveScope` | C2 | `ship.agents.write` | ◇ domain | Profile writes. |
| `Dispatch` | C2 | `ship.runtime.dispatch` | ★ ship-code | "Spawns + controls another agent — orchestration." The handler emits `agent.session.started`; the ship-code `dispatch.rs` picks it up. |
| `Stop` | C2 | `ship.agents.write` | ★ ship-code | Routes through shipd-injected `StopFn` → `cancel.rs` + worktree teardown. |
| `Steer` | C2 | `ship.agents.write` | ★ ship-code | Routes through shipd-injected `SteerFn` → `tmux send-keys` / `actor.handle.send`. |
| `Grant` | C3 | `ship.agents.grant` | ◇ domain (logic) / ★ ship-code (C3 gate) | Two-effect domain logic (live-session upgrade + baseline persist) — domain. C3 gate (operator-only) is shipd's `gate::decide` closure. The fn signature carries the gate check; the body is domain. |
| `PostFeedback` | C1 | `ship.feedback.write` | ◇ domain | Studio canvas feedback. |
| `PostAnnotation` | C1 | `ship.annotations.write` | ◇ domain | Canvas annotation. |

**Cleanest cut:** the `Dispatch`/`Stop`/`Steer` RPCs are the spawned-harness control surface; the rest is domain. Grant is interesting — its **body** is domain (it's a write to `provisioned_grants` projection + live-session upgrade), but its **gate** is operator-only and its **callers** are the spawned-harness session handles. The `GrantFn` closure type in `rpc_wire.rs` is the seam.

The single proto comment that makes this unambiguous (`agents.proto:13-21`):
> "Two scopes meet here, deliberately kept distinct:
> - agent DEFS — space/account-scoped authored config (List/Detail/Get/… + the CRUD writes). Reads are C0; writes are C2 gated ops that write the source file.
> - sessions — RUNTIME-scoped runtime-instance reads (#283). Distinct from def scope; never conflated."

### 4.2 `RuntimeService` (1 method, `runtime.proto`)

| Method | Tier | Capability | Classification | Notes |
|---|---|---|---|---|
| `ListProcs` | C0 | `ship.runtime.read` | ★ ship-code (read) | Reads the `process.*` projection — every spawned process emits one row. The READ is C0; the WRITES that populate it are `process.spawned` / `process.exited` / `process.killed` events emitted by ship-code (`runtime::executors::process::emit_executor_session_destroyed` etc.). The RPC is the introspection read; the events are the spawn-side. |

There are other `ship.runtime.*` capabilities across the proto package
(`PipelinesService.Dispatch` uses `ship.runtime.dispatch` for
`Dispatch` and `EndTurn`, `ship.runtime.write` for
`UpdateStatus`/`StepDone` — see `pipelines.proto:141-163`). Those are
the **pipeline runtime** — not the agent-session runtime. They share
the namespace (intentionally — pipelines ARE the cross-cutting
orchestrator) but the per-method blast tiers and bodies are
pipeline-shaped, not agent-session-shaped. They sit in the SHARED box
(`ship-pipelines` + `ship-pipeline-runtime`).

### 4.3 The "session bytes" / spawned RPC surface — clarification

The job spec asks about "Dispatch / Stop / SessionBytes / ListProcs". `SessionBytes` is not a method name in the current proto; the equivalent is:

- `Sessions` / `SessionsLive` — list RUNTIME-scoped sessions (C0 reads of the `agent_sessions` projection).
- The **stream-json event log** — surfaced as `actor.{created,stopped,output}` events in substrate (`apps/shipd` routes hand SSE-style streams of these to the studio). The byte stream itself lives in the `ActorHandle::subscribe_output` channel — owned by the supervisor primitive. Not a wire RPC.

`SessionBytes` would more naturally map to "the harness's own byte stream while it's running" — that's a session attach, not a method. In the current code, the closest is `apps/shipd/src/routes/agent_sessions_attach.rs` (WebSocket attach over the PTY master handle). The actor routes for this are `services::sessions::actor_registry::pty_actor_handle` + `apps/shipd/src/routes/agent_sessions_attach_tests.rs`.

## 5. Compiler harness coupling — `crates/core/compiler/src/`

`compiler` is a pure transform crate: `cargo check -p compiler --target aarch64-linux-android` succeeds. It takes a `ResolvedConfig` (input), looks up a `HarnessDescriptor` by id from its in-process `compile::harness::HARNESSES` table, and dispatches via fn pointers on the descriptor. No I/O. No platform dep. No ship-agents dep.

### 5.1 Harness-target-specific vs generic

| Path | Role | Harness-specific? |
|---|---|---|
| `src/lib.rs` | Top-level re-exports. | generic |
| `src/resolve.rs` | `resolve()` + `resolve_library()` + `ResolvedConfig` + `WorkspaceOverrides`. The ResolvedConfig has typed per-harness fields: `claude: ClaudeSettings`, `codex: CodexSettings`, `opencode: OpenCodeSettings`, `cursor: CursorSettings` (`resolve.rs:54-57`). | **per-harness typed fields** (generic mechanism, per-harness payload) |
| `src/compile/mod.rs` | `compile(resolved, harness_id)` dispatcher. The single entry. | generic |
| `src/compile/harness.rs` | `HARNESSES` static table + `HarnessDescriptor` (paths, feature flags, fn pointers: `compile_agent`, `build_mcp`, `build_settings`). | **★ per-harness** — adding a harness is one entry here. |
| `src/compile/claude.rs` | `build_claude_settings_patch`, `build_claude_hooks_patch`, `compile_claude_agent` + `ClaudeSettings`. | **★ claude-only** |
| `src/compile/codex.rs` | `build_codex_config_patch` + `CodexSettings`. | **★ codex-only** |
| `src/compile/cursor.rs` | `build_cursor_hooks_patch` + `inject_ship_lifecycle_hooks` + `translate_to_cursor_permission` + `build_cursor_permissions` + `CursorSettings`. | **★ cursor-only** |
| `src/compile/opencode.rs` | `translate_to_opencode_permission` + `build_opencode_permissions` + `OpenCodeSettings`. | **★ opencode-only** |
| `src/compile/pi.rs` | No-op `compile_pi_agent` + `build_settings` (no-op). | **★ pi-only** (config-light) |
| `src/compile/agents.rs` | `compile_agent_profiles` + per-harness subagent formatters (`compile_claude_agent`, `compile_cursor_agent`, `compile_codex_agent`, `compile_opencode_agent`). | **★ per-harness formatters** (one fn per harness) |
| `src/compile/mcp.rs` | `build_mcp_servers` + `server_entry` — generic MCP entry renderer parameterized on the descriptor's `emit_type_field` / `sse_url_field` / `http_url_field`. | generic over `HarnessDescriptor` |
| `src/compile/skills.rs` | `build_skill_files` + `resolve_event_subscriptions` + `build_runtime_vars` + `format_skill_file` + `parse_runtime_vars` — generic, parameterized on `desc.skills_dir`. | generic over `HarnessDescriptor` |
| `src/compile/plugins.rs` | `build_plugins_manifest` — generic. | generic |
| `src/compile/context.rs` | `build_context_content` (rules + active-agent notice) — generic, parameterized on `desc.context_file`. | generic over `HarnessDescriptor` |
| `src/compile/harness.rs::HARNESSES` | The five-entry static (claude / codex / cursor / opencode / pi). | **★ the heart of the table** |
| `src/decompile/*` | `decompile_claude` / `decompile_codex` / `decompile_cursor` / `decompile_opencode` / `detect_harnesses`. | **★ per-harness** (reverse direction) |
| `src/compile/tests/*` | The 13 per-harness test files. | **★ per-harness** |
| `src/types/*` | The shared type vocabulary: `AgentProfile`, `McpServerConfig`, `Skill`, `Rule`, `Permissions`, `HookConfig`, `PluginEntry`, … | generic |
| `src/schemas.rs` | `HARNESS_SCHEMAS` + `HARNESS_MANAGED_KEYS` + `schema_url`. | **per-harness** (one entry per harness) |
| `src/manifest.rs` / `src/lockfile.rs` / `src/vars.rs` / `src/output.rs` / `src/jsonc.rs` / `src/events.rs` | Generic compile/resolve pipeline plumbing. | generic |

**The per-harness coupling is concentrated in `compile/{claude,codex,cursor,opencode,pi}.rs` and `compile/harness.rs::HARNESSES`. The rest of the crate is generic over `HarnessDescriptor`.** The compiled output for each harness is the exact artifact the spawned-harness `process.rs` reads (`.claude/settings.json`, `.codex/config.toml`, `.cursor/cli.json`, `.opencode/opencode.json`, `AGENTS.md`/`.agents/skills/`). The embedded runtime's "compile output" — the scoped SDK for a deno_core isolate — is **not** this compile crate; it is a separate, sparser surface (the spike's `sdkgen.rs` + `dtsgen.rs`, generated from proto∩grant). **The embedded runtime does not need this compiler.** This is a clean ship-code coupling boundary: the compiler exists to produce the on-disk configs the spawned harnesses read; the embedded runtime's configs come from a different mechanism (proto descriptor set).

## 6. Ambiguities + decisions surfaced

Each ambiguity: state, evidence, recommendation.

### 6.1 Is `services/agents/dispatch_core.rs` shared-domain or ship-code?

`dispatch_agent_session_core` builds the `agent.session.started` envelope and persists it. It does NOT spawn the harness — that's `src/dispatch.rs`. The envelope is the SHARED message format both execution models will consume (the embedded runtime would also produce an `agent.session.started` envelope for telemetry, even if it doesn't fork a child). The kernel-side eventing is shared; the spawn atom is ship-code.

**Recommendation: shared-domain.** The seam is `src/dispatch.rs` (the handler that does the spawn), NOT `dispatch_core.rs` (the producer of the message).

### 6.2 Is `services/agents/dispatch_request.rs` shared or ship-code?

`DispatchRequest` is a request body that any execution model could accept. `validate_axes` and `billing_warning_for_axes` lean spawned-harness (`invocation=print` × `claude` is billable at API rates, an embedded in-process call is not billed per-CLI).

**Recommendation: shared.** Move the billing-warning lookup behind a closure or a `harness_kind` enum so the embedded path can opt out. (Or, simpler: make the warning nullable per-harness. The shape can stay.)

### 6.3 Is `services/agents/dispatch_resume.rs` shared or ship-code?

`resolve_resume` + `load_and_validate_session_file` look up the harness's CLI-specific session id (claude's `sessionId`, codex's `threadId`, etc.) and verify the adapter can build a `--resume` argv. This is **purely spawned-harness** — the embedded runtime does not have a "resume" CLI shape.

**Recommendation: ship-code.** Move to `runtime::dispatch_resume` (or a new `ship_code::dispatch_resume`). The reason it lives in `services/agents/` today is co-location with `dispatch_core.rs`/`dispatch_request.rs`, not because it's domain.

### 6.4 Is `hooks/tool_gate` external-only?

`hooks/tool_gate.rs` is a static decision function over a parsed `PreToolUse` payload (cwd + tool_name + tool_input). It does not name the harness — it operates on the normalized stdin shape. It IS used by both:

- spawned-harness (the `ship hook before-tool` binary invokes it via the harness's `PreToolUse` hook, which the compiler injects via `inject_ship_lifecycle_hooks`).
- embedded — would the embedded runtime need this? Currently no, because the embedded path's gate is blast-tier (`gate.rs` in the spike) and the tool gate is a separate layer-2 envelope-pattern match. **BUT** the embedded path COULD use this if a future capability-skill's tools wanted the same path-pattern enforcement.

**Recommendation: shared-domain.** The function is pure; the question is who calls it. Both can.

### 6.5 Is `services/sessions/agent_actor.rs` shared or ship-code?

The whole file is `spawn_actor` + reader middleware for a long-lived harness session + `run_headless_turn` for per-turn. Every `spawn_actor` call here is `ActorKind::Harness` and a real PTY child. The auto-dismiss-claude-menu state is also harness-specific. **Spawned-harness by construction.**

**Recommendation: ship-code.** This is the clearest "embedded would never call this" boundary.

### 6.6 Is `services/sessions/lifecycle.rs` shared or ship-code?

`spawn_agent` is in here. The module also has `send_agent_message` (which spawns headless per-turn harness) and `sweep_idle` (just clock math on the sidecar) and `load_agent_rules` (reads `.ship/agents/<id>.jsonc` from the worktree — domain). **Mixed: half spawned-harness, half domain.**

**Recommendation: split the file** along the line. `sweep_idle` and `load_agent_rules` stay shared (or move to `services/sessions/harness_idle.rs` and the domain). `spawn_agent` and `send_agent_message` go ship-code.

### 6.7 Is `services/compile/projection.rs` (compile state) shared?

Yes — it's a SQLite projection over `compile.completed` events. The events are emitted by the compile service, which is shared-domain. The projection is just SQL fold. **No ambiguity. Shared.**

### 6.8 Is the `agent_domain::harnesses::HARNESSES` table a copy of `compiler::compile::harness::HARNESSES`?

Yes — there are two parallel `HARNESSES` tables. `agent_domain::harnesses::HARNESSES` is the L1 agent-domain mirror; `compiler::compile::harness::HARNESSES` is the transform-crate authoritative copy. The two are kept in sync by hand. `ship_harnesses::harness_adapter::REGISTRY` is a third table (just the trait-impl pointers). This is a known issue (per `HARNESSES.md` and `HARNESSES.md:1-9`): "Add a harness: one entry here plus the adapter module." There is no single source of truth across the three.

**Recommendation: out of scope for the box.** The three-way split is a separate refactor (would unify the tables behind a single generated registry or a `build.rs`-time include).

### 6.9 Does the `RuntimeService` need to ship-code?

The single method `ListProcs` reads the `process.*` projection. The projection is populated by `process.spawned` / `process.exited` / `process.killed` events, which are emitted by the spawned-harness path. The READ itself is generic. The projection is generic. **The ship-code concern is the events that populate it, not the read.**

**Recommendation: keep `ListProcs` in the `runtime.proto` service but understand the projection is populated by ship-code emitters.** The RPC service is the right home for the read; shipd's `process.*` projection is the right home for the data.

## 7. Proposed first-cut fence — LOWEST-RISK way to draw the box

Goal: make the `ship-code` ↔ `shared-domain` edge visible without
moving any code. Three options, with explicit breakage.

### 7.1 Option A — module boundary inside `ship-agents`

Add a new `src/runtime/` submodule called `ship_code` (or just a
module-level doc fence). The current `runtime::*` would be renamed
`runtime::ship_code::*`; the L0 helpers (`worktrees_dir`,
`compile_agent_config`) stay where they are because they're called by
both ship-code (`adapters::GitWorktreeIsolation`) and shipd's
`apps/shipd/src/dispatch/adapters.rs`. **Pure re-export; no
behavioural change.** The visibility is: anything under
`ship_agents::runtime::ship_code::*` is spawned-harness.

What breaks: nothing compiles-side. Documentation: clear.

What we get: the boundary is visible at the import site
(`ship_agents::runtime::ship_code::executors::ProcessExecutor` vs
`ship_agents::services::agents::dispatch_core`). Reviewers can ask
"is this in the right module?" without checking the file.

### 7.2 Option B — `#[cfg(feature = "ship_code")]` feature on `ship-agents`

Adds a `ship_code` feature. When off (default), `runtime::*` is
gated. When on, full surface. The embedded path would compile
ship-agents WITHOUT the feature, the spawned path with it.

What breaks: a LOT. Every consumer that does
`use ship_agents::runtime::executors::process::ProcessExecutor`
silently breaks when the feature is off. The default feature would
have to be `ship_code` (otherwise shipd doesn't compile). The
embedded-only build would need to drop `ship-agents` from its
Cargo.toml and depend on the smaller pieces directly. Net: this is
the right shape for a *future* split, but it is not zero-risk and
not first-cut.

### 7.3 Option C — move ship-code to its own crate, leave domain in `ship-agents`

`ship_code` crate: `Cargo.toml` deps on `ship-runner` (L0),
`ship-harnesses`, `ship-isolation`, `ship-supervisor` (via substrate),
`ship-pipeline-runtime` (for `OrchestratorContext::spawn_policy`),
`ship-agents` (for `services::agents::dispatch_core` types and
`agent_domain::config` for harness settings). The contents: move
`ship-agents::runtime::*` wholesale + the lifted
`services::agents::dispatch_resume` + the `services::sessions::*
spawned-harness` bits (`agent_actor.rs` + the spawn-half of
`lifecycle.rs`). The crate exposes `start_agents` + the `AgentPorts`
port type.

What breaks: every `use ship_agents::runtime::…` site (5+ files in
`apps/shipd`, tests). `ship-agents` would have a `pub use
ship_code::*` re-export for the back-compat, then shipd migrates
file by file. The `dispatch/run_dispatch.rs` composition root
becomes the entry point that wires the `ship_code` crate into the
pipeline orchestrator.

This is the right end state but the wrong first cut. Use it as the
migration plan for **S16.2 / S16.3**, after S16.0 (this doc) is
signed off and the four §4 questions in `s16-graduate.md` are
answered.

### 7.4 Recommendation: **Option A first.**

A module-level fence in `ship-agents::runtime::ship_code` (with
`pub use` re-exports at the parent module for back-compat). The
embedded spike can land as `ship-agents::runtime::embedded::*` in
parallel. No `Cargo.toml` churn, no `#[cfg]` gates, no broken
compiles. The fence is a docstring + a module path. Reviewers see
the boundary; CI sees green.

After S16.0 sign-off and a few slices, Option C becomes the
right move — but only with the four questions from
`spikes/ship-agent/jobs/s16-graduate.md` §4 answered (gate model,
MCP reconcile, embedded-vs-spawned coexistence, feature vs separate
crate).

## 8. Open questions deferred to S16.0 sign-off

(Repeating from `s16-graduate.md` §4; this doc is the evidence base
for the four answers.)

1. **Gate model.** `tools/tool_gate.rs` (Layer 2) and the spike's
   `gate.rs` (blast tiers) coexist today with no overlap. The fence
   is unchanged either way: tool_gate is shared, the blast-tier
   check is the embedded runtime's concern. If the embedded runtime
   grows to also handle the CLI-shape, the gate needs to coexist
   with the same fn-pointer-style extension the compiler uses. *Not
   blocking S16.0.*
2. **MCP reconcile.** `agent_domain::import/mcp.rs` ingests harness
   config files into `.ship/mcp.jsonc`. The spike projects MCP as a
   single `mcp.call` op. **Both models need the projection**:
   spawned-harness needs `.mcp.json` to wire the stdio child; the
   embedded runtime needs `mcp.call` as an op. The two are
   compatible (one is the source, the other is the runtime
   projection). *Not blocking S16.0.*
3. **Embedded vs spawned-harness coexistence.** Confirmed: both
   belong in `ship-agents` (shared domain, two runtimes). This
   document is the evidence.
4. **Feature boundary.** Recommendation: **module boundary first
   (Option A), then feature gate in a later slice, then separate
   crate last.** The blast-radius tier system already exists
   (`BLAST_RADIUS_C0_SAFE` / `C1_SPACE` / `C2_ELEVATED` /
   `C3_CRITICAL` in `proto/ship/ops/v1/policy.proto`); the
   embedded-vs-spawned choice is downstream of that and should not
   be encoded in the gate.

## 9. Acceptance for S16.0

- [x] Every module under `crates/core/ship-platform/crates/ship-agents/src/` is classified (ship-code / shared-domain / ambiguous).
- [x] Cross-boundary edges are listed with file/line citations.
- [x] `ship-harnesses`, `ship-isolation`, `ship-supervisor` confirmed spawned-only; the only domain-tree consumers are flagged in §3.4.
- [x] `AgentsService` (23 methods) and `RuntimeService` (1 method) classified method-by-method.
- [x] Compiler harness coupling mapped (per-harness files vs generic over `HarnessDescriptor`).
- [x] Ambiguities (8) each have a recommendation.
- [x] First-cut fence proposed (Option A) with breakage called out (none).
- [x] `cargo test --manifest-path spikes/ship-agent/Cargo.toml` → 130 passed.

## 10. References

- `spikes/ship-agent/README.md` — spike SSOT.
- `spikes/ship-agent/jobs/s16-graduate.md` — S16 draft plan this doc is the evidence for.
- `crates/core/ship-platform/crates/ship-agents/src/lib.rs` — the crate module map.
- `crates/core/ship-platform/crates/ship-agents/Cargo.toml` — dependency list (§3.4 verification).
- `crates/core/compiler/HARNESSES.md` — harness support matrix + paths.
- `proto/ship/ops/v1/agents.proto` — `AgentsService` proto.
- `proto/ship/ops/v1/runtime.proto` — `RuntimeService` proto.
- `proto/ship/ops/v1/pipelines.proto` — `PipelinesService` (pipeline runtime, shared).
- `apps/shipd/src/dispatch/{mod,adapters,platform,run_dispatch,orphan_cleanup}.rs` — composition root outside the crate to break the L1 cycle.
