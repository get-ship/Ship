# Job S16: graduate the spike into `crates/` + `apps/` (Phase 6 — DRAFT PLAN)

> Status: **draft for architect review.** S16 is the keystone — it unblocks the
> real S14 (daemon orchestration) and real S12 (tsgo + `crates/sui/lsp`), both
> of which the spike could only prove with in-process fakes because a standalone
> workspace can't import the daemon crates. This is a *reconciliation*, not a
> move; read §1 before §2.

## 0. The finding that reshapes S16
`crates/core/ship-platform/crates/ship-agents` is **not** an empty target — it
is a substantial, shipping L1 crate ("Agent domain — agent-session dispatch,
agent-domain policy/resolver/skill/import, hooks engine, L1 service wrappers…
Carved from ship-platform, C8 keystone"). It already has:

- **a real tool gate** — `src/hooks/tool_gate.rs` (+ tests). Enforcement lives in
  the **hooks engine**, not a `gateway.call`/`gate.decide` blast-tier function.
- **agent-session dispatch** — `src/dispatch.rs` drives the L0 `ship-runner`
  mechanism, owns the spawn atom for `ship use` / `ship dispatch` / **MCP
  `dispatch_agent`** sessions, and already skips `parent_pipeline_run_id`
  sessions (the pipeline composer owns those). **S14's "reconcile with
  pipelines" is partly already the architecture here.**
- **sessions** (real S9), **`agent_domain`** = skills / permissions / rules /
  **mcp import** (real S-skills, S-mcp, permissions), **hooks engine**,
  **`runtime/executors`** (Tmux/Process — the real session executors S14's
  design named).

### Two execution models — the crux
`ship-agents` dispatches **external harness CLIs** (claude/omp/codex) via
executors. The spike runs the model **embedded, in-process** (rig drives the
turn; `run_script` executes JS in a deno_core isolate; every op passes the
one-door gate + verify spine). These are *complementary*, not competing — and
the embedded model is exactly the S5 unlock ("AI integrated deeply into the
shell/daemon, not bolted onto a terminal window's PID").

**S16 = bring the embedded in-process runtime into `ship-agents`, alongside its
existing spawned-harness dispatch, sharing the one domain layer.** Not a
wholesale drop-in; a careful graft.

## 1. Gap analysis — spike concept → real-crate disposition
| Spike module | Real `ship-agents` status | S16 disposition |
|---|---|---|
| `gateway.rs` + `gate.rs` (one door, blast tiers C0–C3) | `hooks/tool_gate.rs` gates tools, different shape | **RECONCILE** — decide whether the blast-tier gate becomes a hook, or the tool_gate adopts blast tiers. Architect call. (§4 Q1) |
| `runtime.rs` + `script_executor.rs` + `module_loader.rs` (deno_core code-mode, deny-by-default `ship` import) | **absent** — no V8/deno_core/JS runtime | **GRADUATE AS NEW** — the spike's biggest contribution; gate behind an `embedded` cargo feature (heavy dep). |
| `spine.rs` (post-exec verify; `Unverified` refuses) | **absent** — no `spine`/`verify` anywhere | **GRADUATE AS NEW**. |
| `sdkgen.rs` + `dtsgen.rs` + `registry.rs` (scoped SDK from proto∩grant) | skills/compile exist; SDK-gen does not | **GRADUATE AS NEW**, then point `registry` at the real proto descriptor set (the spike's seam). |
| `agent_loop.rs` (in-process rig turn + the two tools) | dispatch spawns external CLIs; no in-process loop | **GRADUATE AS NEW** (embedded execution path), reusing dispatch/sessions for lifecycle. |
| `auth.rs` (`AuthContext`, operator=false) | real auth via ship-substrate / kernel | **RECONCILE** — map onto the real principal/`SpawnPolicy`. |
| `session.rs` (S9) | `sessions/` exists | **DROP spike version**, use real. |
| `skills.rs` (S-skills) | `agent_domain/skill*` exists | **DROP spike version**, use real. |
| `mcp.rs` (S-mcp `mcp.call` + registry) | `agent_domain/import/mcp.rs` exists | **RECONCILE** — the spike's `mcp.call`-as-op projection vs the real mcp-import. (§4 Q2) |
| `orchestration.rs` (S14 spike-local) | `dispatch.rs` + pipeline composer + `ship-pipeline-runtime` | **REPLACE fakes with real** — wire `runtime.dispatch`/`parallel` to the existing pipeline machinery (the design's whole point). |
| `lsp.rs` (S12 seam) | n/a | **GRADUATE seam**, then S12 wires `crates/sui/lsp` + tsgo behind it. |
| `events.rs` + `EventLog` (S10/S15) | substrate `EventEnvelope` + (future) `ship_subscriber` | **RECONCILE** — `translate` AgentEvent ↔ substrate events; `EventLog` → real subscriber log. |
| `resilience.rs` (S15) | n/a | **GRADUATE AS NEW** (small). |

## 2. Workspace + dependency changes
- `deno_core` + `rig-core` are **not** in the main workspace today. Both are
  heavy. Add them as workspace deps, but gate the embedded runtime behind a
  **`ship-agents/embedded` cargo feature** so the daemon's default build (which
  uses spawned-harness dispatch) doesn't pay the V8/rig cost unless embedded
  agents are enabled. The standalone `spikes/ship-agent` workspace dissolves.
- Resolve edition skew: spike is edition 2021, `ship-agents` is 2024.
- The spike's `[workspace]`/exclude goes away; modules land under
  `ship-agents/src/embedded/` (new submodule) so the carve stays legible.

## 3. Staged sequence (each stage keeps `ship-agents` green)
- **S16.0 — Gap-analysis spike (READ-ONLY, no code).** Confirm the §1 table
  against the real code (esp. tool_gate shape, dispatch↔pipeline wiring, auth
  principal). Output: the reconcile/graduate/drop decision per row, signed off.
- **S16.1 — Scaffold.** Add `deno_core`/`rig` workspace deps + `embedded`
  feature; create empty `ship-agents/src/embedded/` mod. No behavior. Green.
- **S16.2 — Graduate the pure-new pieces** (no reconcile needed): `spine`,
  `resilience`, `registry`+`sdkgen`+`dtsgen`, `module_loader`. Behavioral tests
  move with them. Green.
- **S16.3 — Graduate the deno_core runtime + the in-process rig loop** behind
  `embedded`. This is the load-bearing stage. Green with feature on AND off.
- **S16.4 — Reconcile the gate** (§4 Q1): the embedded path's one-door gate
  routes through (or becomes) the real tool_gate/hooks; blast tiers preserved.
- **S16.5 — Reconcile auth/sessions/mcp/events** to the real domain layer
  (drop spike `session.rs`/`skills.rs`; map `auth`, `mcp.call`, `events`).
- **S16.6 — Wire orchestration to real pipelines** (unblocks **S14-real**):
  `runtime.dispatch`/`parallel` → existing dispatch + `ship-pipeline-runtime`.
- **S16.7 — Expose embedded agent as a service in `apps/shipd`** (the S5 unlock:
  an embedded agent surface in the daemon, not a spawned PID).
- *(then S12-real and S17 e2e follow, no longer blocked.)*

## 4. Open questions for the architect (block S16.0 sign-off)
1. **Gate model:** does the spike's blast-tier `gate.decide` become a hook in
   the existing hooks engine, or does `tool_gate` adopt C0–C3 blast tiers? (The
   invariant "the gate is the only enforcement" must survive whichever way.)
2. **MCP:** the spike projects MCP as a single `mcp.call` op through the gate;
   `agent_domain/import/mcp.rs` already imports MCP. Reconcile to one model.
3. **Embedded vs spawned-harness:** confirm both execution models coexist in
   `ship-agents` (shared domain, two runtimes) — vs. embedded living elsewhere.
4. **Feature boundary:** is `embedded` (deno_core + rig in the daemon) acceptable
   as a default-off cargo feature, or should the embedded runtime be its own
   crate (`ship-agent-embedded`) depending on `ship-agents`?

## 5. Acceptance (for the whole S16, not S16.0)
- `ship-agents` builds and tests green with `embedded` OFF (no V8/rig cost) AND
  ON (the spike's behavioral suite, graduated, passes against real domain types).
- No spike invariant regresses: one door, gate sole enforcement, no
  self-elevation, deny-by-default twice, nothing-accepted-blindly, two tool
  categories, denial-as-data, cooperative cancel.
- The standalone `spikes/ship-agent` workspace is deleted; nothing imports it.

## 6. Recommendation
Do **S16.0 (gap-analysis, read-only) first and stop** — get the four §4 answers
signed off before any code. Q4 especially (feature vs. separate crate) sets the
whole structure. Everything after S16.0 is mechanical given those decisions.
