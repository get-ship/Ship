# Ship Agent — S16 Execution Roadmap (the build sequence)

> The design SSOT is `s16-architecture.md`. This is the **build order**: phases,
> the *how* for each, dependencies, and the one risk we verify before touching
> the trunk. Greenfield-first (prove it in the spike), then graduate.

## Phase A — Greenfield engine (in the spike, zero workspace risk) — IN PROGRESS
The spike already has deno + the EventLog + the one-door Gateway, so the whole
engine is buildable here, behind the existing ports, before any trunk commitment.

- **A1 · durable-execution core** — journal + replay + suspend/resume. ✅ DONE
  (`4fb0856f2`): replay re-executes nothing, suspend parks/resumes without
  re-burn, denial-as-data. `src/engine.rs`.
- **A2 · fan-out / parallel + aggregate** — `parallel` (deterministic journaling
  of K ops) + the min-successful aggregate decision (fold `orchestration.rs`).
- **A3 · retry** — per-op retry policy on the live frontier; only the final
  result is journaled (replay never re-runs the retries). Fold `resilience.rs`.
- **A4 · deno wiring — `script-as-Program` (THE thesis connector).** A ship-js
  script becomes a durable program with **no deno change**: wrap the Gateway in a
  `JournalingGateway` (cursor + journal: replay logged results, execute+append on
  the frontier, return a suspend-signal that unwinds the script on a checkpoint).
  `run_script` with a `JournalingGateway` = a replayable code-mode turn. Resume =
  re-run the script; the gateway memoizes completed ops up to the resolved suspend.
- **A5 · context-projection** — the runtime renders the model's context from the
  journal (`project(journal, budget, strategy)`), swappable strategies, journal is
  the lossless truth. Sub-run journals compartmentalize context.
- **A6 · agent-loop ↔ engine** — a model turn IS a program; `agent()`/`parallel()`
  are sub-runs (own journals). The rig loop drives the program frontier.

**Exit A:** the spike proves the full engine + code-mode-as-durable-program +
context-projection, all green. The hard, novel work is done before the trunk.

## Phase B — Verify the one risk (before touching the trunk)
- **B1 · Connect in-process transport feasibility.** Prove `tower::oneshot` of the
  Connect router works with the `connectrpc` crate: full interceptor stack (the
  `GateFn` runs), in-process, no socket, `AuthContext` injected into request
  extensions. This is the load-bearing assumption for RPC-as-IPC. Tiny harness
  against a real shipd service. **If it doesn't hold cleanly:** fall back to
  UDS-loopback for embedded, or a thin in-proc service adapter — same SSOT, the
  transport differs. This gate protects us from building on a false assumption.

**Exit B:** RPC-as-IPC proven, or the fallback chosen with eyes open.

## Phase C — Workspace foundation (commits deno + the engine to the trunk)
- **C1 · ship-js → deno.** Graduate the spike's `runtime`/`script_executor`/
  `module_loader` into `ship-js` (rquickjs out, deno_core in). Risk-free —
  `ship-js` has zero consumers (`s16a-shipjs-deno-blast-radius.md`). Adds
  `deno_core`/`deno_error` to the workspace; **gate behind a `deno` feature** so
  the default daemon build doesn't pay V8 until embedded agents are on.
- **C2 · the engine → its home.** The Phase-A engine lands as an L0 crate (or in
  `ship-js`); pull `ship-pipelines` DAG/`aggregate_lane_outcomes` types where they
  fit (pure data); leave `ship-runner`'s spawn machinery behind (it's one op).
- **C3 · RPC-as-IPC.** The in-process tower transport (per B's proven shape); the
  scoped SDK as a **codegen'd Connect client**; `op_ship_call` → Connect-request
  marshaller. One proto SSOT → services + gate policy + SDK + `.d.ts`.

**Exit C:** engine + deno + RPC-as-IPC in the trunk, gated, default-off-heavy.

## Phase D — The agent runtime (library + app)
**Library-vs-app handling: ONE runtime crate, two skins.** The library is the
default (embedded in shipd, **many agents per process** — the non-negotiable);
the app is a thin binary wrapping the same crate (process isolation when wanted).
The choice is a deployment detail, not a code fork.
- **D1 · `ship-agent` runtime (library)** — loop + engine + scoped SDK +
  context-projection, behind ports, no L1-service deps. Reaches ops via RPC-IPC.
- **D2 · auth seam** — `embedded_principal(agent_id, role) -> AuthContext`,
  `operator_cred=false` by construction, grants via `resolve_grants`, injected into
  the Connect request extensions. The app skin needs no seam (peer-creds over the
  socket like any agent).
- **D3 · `apps/ship-agent` (app skin)** — the standalone binary; the same runtime,
  a different transport.

**Exit D:** the agent runs embedded (many-per-process) and stands up as its own
process from the same crate.

## Phase E — Integration + relocation (the trunk reshape)
- **E1 · L1 control plane.** `ship-agents` exposes `Run` / `WatchSession` (typed
  `AgentEvent`) / `Approve` (new) / reuse `Steer`/`Stop` over RPC, routing to
  `dyn AgentRuntime` (in-proc impl embedded; RPC-client impl for the app). Knows
  the methods, not the loop. **Agents is the prime name.** RPC-as-control-plane =
  the studio/CLI drive live agents identically.
- **E2 · ship-code relocation.** Boundary-map **Option C**: carve the
  spawned-harness stack (`runtime::executors` + `actor`/`dispatch`/`cancel`/`ports`
  + `dispatch_resume` + the spawn-half of `services/sessions`) into its own box,
  with `ship-harnesses`/`ship-isolation`/`ship-supervisor`. `spawn-agent` becomes
  **one op** the engine can call. (Dead orphans already deleted.)
- **E3 · pipelines refactor.** Re-center the existing pipeline runtime onto the
  durable-execution engine: the user-facing pipeline (scripts / zapier-esque /
  human checkpoints) runs on the SAME engine; process-agent steps become the
  `spawn-agent` op; the old process-orchestrator (`RunnerPorts`/executors/
  `OrchestratorContext`) is demoted to the impl of that one op. Static `jsonc` and
  dynamic code-mode are the two front-ends to one engine. Triggers (cron/webhook/
  event) stay separate (`ship-triggers`) — they start runs, they aren't the engine.

**Exit E:** one engine serves agents AND pipelines; ship-code parked; the trunk's
process-orchestrator demoted to one op behind the gate.

## Phase F — Graduate the rest
- **S12-real** (tsgo + `crates/sui/lsp` behind the slice-4 seam) and **S14-real**
  (orchestration on the unified engine — `agent()`/`parallel()` as sub-runs) land
  on the proven substrate. **S17** e2e.

## Dependency spine + risk
```
A (engine, spike) ──► B (verify Connect-IPC) ──► C (deno+engine+IPC in trunk)
                                                      └─► D (runtime: lib+app) ──► E (control plane,
                                                                                      ship-code carve,
                                                                                      pipelines refactor) ──► F
```
- **The only architectural risk is B** (Connect in-process transport). Everything
  in A is greenfield and reversible; B gates the trunk work; C–F are mechanical-ish
  on a proven core + a verified transport.
- **Reversibility:** Phase A lives entirely in the spike — nothing in the trunk
  changes until C, and the first trunk change (C1) is risk-free (no ship-js
  consumers). The heavy/irreversible commitment (deno in the workspace) is C1,
  gated behind a feature.
