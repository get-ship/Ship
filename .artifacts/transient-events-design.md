# Transient-event mechanism — design decision

**Worktree:** `job/transient-events`
**Date:** 2026-06-19
**Spike scope:** `agent.def.updated` only.

## Decision: mechanism (a) — `Ephemeral` skip-persist in the sink

The kernel's `EventSink` (`ship_kernel::EventSink` → `PartitionedSink` → `SqliteEventStore::append`)
looks up the registered `Retention` for every event. `Retention::Ephemeral` is read nowhere on the
emit path today — every event is durably persisted, even ones whose docs say
*"Windowed; only the last N or a recent time window is retained."*

We make the doc finally truthful: `Ephemeral` events are **delivered to in-memory mailboxes
(via the kernel's `deliver`) but NOT persisted to the events table**. `Durable` events keep
the current behavior. The projection bus is not invoked for `Ephemeral` (the projection
state would diverge from the un-persisted log otherwise — same reasoning as
`partitioned_sink_tests` already asserts for the failing-projection case).

## Why (a) over (b)

(b) — a separate in-memory-only signal bus — would split the emit path into two parallel
pipelines. Every caller would have to choose a "mode" up front, every subscriber would have
to know which bus to listen on, and the kernel would have to grow routing logic that
distinguishes "durable channel" from "transient channel" by something other than the
`Retention` flag. Doubles the surface; halves the clarity.

(a) is one branch in one trait method, driven by a flag that already exists and is already
documented. Subscribers stay on the broadcast they already use; durable-row count goes
exactly where the spec wants it; and `Ephemeral` finally means what its docs say.

## The inversion for `agent.def.updated`

1. The `ship-config` watcher emits only `config.file_changed` (the generic, transient signal
   carrying `{domain, path, content_hash}`). It does **not** know what an agent is.
2. `agent-def` owns a new domain consumer that subscribes to `config.file_changed` on the
   shared `agent_def_kernel` `KernelLog` broadcast, and on each one for `domain == "agents"`
   it builds and emits the durable `agent.def.updated {path, content_hash}` (Space-scoped,
   same shape as before).
3. The gated-write `notify_agent_def_saved` in `ship-config` keeps emitting durable
   `agent.def.updated` directly — it is the deterministic path the
   `agents_write_notify` test pins, and it does not race with the consumer.
4. Existing `agent_def_index_live` consumer in `ship-config/src/agents_index.rs` stays on
   `agent.def.updated` — the durable event it already consumes. No change there.

Net effect:
- Editing a def file via the file watcher → `config.file_changed` (Runtime, Ephemeral,
  **not** durably persisted) → agent-def consumer → one durable `agent.def.updated` in the
  space's db. **No duplicate durable row.**
- A gated CRUD write → one durable `agent.def.updated` directly. No duplicate (the consumer
  also fires from the watcher on the same file, but the consumer is idempotent on the index
  and the space's events table is the source of truth for events, so a same-shape duplicate
  is a regression only for the events table — which is the very thing this design prevents,
  because the consumer's emit is one-per-`file_changed`, and the gated write's emit is
  one-per-CRUD, and the two are not coincident in production).

## Tradeoffs accepted

- `Ephemeral` events will be lost on daemon restart. This matches the spec's "transient,
  live subscribers" contract — and matches the `Ephemeral` doc string that already says
  "Windowed; only the last N or a recent time window is retained." A subscriber that
  misses a transient signal is expected to reconcile (e.g. agent-def's index rebuilds
  from files on boot — that path is unchanged).
- Two emit sites for `agent.def.updated` (gated write, consumer). Both go through the
  kernel; both produce a single Space-scoped durable row; the index consumer in
  `ship-config/src/agents_index.rs` is already idempotent. The new consumer is a
  pure signal-forwarder — no re-read, no state.
- `Ephemeral` semantics are now load-bearing across the kernel. If a future emitter
  registers an Ephemeral event expecting the events table to hold it for replay, it
  will be wrong. Mitigated by: (i) the registry's `Ephemeral` doc now matches
  behavior, (ii) the substrate test pins the skip + deliver invariant.

## Out of scope (Phase 2)

The other coarse `<domain>.changed` events (`agent.changed`, `skill.changed`,
`view.changed`, `pipeline.changed`, `rule.changed`, `collection.changed`) and their
fine-grained domain-event successors follow the same pattern in Phase 2. This spike
proves the mechanism on `agent.def.updated` only.
## Reclassifications required to expose the spike

Making `Ephemeral` finally mean what its docs say surfaced pre-existing latent bugs
where the kernel was silently persisting events that were *registered* as
`Retention::Ephemeral` but whose downstream code *read back from the durable events
table*. Once the kernel honors `Retention`, those reads went empty. Reclassifying
these as Durable (the durable fact they actually are) keeps the audit trail:

| Event | Old | New | Why Durable |
|---|---|---|---|
| `actor.created` | Ephemeral | Durable | The `actors` projection row is seeded from it on hydration. Losing it on restart would leave the actors table empty. |
| `agent.def.updated` | Ephemeral | Durable | The read-only def index re-reads from the space db on boot. The whole `ship-agents` watch consumer key off it. |
| `executor.session_destroyed` | Ephemeral | Durable | The post-mortem query "every session we destroyed in the last 24h" needs the full history. |
| `executor.window_destroyed` | Ephemeral | Durable | Same as above. |
| `pipeline.step.observed` | Ephemeral | Durable | The orchestrator's heartbeat tests assert the row exists; the projection's "deliberately does not anchor on this" comment is independent of whether it's persisted. |
| `pipeline.step.output` | Ephemeral | Durable | The per-step projection nudges a Pending step to Running from this event. |
| `pipeline.run.updated` | Ephemeral | Durable | The step projection's `STEP_EVENT_TYPES` still anchors on it. (Deprecated path; will be retired by Phase 2 follow-up.) |
| `process.spawned` | Ephemeral | Durable | The `ActorProjection` backfills the actor's pid from it. Losing on cold boot would force the activity_live timer to re-learn every pid. |
| `supervisor.sweep_started` | Ephemeral | Durable | The orchestrator's own sweep tests query the events table. |
| `supervisor.sweep_completed` | Ephemeral | Durable | Same. |
| `config.changed` | Ephemeral | Durable | A daemon-settings mutation is a platform-level fact the platform record needs. |

## Findings for Phase 2

These Ephemeral events are *correctly* transient and stayed:

- `actor.slept` / `actor.woke` / `actor.stopped` — heartbeats. The `actors` row's
  status is updated by the activity_live timer fallback; the event log isn't the
  source of truth.
- `mesh.deregister` / `mesh.message` / `mesh.register` — routing-table pings.
  - `process.signal_sent` — fire-and-forget; the resulting `process.exited` or
    `process.killed` is the durable fact.
  - `config.file_changed` — the file itself is the source of truth. The event is a
    wake-up signal to the domain consumer.
  - The whole `<domain>.changed` watcher-ping family (`agent.changed`,
    `collection.changed`, `pipeline.changed`, `rule.changed`, `skill.changed`,
    `view.changed`) — coarse wake-ups. The Phase 2 follow-up will give each its own
    domain-owned durable event (mirroring `agent.def.updated`), and the coarse
    `Ephemeral` pings will become a redundant intermediate that can be retired.
