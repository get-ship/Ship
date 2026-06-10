# Job S11: approval + steer / cancel

## Context
Work in `spikes/ship-agent/` (own Cargo workspace; builds from any location).
Read `README.md` (SSOT). The runtime now drives gated effects through the one
door; what it lacks is a way to handle a human-in-the-loop *Ask* stance and to
let a supervisor interrupt a long-running step. This slice adds both, expressed
through the existing event/translate machinery — not a side channel.

What exists today (S10):
- `events.rs` — `AgentEvent` vocabulary (serde, internally-tagged), the
  `EventSink` trait (`CollectingSink` / `TerminalSink`), and a pure `translate`.
- `agent_loop.rs` — the rig-driven loop with the two tools (`ship_op`,
  `run_script`); `run_script` crosses the `!Send` VM-thread bridge.
- `gate.rs` — `decide(...)`; the gate is the single point of enforcement.

There is **no** approval / steer / cancel surface yet — this is greenfield here.
The `Approver` trait in any sibling spike is NOT to be imported; design fresh for
this runtime.

## Task

### 1 · Approval handshake (the *Ask* path)
When an effect's stance is `Ask` (human-in-the-loop), the loop emits an
`ApprovalRequested` event and awaits a decision (`UserInput::Approve` / deny).
The gate policy itself is unchanged: `Ask` *replaces* `Allow` for ops that
require human approval; it is not a second enforcement point. A denial — from
the gate, from the human, or from a headless run — is surfaced as data, never
as an exception.

### 2 · Steer / cancel
A running loop must accept a steer (inject a user message that the next turn
sees) and a cancel (cooperatively stop the run at the next safe point,
surfaced as an event — not a panic, not a dropped future mid-op). Both go
through the event/translate machinery; no side channel.

### Files
- **Primary:** `agent_loop.rs`, `events.rs`.
- **Small touch:** `gate.rs` — add `Decision::Ask` variant; do NOT change
  `decide`'s policy. Do NOT touch `auth.rs`, `runtime.rs`, `module_loader.rs`,
  `spine.rs` core verify, or any enforcement.

## Design

### Event vocabulary (events.rs)
Extend `AgentEvent` with:
- `ApprovalRequested { op, args, request_id }` — emitted when an `Ask` stance
  surfaces; `request_id` lets an interactive approver correlate.
- `Cancelled { reason }` — emitted when a run is cooperatively cancelled.

`UserInput` is a small enum (NOT an `AgentEvent` variant) returned by the
`Approver`: `Approve { request_id }` or `Deny { request_id, reason }`. The
approver's `Deny` is a normal denial and surfaces as a result.

### Approval flow
1. The op's stance is determined by `gateway.stance(op, args, auth) -> Stance`
   where `Stance = Allow | Ask | Deny(reason)`. The `Ask` set is a small
   intrinsic table on `InProcessGateway` (e.g. `mcp.create`, `account.rotate_key`
   are ask-stance; everything else is `Allow`-stance by default — except the
   gate still produces `Deny` for ungranted elevated ops).
2. The gateway's `call` retains its existing semantics for `Allow`/`Deny`; for
   `Ask`, the call surface is unchanged from the *tool's* perspective: the
   gateway returns `Err(GatewayError::Ask { op, args, request_id })`.
3. The `ShipOpTool` (and the `RunScriptTool` op surface via the same path)
   catches `Ask`, emits `ApprovalRequested` to the sink, then awaits
   `Approver::request(op, args, request_id)`. If `Approve` → re-dispatch
   through `gateway.call` (the gate still runs). If `Deny` → return
   `{ "denied": "approval denied: <reason>" }` as a tool *result* (data, not
   error).

### Approver
A small trait in `agent_loop.rs`:
```rust
#[async_trait]
pub trait Approver: Send + Sync {
    async fn request(&self, op: &str, args: &Value, request_id: &str) -> UserInput;
}
```
Two impls:
- `HeadlessApprover` — `request` resolves to `Deny { request_id, reason: "no human in loop" }` synchronously. **No awaiting a real human ⇒ deny.** A pipeline step or unattended run must never block forever.
- `ScriptedApprover` — wraps an `mpsc::Receiver<UserInput>`. Tests push scripted decisions; the request future awaits. If the channel closes before a decision arrives, deny (fail-closed).

### Steer
A `Steer` is shared state: `Arc<Mutex<VecDeque<String>>>`. Producers push;
the `ShipOpTool` (or `run_step` between rig turns) drains and surfaces the
steered text to the model. Concretely: when a `Steer` is present and the
tool is about to return, the tool result is enriched with the steered text
so the next rig turn observes it. The `translate` machinery maps the
observed `Steered { text }` step to a `Status` event (or a new
`SteerObserved` `AgentEvent` variant — see code).

### Cancel
A `Cancel` is `Arc<AtomicBool>`. The loop checks it at safe points:
- Before each rig prompt.
- At the start of every `ShipOpTool::call` and `RunScriptTool::call` —
  *before* the gateway call, so no effect is left half-applied.
- The `ScriptExecutor` checks it at job dispatch (and the receiver loop
  observes it between jobs). It cannot interrupt a V8 isolate mid-execution,
  but it WILL observe the flag at the next safe boundary and the spawned
  job's `Gateway::call` honors the same flag.

On cancel: emit `Cancelled { reason }` and return. The cancel reason
defaults to `"user-requested"`.

## Invariants (must hold — see README §Invariants)
- **One door.** Approval gates an effect that still passes through `gateway.call`; it does not become a bypass. The gate remains the only enforcement — `Ask` is the *Ask resolution*, not a second enforcement point.
- **No self-elevation** — `operator=false` by construction; approval can never manufacture operator clearance or clear C3. `Ask` is a stance on the *call*, never on the *agent*.
- **Denial is data** — a denied approval, a cancel, and a normal denial are all surfaced consistently as events/results, never as exceptions.
- Don't break the `!Send` bridge: cancel must stop cooperatively at a safe point.

## Acceptance (behavioral tests — NOT brittle)
Add `#[cfg(test)]` tests asserting *semantics*:
- an `Ask` effect with an approving decision proceeds; with a deny it does not — and a denial is surfaced as data, not an error/panic.
- headless / no-human resolves `Ask` to deny deterministically (does not hang).
- a steer message is observable by the subsequent turn.
- a cancel stops the run at a safe point and is surfaced as an event; no effect is left half-applied.

Drive these with a test sink / scripted input — do NOT depend on a real
terminal or real model. `cargo test` in `spikes/ship-agent`: ALL existing
tests (87 at dispatch time) + new ones pass.

## Process
Run `cd spikes/ship-agent && cargo test` **in the foreground and WAIT for
the final `test result:` line** before reporting. Do NOT background the
build and return early. The first build is slow (deno_core/V8) — wait it
out. When green, `git add` + `git commit` on the current branch.

## Constraints
- Primary work: `agent_loop.rs` + `events.rs` (and a small `gate.rs` touch
  ONLY to route the `Ask` resolution through the approval handshake — NOT
  to change `decide`'s policy). Do NOT change `auth.rs`, `runtime.rs`,
  `module_loader.rs`, `spine.rs` core verify, or the gate decision itself.
- Behavioral tests only — no hardcoded counts, no exact generated strings.
- Update `README.md` (module map / Seams / invariants) if they change.
