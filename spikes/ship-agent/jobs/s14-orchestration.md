# Job S14: Orchestration ↔ Ship pipelines reconciliation (DESIGN)

## Context

Work in `spikes/ship-agent/` (own Cargo workspace; builds from any location).
Read `README.md` (SSOT for invariants, module map, spine).

The spike README is explicit and is the brief:

> PHASE 5 — orchestration: S14 agent()/parallel — **RECONCILED WITH SHIP
> PIPELINES (shared DAG/control-flow), not a parallel system.**
> "Orchestration and ship pipelines are the same capability (DAG + control
> flow + parallelization) at different altitudes and must share machinery."

So this is **not** "design a new agent orchestrator." It is: *what is the ONE
shared DAG/control-flow machine that serves both (a) an agent spawning
sub-agents / parallel branches in-loop, and (b) the Ship pipeline runtime —
and how does each altitude consume it without forking into two systems?*

This is design-only. Deliverable: this document. No code, no new modules.
Constraint honored: `gate.rs` / `gateway.rs` / `runtime.rs` / `agent_loop.rs`
in the spike are untouched. `crates/ship-pipeline-runtime` is read, cited,
designed against — never modified.

## How the two altitudes frame the question

### Altitude A — in-loop (the agent runtime)

The agent loop drives the model turn via rig (`agent_loop.rs:162-175`). Tools
the model can call are two: `ship_op` (one gated op through the one door)
and `run_script` (multi-step code-mode on a dedicated VM thread). Every
effect the model causes enters `gateway.call(op, args, auth)`
(`gateway.rs:34`); the gate runs there (`gate.rs:37-56`); C3 is unreachable
because `operator_cred` is hard-wired false in-process (`auth.rs:23-26`).

The latent need: an in-loop agent that wants to *fan out work* — e.g.
"research these 5 sub-questions in parallel, then synthesize" — has nowhere
to express that within the current two-tool surface. Today the model can
only `ship_op` / `run_script`; it cannot say "spawn a child agent to do X
with this spec, then wait for K of N to come back, then advance." It
certainly cannot do that without going around the gate.

The spike already surfaces `agent()` at the SDK / `.d.ts` level as the
orchestration entry point — it appears only with the `runtime` grant
(`sdkgen.rs:165-174`, `sdkgen.rs:189-193`); the description: "The runtime
grant surfaces as the orchestration entry point, not as a
`runtime.dispatch` method — spawning is a capability, not a service call"
(`sdkgen.rs:166-170`). So the *type-level* capability shape exists. What
it *means* — the actual control-flow semantics — is what S14 decides.

### Altitude B — pipeline runtime (the daemon)

The pipeline runtime is a DAG of `PipelineStep` nodes declared in
`.ship/pipelines/*.jsonc`. Each step is one of `Agent | Review | Run`
(`ship-pipelines/src/types.rs:24-34`). Edges are `after: Vec<String>`
declarations on each step (`types.rs:131-132`); the unblocking rule is
`Pipeline::unblocked_steps(completed) -> Vec<&PipelineStep>`
(`types.rs:274-280`).

Today, fanout/parallelism inside a pipeline is expressed via a single step
kind: `FanoutSpec { lanes, judge, min_successful_lanes }`
(`types.rs:95-106`). Each lane spawns its own worktree + agent in parallel
(`ship-pipeline-runtime/src/orchestrator/fanout.rs:39-187`), the lanes'
oneshot `LaneOutcome` senders collect into a background aggregator
(`fanout.rs:194-267`), and the pure `aggregate_lane_outcomes` decision
(`ship-pipelines/src/fanout.rs:31-47`) emits a synthetic
`pipeline.step.observed` for the parent so DAG advancement continues.

Control flow today is linear-with-fanout: a `PipelineStep::after` declares
its dependencies, and once they all complete the step dispatches
(`pipeline.rs:174-238`). There is no explicit conditional, no loop, no
in-step sub-graph. The Review step is the only "judge a previous step's
work" primitive (`review_retry.rs:18-109`). The post-step diff gate
(`ship-pipelines/src/gate.rs`) is declarative, not an agent — it never
spans nodes.

### The question in one sentence

Altitude A needs *new* primitives the pipeline runtime doesn't have
(arbitrary branching, in-loop spawning of child agents, joining on
sub-graphs from inside a turn). Altitude B has the machinery for
*static* DAG + fanout + judge. They MUST share it. What is the shared
machine, what is altitude-specific, and what new primitives the agent
altitude genuinely needs?

## 1 · Shared model — the one DAG/control-flow machine

### 1.1 The node vocabulary

A node is a *unit of gated work* with input ports, output ports, and a
verdict. Three node kinds exist:

1. **Effect node** — the unit of agent or shell work. A `PipelineStep`
   in the L0 spec types (`ship-pipelines/src/types.rs:118-180`) is the
   concrete shape: `id`, `step_type` (`Agent | Review | Run`), `after`,
   `agent`, `model`, `harness`, `command`, `produces`, `consumes`,
   `promotes`, `gate`, `fanout`, `max_retries`, `invocation`. The same
   shape serves both altitudes; altitude-specific fields are *omitted*
   by the other altitude, not a different struct.

2. **Fanout node** — an effect node whose `fanout` is set. Conceptually
   a parent with K lanes + a judge. In the L0 type it is **not a
   separate kind**: fanout is an *attribute* of an effect node
   (`types.rs:163-165` and the validator `validate_fanout` in
   `ship-pipelines/src/validate.rs:168-213`). The shared machine
   lowers a fanout node to "spawn K child effect nodes, then a judge
   effect node that depends on all of them" — i.e. the lanes and the
   judge *are* effect nodes in the same DAG, the parent is just a
   convention/structuring device.

3. **Judge node** — an effect node of `StepType::Review` or a `Run`
   scorer whose `after` includes the fanout parent (or the lanes
   directly, by the lane-scoped consume convention
   `ship-pipelines/src/validate.rs:215-244`,
   `parse_lane_scoped_ref` in `validate.rs:18-23`). It is an ordinary
   effect node whose job is to consume lane outputs and emit a
   verdict. The fanout aggregator's `aggregate_lane_outcomes` is the
   *shortcut* decision (`Advance` if `min_successful_lanes` is met,
   `Fail` otherwise) — the judge node is the *rich* decision (a real
   agent arbitrating).

There is **no fourth node kind**. A conditional, a loop, a manual
decision — when added — are composed from these three, not new node
kinds. (See §6 for the conditional/loop work that does *not* land in
S14.)

### 1.2 The edge vocabulary

Edges are `PipelineStep::after: Vec<String>` (`types.rs:131-132`).
Resolved via `Pipeline::unblocked_steps(completed) -> Vec<&PipelineStep>`
(`types.rs:274-280`). The rule is "all dependencies completed ⇒
dispatchable." That's the only rule.

Fanout lowers into edges:
- lane `L` of fanout step `F` has `after: F.after ∪ {F.id}` (i.e. the
  same upstream deps as the parent, plus the parent itself, which
  is a no-op DAG-wise but keeps the lane a normal node).
- judge step `J` has `after: F.id` (it depends on the fanout step
  completing as a whole).
- The synthetic `pipeline.step.observed` emitted by the aggregator
  (`fanout.rs:244-266`) advances the parent step in the projection
  so the judge's `after` resolves.

Lane-scoped consumes (`<lane_id>.<output_name>`) are a *naming
convention* over the same edge structure — they don't add edges,
they just resolve inputs to a specific lane's outputs during dispatch
(`step_io.rs:49-134`).

### 1.3 Parallelism, fan-in, and joins

Parallelism is a property of *indegree*: a node with N=0 indegree and
M unblocking edges dispatches in parallel with all other N=0-indegree
nodes. There is no explicit `parallel { ... }` block; the DAG
implicitly parallelizes across root nodes, and the fanout node
explicitly parallelizes K lanes.

Fan-in is implicit: a node's `after` is the fan-in point. K upstream
nodes converging on one `after` is the standard K-fan-in. The
fanout/judge pattern is the dedicated K-fan-in shape: K lanes →
aggregator (synthetic observed) → judge (real or shortcut).

Joins: there is no "wait for any" or "wait for N" semantic distinct
from "all dependencies complete." The fanout's
`min_successful_lanes: usize` (`types.rs:103-105`,
`ship-pipelines/src/fanout.rs:31-47`) is the dedicated N-of-K join —
the *only* place the shared machine relaxes "all dependencies." That
is the right place to keep it: a generic N-of-K join over arbitrary
edges would let one slow sibling block progress invisibly; a
fanout's lanes are scoped to the same spec, expected to finish in
comparable time, and the *judge* is the place where semantic
arbitration happens.

### 1.4 Control flow — what's in, what's deferred

**In:**
- Sequencing: `after` is the only mechanism.
- Parallelism: implicit across roots + explicit via `fanout`.
- Aggregation: `aggregate_lane_outcomes` (shortcut) + judge node
  (rich).
- Retries: `PipelineStep::max_retries: Option<u32>`
  (`types.rs:143-145`) + `review_retry.rs` (review-failure retry) +
  `gate` attribute (declarative diff-rule gate, on any step).

**Out of scope for S14, deferred to a later slice:**
- Conditionals ("if X then A else B"). Adding a `Branch` node kind
  with `condition: <op-result-binding>` would work but is its own
  design — see §6.1.
- Loops ("repeat until X"). Same — its own design, see §6.2.
- Recursion / in-loop sub-DAG. The agent-altitude §3 design is the
  first cut; it should not bleed into pipeline-altitude spec types
  before the first slice proves out.

**Out of scope permanently:**
- A "general" join primitive distinct from fanout — see §1.3.

### 1.5 Cancellation

The pipeline altitude already has the shape: `pipeline.run.cancelled`
event (`pipeline.rs:5` in the `event_types` block), `cancel_job_fanout_lanes`
tears down registered lanes
(`ship-pipeline-runtime/src/orchestrator/fanout_cancel.rs:23-85`),
`handle_cancelled` stops the agent process
(`handlers.rs:148-236`). Terminal-state guards
(`ship-runner/src/router.rs:terminal_guard`) drop `step.failed` events
for terminal runs so cancellation isn't miscategorized as failure.

The agent altitude inherits the same shape: any effect spawned from
within a run belongs to that run. Cancellation drops the in-flight
oneshot senders (`fanout_cancel.rs:75-84`) so the aggregator's `rx.await`
resolves to `Err(RecvError)` and the fanout background task exits
promptly (`fanout.rs:206-227`). The agent altitude MUST use the same
mechanism: each spawned child's lifecycle is registered to the parent's
`PipelineRun`-equivalent in the agent altitude (see §2.1), and
operator cancellation of the parent tears down children with the same
oneshot-dropping + terminal-guard pattern.

### 1.6 The shape proof — one machine, not two

The shared model is the L0 spec types in `ship-pipelines` plus the
runner-mechanism primitives in `ship-runner`:
- `Pipeline { steps, auto_merge }` + `PipelineStep` +
  `FanoutSpec` + `GateSpec` + `StepOutput` + `StepPromote` —
  pure data, no I/O, portable to the cloud worker and the browser
  embed (`ship-pipelines/src/lib.rs:1-7`).
- `aggregate_lane_outcomes` (pure decision function) +
  `validate_pipeline` (pure validator) — same input, same output
  everywhere.
- `RunnerPorts` (core ports), `SpawnPolicy`, `ProjectionReader`
  (`ship-substrate/src/services/dispatch_ports/runner_ports.rs:69-84`)
  — the runner mechanism relocated to L0 already (`ship-runner/src/lib.rs:5-28`).

The pipeline altitude composes all of this. The agent altitude composes
the same `Pipeline` / `PipelineStep` / `aggregate_lane_outcomes` /
`RunnerPorts` — not a new control-flow layer. The next sections prove
that.

## 2 · Two altitudes, one engine

### 2.1 The agent altitude — the in-loop DAG

A sub-agent / parallel-branch is *itself* a `Pipeline`. The agent
altitude composes a transient `Pipeline` per in-loop spawn; the parent
turn waits on it the way the pipeline runtime's `await` would. Concretely:

- `agent(spec, input)` in the scoped SDK
  (`sdkgen.rs:189-193`, `dtsgen.rs:48-51`) lowers to
  `runtime.dispatch { spec, input }` — an op on the `runtime` service
  domain. The `runtime` service is the orchestration entry point
  (`sdkgen.rs:115-122`); the gate sees `runtime.dispatch` and
  applies the C2 grant check on `runtime`.
- The runtime impl of `runtime.dispatch` is a *thin handler* in
  the agent-altitude control plane: it builds a `Pipeline { steps:
  [<one agent step>] }` (or, for `parallel([spec, spec, ...])`, a
  `Pipeline { steps: [<fanout step>], ... }` with a single judge), and
  hands it to the shared advancement routine. The handler is a
  payload-shape adapter, not a new orchestrator.
- The transient `PipelineRun` projection row uses
  `pipeline_run_id = <parent_run>::<child_id>::<ulid>` and `slug =
  <parent_slug>::<child_slug>` — the same scheme the fanout lane
  uses today (`fanout.rs:73-76`,
  `dispatch.rs:339-341` `parent_pipeline_run_id` lineage for
  composed sessions). The composed-session observability pair
  (`emit_composed_step_session` in
  `ship-pipeline-runtime/src/orchestrator/dispatch.rs:355-398`)
  is the read-model that surfaces both bare agent sessions and
  composed-pipeline-step sessions identically.
- The parent turn's `run_step` awaits the child run's
  `pipeline.run.completed` (or `pipeline.run.failed`) event the way
  the pipeline router awaits any step. The child runs through
  the *same* runner-mechanism code path in `ship-runner`.

The shape of the agent altitude's `agent()` is therefore:

```text
agent(spec, input) ─► runtime.dispatch op ─► gate (C2 on `runtime`)
                                            │
                                            ▼
                          build transient Pipeline, spawn, await
                                            │
                                            ▼
                          shared runner advances the child DAG
                                            │
                                            ▼
                          events flow through the existing
                          `events.rs` (spike) /
                          `ship-substrate::events::EventEnvelope`
                          (daemon) stream
```

`parallel([s1, s2, ...])` is the same shape with a `FanoutSpec` and
a small judge step. The judge is the fanout's `min_successful_lanes`
shortcut decision for a default configuration; richer arbitration
(an LLM judge) is a normal `StepType::Review` step that depends on
the fanout and consumes the lanes' outputs via lane-scoped
references.

### 2.2 The pipeline altitude — the daemon DAG

The pipeline altitude is what exists today: a `Pipeline` declared
in `.ship/pipelines/*.jsonc` (see `.ship/pipelines/sui-eval.jsonc`
or `multi-harness-smoke.jsonc` for live examples), loaded by
`pipeline_loader::ensure_pipeline` into a `PipelineRun` projection
row, advanced by the router
(`ship-pipeline-runtime/src/orchestrator/router.rs:95-285`) and
the per-event handlers (`pipeline.rs:25-382`).

Fanout already lowers to lane effect nodes + judge effect node
(`fanout.rs:39-273`); the aggregator's `aggregate_lane_outcomes`
is the pure decision function (`ship-pipelines/src/fanout.rs:31-47`).
The pipeline altitude today is the *only* altitude; the agent
altitude is what S14 stands up against it.

### 2.3 What is shared vs altitude-specific

| Concern | Shared | Pipeline-altitude-specific | Agent-altitude-specific |
|---|---|---|---|
| Spec types | `Pipeline`, `PipelineStep`, `FanoutSpec`, `GateSpec`, `StepOutput`, `StepPromote` | — | — |
| Validation | `validate_pipeline`, `validate_pipeline_io`, `validate_fanout`, `validate_output_name` | — | — |
| Aggregation decision | `aggregate_lane_outcomes`, `AggregateDecision` | — | — |
| Variable interpolation | `interpolate`, `interpolate_step` | — | — |
| Step status semantics | `PipelineStepObservedPayload::compute_verdict` (`events/pipeline.rs:383-449`) | — | — |
| L0 runner mechanism | `ship-runner` mailbox, `RunnerPorts`, `EventDedup`, `terminal_guard`, `KernelEventSink`, `build_spawn_env` | — | — |
| Executor / isolation | — | `SessionExecutor` impls (`ship-supervisor/executors/`), `IsolationStrategy` impls, `AgentPlatform` for worktree/tmux/merge | Same impls reused (the child run is a pipeline run — same executor) |
| Process / worktree | — | `ProcessRegistryPort` keyed on `slug`; `AgentPlatform::cleanup_job` | Same registry; child slugs are namespaced under the parent (e.g. `<parent>::<child>`) |
| Per-step spec body | — | `pipeline_specs::write_pipeline_specs` writes `.artifacts/pipeline/NN-<step_id>.md` from the JSONC's `step_specs[]` block | The agent provides a single inline spec body; no `.artifacts/pipeline/` filesystem path |
| Dispatch prompt | — | `pipeline_specs::write_dispatch_prompt` (RUNTIME_CONTRACT appended via `prompt_composer::RUNTIME_CONTRACT`) | A shorter prompt: "Execute this spec, return the result" — no RUNTIME_CONTRACT because no `ship step-done` is needed (we await the whole run) |
| Consumed / produced | — | `step_io::resolve_consumes` (artifact service) | A trivial version: the parent's input is the child's input, the parent's output receives the child's terminal payload (no artifact scope — just a structured value) |
| Replay / reconciliation | — | `replay.rs::replay_unprocessed_completions` / `reconcile_dispatched_jobs` | Same mechanism — the child run is in the same projection; if the daemon dies mid-child, the same startup path reconciles it |
| Cancellation | — | `cancel_job_fanout_lanes`, `handle_cancelled`, `terminal_guard` | Same; the parent run's cancel tears down all registered children |

The agent altitude does **not** invent a new lifecycle, a new
projection table, a new event vocabulary, a new cancel path, or a
new retry path. It composes the existing ones.

### 2.4 What the agent altitude genuinely adds

1. **A dispatch handler** for `runtime.dispatch` that builds the
   transient `Pipeline` (one step, or a fanout-of-K with judge)
   and hands it to the existing advancement code. The handler
   is the *only* net-new code. It is a payload-shape adapter.

2. **A "compose into parent" rule**: when a child run completes,
   its terminal payload becomes a structured result the parent's
   turn can read. The plumbing is the existing event bus; the
   parent subscribes (or awaits) `pipeline.run.completed` for the
   child's `pipeline_run_id`. The "result" wire is just the
   child's structured `AgentOutput`-equivalent — it isn't a new
   projection.

3. **A `parallel([...])` surface in the SDK**. Today the
   `runtime` service exposes a single op (`sdkgen.rs:115-122`).
   The shape that fits the design: `parallel` is either (a) a
   second op (`runtime.parallel { specs: [...] }`) or (b) a
   `runtime.dispatch` payload shape that contains a fanout.
   (a) is the cleaner split: one op = one semantic. See §5.1.

4. **Composition with the model loop's turn semantics**. The
   `run_step` function (`agent_loop.rs:143-178`) is currently
   `async fn ... -> AgentOutput { ... }`. To support
   `agent()` / `parallel()` calls, the model's tool result for
   `runtime.dispatch` must be a future that resolves when the
   child completes. Rig's tool future is `Send`; awaiting a
   child `pipeline.run.completed` from within a tool future is
   the natural fit. The wire shape is "subscribe to child events,
   filter on `pipeline_run_id`, map terminal event to a JSON
   payload, return as tool result." No new runtime primitive is
   needed for the await; the existing event bus is the
   synchronization point.

The first three are local changes. (4) is the design decision that
makes agent-altitude orchestration feel native to a model loop.

## 3 · Invariant fit — every spawned effect still passes the one door

The spike's invariants (README §Invariants) must continue to hold when
orchestration lands:

1. **One door** — every effect enters `gateway.call`. S14 does not
   add a new door. The `runtime.dispatch` and `runtime.parallel`
   ops are two more ops on the registry (`sdkgen.rs:115-122`); both
   route through the existing gateway. A child agent spawned by
   `runtime.dispatch` is, when it runs, *itself* subject to the
   one door via the pipeline altitude's executor: every tool call
   the child agent makes — `ship_op`, `run_script`, the model
   itself — enters `gateway.call` at the child process's own
   `Gateway` impl. There is no path that bypasses the gate.

2. **The gate is the only enforcement.** The gate is unchanged.
   `C0/C1 baseline, C2 needs the granted service, C3 operator-only`
   (`gate.rs:7-11`). The `runtime` service domain is C2 (it grants
   "you may spawn"), so a child-spawning op requires the agent
   to hold the `runtime` grant. A child agent runs in its own
   projection row with its *own* `AuthContext` derived from its
   own spec — its grants are not inherited from the parent
   automatically (see §3.4 below).

3. **No self-elevation** — `operator_cred = false` by construction
   (`auth.rs:23-26`). The child run's executor runs through
   `RunnerPorts::SpawnPolicy` (`runner_ports.rs:38-44`), which
   mints a domain-free `Authority` principal; the agent-specific
   role/token resolution lives in the L1 impl
   (`spawn_policy.rs:1-11`). The child run's principal is derived
   from the child's spec + session — not from the parent's
   session. C3 remains unreachable.

4. **Deny-by-default, twice** — imports (only `ship`) and
   capabilities (only granted services). Unchanged. A child agent's
   generated SDK is generated from *its* grants; a parent that
   holds `runtime` plus `collections` and spawns a child with
   spec `Capability { grants: ["runtime"] }` produces a child SDK
   that exposes only `runtime` (and, transitively, the ability to
   spawn a *grand-child*, not the parent's `collections` op).

5. **Nothing accepted blindly** — spine verify on ops; LSP
   diagnostics on edits. The `runtime.dispatch` op's result is
   the child `AgentOutput`. The spine wraps the tool call
   (`agent_loop.rs:74-92`): a denial returns as data
   (`{ "denied": "..." }`), a real failure errors, an
   unconfirmed ok returns `SpineError::Unverified`. A child
   run's terminal payload that doesn't confirm
   (`done_result == Some(true)`) is treated the same way —
   `aggregate_lane_outcomes` already routes to
   `AggregateDecision::Fail` for failed lanes
   (`ship-pipelines/src/fanout.rs:36-46`).

6. **Two tool categories, period** — gated ops + JS execution.
   `agent()` and `parallel()` are *new* op names on the `runtime`
   service. They do not introduce a third tool category; the
   model still calls `ship_op` (atom) or `run_script`
   (composition) — and one of those ops can be
   `runtime.dispatch` / `runtime.parallel`. Code-mode composition
   via `run_script` (multi-op, no `agent()`) is unaffected.

7. **Denial is data** — the existing denial path returns
   `{ "denied": "..." }` to the model
   (`agent_loop.rs:87-88`). A `runtime.dispatch` denial (the
   parent doesn't hold `runtime`, or the child spec is invalid)
   is a denial, surfaced identically. A child-spawn *failure*
   (the child run failed or the gate at the child level denied
   the child's op) is also a result: the child run's terminal
   event carries the failure reason, which the parent turn reads
   as a tool result.

### 3.4 Grant propagation — the load-bearing rule

This is the rule that, if wrong, breaks everything: **a child run
inherits NO grants from the parent automatically.** The parent's
`runtime` grant lets it *call* `runtime.dispatch`; it does NOT
grant the child anything. The child runs with the grants in its
own `AgentSpec.capability.grants`. If the parent wants the child
to act with the parent's authority, the parent must explicitly
construct a child spec with that authority — and the gate
enforces that authority was *granted* to the parent in the first
place (otherwise the parent couldn't pass it on).

Concretely: a parent with `["runtime", "collections"]` that calls
`agent(child_spec, input)` where `child_spec.capability.grants =
["collections"]` is fine. A parent with `["runtime"]` that tries
to pass `["collections"]` to a child fails at the *child's*
gate, because the child has no such grant authority and the
spine treats unconfirmed grants as a result error
(`agent_loop.rs:81-84`). This is the same property the
ship-agents keystone has: the L1 `SpawnPolicy` impl mints a
session token scoped to the *child's* role, not the parent's
(`spawn_policy.rs:1-11`, the role resolution
`spawn_policy.rs:31-39`). A future "pass parent grants" affordance
would have to be a *new* opt-in op, not a silent default.

### 3.5 Events still flow through the existing `events.rs` stream

The agent altitude composes into the substrate event bus:

- `pipeline.run.started` — emitted when the transient `Pipeline`
  is built and dispatched.
- `pipeline.step.started` / `pipeline.step.completed` /
  `pipeline.step.failed` / `pipeline.step.observed` — same as
  pipeline altitude.
- `pipeline.run.completed` / `pipeline.run.failed` — same.

The spike's `events.rs` (`AgentEvent` enum) is the *user-facing*
event vocabulary. The substrate event stream (`EventEnvelope`)
is the *runtime* event vocabulary. The agent altitude emits
into the substrate stream (so a child run is event-sourced
identically) and translates substrate events to `AgentEvent` via
the existing `translate` function (`events.rs:108-124`).

A parent's turn that awaits a child subscribes to substrate
events for the child's `pipeline_run_id`; the same `EventSink`
impls (`CollectingSink`, `TerminalSink`) and the same
`translate` mapping work — the parent's `LoopItem::ToolResult`
maps to `AgentEvent::ToolResult` with the child's terminal
payload as the result string, exactly as it would for a single
op. The CLI / SUI view sees a single tool call's result, not a
spurious nested event stream.

## 4 · Reuse vs. build — concretely

### 4.1 Reuse — directly, no changes needed

- `ship-pipelines` (L0 data crate) — `Pipeline`, `PipelineStep`,
  `FanoutSpec`, `StepType`, `StepOutput`, `StepPromote`,
  `OutputScope`, `aggregate_lane_outcomes`, `validate_*`,
  `interpolate*`. The agent altitude builds `Pipeline` /
  `PipelineStep` values directly from a transient spec and feeds
  them to the runner.
- `ship-runner` (L0 mechanism) — `dispatch-concurrency gate`,
  `EventDedup`, `terminal_guard`, `build_spawn_env`,
  `persist_and_route`, `resolve_step_invocation`. Reused
  identically.
- `ship-substrate::events::pipeline::event_types::*` — the
  full `pipeline.*` vocabulary. Reused.
- `ship-substrate::services::dispatch_ports::{RunnerPorts,
  SpawnPolicy, ProjectionReader, SessionExecutor,
  IsolationStrategy, AgentPlatform, ProcessRegistryPort,
  ProcessTreeProbe}` — the existing ports
  (`dispatch_ports.rs:163-316`). Reused.
- `ship-supervisor` (L0) — `TmuxWindowExecutor`,
  `ProcessExecutor`, `RoutingExecutor`. The child run is a
  pipeline step; the same executors drive it. Reused.
- `ship-pipeline-runtime::orchestrator::{fanout,
  fanout_aggregate, fanout_cancel, fanout_recover, pipeline,
  pipeline_specs, step_io, run_step, step_executor, review_retry,
  sweep, replay, pending, handlers, dispatch, router}` — the
  L1 policy. The agent altitude dispatches through the *same*
  router; the transient Pipeline lands in the same projection.
- The composed-session observability pair
  (`dispatch::emit_composed_step_session`,
  `dispatch::emit_composed_session_terminal`) — the read model
  for pipeline-step agent sessions is the read model for
  in-loop-spawned child runs, with `parent_pipeline_run_id`
  set to the child's actual parent.

### 4.2 Build — net new for the agent altitude

- **A `runtime.dispatch` handler in the agent altitude.** A
  function that takes `{ spec, input }`, builds a
  `Pipeline { steps: [<one agent step with goal = input,
  agent = spec.disposition.model (or a sensible default),
  harness/model from spec>] }`, and calls
  `ship_runner::router`-style advance + `RunnerPorts` spawn. The
  handler lives *where the agent runtime lives* — in the spike
  initially, in the L1 `ship-agents` crate after S16 graduation.
  It is NOT in `ship-pipeline-runtime` (that crate is the
  pipeline altitude's L1 policy; the agent altitude's L1 policy
  is `ship-agents`).
- **A `runtime.parallel` handler.** Same shape, but builds a
  `Pipeline { steps: [<one fanout step with the K lanes as
  FanoutLane, a default judge with min_successful_lanes
  semantics>] }`. The aggregator shortcut is the default; the
  judge step is a `Run` step that returns the aggregated
  outcomes as a structured result.
- **A spawn-wiring for `parent_pipeline_run_id`.** The child
  run's `PipelineRunStartedPayload` carries a
  `parent_pipeline_run_id: Option<String>` (analogous to the
  existing `parent_pipeline_run_id` field in
  `dispatch.rs:339-341`). This is the field the
  `emit_composed_step_session` already keys on to skip
  re-emitting the bare agent-session started event. No new
  field; the existing field is sufficient.
- **A turn-level await primitive.** The model's tool future
  awaits the child `pipeline.run.completed` /
  `pipeline.run.failed` event. This is a thin subscription
  over `ship_substrate::events::EventBus` filtered on
  `pipeline_run_id` — not a new runtime. (When S15 lands, the
  bus is the event-log emit from
  `ship_subscriber::EventSubscription`; same shape.)

### 4.3 What we deliberately do NOT build

- A new projection table. Children land in `pipeline_runs`.
- A new event namespace. Children emit `pipeline.*` events.
- A new cancel path. Children register with
  `ProcessRegistryPort` keyed on their slug; parent cancel
  iterates and tears down, the way
  `cancel_job_fanout_lanes` already does.
- A new control-flow node kind. `agent()` is one effect node;
  `parallel()` is one fanout node; both are existing kinds.
- A new retry path. `PipelineStep::max_retries` is reused; a
  child-run failure inside a parent turn is the parent's tool
  returning a denial-style result (`{ "denied": "child failed:
  ..." }`), the way any op failure surfaces.
- A new executor. Children run under the same
  `SessionExecutor` impls as bare pipeline steps.

## 5 · The seam and the first slice

### 5.1 The trait/seam for impl

A single new handler in the agent altitude, of the shape:

```rust
#[async_trait]
pub trait OrchestrationDispatch: Send + Sync {
    /// Run a single child agent to completion; return its terminal
    /// payload as a structured value. The gate runs on every op the
    /// child causes; the child's lifecycle is registered to the
    /// parent's pipeline run for cancellation.
    async fn dispatch_agent(
        &self,
        spec: &AgentSpec,
        input: &str,
        parent_run: &ParentContext,
    ) -> Result<DispatchedOutput, OrchestrationError>;

    /// Run K child agents in parallel under a fanout node; return
    /// the aggregator's decision plus the surviving lane payloads.
    async fn dispatch_parallel(
        &self,
        specs: &[AgentSpec],
        inputs: &[String],
        parent_run: &ParentContext,
    ) -> Result<DispatchedOutput, OrchestrationError>;
}
```

`ParentContext` carries the parent's `pipeline_run_id`, `slug`,
`branch`, the auth context (for the gate on the dispatch op
itself, C2 on `runtime`), and the registry/executor/probe
references the child run must register with. The impl is a thin
adapter that builds a transient `Pipeline` and hands it to
`ship-runner`-driven advancement — *not* a parallel orchestrator.

`DispatchedOutput` is the structured form of the child
`AgentOutput` plus the child's `pipeline_run_id` (so the parent
turn can correlate). `OrchestrationError` enumerates the failure
modes that map to a tool `Result::Err` in the spike
(`GatewayError::Denied`, dispatch failure, child run failure,
gate denial in the child) — same shape as
`spine::SpineError::Execute` /
`SpineError::Unverified` (`agent_loop.rs:80-92`).

This is the only net-new public type in the agent altitude.
Everything else is composition of existing types.

### 5.2 The smallest first slice

The smallest implementable slice that *proves the shared
machine*:

1. **One agent calls one child.** A test that builds a parent
   spec with `grants: ["runtime"]`, calls `agent(spec, input)`
   on a child spec with `grants: ["collections"]`, and asserts:
   - the child ran (the child projection row exists with status
     `Completed`);
   - the child emitted its `pipeline.run.completed` event;
   - the parent's tool result carries the child's structured
     output;
   - the child's grants did NOT include any parent grant that
     was not in its own spec (no silent inheritance).
2. **Three children in parallel, judge = `min_successful_lanes`.**
   A test that calls `parallel([s1, s2, s3])` with two
   succeeding children and one that fails an assertion, asserts
   the aggregator returned `Advance { ok_count: 2 }` (assuming
   `min_successful_lanes: 2`), and that the parent's tool result
   carries the structured `LaneOutcome` list.
3. **The gate runs at the child level.** A test that has the
   child attempt a `collections.read` op (granted) and a
   `mcp.create` op (not granted), asserts the granted op
   succeeded and the ungranted op was denied, *at the child
   process* — i.e. denial is data in the child's tool result,
   not an exception in the parent.
4. **Cancellation cascades.** A test that cancels the parent
   mid-child-run, asserts the child's executor was stopped
   (SIGTERM emitted; `terminal_guard` suppresses the resulting
   `pipeline.run.failed` since the parent is terminal), and the
   parent's tool future resolves promptly (no hang on the
   child's oneshot senders).
5. **The child cannot self-elevate.** A test that has a child
   attempt an op requiring a grant it doesn't have (even if
   the parent holds that grant), asserts the child's gate
   denied — the parent did NOT pass its grant down.

These five tests prove: (a) the shared machine runs the same
code path in both altitudes, (b) every effect still passes the
one door, (c) events still flow through the existing stream,
(d) cancellation works, (e) no self-elevation. That is the
invariants contract for S14.

### 5.3 Where the slice lives

- **In the spike (`spikes/ship-agent/`)**: a new module
  `src/orchestration.rs` (and `mod orchestration;` in
  `main.rs`) containing the trait, the impl, and the five
  tests. No changes to `agent_loop.rs` / `gateway.rs` /
  `gate.rs` / `runtime.rs` — the slice is
  composition-on-the-side, the same way
  `script_executor.rs` is the !Send bridge. The
  `runtime.dispatch` and `runtime.parallel` ops are added to
  `sdkgen.rs::ops_for` and the test that proves the surface
  shape (`sdkgen.rs:235-238`,
  `sdkgen.rs:248-264`) extends naturally.
- **In the daemon side** (`ship-pipeline-runtime`,
  `ship-agents`): nothing changes in S14. The slice is
  the spike proving the design; S16 graduation moves the
  module into `ship-agents` and wires it into the daemon's
  service registry. The pipeline runtime itself is
  untouched — children of `runtime.dispatch` flow through
  it as regular pipeline runs.

## 6 · Decision log — options considered

### 6.1 Option A — build a parallel "in-loop orchestrator" with its own DAG, its own fanout, its own cancel

**Why rejected.** Forks the system. The README is explicit:
"orchestration and ship pipelines are the same capability at
different altitudes and must share machinery." A second DAG
means a second validator, a second aggregator decision, a
second cancel path, a second replay path. The first time
someone wants to express "if X then dispatch A else B" in
*both* altitudes, they pay twice. The whole history of this
runtime has been a steady march *toward* a single DAG
(`PipelinePhase` legacy → `Pipeline` DAG, `merge` step kind
→ `auto_merge: bool` on the pipeline, etc.) — going backward
to fork orchestration would be the wrong direction.

### 6.2 Option B — extend `PipelineStep` with `Branch` and `Loop` node kinds

**Why deferred.** A conditional / loop primitive is genuinely
useful in both altitudes (pipelines today fake conditionals
via the gate-attribute + max_retries pattern, which is
painful). But adding a new node kind is a *spec-type* change
that ripples to every validator, every dispatch path, every
fanout aggregator. Doing it before the shared-machine
invariant is proven risks two things: (1) baking a
condition-on-result-binding shape into the L0 types before
we know whether the agent-altitude's in-loop case wants a
deferred future-based conditional or a strict value match;
(2) entangling the orchestrator-reconciliation slice with
the control-flow-extensions slice. S14 ships the shared
machine; S-conditional (later) adds the new node kind on top.
The decision record for adding the new kind will reference
this doc as the "shared machine exists, here is the new
node it composes" reference.

### 6.3 Option C — expose a low-level "spawn child process" op and let the agent implement DAG composition in code

**Why rejected.** It leaks the kernel: a model that wants to
fan out has to know about the executor, the projection, the
event bus, the cancel registry, the lane oneshot — every
invocation re-implements fanout, badly, in JS. The point of
the `runtime` service domain is the *abstraction*: a model
that wants fanout says "I want fanout" and the runtime
provides it correctly. The single-door invariant and the
deny-by-default loader both benefit: the model never has to
hand-build an ungranted op call to spawn a child.

### 6.4 Option D — make `agent()` an alias for `runtime.dispatch` only; `parallel()` is a separate op

**Why this wins.** Two ops, two semantics, one SDK surface.
`agent()` lowers to `runtime.dispatch { spec, input }`
(transport: build a one-step Pipeline, await its
`pipeline.run.completed`); `parallel([s1, s2, ...])` lowers to
`runtime.parallel { specs: [...], inputs: [...] }` (transport:
build a fanout-step Pipeline, await its
`pipeline.step.observed`). Same gate (C2 on `runtime`), same
auth (the agent's `AuthContext` with `runtime` grant), same
op layer, different payload shape. The `runtime` service def
in `sdkgen.rs:115-122` already has the placeholder for two
ops; the slice adds the second.

The alternative — `parallel` as a `runtime.dispatch` payload
shape — was rejected because: (1) one op = one payload shape
is the existing convention (`ops_for` is keyed on `op.name`,
not on a payload's `mode` discriminator), (2) it keeps the
gate decision simple (one blast tier, one service domain, one
auth check), and (3) the `runtime.parallel` op's *result
shape* (a list of `LaneOutcome`s plus the aggregator's
`AggregateDecision`) is genuinely different from
`runtime.dispatch`'s result shape (a single `AgentOutput`),
so the type-level distinction is real, not cosmetic.

### 6.5 Option E — composition with the model turn via a custom rig tool future

**Why this wins.** The model loop is rig-driven
(`agent_loop.rs:162-178`); rig's `Tool::call` returns a
`Send + 'static` future. Awaiting a child
`pipeline.run.completed` event over the substrate event bus
is a natural Send future. The wire shape is "subscribe to
events for the child's `pipeline_run_id`, filter, return
the terminal event's payload as the tool result." No new
runtime primitive, no new concurrency model. The alternative
— a hand-rolled `LocalSet` per child, or a oneshot per
child with the parent turn's executor blocked on a JoinSet
— was rejected as reinventing `tokio::sync::oneshot` +
`tokio::select!` with worse properties (no cancellation
propagation, no per-child logging, no event-bus
intermediation).

## 7 · Acceptance (this document)

This deliverable is a design spec. Its acceptance is
grounded in:

- every "pipelines already do X" claim cites the file path
  and the function or type that does X
  (`ship-pipeline-runtime/src/orchestrator/fanout.rs:39-273`,
  `ship-pipeline-runtime/src/orchestrator/pipeline.rs:25-382`,
  `ship-pipelines/src/types.rs:118-180`,
  `ship-pipelines/src/fanout.rs:31-47`,
  `ship-pipelines/src/validate.rs:25-150`,
  `ship-substrate/src/services/dispatch_ports/runner_ports.rs:69-84`,
  `ship-runner/src/lib.rs:5-28`,
  `ship-substrate/src/events/pipeline.rs:300-473` for the
  event vocabulary,
  `ship-pipeline-runtime/src/orchestrator/pipeline_specs.rs:36-129`
  for the per-step spec materialization,
  `ship-pipeline-runtime/src/orchestrator/step_io.rs:49-134`
  for consume resolution,
  `ship-pipeline-runtime/src/orchestrator/fanout_cancel.rs:23-85`
  for the cancel path, and
  `ship-pipeline-runtime/src/orchestrator/replay.rs:73-152`
  for the startup-reconciliation path);
- the five-test slice in §5.2 maps 1:1 to the invariant
  contract in §3 and to the spike's existing invariant
  list (README §Invariants);
- the seam (§5.1) is the *only* net-new public type, and
  it composes existing types;
- the "do not build" list (§4.3) is the "we deliberately
  do not duplicate this" list, which is what makes the
  shared machine a shared machine and not two machines
  that happen to share a name.

## 8 · Process

Run `cd spikes/ship-agent && cargo test` once at the end to
confirm the spike is unchanged (87 pass, no new tests in this
slice — design only). Commit this file to the current
`job/s14-orchestration` branch. **No push** — the
orchestrator gates locally.

## Constraints honored

- **DESIGN ONLY.** No new runtime modules, no changes to
  `gate.rs` / `gateway.rs` / `runtime.rs` / `agent_loop.rs`.
  Only the new `jobs/s14-orchestration.md` and a one-line
  README arc note if appropriate.
- **`crates/ship-pipeline-runtime` is read, not modified.**
  Every claim about it cites the file path and the line
  range.
- **Crate stays green.** One `cargo test` at the end; 87
  pass expected.
