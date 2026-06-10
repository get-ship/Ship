# Job S16 — orchestration/dispatch entry points: the single port (READ-ONLY, design only)

> Status: **read-only map + design.** No code moved, no renames, no
> module reshuffles. This is the follow-up to the
> `s16-shipcode-boundary.md` keystone (which drew the
> `ship-code` ↔ `shared-domain` line at `runtime::executors::*` + L1
> lifters). It answers a different question: *the dispatch path
> itself — "spawn this / advance that step" — is smeared across
> ~5 layers. What is the one port every entry should route
> through, and where does the embedded runtime plug in?*
>
> Spike baseline: `cargo test --manifest-path
> spikes/ship-agent/Cargo.toml` → 130 passed (unchanged). Boundary
> map commit: `9114fc49d`.

## 0 · North star (one paragraph)

There are **five entry points** that can today "initiate or advance
a dispatch/orchestration":

1. `apps/shipd/src/dispatch/run_dispatch.rs::subscribe_job_events` —
   the composition root that wires the **one** `OrchestratorContext`
   + the `AgentPorts` bundle and starts the pipeline orchestrator
   actor and the `service.agents` actor.
2. `apps/shipd/src/routes/runtime_api/dispatch/mod.rs::
   dispatch_agent_session` and the ConnectRPC twin in
   `ship-agents::services::agents::rpc_wire.rs::dispatch` — both
   call the transport-neutral
   `ship-agents::services::agents::dispatch_core::
   dispatch_agent_session_core`, which builds the bare
   `agent.session.started` envelope.
3. `apps/shipd/src/routes/runtime_api/pipeline_run.rs::run_pipeline`
   + the `pipelines.dispatch` write-closure in
   `apps/shipd/src/ops_rpc.rs:497-543` — both call
   `ship-pipeline-runtime::run_core::run_pipeline_core` →
   `dispatch_one`, which builds the `pipeline.run.started` envelope.
4. `ship-agents::actor::start_agents` +
   `ship-agents::dispatch::handle_agent_session_started` — the
   bare-session actor (S3b carve) that is the **sole** consumer of
   `agent.session.started` and `agent.session.cancelled`.
5. `ship-pipeline-runtime::orchestrator::router::start_orchestrator`
   + `pipeline::advance_pipeline` + `dispatch::
   handle_pipeline_run_started` — the **sole** consumer of
   `pipeline.run.started` (first-step) and `pipeline.step.observed/
   completed` (advance). The router deleted its arms for
   `agent.session.{started, cancelled}` in S3b; it keeps
   `agent.session.{completed, failed}` ONLY for
   `parent_pipeline_run_id != None` to advance composed sessions.

Three of these (1, 4, 5) are the **real** entries — they build or
act on events that flow into the kernel. Two (2, 3) are
HTTP/ConnectRPC adapters. The pass-through adapters are correct —
they exist because the wire types need transport-boundary guards
(MCP-workspace violation, scope gate, blast-tier gate) before
calling the domain core. **All five route through one shared
substrate: `RunnerPorts` + `SpawnPolicy` + `ProjectionReader`.**
The smearing is in the L1 port bundles (two of them:
`OrchestratorContext` and `AgentPorts`) that **compose** that
substrate. The single port to unify on is **`RunnerPorts`** + a
**single new `Dispatcher` trait** that owns the "build a transient
run / advance / cancel" verb set. The embedded runtime enters as
a new `Dispatcher` impl in `ship-agents::runtime::embedded`,
alongside a future `ShipdDispatcher` that the L1 domain functions
delegate to. The L1↔L0 cycle forces the **composition root to
remain at `apps/shipd`** (or any other host that depends on both
`ship-agents` and `ship-pipeline-runtime`); the `Dispatcher`
trait itself is L1 and lives in `ship-agents::runtime` next to
the existing executors.

## 1 · Entry-point inventory

Every place that *initiates or advances* a
dispatch/orchestration. Each row: path → verb → what it does →
what it calls → who calls it → genuine entry or pass-through.

### 1.1 Production entry points

| # | Path (file:line) | Verb | What it does | Calls | Genuine entry? |
|---|---|---|---|---|---|
| 1 | `apps/shipd/src/dispatch/run_dispatch.rs:49-135` `subscribe_job_events` | bootstrap | Constructs the **ONE** `OrchestratorContext` + the `AgentPorts` bundle, starts the pipeline orchestrator actor and the `service.agents` actor. | constructs `Arc<RoutingExecutor>` (L1, `ship_agents::runtime::executors::routing::RoutingExecutor::new`), `Arc<adapters::GitWorktreeIsolation>` (host), `Arc<ShipdPlatform>` (host), `Arc<ConfigSpawnPolicy>` (L1, `ship_pipeline_runtime::orchestrator::spawn_policy::ConfigSpawnPolicy`), `Arc<ProjectionsReader>` (L1, `ship_pipeline_runtime::orchestrator::projection_reader::ProjectionsReader`), then `Arc<OrchestratorContext { … }>` + `Arc<AgentPorts { … }>`. Starts `start_orchestrator` (L0 router) and `start_agents` (L1). | **YES** — the single composition root. The kernel-side event bus is the substrate. |
| 2a | `apps/shipd/src/routes/runtime_api/dispatch/mod.rs:37-84` `dispatch_agent_session` | REST `POST /api/v1/agent_sessions/dispatch` | HTTP transport adapter. MCP-workspace guard at the transport boundary, then `dispatch_agent_session_core(&port, req)` where `port = AgentDispatchAdapter(state)` wraps `state.persist_and_route`. | `ship_agents::services::agents::dispatch_core::dispatch_agent_session_core` (the transport-neutral core). | PASS-THROUGH to (4). |
| 2b | `crates/core/ship-platform/crates/ship-agents/src/services/agents/rpc_wire.rs:624-696` `AgentsWire::dispatch` | ConnectRPC `AgentsService.Dispatch` | Same shape as 2a, different transport. `agent.dispatch` gate at the wire, MCP-workspace guard, then `dispatch_agent_session_core(self.dispatch_port.as_ref(), dreq)`. | `dispatch_agent_session_core` (same as 2a). | PASS-THROUGH to (4). |
| 3a | `apps/shipd/src/routes/runtime_api/pipeline_run.rs:21-130` `run_pipeline` | REST `POST /api/v1/pipelines/run` | HTTP transport adapter. `Scope::CreateRuns` gate, then `run_pipeline_core(&port, req)`. | `ship_pipeline_runtime::run_core::run_pipeline_core` → `dispatch_one`. | PASS-THROUGH to (5). |
| 3b | `apps/shipd/src/ops_rpc.rs:497-543` `pipelines.dispatch` write-closure | ConnectRPC `PipelinesService.Dispatch` | The wire-layer `write_fn` closure (see `crates/ship-pipeline-runtime/src/wire.rs:52-53,203-229`). On `op == "pipelines.dispatch"`, calls `run_pipeline_core` with the same `PipelineDispatchAdapter`. | Same as 3a. | PASS-THROUGH to (5). |
| 3c | `apps/shipd/src/routes/runtime_api/jobs.rs:399` `end_job` | REST stop-hook turn-end | HTTP transport for the harness's Stop hook → `end_turn_core`. Sends `TurnEnded` to the step monitor. | `ship_pipeline_runtime::run_core::end_turn_core` → `TurnSignalPort::signal_turn_ended`. | PASS-THROUGH to the live step monitor. |
| 4 | `crates/core/ship-platform/crates/ship-agents/src/services/agents/dispatch_core.rs:85-194` `dispatch_agent_session_core` | domain core | Transport-neutral core behind 2a/2b. Validates the request, resolves resume state, builds the `agent.session.started` envelope, persists via `AgentDispatchPort::persist_and_route`. | `port.persist_and_route(envelope)` via the `AgentDispatchAdapter` (which delegates to `state.persist_and_route`). | **YES (message-producer)** — the only place bare `agent.session.started` is built. Distinct from (5): this is the producer; (5) is the consumer. |
| 5 | `crates/core/ship-platform/crates/ship-agents/src/actor.rs:38-66` `start_agents` + `dispatch.rs:39-100` `handle_agent_session_started` | domain actor | Subscribes to `agent.session.{started, cancelled}` (the S3b carve). For `started`: detach under the L0 `DISPATCH_GATE`, run isolation (worktree + `compile_agent_config`), spawn harness through `AgentPorts.executor`, register, attach exit handler, emit `agent.session.dispatched`. Skips events with `parent_pipeline_run_id` set (composed sessions; the orchestrator owns their spawn). | `AgentPorts` (the bare-session subset — no `done_flags`/`activity_txs`/`fanout_lane_txs`). | **YES (agent-session-side)** — the only actor that owns bare-session spawn after S3b. |
| 5b | `crates/core/ship-platform/crates/ship-agents/src/cancel.rs:34-99` `handle_agent_cancelled` | domain actor | Sole handler of `agent.session.cancelled`. Stops the handle (L0 `ship_runner::stop_by_slug`), cleans up worktree/tmux/branch, emits `process.killed`. | `ports.executor`, `ports.platform.cleanup_job`, the L0 `stop_by_slug`. | **YES (cancel-side)** — paired with (5). |
| 6 | `crates/ship-pipeline-runtime/src/orchestrator/router.rs:95-285` `handle_orchestrator_event` | router dispatch | Single switch over envelope event type. Routes `pipeline.run.started` to (7), `pipeline.run.completed` / `pipeline.step.completed` to (8), `pipeline.step.observed` to (8) with fanout-lane special case, `pipeline.run.cancelled` to `handle_cancelled`, `pipeline.run.failed` to cleanup, `agent.session.completed/failed` to (8) for composed sessions, `pipeline.step.done_declared` to monitor bookkeeping, `pipeline.run.merged` to cleanup. **Deleted arms**: `agent.session.started/cancelled` (S3b carve). | The handlers named above + `terminal_guard` (L0 dedup + terminal-state guard). | YES (router) — but the router is a **subscriber for kernel events**, not a producer. It advances; it does not initiate. It is the kernel-side consumer of (4), (5), and the L0 dispatch helpers. |
| 7 | `crates/ship-pipeline-runtime/src/orchestrator/dispatch.rs:41-320` `handle_pipeline_run_started` + `dispatch_pipeline_run_internal` | first-step dispatch | Called by 6 when `pipeline.run.started` arrives. Runs the same first-step logic that (8) does for steps 2+: handle `StepType::Run` (pure shell), `StepType::Agent|Review` (worktree + compile + spawn), or a fanout first-step (delegate to (9)). The first-step path is mirrored in the advance path so the two cannot drift (`dispatch.rs:99-109` explicitly cites the same fanout branch as `pipeline.rs:174-238`). | `octx.isolation`, `octx.executor`, `octx.platform`, `octx.registry`, `octx.spawn_policy` — the `RunnerPorts` core. | **YES (first-step-side)** — paired with (8). |
| 8 | `crates/ship-pipeline-runtime/src/orchestrator/pipeline.rs:25-382` `advance_pipeline` (calls `advance_pipeline_inner`) | step advance | Called by 6 when a step terminates (`step.observed` or `step.completed`). Acquires per-job `run_locks` mutex, reloads projection, builds the "completed steps" set, computes `unblocked_steps` (the L0 `Pipeline::unblocked_steps`), dispatches each via `octx.isolation.prepare` → `octx.executor.spawn` (or fanout via (9)). | Same `RunnerPorts` core as (7). | **YES (advancement-side)** — the inner step-dispatch loop. |
| 9 | `crates/ship-pipeline-runtime/src/orchestrator/fanout.rs:39-273` `dispatch_fanout_step` | fanout dispatch | Called by (7) and (8) when a step declares `fanout: FanoutSpec`. Dispatches K lanes concurrently, each with its own worktree, branch, slug, and `StepMonitor`. Registers each lane's `oneshot::Sender<LaneOutcome>` in `octx.fanout_lane_txs` + `FanoutLaneRef` in `octx.fanout_lane_refs`. A background aggregator awaits all/min lanes then calls the pure `aggregate_lane_outcomes(outcomes, min_ok, total)` (`ship-pipelines/src/fanout.rs:31-47`); on Advance, emits a synthetic `pipeline.step.observed` for the parent fanout step so (8) re-runs to dispatch the judge. | Same `RunnerPorts` core. | **NO** — sub-routine of (7)/(8) using the same ports. The synthetic `step.observed` event re-enters through 6, completing the feedback loop. |

### 1.2 Near-entries (pass-throughs that LOOK like entries but are not)

- `ship-pipeline-runtime::run_core::run_pipeline_core`
  (`run_core.rs:125-200`): called by 3a + 3b. Locates the pipeline
  JSONC, builds the `PipelineRunStartedPayload`, delegates to
  `dispatch_one`. **Same shape as (4) but for pipelines.** Calls
  the same `port.persist_and_route` port. Already lives in the
  same module as `dispatch_one` (the transport-neutral pipeline
  core), so the call graph is: 3a/3b → `run_pipeline_core` →
  `dispatch_one` → `port.persist_and_route` (pipeline-side
  `PipelineDispatchPort` trait). One port, one core function.
- `ship-pipeline-runtime::run_core::dispatch_one`
  (`run_core.rs:25-52`): the only function that *constructs* a
  `PipelineRunStartedPayload` for a fresh run. **YES (pipeline
  message-producer)**, but the producer and the consumer are
  colocated in the same crate, so the smearing here is internal.
  Adding it to the "5 entries" tally above gives: (1) composition
  root, (4) bare-session core, (5) bare-session actor, (5b)
  bare-session cancel, (6) router, (7) first-step, (8) advance —
  seven if you count each as a peer. The job spec lists seven
  ("apps/shipd/src/{dispatch, routes/runtime_api/dispatch,
  services/pipelines}, crates/ship-pipeline-runtime/src/
  orchestrator/*, ship-agents::{dispatch, services/agents/
  dispatch_core}, crates/core/ship-substrate/src/services/
  dispatch_ports/*, ship-pipelines (L0 types)"); my count of
  5 + 4 internal peers matches.
- `ship-agents::runtime::run_dispatch.rs:46-132`
  `subscribe_job_events`: **dead code** (orphan from a
  half-completed refactor). Identical body to
  `apps/shipd/src/dispatch/run_dispatch.rs:49-135`, and
  references `super::platform::ShipdPlatform` which **does not
  exist** in this crate (the only `ShipdPlatform` lives in
  `apps/shipd/src/dispatch/platform.rs`). Verified with `rg
  "ship_agents::runtime::run_dispatch" .` — zero callers. This
  file should be deleted but per the job constraint (no code
  moves) we just flag it. **Smeared.**
- `ship-agents::runtime::adapters::GitWorktreeIsolation` is
  *defined* but **not wired AND not even compiled into the
  crate**. The composition root at
  `apps/shipd/src/dispatch/run_dispatch.rs:83` uses the host's
  `apps/shipd::dispatch::adapters::GitWorktreeIsolation` instead.
  Two identical `GitWorktreeIsolation` impls, ~60 lines each. The
  L1 one (`ship-agents::runtime::adapters.rs`) and the
  `ship-agents::runtime::run_dispatch.rs` companion **are not
  declared in `ship-agents::runtime::mod.rs`** — verified with
  `grep "pub mod" crates/core/ship-platform/crates/ship-agents/
  src/runtime/mod.rs` → only `agent_lifecycle`, `executors`,
  `helpers` are wired. The files exist on disk but the compiler
  never sees them. They are preserved (not deleted) so the next
  slice can move them into a `ship_code` crate and re-export
  through the new module tree. **Smeared**, but with a
  deliberate "frozen" seam: the files document what the crate
  WOULD need if `ship-pipeline-runtime` were an explicit dep.
- `ship-agents::services::sessions::lifecycle::spawn_agent` and
  `agent_actor.rs::{spawn_or_attach, run_headless_turn}`: a
  SEPARATE, older session-sidecar spawn path that calls
  `ship_substrate::supervisor::spawn_actor` directly. Distinct
  from (4)/(5); it bypasses the kernel envelope and registers a
  PtySession + an `actor_id` directly. Used by the chat / studio
  pane surface (`apps/shipd/src/sessions/api/`), not by the
  pipeline + agent-session dispatch path. Per
  `s16-shipcode-boundary.md §1.3.3`: spawned-harness by
  construction; the embedded path doesn't need this. **Not a
  second entry into the dispatch path; a different feature
  (chat attach).** Worth noting because the "smeared across
  ~5 layers" worry confuses this with the dispatch path.
- `ship-agents::services::sessions::shell_actor::spawn_shell`:
  shell-session spawn. Same direct `spawn_actor` style; used by
  the chat-attach surface, not dispatch. Same status.

### 1.3 Genuine entry count: **5 producer/initiator**, **4 advancer** (4 distinct from 5)

Producers/initiators (build envelopes that initiate work):
(1) composition root, (2a/2b→4) bare-session REST+RPC producers,
(3a/3b→run_core) pipeline REST+RPC producers, (4) bare-session
core, (5) bare-session consumer. The composition root is not
itself an entry; it's a wiring site that starts the actors which
*are* the entries. So: 4 actors + cores, 2 transport handlers
that pass through. The job spec's "every place that initiates or
advances" includes the advancement path, which adds 3 (router,
first-step, advance) + 1 (fanout). The shape:

```
initiate    → transport (REST/RPC) → domain core → kernel envelope
advance     → kernel envelope → router → first-step|advance → fanout
```

## 2 · Current control-flow graph

Two end-to-end traces. Both render in the same `kernel` event
bus (`ship_substrate::events::KernelRouter`); both consume the
SAME `RunnerPorts` core. **The only divergence is the actor that
owns the envelope**: `service.agents` (5) owns
`agent.session.*`; `service.orchestrator` (6) owns
`pipeline.*` + `agent.session.{completed,failed}` for composed
sessions.

### 2.1 Trace A — bare agent-session dispatch (`ship dispatch` / REST / ConnectRPC)

```
client (CLI / MCP)
  │
  └─ 2a (REST)              POST /api/v1/agent_sessions/dispatch
      │                     handler: apps/shipd/src/routes/runtime_api/dispatch/mod.rs:37
      │                       • MCP-workspace violation check (transport boundary)
      │                       • port = AgentDispatchAdapter(state)
      │                       • call dispatch_agent_session_core(&port, req)
  │  or
  └─ 2b (ConnectRPC)        AgentsService.Dispatch
                            handler: ship-agents::services::agents::rpc_wire.rs:624
                              • (self.gate)(&ctx, "agent.dispatch")   ← C2 gate
                              • port = self.dispatch_port
                              • dispatch_agent_session_core(port.as_ref(), dreq)
                              • map DispatchCoreError → ConnectError
  │
  └─► 4   dispatch_agent_session_core          ship-agents::services/agents/dispatch_core.rs:85
            • validate_axes(req)               (billed-by-CLI warnings)
            • resolve_resume(req)              (spawned-harness only)
            • let payload = AgentSessionStartedPayload { session_id, slug, branch, … };
            • port.persist_and_route(envelope)  ──► ApiState::persist_and_route
            • return DispatchOutcome { session_id, slug, branch, resume, … }
                  │
                  ▼
            kernel: agent.session.started   (persist to platform.db + route to subscribers)
                  │
                  ▼
            (5) start_agents mailbox     ship-agents::actor.rs:38
            handle_agents_event          (terminal_guard, then switch)
            handle_agent_session_started (event has parent_pipeline_run_id == None)
                  │
                  ├── ensure_pipeline-stuff: parent_pipeline_run_id? → SKIP for bare
                  ├── slow prep under L0 DISPATCH_GATE (detached from mailbox)
                  ├── dispatch_agent_session_internal:
                  │     octx.isolation.prepare(ctx)        (worktree + .dispatched marker + compile_agent_config)
                  │     octx.executor.spawn(ctx)            (RoutingExecutor → Process/Tmux/Server)
                  │     octx.registry.register(slug, handle)
                  │     exit_handler::start(...)
                  │     emit_agent_session_dispatched(...)
                  │
                  └── return to mailbox loop
```

**Smearing in this trace:** none at the entry boundary (2a/2b →
4 is a single hop, both transports converge on the same core).
The smearing is *one level up*: there are TWO `GitWorktreeIsolation`
impls (the host one used at composition time and the L1 one not
used) and TWO `run_dispatch` files (the host one used and the
L1 one dead). Both are pure duplication; the working code lives
in the host crate. **Decision: the boundary map's Option A
(module-level fence) would be the wrong move here** — Option C
(move to a `ship_code` crate) is the right move precisely for
this code: the executor assembly + `GitWorktreeIsolation` +
`ShipdPlatform` + `subscribe_job_events` all belong in a crate
named after what they are, not in `apps/shipd` next to web routes
and the L1 domain. But that crate move is S16.2/S16.3 work
(deferred), not S16.0.

### 2.2 Trace B — pipeline-run advance (`ship run` / `pipeline.run.started` → step N+1 advance)

```
client (CLI / studio)
  │
  └─ 3a (REST)              POST /api/v1/pipelines/run
      │                     handler: apps/shipd/src/routes/runtime_api/pipeline_run.rs:21
      │                       • Scope::CreateRuns gate (transport boundary)
      │                       • port = PipelineDispatchAdapter(state)
      │                       • call run_pipeline_core(&port, req)
  │  or
  └─ 3b (ConnectRPC)        PipelinesService.Dispatch
                            handler: ship-pipeline-runtime::wire.rs:203
                              • (self.write_fn)("pipelines.dispatch", args, ctx)
                              • → ops_rpc.rs:497 closures
                              • call run_pipeline_core(&port, req)
  │
  └─► run_pipeline_core    ship-pipeline-runtime::run_core.rs:125
            • locate_pipeline_file(req.pipeline)
            • parse JSONC → Pipeline
            • let payload = PipelineRunStartedPayload { pipeline_run_id, slug, branch, spec_path, …, steps: Some(pipeline) };
            • dispatch_one(&port, payload, &pipeline_run_id)
                  │
                  ▼
            dispatch_one    run_core.rs:25
            • ensure_pipeline(payload) → PipelineRunStartedPayload
            • port.persist_and_route(envelope)  ──► ApiState::persist_and_route
            • return DispatchOneOutcome { pipeline_run_id, slug, branch, … }
                  │
                  ▼
            kernel: pipeline.run.started    (persist + route)
                  │
                  ▼
            (6) start_orchestrator mailbox  ship-pipeline-runtime::orchestrator::router.rs:28
            handle_orchestrator_event → switch "pipeline.run.started" →
            (7) handle_pipeline_run_started   orchestrator/dispatch.rs:41
                  • ensure_pipeline  (idempotent re-resolve, picks up JSONC)
                  • dispatch_pipeline_run_internal:
                       • if first.step_type == Run → dispatch_first_run_step (run_step::execute_run_step, no spawn)
                       • if first.fanout is Some → dispatch_first_fanout_step ─► (9) fanout
                       • else (Agent|Review):
                            • resolve harness/model from first step
                            • SessionContext { …, env: build_spawn_env(slug, agent, octx.spawn_policy.as_ref()) }
                            • octx.isolation.prepare(ctx)        (worktree + write_phase_spec_from_context + compile_agent_config)
                            • write_pipeline_specs + write_dispatch_prompt (the L1 spec materialization)
                            • if cancelled: bail
                            • octx.executor.spawn(ctx)            (RoutingExecutor)
                            • octx.registry.register(slug, handle)
                            • emit_process_spawned, emit_composed_step_session (composed-session observability)
                            • if is_multi_step: start_step_monitor
                              else: exit_handler::start(ExitVocabulary::PipelineRun)
                            • emit_pipeline_run_dispatched
                  │
                  ▼
              (harness runs in the worktree, eventually emits actor events)
                  │
                  ▼
              substrate: agent.exited, session.ended, step.done_declared, step.observed
                  │
                  ▼
            (6) router → "pipeline.step.observed" → (8) advance_pipeline
            (8) advance_pipeline_inner   orchestrator/pipeline.rs:44
                  • load pipeline_run by id; acquire run_lock; reload under lock
                  • compute "completed" set from current_step / completed_steps / root_steps()
                  • let next = pipeline.unblocked_steps(&completed)
                  • for each next step:
                       • if fanout → (9) dispatch_fanout_step
                       • if Run → run_step::execute_run_step
                       • if Agent|Review → similar to (7)'s first-step path
                  • return true if any step dispatched
```

**Smearing in this trace:** the first-step (7) and advance (8)
paths are deliberately mirrored so they cannot drift
(`dispatch.rs:99-109` cites the same fanout branch as
`pipeline.rs:174-238`). The mirror is a real maintenance cost but
not smearing — it's a fork in the dispatch flow that exists
because the first step has no "current_step" to advance from. The
genuine smearing in the pipeline path is the **dead `run_dispatch`
copy in `ship-agents::runtime`** (same as in Trace A) and the
`StepMonitor` / `exit_handler` / `start_step_monitor` machinery
that lives half in `orchestrator/step_monitor/`, half in
`runner/exit_handler`, half inline in `pipeline.rs:286-317` — a
future slice can fold this into a single `start_step` helper.

### 2.3 Where the two traces converge / diverge

- **Converge**: every effect passes through `octx.executor.spawn`
  (Trace B) or `ports.executor.spawn` (Trace A). Both are
  `SessionExecutor` trait objects; the composition root at
  (1) wires both bundles to the **same** `Arc<RoutingExecutor>`
  instance. The two actors share one `ConfigSpawnPolicy` too
  (cloned into both bundles), so role/token resolution is
  identical on every dispatch path.
- **Converge**: every effect writes to the same kernel event bus.
  The two envelopes are different (`agent.session.started` vs
  `pipeline.run.started`), but the *projection* that materializes
  them (`agent_sessions` projection, populated by
  `agent.session.*` events emitted by `service.agents` and by
  `emit_composed_step_session` in `dispatch.rs:355-398`) is one
  read model. The pipeline-composed session IS a row in
  `agent_sessions` with `parent_pipeline_run_id` set.
- **Diverge**: composed vs bare. Composed sessions (Trace B) are
  owned by the orchestrator; the `service.agents` actor SKIPS them
  in `handle_agent_session_started` when `parent_pipeline_run_id`
  is set. The composed session is advanced when the orchestrator
  sees `agent.session.completed/failed` with that field set
  (`router.rs:113-117, 138-151`). Bare sessions (Trace A) skip
  the orchestrator's composed-session advance.

## 3 · The canonical port

`RunnerPorts` is already the substrate. The L1 smearing is that
two distinct L1 port bundles (`OrchestratorContext` and
`AgentPorts`) both *compose* `RunnerPorts` + their own per-actor
bookkeeping. Three candidate single ports:

1. **`dispatch_ports::RunnerPorts`** (substrate, already there) +
   a new **`Dispatcher` trait** in L1. `RunnerPorts` is the
   data: the ports the runner mechanism needs. `Dispatcher` is
   the verbs: `dispatch_run`, `dispatch_step`, `cancel_run`.
2. **`OrchestratorContext`** as the single port. Bad idea: it
   is 11 fields, 4 of which are L1 bookkeeping (`done_flags`,
   `activity_txs`, `fanout_lane_txs`, `fanout_lane_refs`) that
   don't belong in the bare-session actor (5) and don't belong
   in the embedded runtime. Bloat.
3. **A new single dispatch port** (no name proposed). Risk:
   introduces a third "L1 port bundle" instead of unifying on
   the existing substrate.

**Choice: option 1.** The substrate `RunnerPorts` + a new
`Dispatcher` trait at L1.

`RunnerPorts` (already defined in
`crates/core/ship-substrate/src/services/dispatch_ports/
runner_ports.rs:75-84`):

```rust
pub struct RunnerPorts {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor:  Arc<dyn SessionExecutor>,
    pub platform:  Arc<dyn AgentPlatform>,
    pub registry:  Arc<dyn ProcessRegistryPort>,
    pub probe:     Arc<dyn ProcessTreeProbe>,
    pub spawn_policy: Arc<dyn SpawnPolicy>,
    pub projections: Arc<dyn ProjectionReader>,
}
```

`RunnerPorts` is the *movable* half of the historic
`OrchestratorContext` (per `dispatch_ports/runner_ports.rs:69-84`).
The non-movable half is the per-actor bookkeeping, currently
held alongside in the same struct but explicitly named
`RunnerDomainState` in the substrate
(`runner_ports.rs:86-102`) so the next slice can split it off.

The new `Dispatcher` trait, proposed in
`ship-agents::runtime::dispatcher` (L1, peer of `executors`):

```rust
#[async_trait]
pub trait Dispatcher: Send + Sync {
    /// Begin a fresh pipeline / bare-session / transient-embedded run.
    async fn dispatch_run(&self, spec: DispatchSpec) -> Result<RunHandle, DispatchError>;

    /// Advance a step in a running pipeline.
    async fn advance_step(&self, run_id: &str, terminal: StepTerminal) -> Result<bool, DispatchError>;

    /// Cancel a run; cascade to fanout lanes and embedded children.
    async fn cancel_run(&self, run_id: &str, cause: CancelCause) -> Result<(), DispatchError>;
}
```

Where `DispatchSpec` is the union of `{ BareSessionSpec, PipelineSpec, EmbeddedSpec }`,
`StepTerminal` carries `StepVerdict` + commits + exit code, and
`CancelCause` mirrors the L0 `operator`/`executor_exit` reasons.

### 3.1 What each of the ~5 entries collapses into

| Entry today | What it does today | Collapses to |
|---|---|---|
| (1) `subscribe_job_events` | constructs `OrchestratorContext` + `AgentPorts` from concrete impls; starts actors | **composition root** (stays at `apps/shipd`; builds the `Dispatcher` impl) |
| (2a/2b) REST/ConnectRPC dispatch | validates, calls `dispatch_agent_session_core` | `dispatcher.dispatch_run(BareSessionSpec)` (the wire handlers become transport adapters) |
| (3a/3b) REST/ConnectRPC pipeline run | validates, calls `run_pipeline_core` → `dispatch_one` | `dispatcher.dispatch_run(PipelineSpec)` (the wire handlers become transport adapters) |
| (4) `dispatch_agent_session_core` | builds the bare `agent.session.started` envelope, persists | `dispatcher.dispatch_run(BareSessionSpec).build_envelope()` — the producer half |
| (5) `start_agents` + `handle_agent_session_started` | consumes the envelope, spawns harness through `AgentPorts` | `dispatcher.dispatch_run(BareSessionSpec).consume_envelope()` — the consumer half |
| (5b) `handle_agent_cancelled` | stops the handle, emits `process.killed` | `dispatcher.cancel_run(run_id, operator)` |
| (6)/(7)/(8) router + first-step + advance | the runner mechanism, advances pipeline DAG | `dispatcher.advance_step(run_id, terminal)` (the fanout/sub-routines stay in the orchestrator as the "runner mechanism" the trait is the port to) |

The "thin adapter over" rows: the REST/ConnectRPC handlers
(2a/2b/3a/3b) stay as transport adapters — they enforce the
MCP-workspace violation, scope gate, and blast-tier gate that
are transport-boundary concerns. The composition root (1) stays
at `apps/shipd` because of the L1↔L0 cycle. The runner
mechanism (6/7/8) stays in `ship-pipeline-runtime` because
*it is* the L1 pipeline policy; the embedded runtime's
`Dispatcher` impl reuses the same runner via
`RunnerPorts::clone()`.

**The shape is: ONE `Dispatcher` trait, ONE `RunnerPorts` core,
ONE composition root at `apps/shipd` (or any other host).** The
~5 entries collapse to 3 (REST+RPC, bare-session core, pipeline
core) for the "initiate" side and 1 (`Dispatcher::advance_step`)
for the "advance" side, with the fanout aggregator as an
internal sub-routine.

## 4 · The L1↔L0 cycle — precise map

Verified by reading `crates/core/ship-platform/crates/
ship-agents/Cargo.toml` and `crates/ship-pipeline-runtime/
Cargo.toml`.

```
ship-agents (L1) ── depends on ── ship-substrate (L0)
                ── depends on ── ship-runner (L0)
                ── depends on ── ship-harnesses (L1 sibling)
                ── depends on ── ship-isolation (L0)   (default-off)
                ── depends on ── ship-files (L1 sibling)
                ── does NOT depend on ── ship-pipeline-runtime

ship-pipeline-runtime (L1) ── depends on ── ship-substrate (L0)
                            ── depends on ── ship-runner (L0)
                            ── depends on ── ship-pipelines (L0 data)
                            ── depends on ── ship-platform (L1) → re-exports ship-agents facade
                            ── depends on ── ship-agents (L1)      <-- L1->L1
                            ── depends on ── ship-auth, ship-files, ship-store, ship-artifacts (L1 siblings)
```

The cycle that exists if `ship-agents` directly depended on
`ship-pipeline-runtime`:

```
ship-agents ──► ship-pipeline-runtime ──► ship-agents
```

**What this constrains the design to:**
- The `Dispatcher` trait (proposed in §3) must live in L1, not
  in `ship-pipeline-runtime`, and must not transitively depend
  on `ship-pipeline-runtime`. Proposed home:
  `ship-agents::runtime::dispatcher` — peer of the existing
  `executors` module. The trait names **verbs**
  (`dispatch_run`, `advance_step`, `cancel_run`) and an impl
  takes `RunnerPorts` + a small per-impl `Bookkeeping` struct.
  The trait itself is pure data + verbs; it has no
  `ship-pipeline-runtime` types in its surface. The
  `ShipdDispatcher` impl (composition root) can hold
  `Arc<OrchestratorContext>` internally because the impl is
  constructed in `apps/shipd` where both crates are visible.
- The composition root (1) **stays at `apps/shipd`** for as long
  as the `ship-pipeline-runtime` ↔ `ship-agents` edge exists.
  Any other host (a future browser embed, the cloud worker) that
  also depends on both crates can host its own composition root.
  The trait can be moved to substrate only after the
  orchestrator-side L1 state (`done_flags`/`activity_txs`/fanout
  maps) is split into `RunnerDomainState` per
  `dispatch_ports/runner_ports.rs:86-102`.
- The embedded runtime enters as a `Dispatcher` impl (the
  `EmbeddedDispatcher` variant of `DispatchSpec`) that does NOT
  need pid/worktree/sandbox machinery. The trait surface must
  not require `SessionContext` (which carries `slug`, `branch`,
  `work_dir`, `pid`-shaped env) — the embedded impl ignores
  those fields. **Concretely: `DispatchSpec::Embedded { spec,
  input }` is enough; the impl builds an in-process turn and
  returns the result as `RunHandle::Embedded(id, output)`.**
  See §5 for the full embedded design.

## 5 · Where the embedded runtime plugs in

Per S14 (`spikes/ship-agent/jobs/s14-orchestration.md`): the
embedded runtime is the spike's in-process agent (`rig` + V8 +
deno_core + scoped SDK). It has **NO pid, NO worktree, NO
harness process, NO supervisor child, NO daemon auth scope**.
It runs inside the host process, directly driven by the rig
loop. S14's design choice: a sub-agent / parallel-branch is
*itself a transient `Pipeline`* — the embedded path composes a
`Pipeline { steps: [<one agent step>] }` (or a fanout) and
hands it to the shared advancement code. The transient run
projects onto the same `pipeline_runs` table, emits the same
`pipeline.*` events, and the parent turn awaits the child's
`pipeline.run.completed` over the event bus.

**The choice for S16.0: the embedded runtime enters as a new
`Dispatcher` impl**, NOT as a new `SessionExecutor` kind.

### 5.1 Why not as a `SessionExecutor` (alongside Process/Tmux/Server/Routing)?

`SessionExecutor::spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle>`
(`crates/core/ship-substrate/src/services/dispatch_ports.rs:175`)
is shaped around a `SessionContext` that carries `slug`,
`branch`, `work_dir`, `pid`-shaped env vars, and `LaunchProfile
{ harness, model, persist_session, resume_session, … }`. It
returns an `ExecutorHandle` with a `pid: Option<u32>`. The
`RoutingExecutor` selects per-job based on `adapter.caps()` and
`headless` (see
`crates/core/ship-platform/crates/ship-agents/src/runtime/
executors/routing.rs`). **None of this makes sense for the
embedded runtime**: there is no harness to argv-build, no
`headless` flag, no `pid`. The embedded runtime is not a way
to run a harness; it is a way to run an in-process model loop.
The `SessionExecutor` trait's shape forces an awkward
"embedded-as-harness" translation, and the `RoutingExecutor`'s
decision criteria would have to be expanded to "is this harness
an embedded stub?" — a category error.

### 5.2 Why as a `Dispatcher` impl (alongside the future `ShipdDispatcher`)?

A `Dispatcher::dispatch_run(&self, spec: DispatchSpec) ->
Result<RunHandle, DispatchError>` is shaped around a **verb on
a spec**, not a context for spawning a process. The
`DispatchSpec::Embedded { spec: AgentSpec, input: String }`
variant carries exactly what the embedded runtime needs and
nothing else. The `RunHandle::Embedded { id: String, output:
Value }` return shape carries the in-process turn result back
to the parent. No `pid`, no `worktree`, no `SessionContext` —
and crucially, no "fake" `ExecutorHandle::Embedded` that
`is_alive`/`stop` would have to special-case.

The fanout case (`parallel([s1, s2, s3])`) lowers to a
`DispatchSpec::EmbeddedFanout { specs: Vec<AgentSpec>,
inputs: Vec<String>, min_successful: usize }`. The impl runs
each spec in a child task, collects the per-child `RunHandle`s,
and calls the pure `aggregate_lane_outcomes` (re-exported from
`crates/core/ship-substrate/crates/ship-pipelines/src/fanout.rs::aggregate_lane_outcomes`) — same shortcut decision the
orchestrator uses. The fanout aggregator task is the embedded
runtime's analog of `dispatch_fanout_step`; it just runs
in-process instead of forking harnesses.

### 5.3 What stays the same vs what the embedded path DOESN'T need

| Concern | Spawned-harness path | Embedded path |
|---|---|---|
| `RunnerPorts` fields used | all 7 | `spawn_policy` (for env), `projections` (for terminal-state guard), `platform` (NOT for the embedded case — see below) |
| `IsolationStrategy` | `GitWorktreeIsolation` (worktree + compile) | **NOT NEEDED** — no worktree, no compile-to-disk |
| `SessionExecutor` | `RoutingExecutor` (Process/Tmux/Server) | **NOT NEEDED** — no harness process |
| `AgentPlatform` | `ShipdPlatform` (worktrees_dir, graceful_agent_exit, cleanup_job, mint_session_token) | **NOT NEEDED** — no PID, no worktree, no session token (the embedded path uses the parent's token; the SDK is scoped to the parent's grants per S14 §3.4) |
| `ProcessRegistryPort` | `SharedRegistry` (handle live map) | **NOT NEEDED** — no handle; the child is a `JoinHandle` |
| `ProcessTreeProbe` | `SysinfoProcessTreeProbe` (CPU ticks) | **NOT NEEDED** — no process tree to probe |
| `ProjectionReader` | `ProjectionsReader` (orphan-slugs, terminal guard) | **NEEDED** — terminal-state guard still applies; the embedded run lands in `pipeline_runs` projection |
| `SpawnPolicy` | `ConfigSpawnPolicy` (role/token mint) | **NEEDED** (partially) — the embedded child needs a scoped `SHIP_DAEMON_URL` and possibly a scoped `SHIP_TOKEN`, but not the agent-specific role mapping (the child inherits the parent's authority per S14 §3.4) |
| Per-step spec body | `pipeline_specs::write_pipeline_specs` (writes `.artifacts/pipeline/NN-<step>.md`) | **NOT NEEDED** — the spec body is in the parent's memory; the embedded child reads it from a closure capture, not from disk |
| Per-step dispatch prompt | `pipeline_specs::write_dispatch_prompt` (writes RUNTIME_CONTRACT) | **DIFFERENT** — embedded children don't need a RUNTIME_CONTRACT (they don't emit `step-done`); a shorter "execute this spec, return the result" prompt |
| Fanout aggregator | background task + oneshot lanes | same algorithm, runs in-process (no kernel hop) |
| `EventDedup` | L0 mechanism | **NEEDED** — the embedded run still emits `pipeline.*` events on the parent process's kernel bus |
| `run_locks` | per-job advancement mutex | **NEEDED** — concurrent events can still race the embedded run's projection |
| `done_flags` / `activity_txs` | step-monitor book-keeping | **NEEDED for fanout**, NOT for single embedded children |
| `fanout_lane_txs` / `fanout_lane_refs` | fanout lane oneshot + ref maps | **NEEDED for fanout** |
| Cancel path | `pipeline.run.cancelled` → `handle_cancelled` → `cancel_job_fanout_lanes` | `pipeline.run.cancelled` → embedded impl drops the in-flight child `JoinHandle`; fanout lanes drop their senders (same pattern as `fanout_cancel.rs:75-84`) |
| Resume CLI | `dispatch_resume::resolve_resume` (claude `--resume`, codex `--resume`) | **NOT NEEDED** — the embedded runtime has no resume CLI; "resume" for an embedded child is just running the same `AgentSpec` against the prior context window |

**The port's shape is already right:** `RunnerPorts` is
domain-free (`runner_ports.rs:9-12` is explicit: "EVERY
signature below uses only substrate / primitive types"), and
the seven fields are independently usable. The embedded impl
just doesn't construct the ones it doesn't need — but the
**trait surface cannot require all of them**. This is exactly
the constraint §4's "What this constrains the design to" called
out: `Dispatcher::dispatch_run` takes a `DispatchSpec`, not a
`SessionContext`; the trait carrier is the spec, not the port.

### 5.4 The embedded impl's collaboration with the pipeline runner

This is the subtle bit. Per S14: the embedded path *uses* the
`ship-runner` mechanism (the L0 mailbox-loop primitives in
`crates/core/ship-substrate/crates/ship-runner/src/router.rs`).
The child is a `Pipeline`; the parent's tool future awaits
the child's `pipeline.run.completed` event. **The child
dispatch goes through the SAME `start_orchestrator` actor** —
not a separate embedded-only actor — because the L0 mailbox
dedup and terminal-state guard (`terminal_guard` in
`ship-runner/src/router.rs`) are the same guarantees the
embedded path needs. The embedded impl's job is just to:

1. Build a transient `Pipeline { steps: [<one Agent step with
   goal = input>] }`.
2. Build a `PipelineRunStartedPayload` with a
   `pipeline_run_id = <parent_run>::<child_id>::<ulid>` (the
   lineage scheme from S14 §2.1, mirroring
   `parent_pipeline_run_id` for composed sessions).
3. Hand the payload to the L1 pipeline-dispatch core
   (`dispatch_one` in
   `crates/ship-pipeline-runtime/src/run_core.rs:25`) — which
   builds the `pipeline.run.started` envelope, persists, and
   routes.
4. Subscribe to the event bus for the child's
   `pipeline_run_id`, return a tool future that resolves on
   `pipeline.run.completed` / `pipeline.run.failed`.

The orchestrator's `handle_pipeline_run_started` (7) then
drives the embedded child exactly as it drives a
spawned-harness step. The **only** difference is that the
embedded child has `step.step_type = Agent` with
`step.agent = <embedded-stub>` and `step.harness = None` — and
the orchestrator's `advance_pipeline_inner` (8) dispatches
via `octx.executor.spawn(ctx)`. **This is where the embedded
impl WOULD need a `SessionExecutor`** — but a different one:
a stub `EmbeddedExecutor` that just acknowledges the spawn
(returns a `None` pid, no real work) because the work is being
done by the in-process tool future waiting on the
`pipeline.run.completed` event. **OR** we sidestep the
orchestrator entirely and have the embedded impl emit the
`pipeline.run.completed` event directly when the in-process
turn finishes, without going through `advance_pipeline`. The
second option is cleaner: the embedded path is a peer of the
orchestrator, not a sub-routine.

**Decision (proposed, not ratified):** the embedded path runs
*without* the orchestrator's `advance_pipeline` loop. The
embedded impl subscribes to its own `pipeline.run.completed`
event (filtering on the child's `pipeline_run_id`); when the
in-process turn resolves, the impl emits the terminal event
itself. This is the same pattern the fanout aggregator uses
to emit the synthetic `pipeline.step.observed`
(`crates/ship-pipeline-runtime/src/orchestrator/fanout.rs:244-266`).
The orchestration is the orchestrator *in the parent's run*
(Trace B) but **the embedded child's single step has no
parent pipeline** — the child IS its own transient pipeline,
with the embedded impl acting as both dispatcher and
aggregator. The first-slice proof in
`spikes/ship-agent/jobs/s14-orchestration.md §5.2` tests
upholds this: "the child ran (the child projection row exists
with status `Completed`); the child emitted its
`pipeline.run.completed` event."

## 6 · Compiler prompt/context angle

The job spec asks: does the compile step produce
**context/prompt content** (`compiler::compile::context.rs
::build_context_content` etc.) that the EMBEDDED runtime
should also consume at turn-start? Is there a shared
"resolve agent context" seam, distinct from the
on-disk-harness-config output?

### 6.1 What `build_context_content` produces

`build_context_content(desc, resolved) -> Option<String>`
(`crates/core/compiler/src/compile/context.rs:5-33`) emits a
single string: rules content (trimmed, joined by `\n\n`) + an
optional active-agent notice (`<!-- ship: active mode = {id}
-->`). It is one of several fields in `CompileOutput` (the
others: `skill_files`, `event_subscriptions`, `settings`,
`mcp`, `plugins`, `context_content`). The compiler then writes
the output to a path that depends on the harness's
`ContextFile` (e.g. `CLAUDE.md` for claude). The string is
*content the harness file system reads at start-up*; it is
not structured data the model is queried against.

### 6.2 What the embedded runtime would want

The embedded runtime drives a model turn. The turn starts
with a structured prompt composed of:

1. **Rules content** (the same as the spawned-harness path's
   `context_content`, but the embedded child receives it
   *inline* as part of the in-process turn input — not as a
   file the spawned process reads).
2. **Active-agent notice** (the `<!-- ship: active mode = … -->`
   block).
3. **Skill bodies** (the embedded child's SDK surfaces the
   scoped skills via `import "ship"` in deno_core; the
   `skill_files` field is the equivalent of what the spawned
   harness reads off disk).
4. **Resolved grants** (the embedded child's gate enforces
   the granted services — the spike's `gate.rs` already does
   this; the daemon's L1 `SpawnPolicy` mints the right
   session token).
5. **Resume context** (the previous turn's output, the
   compaction policy, etc. — the embedded runtime's
   `HistoryStore` seam from S9).

Of these, **(1) and (2) are the "context content" the job
spec asks about** — and they ARE shared between the
spawned-harness path and the embedded path. (3), (4), (5) are
embedded-specific shapes that the L1 domain already has
seams for.

### 6.3 Is there a shared "resolve agent context" seam?

**Yes, partial.** The L1 `ship-agents::sessions::compile.rs`
exports `resolve_session_agent_config(ship_dir, agent_id) ->
ProviderSettings` (line 11-24) and
`resolve_session_context_root(ship_dir, explicit_path) ->
PathBuf` (line 46-57). These resolve:

- Which `AgentProfile` is named by the spec / dispatch.
- Where the on-disk harness config files live (project root
  or worktree).
- The harness list to compile for.

But these produce **on-disk-harness-config** output: the
`compile_session_context` helper at
`crates/core/ship-platform/crates/ship-agents/src/sessions/
compile.rs:66-101` calls
`crate::services::compile::compile_project(...)` which writes
harness-specific config files (`.claude/settings.json`,
`.codex/config.toml`, etc.). The embedded runtime doesn't
need those files — it needs the **structured** data: rules,
permissions, grants, skill list, context content string.

**The gap:** the L1 compile service exposes a *filesystem
output* (write these files, emit `compile.completed`). It
does NOT expose a *structured* `ResolvedAgentContext` value
that the embedded runtime could consume directly. A new
seam is needed:

```rust
// proposed, in ship-agents::sessions::compile.rs (or
// services::compile::resolve) — pure data, no I/O
pub struct ResolvedAgentContext {
    pub active_agent: Option<String>,
    pub rules: Vec<Rule>,
    pub skills: Vec<Skill>,
    pub permissions: Permissions,
    pub requested_grants: Vec<String>,
    pub context_content: Option<String>, // build_context_content output
    pub harnesses: Vec<String>,
}

pub fn resolve_agent_context(
    ship_dir: &Path,
    agent_id: Option<&str>,
) -> Result<ResolvedAgentContext> { ... }
```

This fn takes the same `ResolvedConfig` the compile step
takes (per `crates/core/compiler/src/compile/mod.rs:177`:
`out.context_content = context::build_context_content(...)`),
but stops before writing to disk. **Both runtimes call this
same fn** to get the structured input to their turn: the
spawned-harness path writes it to `.claude/settings.json`'s
`context` field (or equivalent), the embedded path injects
it into the rig prompt.

### 6.4 What this is NOT

The job spec is explicit: "SDK-gen is already decided OUT of
the compiler — do not relitigate that; only the
prompt/context-resolution question is open." The proposed
`resolve_agent_context` is **NOT** the SDK-gen path (that's
spike's `sdkgen.rs` + `dtsgen.rs` per S14). It is the
**structured content** the model sees at turn-start, which
is independent of the generated SDK the model imports at
code-mode. The compile service currently mixes the two
(writes the harness file *and* produces the structured
content); the proposal splits them.

### 6.5 Is this in scope for S16.0?

**No.** S16.0 is a *map*, not a design. The proposed
`resolve_agent_context` seam is flagged for S16.2 or a
follow-up slice: it's a non-trivial refactor of the L1
compile service, and it touches every code path that calls
`compile_project` today. The compile-step output today is
ON-DISK files; turning it into ON-DISK + STRUCTURED requires
changing the call sites (the spawned-harness path still
wants the files; the embedded path wants the struct). A
future slice can do this; S16.0 only names the seam.

## 7 · Proposed unified single-entry design + lowest-risk first step

### 7.1 The design

ONE `Dispatcher` trait in L1, ONE `RunnerPorts` core, ONE
composition root at `apps/shipd`:

```text
                        transport (REST / ConnectRPC)
                                |
                                v
  +=====================================================+
  |  domain core (transport-neutral)                    |
  |  +* dispatch_agent_session_core   (bare-session)     |
  |  +* run_pipeline_core -> dispatch_one  (pipeline)    |
  |  +* dispatch_embedded_core?  (proposed, see below)  |
  +======================+===============================+
                         |
                         v
  +=====================================================+
  |  Dispatcher (NEW L1 trait)                          |
  |  dispatch_run(spec: DispatchSpec) -> RunHandle      |
  |  advance_step(run_id, terminal) -> bool             |
  |  cancel_run(run_id, cause) -> ()                    |
  +======================+===============================+
                         |
            +============+============+
            v                         v
   +==================+      +=====================+
   | ShipdDispatcher  |      | EmbeddedDispatcher  |
   | (composition     |      | (NEW - peer entry)  |
   |  root at         |      |                     |
   |  apps/shipd)     |      | in-process rig turn |
   |                  |      | + transient         |
   | holds:           |      |   Pipeline          |
   |  Arc<Orch-       |      |                     |
   |   Ctx>           |      | holds:              |
   |  Arc<AgentPorts> |      |  Arc<Model>         |
   |                  |      |  Arc<Gateway>       |
   |  Arc<Kernel>     |      |  Arc<ProjectionRead>|
   +========+=========+      +==========+==========+
            |                          |
            +==========+===============+
                       v
              +==================+
              |   RunnerPorts    | (substrate, already there)
              |  + L0 mechanism  |
              |  + kernel bus    |
              +==================+
```

The `Dispatcher` trait itself lives in
`ship-agents::runtime::dispatcher` (peer of `executors`,
`helpers`, `agent_lifecycle`). The trait surface is
domain-free where possible: verbs over a `DispatchSpec`
enum, returning a `RunHandle` enum. The `ShipdDispatcher`
impl carries an `Arc<OrchestratorContext>` + `Arc<AgentPorts>`
(visible only at the impl site in `apps/shipd`, never in the
trait). The `EmbeddedDispatcher` impl carries
`Arc<dyn Model> + Arc<dyn Gateway> + Arc<dyn
ProjectionReader>` — none of the runner mechanism's
process-shaped ports.

### 7.2 What changes for each entry

| Entry | Today | After `Dispatcher` |
|---|---|---|
| (1) composition root | constructs `OrchestratorContext` + `AgentPorts` and starts actors | constructs `ShipdDispatcher` (which owns the actors) + `EmbeddedDispatcher` and starts them |
| (2a/2b) bare-session dispatch | `dispatch_agent_session_core` builds envelope, `AgentPorts` consumes it | `dispatch_agent_session_core` calls `dispatcher.dispatch_run(BareSessionSpec)`; the impl builds the envelope + registers the consumer |
| (3a/3b) pipeline dispatch | `run_pipeline_core` + `dispatch_one` build envelope, `OrchestratorContext` consumes it | same, with the `Dispatcher` impl owning the orchestrator-side wiring |
| (4)/(5) | bare-session actor is a free function in `ship-agents::actor` | folded into `ShipdDispatcher`; the actor becomes a method on the impl |
| (6) router | free function in `ship-pipeline-runtime::orchestrator::router` | the router becomes the internal `runner_mechanism` module of `ShipdDispatcher` (the actors it starts become "started by the impl") |
| Embedded | doesn't exist | `EmbeddedDispatcher::dispatch_run(EmbeddedSpec)` — the new entry, peer of `ShipdDispatcher` |
| `dispatch_resume` | in `ship-agents::services::agents::dispatch_resume.rs` (per `s16-shipcode-boundary.md §6.3`: spawned-harness only) | moves to `ship_code::runtime::dispatch_resume` (the S16.2 carve) — NOT in scope for S16.0 |
| `AgentPlatform` impl | `ShipdPlatform` in `apps/shipd::dispatch::platform` | moves to `ship_code::runtime::platform` (S16.2); until then, stays in `apps/shipd` and the `ShipdDispatcher` constructor captures it |
| `GitWorktreeIsolation` (host) | in `apps/shipd::dispatch::adapters` | same move as `AgentPlatform` |

### 7.3 The lowest-risk first step (READ-ONLY, in this slice)

**No code moves.** This doc IS the first step. The work the
slice delivers is:

1. The entry-point inventory above (§1) — which currently
   lives in the boundary map's head, in the code, and in
   tribal knowledge. Written down.
2. The control-flow graph (§2) — bare-session and pipeline
   side-by-side, with the convergence and divergence points
   called out.
3. The port choice (§3) — `RunnerPorts` + new `Dispatcher`
   trait, with the alternative considered and rejected.
4. The cycle map (§4) — verified by `cargo tree`; the
   cycle wall is the un-mod'd `adapters.rs` and
   `run_dispatch.rs` files in `ship-agents::runtime/`.
5. The embedded entry design (§5) — peer `Dispatcher` impl,
   with a table of which `RunnerPorts` fields are needed
   and which are explicitly not.
6. The compile-context seam (§6) — flagged for a follow-up
   slice, not in S16.0 scope.
7. The proposed design + first-step convention (§7) — the
   shape, the impl list, and the explicit "no code moves
   in S16.0" rule.

### 7.4 The convention to keep (S16.0 fence)

Per the boundary map's Option A, **this slice is a doc**. No
`git mv`, no `pub use` reshuffles, no `#[cfg(feature = ...)]`
gates. The next slice (S16.2) is the implementation: at that
point, the boundary map's `Option C` ("move to a `ship_code`
crate") is the right move for the host-side `apps/shipd/
dispatch/` files; the trait + impl design in this doc is the
shape the S16.2 slice implements.

## 8 · Open questions deferred to S16.0 sign-off

(Repeating from `s16-shipcode-boundary.md §8` + adding
slice-specific items; this doc is the evidence base for
the S16.2/S16.3 follow-ups.)

1. **Embedded impl's "transient pipeline without a
   parent-run" detail (§5.4).** Proposed: the embedded
   child emits its own `pipeline.run.completed` event when
   the in-process turn finishes, sidestepping the
   orchestrator's `advance_pipeline` loop. Alternative: the
   orchestrator's first-step path drives the embedded child
   via a stub `EmbeddedExecutor` (a fake spawn that
   acknowledges but does no work). The first option is
   cleaner (no `ExecutorHandle::Embedded` shim); the second
   is more uniform (every step goes through the same
   dispatch path). *Not blocking S16.0.*
2. **Compile-context seam (§6).** The
   `resolve_agent_context` proposal is non-trivial and
   touches every call site of `compile_project` today.
   Defer to a follow-up slice (call it S16.2.1 or
   S-compile-resolve).
3. **Trait location.** The proposed
   `ship-agents::runtime::dispatcher` is the right home
   *now* (L1, no cycle). Long-term the trait can move to
   substrate (`ship-substrate::services::dispatch_ports::
   dispatcher.rs`) once the orchestrator-side L1 state is
   split per `runner_ports.rs:86-102`. The S16.0 trait
   signature is designed to be substrate-movable: no
   domain types cross the trait surface.
4. **`DispatchSpec` shape.** Bare session + pipeline +
   embedded + embedded-fanout. Should it be one enum, or
   should the trait be generic over a `Spec` type
   parameter? Proposed: one enum, for now. A future slice
   can generic-ify if a fourth variant appears.
5. **Gate model.** Per `s16-shipcode-boundary.md §8.1`:
   `tools/tool_gate.rs` (Layer 2) and the spike's
   `gate.rs` (blast tiers) coexist. The dispatch port
   doesn't carry a gate — the gate runs at every op
   invocation, not at the dispatch boundary. The
   `Dispatcher` trait carries `auth: AuthContext` for
   the C2 grant check on the *dispatch* op itself; per-op
   gates run inside the runtime that the dispatcher
   instantiates (harness adapter, or embedded model loop).
   *Not blocking S16.0.*
6. **`dispatch_resume::resolve_resume` and the embedded
   path's "resume" affordance (§5.3).** Spawned-harness
   has a CLI shape (`claude --resume <sessionId>`);
   embedded has the in-process `HistoryStore`. The
   embedded path's resume is "re-run with the prior
   context", not "re-spawn a child". The
   `DispatchSpec::Embedded { spec, input }` carries
   `input` as the new turn's input; the parent's
   `HistoryStore` is closed-over in the embedded impl's
   constructor. No CLI shape needed. *Not blocking S16.0.*

## 9 · Acceptance for S16.0

- [x] Every entry point that initiates or advances a
      dispatch/orchestration is inventoried with file:line
      citations (§1).
- [x] The control-flow graph for one full bare-session
      dispatch AND one full pipeline-step advance is drawn
      with convergence/divergence called out (§2).
- [x] The canonical port is chosen with alternatives
      rejected on the record; the trait shape is proposed
      (§3).
- [x] The L1↔L0 cycle is mapped precisely (cargo tree
      verified) with the design constraint stated (§4).
- [x] The embedded runtime's entry is designed with the
      "no pid/worktree/harness" constraint honored (§5).
- [x] The compiler context/prompt seam is named as a
      follow-up slice, with the proposed shape (§6).
- [x] The unified single-entry design + lowest-risk first
      step are stated explicitly; the S16.0 fence is
      "doc-only" (§7).
- [x] `cargo test --manifest-path spikes/ship-agent/
      Cargo.toml` → 130 passed (unchanged from baseline).

## 10 · References

- `spikes/ship-agent/jobs/s16-shipcode-boundary.md` —
  S16.0 keystone: the `ship-code` ↔ `shared-domain`
  boundary, the cross-edge inventory, the first-cut
  fence options. This doc extends §1.1/§2/§7 with the
  *orchestration* perspective.
- `spikes/ship-agent/jobs/s14-orchestration.md` — the
  agent-altitude design. §1 (shared model) + §2 (two
  altitudes) + §5 (the seam) are the design contract
  this doc ratifies the port for.
- `spikes/ship-agent/README.md` — spike SSOT.
- `crates/core/ship-platform/crates/ship-agents/src/
  lib.rs` — the crate module map; line 25 declares
  `pub mod runtime;` and `runtime/mod.rs:21-23` wires
  the visible submodules.
- `crates/core/ship-platform/crates/ship-agents/
  Cargo.toml` — dependency list (§4 cycle verification).
- `crates/ship-pipeline-runtime/Cargo.toml` —
  dependency list (§4 cycle verification).
- `apps/shipd/src/dispatch/{mod, adapters, platform,
  run_dispatch, orphan_cleanup}.rs` — composition root
  outside the crate to break the L1↔L0 cycle.
- `crates/ship-pipeline-runtime/src/orchestrator/
  {router.rs, dispatch.rs, pipeline.rs, fanout.rs,
  handlers.rs, run_core.rs, projection_reader.rs,
  spawn_policy.rs}` — the runner mechanism.
- `crates/core/ship-substrate/src/services/
  dispatch_ports/{dispatch_ports.rs, runner_ports.rs}` —
  the substrate port bundle (§3 the canonical port).
- `crates/core/ship-substrate/crates/ship-pipelines/
  src/{types.rs, fanout.rs, gate.rs, validate.rs,
  variables.rs}` — the L0 data crate (DAG types,
  aggregator decision, validators).
- `crates/core/compiler/src/compile/{context.rs, mod.rs,
  harness.rs, agents.rs, claude.rs, codex.rs, cursor.rs,
  opencode.rs, pi.rs}` — the compile pipeline
  (§6 the context/content seam).
- `proto/ship/ops/v1/{agents.proto, runtime.proto,
  pipelines.proto}` — the RPC surface; `AgentsService
  .Dispatch` and `PipelinesService.Dispatch` are the
  two producer surfaces that converge on the dispatcher
  port.
