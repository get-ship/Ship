---
group: Architecture
order: 3
title: Kernel
description: Event routing primitives — the persist-before-deliver contract, actor mailboxes, namespace subscriptions, and the split between the pure crate and the runtime wrapper.
audience: internal
---

The kernel is the event routing core. It decides when an event is durable, who sees it, and in what order. Nothing about files, databases, or HTTP — that's for the runtime. This page walks through the contract and how the pieces fit.

## Two crates, one name

There are two types named `KernelRouter` in the codebase:

| | Location | What it is | Used where |
|---|---|---|---|
| **Pure** | `crates/core/ship-kernel/src/router.rs` | Abstract routing core. Takes `Arc<dyn EventSink>`. No filesystem. | Composed inside the production wrapper as `self.inner`. |
| **Production** | `crates/core/ship-substrate/src/events/kernel_router.rs` | Composition wrapper. Takes `base_dir: PathBuf`, builds a `SqliteEventStore` sink, hands it to the pure router. | Every real kernel operation. |

The pure crate defines the abstractions (`Mailbox`, `EventSink`, `ActorConfig`, `EventValidator`) and a `KernelRouter` that composes them. The production wrapper does **not** reimplement the router — it owns the pure `KernelRouter` as `self.inner` and delegates actor registration, mailboxes, subscriptions, and persistence to it. The wrapper contributes `EmitContext`-aware validators, `ActorStore` construction, write-namespace enforcement, and (under the `unstable` feature) scope-filtered delivery. The pure router is live production code, not dead code.

The rest of this page describes the *production* kernel in `ship_substrate::events::kernel_router`. Where the pure crate's behaviour differs, it's called out explicitly.

## The routing contract

The production entry point is `emit(event, ctx)` on the substrate wrapper. There is no `route` method on the wrapper — `emit` is the full validate-persist-deliver path, and `deliver(&event)` is the delivery-only path for callers that have already persisted through a canonical writer. `emit` runs:

1. **Validate.** Each registered validator runs synchronously, then write namespaces are enforced. First failure returns error — the event is neither persisted nor delivered.
2. **Persist.** The event is appended to the shared `events` table in `platform.db` via the inner router's sink. This is synchronous and completes before any mailbox sees the event.
3. **Deliver.**
   - If `event.target_actor_id` is set, the event goes to that actor's mailbox only (direct message).
   - Otherwise, the router fans out: for each registered subscription namespace, if `event.event_type` starts with the namespace, every subscribed actor's mailbox receives a clone.

Under the hood `emit` calls `inner.route()` — the pure router's combined persist-then-deliver. (The `unstable` feature path instead calls `inner.persist()` plus a scope-filtered delivery.)

The persist-before-deliver ordering is the kernel's central invariant. No actor reacts to an event that isn't already in the log. Crash recovery can replay from the log without risking a "we delivered but never persisted" state.

Delivery uses `mpsc::Sender::try_send`. If a mailbox is full (capacity 256), the event is dropped silently for that actor. The persist is not affected. Actors need to drain their mailboxes promptly — there is no backpressure from the kernel.

## Actors

An actor is a logical identity. It has:

| Field | Purpose |
|---|---|
| `namespace` | The actor's own namespace — what it owns. |
| `write_namespaces` | Namespaces the actor is permitted to emit into. |
| `read_namespaces` | Namespaces the actor may query via filtered reads. |
| `subscribe_namespaces` | Namespaces the actor wants pushed to its mailbox. |

`ActorConfig` lives in `ship-types::caller`, re-exported by both the pure crate and the runtime router. The runtime router exposes `spawn_actor(id, config)` always, and `spawn_actor_with_permissions(identity, perms)` under the `unstable` feature — the permissions variant pulls namespaces from a higher-level `Permissions` type and is what service code uses when the feature is enabled.

`spawn_actor` returns a `Mailbox`. The caller is responsible for consuming it — typically by spawning a tokio task that loops on `mailbox.recv()`. The kernel does not run actor code; it only delivers events and hands out mailboxes.

## Namespaces and subscription matching

Subscriptions match by **prefix** on `event_type`. Given an event with type `workflow.job.started`:

- Actor subscribed to `workflow.` — matches.
- Actor subscribed to `workflow.job.` — matches.
- Actor subscribed to `workflow.job.started` — matches.
- Actor subscribed to `workflow.task` — does not match.

Matching is string prefix, not segment-wise. A subscription to `work` would match `workflow.*` events; subscriptions are usually written with trailing `.` to prevent this.

Writing is scoped similarly. An event's `event_type` must have a prefix in the emitting actor's `write_namespaces`, or the write is rejected at the `ActorStore` boundary (not inside the router — see next section).

## Physical vs logical isolation

Historical Ship gave each actor its own SQLite file at `~/.ship/actors/{id}/events.db`. Current Ship does not. Every event goes into the single `events` table in `~/.ship/platform.db`. Actor isolation is **logical**:

- Reads through `ActorStore` filter by `actor_id` column; an actor's unfiltered queries only return its own events.
- Writes through `ActorStore` check the event's namespace against the actor's `write_namespaces` and reject mismatches before hitting SQL.

The benefit: single-file backups, one WAL, fewer open file descriptors, simpler cross-actor queries when needed. The cost: actors share a write path, so there's no physical guarantee — a direct SQL write bypassing `ActorStore` could put anything anywhere. The boundary is the `ActorStore` API.

## EventSink — the extension point

`ship_kernel::EventSink` is a one-method trait:

```rust
pub trait EventSink: Send + Sync {
    fn persist(&self, event: &EventEnvelope) -> Result<()>;
}
```

The pure `KernelRouter` calls `sink.persist(&event)` during step 2 of route. Implementations decide where events go: SQLite, an in-memory vec for tests, a network appender, a multiplexer.

The substrate's `SqliteEventStore` at `ship_substrate::persistence::event_store` is the production implementation. The substrate's `KernelRouter` constructs a `SqliteEventStore`, hands it to the pure router as the sink, and persistence flows through the trait — `inner.route()` (or `inner.persist()` on the `unstable` path) calls `sink.persist()`. The trait is live, not bypassed.

## Validators

`EventValidator` runs before persist. One method:

```rust
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &EventEnvelope) -> Result<(), KernelError>;
}
```

A blanket impl makes closures work directly:

```rust
router.with_validator(Box::new(|ev: &EventEnvelope| {
    if ev.event_type.is_empty() {
        return Err(KernelError::ValidationFailed { reason: "empty type".into() });
    }
    Ok(())
}));
```

Use cases: enforce schema on payload, reject events with stale causation IDs, cross-check namespaces against a security policy. Validators are the only hook the kernel provides between receiving an event and persisting it — heavier pipelines (enrichment, dispatch to external services) belong in actor code, not validators.

## Mailbox

`Mailbox` is a thin wrapper around `tokio::sync::mpsc::Receiver<EventEnvelope>`.

- Capacity: 256 (constant `MAILBOX_CAPACITY`).
- `recv().await` — suspends until an event arrives or all senders drop.
- `try_recv()` — non-blocking. Returns `Empty` or `Disconnected`.
- `from_receiver(rx)` — construct from an external receiver (used when the actor wants to compose the channel itself).

The kernel owns the sender side. Losing the `Mailbox` means losing events — if an actor's task panics and drops its receiver, subsequent routes still call `try_send` against that dead channel and drop silently. Supervising actor tasks is the caller's responsibility.

## Direct messages

Setting `event.target_actor_id = Some(id)` bypasses namespace fanout. The router delivers only to the named actor, regardless of its subscribe set. The event is still persisted normally.

Use this for responses, targeted notifications, and cross-actor RPC patterns. Don't use it to implement general routing — that's what namespaces are for.

## Where it ends and the runtime begins

| Concern | Kernel | Runtime |
|---|---|---|
| Mailbox capacity | ✓ | |
| Namespace fanout | ✓ | |
| Validator hooks | ✓ | |
| Persist ordering | ✓ | |
| SQLite schema, migrations | | ✓ |
| `ActorStore` + `actor_id` filters | | ✓ |
| Projections, read models | | ✓ |
| Services (orchestrator, mesh, human gateway) | | ✓ |
| HTTP, MCP, CLI | | transport, not runtime |

The kernel makes no policy decisions about what events mean. It is a message bus with a durability guarantee.

## Current limitations

These are real constraints in the code today. Know them before designing against the kernel.

**Silent drops on full mailboxes.** `try_send` drops events when a receiver is saturated. No counter, no log, no metric. If an actor appears to be missing events, mailbox saturation is an invisible cause. No backpressure — the router returns `Ok(())` whether every mailbox accepted or none did.

**Prefix-match subscription foot-gun.** Subscriptions match by string prefix on `event_type`. A subscription to `work` matches `workflow.*` events. Convention is to write namespaces ending in `.` (`workflow.`) but it's not enforced. No compile-time or runtime validation of namespace strings.

**Global mailbox capacity.** `MAILBOX_CAPACITY = 256` is a module constant. Cannot be tuned per actor. A bursty high-volume actor gets the same queue size as a lazy one.

**No actor supervision in the kernel.** If an actor's consumer task panics and drops its `Mailbox`, the router keeps calling `try_send` on the closed channel. Events intended for that actor drop silently until the router's `stop_actor` is called by someone else. Supervision policy (restart, escalate, drop) lives in `ship_substrate::supervisor` today, loosely coupled.

**Only in-process routing.** All actors share the same process. No protocol for cross-process or cross-machine delivery. Multi-process Ship would need serialization, transport, and a distributed sink.

**Untyped payloads.** `EventEnvelope.payload_json` is a JSON blob. Type is carried in `event_type`. No schema validation at routing time — validators can do it but there's no first-class contract system.

## Potential future work

Where the kernel could grow as Ship's scope expands. None of this is committed — it's a map of what a browser/OS-scale Ship would likely demand from the routing layer.

**Backpressure policy.** Replace `try_send` drop-on-full with configurable policy: block the router, return an error, drop with a metric increment, or spill to the sink and reattempt. The right default is probably "return error and let the caller decide" — drop is a quiet lie about delivery.

**Priority / scheduling.** Human-interaction events (UI clicks, feedback, approvals) shouldn't wait behind background sync. Priority lanes at the mailbox level would let the kernel schedule what an actor receives first when it drains.

**Supervision in the kernel.** Lift `ship_substrate::supervisor` into `ship-kernel` as a first-class concept. Actors declare restart policies, the kernel owns the lifecycle. Gives the kernel enough power to actually be a kernel.

**Capability tokens.** Namespaces are strings today — fakeable, forgeable in tests, leaky. A browser/OS Ship wants capability objects: unforgeable handles issued by the kernel, delegated down to actors, revocable centrally. This is the substrate for real security posture.

**Typed event contracts.** `EventEnvelope` carries a schema reference; the kernel validates payloads against registered schemas at route time. Cheaper than ad-hoc validators, stronger than JSON-blob convention.

**Cross-process routing.** If the daemon process becomes one of several — a render process, a compute process, a sync process — the kernel needs to route events across process boundaries. This is the largest lift: serialization format, transport, sink replication, ordering guarantees across transports.

**Projection subscriptions.** Not strictly a kernel concern (this lives in runtime), but the watch pattern belongs adjacent: subscribe to a query result, tail an entity's event history, hook into projection rebuild completion. Today clients poll or run their own listeners.

