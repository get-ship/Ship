# agent-def

**Layer:** L1 — the unified agent-definition domain.

agent-def holds the single `Agent` type that both Floki (the embedded code-mode runtime) and ship-code (third-party harness drivers) read. The control plane and shipd also depend on it for config CRUD without pulling a full runtime.

## What it contains

- `agent` — the `Agent` struct (identity, capabilities, harnesses, blast radius, disposition).
- `config` — config CRUD seam for reading and writing `.ship/agents/*.jsonc`.
- `harnesses` — harness descriptors and selection.
- `guards` — the 4a validator: payload validation is mandatory on every registered `EventDef`.
- `permissions` — permission evaluation and grant baseline.
- `rule` — agent rules.
- `skill` — skill loading, paths, and variable persistence (backed by `ship-skill-vars`).
- `import` — agent import resolution.
- `compile` — schema compilation and validation.
- `emit` — session event types (`SessionStarted`, `SessionCompleted`, `TurnStarted`, `TurnEnded`, etc.) and `SessionEventSink`.
- `op` — op tool-use event payloads.
- `dispatch_intent` — the `dispatch.requested` spawn-intent event (keystone-A channel).

## Key seam

agent-def was carved from ship-agents (Floki carve D2a). The schema unification into a single `Agent` + runtime discriminant is D2b (pending).
