# Job S10: AgentEvent stream + sinks

## Context
Work in `spikes/ship-agent/` (own Cargo workspace; builds from any location).
Read `README.md` (SSOT). The agent needs a **user-facing event channel** so an
interactive run can be observed/driven (the shell, a UI, the CLI). This slice
builds the event vocabulary + sink + a translation, standalone. Wiring it into a
live streamed loop is a deliberate follow-up — do NOT modify `agent_loop.rs`.

## Task — add `src/events.rs` (+ `mod events;` in main.rs)
1. `enum AgentEvent` (serde, internally-tagged), minimal and ACP-shaped:
   `Status { message }`, `AssistantDelta { text }`, `ToolCall { name, args }`,
   `ToolResult { name, result }`, `Completed { turns }`, `Error { message }`.
2. `trait EventSink { fn emit(&self, ev: &AgentEvent); }` + two impls:
   - `CollectingSink` (stores emitted events; for tests/inspection)
   - `TerminalSink` (prints — the stand-in for a SUI view)
3. A **pure translation** `fn translate(item: &LoopItem) -> Option<AgentEvent>`
   where `LoopItem` is a small LOCAL enum mirroring the loop's observable steps
   (text delta, tool call, tool result, final). This keeps the translation
   testable without a live model. Map each `LoopItem` to its `AgentEvent`
   (some items may map to `None`).

## REQUIRED acceptance (behavioral tests — NOT brittle)
- `CollectingSink` records events in emission order.
- `translate` maps each `LoopItem` variant to the expected `AgentEvent` variant
  (assert variants/semantics, not serialized bytes); items with no event → `None`.
- `AgentEvent` round-trips through serde (serialize+deserialize preserves variant).

## Optional (only if clean)
A `run_turn(model, sink, ...)` that drives rig streaming with a
`MockCompletionModel` and emits events to the sink — tested deterministically
(scripted turn → ordered events ending in `Completed`). If rig's streaming API
is gnarly, SKIP this and leave a `// TODO(stream-wiring)` — the required pieces
above are the deliverable.

## Process (IMPORTANT)
Run `cd spikes/ship-agent && cargo test` **in the foreground and WAIT for the
final `test result:` line** before reporting. Do NOT background the build and
return early. The first build is slow (deno_core/V8) — wait it out.
When green (existing 67 + new), `git add` + `git commit` on the current branch.

## Constraints
- New file `src/events.rs` (+ its `mod` line) only. Do NOT touch `agent_loop.rs`,
  enforcement/execution modules, `provider.rs`, `session.rs`, or `Cargo.toml`
  (no new deps; serde is already available).
- Behavioral tests only. Update `README.md` module map (add an `events.rs` row).
