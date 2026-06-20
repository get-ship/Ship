# durable-execution

**Layer:** L0 — the journaling engine for durable program execution.

Runs a program (a DAG of op-calls) and journals every op outcome. Because the journal is the source of truth, a program can suspend and later resume by replaying the journal: already-completed ops return their logged results instead of re-executing. Replay reconstructs state without re-running effects or re-burning tokens.

## Key seams

- `engine` — `ExecCtx` (op / suspend / parallel / retry / record), `Program`, `execute`, `resume`, `JournalingDispatch` (the bridge that makes an externally-driven op stream durable), `Journal`, `JournalEntry`, `OpRecord`.
- `Dispatch` — the op-execution seam, re-exported from `ship-js`. The engine and the JS runtime share ONE contract; the host captures auth/gate/transport inside its impl.
- `store` — `JournalStore` trait + `InMemoryJournalStore`; `SqliteJournalStore` lives in ship-substrate/src/persistence/journal_store.rs.
- `projection` — the model's context as a disposable, re-derivable view of the journal (`ContextProjector`, `KeepRecent`, `Verbatim`).

## Wiring status

The journal engine is live and tested (P1.0b). Wiring the live Floki agent loop to run durable is Stage 2 (tracked in #30); wiring pipelines is P1.0b part 2.
