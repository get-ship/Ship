# ADR: Event retention semantics

## Status
Accepted — 2026-06-19.

## Context
Every event carries a `Retention` flag, but it was read **nowhere** on the emit path — every event was durably persisted regardless, and `Ephemeral`'s docstring promised "windowed / last-N" behaviour that was never implemented. We need two distinct behaviours: durable history for facts that must replay, and transient delivery for signals that should not bloat the log.

## Decision
Two retention classes, honoured by the kernel sink:

- **Durable** — persisted to the per-scope event store; replayed on projection bootstrap. The default for any fact a projection or query depends on.
- **Ephemeral** — delivered to live in-memory subscribers but **not** persisted. For transient signals; lost on restart by design (subscribers reconcile).

The sink branches on retention: `Durable → persist + projections`, `Ephemeral → deliver only`.

**Rule:** any event whose downstream code reads the durable table MUST be `Durable`. Making `Ephemeral` real surfaced ~10 events mis-tagged `Ephemeral` whose consumers read the durable log — reclassified to `Durable`: `actor.created`, `executor.session_destroyed`/`window_destroyed`, `pipeline.step.observed`/`output`, `pipeline.run.updated`, `process.spawned`, `supervisor.sweep_started`/`completed`, `config.changed`, `agent.def.updated`. Genuinely-transient events stay `Ephemeral`: `mesh.*`, actor heartbeats, `process.signal_sent`, `config.file_changed`, the coarse `<domain>.changed` watcher pings.

## Inversion pattern
The generic file watcher emits only a transient `config.file_changed`; the owning domain consumes it and emits its **durable** domain event. Proven on `agent.def.updated`; the other `<domain>.changed` families follow in a Phase-2 follow-up. The watcher carries no domain knowledge.

## Follow-up (NOT in this implementation)
A third class, **`Windowed`** (persist, then prune after a TTL / last-N), is planned. It requires a pruner — the first event-log deletion in the system. Several events now `Durable` are really windowed candidates ("post-mortem, last 24h": `executor.*`, `supervisor.sweep_*`, pipeline heartbeats) and will move to `Windowed` when it lands. Until then they stay `Durable`.

## Consequences
- `Ephemeral` finally means what its (now-updated) docstring says.
- Event fixtures encode retention → reclassifications require a fixture regen.

## Supersedes
The no-op `Retention` tag (previously read nowhere).
