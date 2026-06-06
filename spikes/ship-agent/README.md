# ship-agent

The Ship Agents Framework runtime — a programmable agent whose **every effect,
atom or code, model-driven or scripted, passes one gate and one verify**, in a
contained VM, against a surface scoped to exactly what it's granted.

This is the **framework**, not "the agent": `AgentSpec` is the unit of extension;
shipwright and every user agent are *instances* that inherit the spine by
construction. Architecturally it is an **app + library, a client of shipd** — it
consumes the platform over the gateway; it is not a substrate layer or a gated
platform service.

## The substrate (borrowed) vs. the spine (ours)

| Borrowed | For |
|---|---|
| `rig` | model loop + provider API (commodity — we don't reinvent the loop) |
| `deno_core` / V8 | JS execution, one isolate per run |
| `tokio` | async |

We own the *policy*: gating, verification, containment, surface generation.
The loop is commodity; the spine is the product.

## The flow — one effect, every layer

```
spec.rs ─ AgentSpec (Identity · Disposition · Capability · Memory)
   │        capability.grants ──► auth.rs ─ AuthContext { grants, operator=false }
   ▼
agent_loop.rs ─ run_step: rig drives the model turn
   ├── ship_op  (atom) ─────────────────────────────────┐
   └── run_script (code) ─► script_executor.rs (!Send bridge, VM thread)
                             └─ runtime.rs ─ deno_core isolate
                                 import "ship" ─ module_loader.rs (deny-by-default: ONLY "ship")
                                 module = sdkgen.rs (scoped to grant); types = dtsgen.rs
                                 op_ship_call ───────────────────┐
                                                                 ▼
                  ┌──────────────── THE ONE DOOR ───────────────────┐
                  │ gateway.rs ─ gateway.call(op, args, auth)        │
                  │ gate.rs   ─ decide(blast, service, auth)         │  ← only enforcement
                  │   C0/C1 allow · C2 needs granted service · C3 operator-only
                  │ dispatch  (stub echo today → command::call)      │
                  └────────┼────────────────────────────────────────┘
                           ▼  spine.rs ─ verify (nothing accepted blindly)
                           ▼  result back to the model (denial = data, not exception)

  lsp.rs ─ verify_edit(path, src): .ts→tsgo / .sui→SUI seam,
           diagnostics → SpineError::Unverified (edit refused, not accepted)
```

## Module map by concern

| Concern | Modules | Role |
|---|---|---|
| Definition | `spec.rs`, `auth.rs`, `loader.rs` | four-facet agent + the credential it presents + `.ship/agents/<id>.jsonc` → `AgentSpec` projection |
| Sessions | `session.rs` | multi-turn `Session` + `HistoryStore` seam (`InMemoryHistoryStore` stub) + `CompactionPolicy` (lossless projection; `KeepRecent` impl) |
| Enforcement | `gateway.rs` → `gate.rs` → `spine.rs` | one door, one gate, post-verify |
| Surface | `sdkgen.rs` + `dtsgen.rs` (shared registry), `module_loader.rs` | scoped SDK (JS + types) from `proto∩grant`; deny-by-default imports |
| Execution | `runtime.rs`, `script_executor.rs` | deno_core isolate; the `!Send` VM-thread bridge |
| Intelligence | `lsp.rs`, `agent_loop.rs` | dual-LSP-fed verify; rig loop + the two tools |

## Invariants (must always hold)

1. **One door** — every effect enters `gateway.call`. No side channel.
2. **The gate is the only enforcement.** Scoped SDK + deny-by-default loader are
   *soft* (ergonomics/legibility); a raw `op_ship_call` to an ungranted op is
   still denied at the door.
3. **No self-elevation** — `operator=false` by construction ⇒ C3 unreachable.
4. **Deny-by-default, twice** — imports (only `ship`) and capabilities (only
   granted services).
5. **Nothing accepted blindly** — spine verify on ops; LSP diagnostics on edits.
6. **Two tool categories, period** — gated ops + JS execution; JS reaches the
   world *only* through ops.
7. **Denial is data** — consistent across the deno_core op and the rig tool.

Dependency arrows point inward toward the door — Definition → Surface →
Execution → Loop all converge on Enforcement, never around it.

## Seams (stub today → real later, all behind traits)

| Seam | Stub | Real |
|---|---|---|
| `Gateway` | `InProcessGateway` (echo + gate) | shipd `command::call` (refactor-gated swap) |
| `LspBackend` | `TsgoStub` / `SuiLspStub` | tsgo + `crates/sui/lsp` (+ warmup/writethrough) |
| model | `MockCompletionModel` (tests) | BYOK provider via rig |
| op registry | hardcoded `ops_for` / `blast_of` | proto descriptor set |
| auth | locally-constructed `AuthContext` | handshake to shipd (peer-cred socket) |

The arc builds against **contract-faithful fakes** so it is unblocked by the
in-flight ops-SSOT refactor; swapping fakes for live shipd is a distinct,
refactor-gated slice.

## v1 arc

```
PHASE 1 — make it real      S6 spec loader · S7 provider/BYOK · S8 scoped-SDK-from-proto · S5 auth handshake seam
PHASE 2 — make it usable    S9 sessions+history+compaction · S10 AgentEvent stream · S11 approval+steer/cancel
PHASE 3 — differentiators    S12 tsgo+SUI LSP real · S13 file ops + hashline
PHASE 4 — customization      S-skills (the user-extension surface) · S-mcp (third-party integration, design-first)
PHASE 5 — orchestration      S14 agent()/parallel — RECONCILED WITH SHIP PIPELINES (shared DAG/control-flow), not a parallel system
PHASE 6 — harden + graduate  S15 resilience+event-log emit · S16 graduate to crates/+apps/ · S17 e2e harness
```

Orchestration and ship pipelines are the same capability (DAG + control flow +
parallelization) at different altitudes and must share machinery. Skills are the
primary user-customization surface. MCP needs a design pass before wiring.

## Conventions

- **Docs:** this README is the SSOT (update the module map / invariants when they
  change); rustdoc `//!` headers carry per-module "why"; `jobs/*.md` are the slice
  + decision log. No scattered docs.
- **Tests:** unit/behavioral inline (assert *semantics* — Allow/Deny, present/
  absent, Ok/Err — never counts or generated-string literals); integration +
  e2e in `tests/` against contract-faithful fakes + mock provider. Every slice
  proves its behavior.
- **Slices:** each job spec lists required behavioral tests, "commit on the
  branch when green," and "update README if invariants/module-map change."

## Run

```
cd spikes/ship-agent
cargo test          # 38 behavioral tests
cargo run           # demo: gate tiers, deno_core one-door, scoped SDK,
                    #       deny-by-default, model loop, code-mode, LSP verify
```
