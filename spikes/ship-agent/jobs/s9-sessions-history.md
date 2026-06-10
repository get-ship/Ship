# Job S9: Sessions + history + compaction-as-projection

## Context
Work in `spikes/ship-agent/` (own Cargo workspace). Read `README.md` (SSOT).
`run_step` is single-shot today. v1 needs **multi-turn sessions**: durable
conversation history and **lossless compaction** (compact the *view* the model
sees while preserving the full log, so it's re-expandable). Build the machinery
standalone; wiring history into the model loop is a deliberate follow-up (do NOT
modify `agent_loop.rs`).

## Task — add `src/session.rs` (+ `mod session;` in main.rs)
1. `struct Session { id: String, history: Vec<Turn> }` where `Turn` records a
   role + content (use a simple local type; do not depend on rig message types).
   - `fn record(&mut self, turn: Turn)` appends.
2. `trait HistoryStore { fn load(&self, session: &str) -> Vec<Turn>; fn save(&self, session: &str, history: &[Turn]); }`
   + `struct InMemoryHistoryStore` impl (the durable event-sourced store is a
   later seam — note it).
3. **Compaction as projection** — `trait CompactionPolicy { fn view(&self, full: &[Turn]) -> Vec<Turn>; }`
   + a `KeepRecent { keep: usize, trigger_at: usize }` impl: when `full.len() >
   trigger_at`, return a compacted view (a synthesized summary placeholder turn +
   the last `keep` turns); otherwise return `full` unchanged. The **full log is
   never mutated** — `view` is a pure projection. Re-running with the full log
   must recover everything.

## Acceptance (behavioral tests — NOT brittle)
- a session accumulates turns across multiple `record` calls (length grows).
- `InMemoryHistoryStore` round-trips: save then load returns the same turns.
- below `trigger_at`, `view(full) == full` (no compaction).
- above `trigger_at`, the view is **shorter** than the full log AND the full log
  is **unchanged** (assert both — lossless projection), and the view ends with the
  most-recent turns (continuity preserved).
- compaction is a pure function of the input (same full log → same view).

Run `cargo test`; ALL (existing 53 + new) pass. Commit on this branch when green.

## Constraints
- New file `src/session.rs` (+ its `mod` line) only. Do NOT modify `agent_loop.rs`
  / `run_step`, the enforcement or execution modules, `provider.rs`, or `Cargo.toml`
  (use only existing deps: serde/serde_json/std — add NO new dependency).
- Behavioral tests only — assert *semantics* (grows / round-trips / shorter-but-
  lossless / pure), never exact counts of a specific fixture or serialized bytes.
- Memory (memento `decisions.*`) is a SEPARATE later slice — do not build it here.
- Update `README.md` module map only if you add a module (you will: add `session.rs`
  under Definition or a new "Sessions" row).
