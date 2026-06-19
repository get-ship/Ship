# Job Spec: transient-event mechanism + invert agent.def.updated emission (SPIKE)

**Branch:** job/transient-events
**Agent:** commander
**Mode:** autonomous (orchestrator — decompose, dispatch specialists, gate)

## Goal

Make the **domain** emit its durable event in response to a file change, instead of
the file watcher emitting domain events. Prove it on ONE event — `agent.def.updated`:

- The `ship-config` watcher emits only a **generic, transient** `config.file_changed`
  signal (delivered to live subscribers, NOT written to any durable event store).
- `agent-def` **consumes** that signal and emits the **durable** `agent.def.updated`
  (registered in agent-def, where it's now emitted).
- Result: **zero duplicate durable rows** (only the domain event persists), and the
  watcher carries **no agent/domain knowledge**.

This is a **spike**: if the design proves worse than the interim register-where-emit
fix, that is a valid outcome — report it; do not force a bad design through.

## Findings to build on (verified)
- `Retention::Ephemeral` is currently a **no-op** — retention is read nowhere in the
  emit/persist path; every event (incl. `config.file_changed`) is durably persisted.
  So "transient" does not exist yet and must be built.
- Two candidate mechanisms — **pick one and document the tradeoff in the worktree**:
  (a) implement `Ephemeral` skip-persist in the kernel persist path (`partitioned_sink` /
  `event_store::append`), so Ephemeral events deliver-but-don't-store; or
  (b) a separate in-memory-only signal bus (the kernel broadcast tx without the persist
  step). Prefer (a) if it makes `Ephemeral` finally mean what its docs say; choose by reasoning.

## Orchestration (you are commander)
1. **Decide + record** the mechanism (a vs b) with rationale — short design note in the worktree.
2. Dispatch **test-writer**: failing tests asserting (i) editing an agent def → agent-def
   emits a durable `agent.def.updated`; (ii) the `config.file_changed` signal is NOT a
   durable row; (iii) `ship-config` references no agent/domain types.
3. Dispatch **rust-lane**: implement the transient mechanism + the inversion to pass those tests.
4. **Gate**: `just check` + the new tests + the previously-failing ship-config event tests.

## Constraints
- Scope = `agent.def.updated` ONLY. Do NOT invert the other config domains (phase 2).
- Behavioral tests are the contract. Don't weaken tests to go green.
- Never push. Document the design decision in the worktree for human review.
- If `Ephemeral` stays a no-op after this, update or remove its misleading doc.

## Acceptance criteria
1. New tests prove: domain-emitted durable `agent.def.updated`; transient (non-persisted) `config.file_changed`; watcher free of domain types.
2. `just check` green; previously-failing ship-config event tests green.
3. A design note in the worktree stating the chosen mechanism + tradeoffs.

---
> Orchestrate per the spec. Commit cleanly in the worktree. DO NOT push. This is exploratory — a well-reasoned "this design isn't worth it" with evidence is an acceptable deliverable.
