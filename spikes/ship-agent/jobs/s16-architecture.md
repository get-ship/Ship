# Ship Agent — Consolidated Architecture (the spec foundation)

> Status: **authoritative.** This is the corrected, agreed architecture that the
> implementation spec is written from. It SUPERSEDES the parts of the four S16
> investigation docs (`s16-shipcode-boundary`, `s16-orchestration-entry`,
> `s16-daemon-integration`, `s16-graduate`) that we overturned in design —
> specifically the `invoke` chokepoint and the "embedded-runtime-as-a-Dispatcher-
> impl-inside-ship-agents-L1" framing. Where this doc and those disagree, this
> wins.

## 0 · What we're building
The **ship agent**: an embedded-first, programmable agent that is, at its core, a
**durable-execution pipeline engine with a model in the loop.** Its tool-calling,
its orchestration, and Ship's user-facing pipelines are **one engine**; the agent
is the dynamic front-end (a model writing a program), a static `pipeline.jsonc`
is the declarative front-end, and both execute the same way. The coding-harness
(spawned external CLIs) is a **parallel, supported feature**, not the design
center — we no longer need it for launch.

## 1 · Layering — library + app, integration first-class
```
apps/ship-agent (roadmap)   standalone binary — process isolation when wanted.
        │ (same runtime, two skins)
ship-agent runtime (lib)    THE agent: rig loop · ship-js code-mode · scoped SDK ·
        │                    context-projection · durable-execution engine.
        │                    Everything real is behind a port. No L1-service deps.
ship-agents (L1 service)    CONTROL PLANE only: exposes the Agent RPC methods and
        │                    organizes calling them. Knows the methods, NOT the loop.
L0 substrate                proto/policy SSOT · Connect services + gate interceptor ·
                            ship-js (deno) · pipelines (DAG/fan/aggregate) · events/journal
ship-code (parked)          spawned-harness strategy — its own box, sidecar-partnered.
```
- **"If it's only ever embedded it's L0; the moment it's its own process it's an
  app."** The runtime is a library (hosted in-process, **many agents per
  process** — the non-negotiable: 25 agents are not 25 processes); the app is the
  binary skin for isolation/scale-out. Both consume the same crate.
- **Agents is the prime name.** The runtime is THE Agent. `ship-agents` (plural)
  is the agent *domain + control plane*; it does not own the loop.

## 2 · The engine — deno, one runtime
- **ship-js becomes deno.** One JS engine serves BOTH pipeline `script` steps and
  agent code-mode. (deno chosen deliberately: Tauri/V8 alignment, V8 security
  maturity, and — load-bearing for this design — mature Rust interop/FFI.)
- The agent's `run_script` and a pipeline `script` step are the same thing:
  a ship-js program that calls ops.

## 3 · The gate + RPC — shipd stays the gate, unify on RPC
- **The gate is the shipd Connect interceptor.** It does NOT move to a new core
  `invoke` chokepoint (that would re-create the ops registry we gutted). There is
  ONE dispatch mechanism: ConnectRPC, proto-derived.
- **RPC is the IPC.** Internal callers (the embedded runtime) reach ops via an
  **in-process tower `oneshot` of the Connect router** — no socket, full
  interceptor stack (so the same `GateFn` runs), pays only proto marshalling.
  External callers use HTTP/UDS. The standalone app skin is just another Connect
  client over `ship.sock`, peer-cred'd like any agent.
- **The scoped SDK is a codegen'd Connect client.** One proto SSOT feeds: the
  Connect services (Rust), the gate policy (`ship_ops_rpc::policy`), the agent's
  JS SDK (Connect client over the in-process transport), and the `.d.ts` types.
  The spike's `op_ship_call` becomes the FFI bridge that marshals a JS call into
  a Connect request and oneshots the router.
- **Cost accepted:** in-process Connect pays proto encode/decode per op
  (microseconds, no socket). We do NOT optimize this away up front — uniformity
  over a second door.

## 4 · The pipeline engine — durable execution, everything is an op
- **One universal node: the op.** There is no step taxonomy. Human-in-the-loop is
  a *suspending* review/approval op (artifacts as payload); spawn-agent is an op
  (it spins up `RunnerPorts` internally); a fetcher is `http.fetch`. The
  human-vs-agent capability difference is **grants, not tools.** The only
  non-ops are the engine itself: **DAG/control-flow** and **suspend/resume.**
  Retry is a per-op policy. Triggers (cron/webhook/event) are **out of the engine**
  — they decide how a run *starts*, not what it executes.
- **Durable execution via deterministic replay.** This is the spine. To get BOTH
  code-mode (dynamic JS control flow) AND resumability, the engine replays:
  every op result (including **LLM calls**) is written to the **journal**; on
  resume, the program re-runs and completed ops return their *logged* results
  instead of re-executing. **Replay never re-burns tokens** — a completed LLM
  call is a journaled op; its output replays from the log. The lighter
  alternative (static-DAG durability only, scripts run-to-completion) is rejected
  because it makes code-mode non-durable and **re-burns all tokens on any crash.**
- **The determinism boundary (load-bearing rule):** anything non-deterministic —
  LLM, time, random, network — MUST be a journaled op, never inline orchestration.
  The spike already bans `Date.now`/`Math.random`; this generalizes it.
- **The journal IS the EventLog (S15), graduated** from in-memory to a durable,
  ordered, persisted log. Static `jsonc` and dynamic code-mode both compile to the
  same replayable execution; the op is the leaf either way.
- **Suspend/resume** covers human-checkpoint, delay, webhook-wait, and
  spawn-agent-await — one mechanism. The **control plane (RPC)** drives resume.
- **Pull vs build:** `ship-pipelines` DAG types + `aggregate_lane_outcomes` are
  pure data → pull if they fit. `ship-runner`'s spawn machinery is process-centric
  → it becomes the impl of the *one* `spawn-agent` op, not the engine.

## 5 · Context engineering — the message array is a cache, not the state
- **The journal is the lossless truth; the model's context window is a derived,
  disposable projection of it.** A typical agent appends-forever then truncates
  destructively because the message array is its only memory; with the journal,
  you append to the journal (lossless) and **project** to the context (lossy,
  optimized, re-derivable). Compaction becomes *rendering*, repeatable arbitrarily.
- **The agent runtime owns context-projection** (not the agent author). Multiple
  swappable strategies; experimental; we engineer it, expose it flexibly via
  settings on the runtime. Unlocks: context-as-projection (`project(journal,
  budget, strategy)` per turn, not `append`), multi-resolution/hierarchical
  compaction (re-expandable from the journal), **sub-agent compartmentalization**
  (a spawned agent has its own journal; the parent sees only its result —
  orchestration caps context for free), and structured/queryable memory.
- **LLM-driven compaction is itself a journaled op** → replay-safe (the summary is
  logged; resume never re-burns it). The projection function must be deterministic;
  pushing the LLM-summarization into an op satisfies that.
- Two distinct levers, not to be conflated: **forward projection** (smaller context
  each new turn — where token savings live) vs **replay-memoization** (skip
  re-calling the model on resume — needs identical input).

## 6 · Auth — operator=false survives, narrow seam added
- The embedded runtime needs an in-process principal (no connecting PID to
  peer-cred-attest). Add a small `embedded_principal(agent_id, role) -> AuthContext`
  seam next to `auth_context_from_peer`, grants via `ship_auth::resolve_grants`
  (never self-asserted), **`operator_cred=false` hard-wired.** The "operator=false
  ⇒ C3 unreachable" invariant holds because operator authority is set ONLY by a
  pid-attested operator session. The in-process call injects this `AuthContext`
  into the Connect request extensions before the tower `oneshot`.
- The app skin needs no new seam — it peer-creds over the socket like any agent.

## 7 · The control plane (ship-agents L1)
- Exposes the Agent execution surface over RPC, routing to `dyn AgentRuntime`:
  `Run` (drive a turn) · `WatchSession` (stream **typed `AgentEvent`** — the
  legibility bet, not `JsonResponse`) · `Approve` (the inbound half of the S11
  approval round-trip — does not exist today; `Grant` is C3 agent-baseline, not
  per-call) · reuse `Steer`/`Stop`. RPC-as-control-plane is the frontend/UX win:
  the studio drives live agents the same way the CLI does.
- The L1 service knows the methods, never the loop. Embedded: the port is wired to
  the in-process runtime. Process: the port is an RPC client to the app.

## 8 · Invariants (must hold)
1. **One door** — every effect is a Connect op through the gate interceptor.
2. **The gate is the only enforcement** (shipd's interceptor; no second gate).
3. **No self-elevation** — `operator_cred=false` by construction ⇒ C3 unreachable.
4. **Deny-by-default** — only granted services; the SDK is a soft layer.
5. **Nothing accepted blindly** — verify on results.
6. **Denial is data**, never an exception.
7. **Determinism for replay** — all non-determinism is a journaled op.
8. **One SSOT** — proto drives services, gate policy, SDK, and types.

## 9 · What supersedes the investigation docs
- **`invoke` (orchestration-entry §3, my own proposal): KILLED.** Replaced by
  RPC-as-IPC (in-process tower `oneshot`); the gate stays the Connect interceptor.
- **"embedded as a `Dispatcher` impl inside `ship-agents` L1": KILLED.** The
  runtime is a library+app; `ship-agents` L1 is a thin control plane that routes
  to it. The runtime is not an L1-service module.
- **"pipeline engine = the process orchestrator (`RunnerPorts`)": REFRAMED.** The
  engine is durable-execution over ops; `RunnerPorts`/spawn is the impl of one op.
- Kept from the docs: the ship-code↔domain boundary (boundary map), the
  `embedded_principal` seam + gate-route + proto-blast SSOT (daemon-integration),
  the dead-orphan finding (orchestration-entry §1.2).

## 10 · Implementation slices (the roadmap to spec into jobs)
- **S16.a — ship-js → deno.** Blast-radius check (dispatched), then migrate the L0
  `ship-js` crate's engine to deno; preserve its eval API for current consumers.
- **S16.b — the durable-execution engine (greenfield).** The journal (graduate
  EventLog → durable/ordered), the op-node + data-flow model, deterministic replay,
  suspend/resume, retry. Pull `ship-pipelines` DAG/aggregate where it fits.
- **S16.c — RPC-as-IPC.** The in-process tower `oneshot` transport; the scoped SDK
  as a codegen'd Connect client; `op_ship_call` → Connect-request marshaller.
- **S16.d — auth seam.** `embedded_principal` + extension injection.
- **S16.e — context-projection.** Runtime-owned, strategy-pluggable, journal-backed.
- **S16.f — ship-code carve.** Boundary-map Option C: move the spawned stack to its
  own box; delete the two dead orphans; `spawn-agent` becomes one op.
- **S16.g — control plane.** `Run`/`WatchSession`/`Approve` on the Agent surface;
  typed `AgentEvent` proto; the `dyn AgentRuntime` port.
- **S16.h — graduate + app skin.** The library lands in the tree; the `apps/ship-agent`
  binary skin; S12/S14-real unblocked.

Cleanup already started: the two dead orphan files
(`ship-agents::runtime::{run_dispatch,adapters}.rs`) are **deleted** (commit
`ddfd5dc61`). S16.a is **de-risked** — `ship-js` has zero consumers, so the deno
swap is risk-free (`s16a-shipjs-deno-blast-radius.md`).

## 11 · Reasoning log — why these calls (the forks we worked through)
This is the SSOT for the *why*, so the build doesn't re-litigate settled forks.

1. **deno, not QuickJS.** Tauri/V8 alignment, V8's security maturity, and —
   load-bearing for this engine — mature Rust interop/FFI. The many-agents-per-
   process density concern is handled by **isolate pooling + startup snapshot**,
   not by switching to a lighter engine. (ship-js is unused today, so the swap
   costs nothing.)
2. **RPC-as-IPC, NOT a core `invoke`.** A core `invoke(op,args,auth)` chokepoint
   would re-create the string-dispatch ops registry that was deliberately gutted
   when REST/MCP/CLI collapsed onto ConnectRPC. ConnectRPC *is* the SSOT dispatch.
   The embedded caller reaches ops via an **in-process tower `oneshot`** of the
   same Connect router — same interceptor, same gate, same services. rpc=external,
   ipc=internal, one serving stack. The gate does **not** move; shipd stays the gate.
3. **Everything is an op.** No step taxonomy. Human-in-the-loop = a suspending
   review op (artifacts payload); spawn-agent = an op; a fetcher = `http.fetch`.
   The human-vs-agent capability difference is **grants, not tools**. The only
   non-ops are the engine itself: DAG/control-flow + suspend/resume.
4. **Durable execution via deterministic replay, not a lighter scheme.** Code-mode
   (dynamic JS) AND resumability is only achievable by replay: ops are journaled,
   resume re-runs the program and completed ops (incl. **LLM calls**) return their
   *logged* results. **Replay never re-burns tokens** — the worry argues *for*
   replay: the lighter option (static-DAG durability, scripts run-to-completion)
   re-burns everything on any crash and makes code-mode the non-durable path. The
   determinism boundary is the price: all non-determinism must be a journaled op.
5. **Context = projection, not accumulation.** The message array is a cache; the
   journal is the truth. Compaction is *re-rendering a view*, repeatable
   arbitrarily, because the journal is lossless. The **runtime** owns projection
   (swappable strategies); sub-agents compartmentalize context for free (own
   journal, parent sees only the result); LLM-driven compaction is itself a
   journaled op (replay-safe). Forward-projection (token savings) and
   replay-memoization (crash-resume) are distinct levers.
6. **Library + app, NOT an L1-service module.** The agent is a self-contained
   runtime behind ports — library (many-per-process embedded) + app (isolation
   skin). `ship-agents` L1 is a thin **control plane** that exposes the methods and
   knows nothing of the loop. **Agents is the prime name.**

## 12 · Doc status
**This doc is the single design SSOT.** The four investigation docs
(`s16-shipcode-boundary`, `s16-orchestration-entry`, `s16-daemon-integration`,
`s16a-shipjs-deno-blast-radius`) are **evidence only** — their file:line maps feed
the build; their design framings are superseded by §9 above where they differ.
`s16-graduate.md` (the original "reconcile into L1" plan) is **deleted** — fully
superseded by this doc.
