---
name: ship-kernel
stable-id: ship-kernel
description: Use when working on event routing, actor registration, or the persist-before-deliver contract. Covers the pure `ship-kernel` crate and the concrete runtime wrapper.
tags: [ship, kernel, events, actors]
authors: [ship]
audience: internal
---

# Ship Kernel

The `ship-kernel` crate defines event routing primitives with no persistence or filesystem dependencies. The runtime wraps the kernel's types with a concrete `KernelRouter` backed by `platform.db`.

## Pure crate (`crates/core/ship-kernel/`)

- **`KernelRouter`** (`router.rs`) — in-memory actor registry. Holds mailboxes, actor configs, namespace subscriptions, validators, and an `Arc<dyn EventSink>`. The runtime wrapper delegates all routing state and persistence to this type.
- **`Mailbox`** (`mailbox.rs`) — receive end of a tokio `mpsc` channel. Capacity 256. `recv()` async, `try_recv()` sync.
- **`EventSink`** (`sink.rs`) — one-method trait: `fn persist(&self, event: &EventEnvelope) -> Result<()>`. Implemented by `SqliteEventStore`.
- **`EventValidator`** (`validator.rs`) — pre-persist hook trait. Closures auto-impl via blanket.
- **`ActorConfig`** — re-exported from `ship-types::caller` (not defined here). Fields: `namespace`, `write_namespaces`, `read_namespaces`, `subscribe_namespaces`.

## Routing contract

`KernelRouter::route(event)`:

1. Run each registered validator. Fail = error, event discarded.
2. `sink.persist(&event)` — synchronous. Durable before any delivery.
3. Deliver:
   - If `event.target_actor_id` set → direct to that mailbox only.
   - Otherwise broadcast: deliver to every actor whose `subscribe_namespaces` contains a prefix of `event.event_type`.

Delivery uses `try_send` — mailbox full = silent drop. No backpressure on the router.

## Runtime wrapper (`ship_substrate::events::kernel_router`)

Thin shell around `ship_kernel::KernelRouter`. Takes `base_dir: PathBuf`
(e.g. `~/.ship/`), constructs a `SqliteEventStore` sink, and hands it to
the inner router. The wrapper adds:

- `emit(event, ctx)` — validates with `EmitContext`-aware validators,
  enforces write namespaces, then calls `inner.route()` (persist + deliver).
- `deliver(&event)` — delivery-only path for callers that have already
  persisted through a canonical writer. Calls `inner.deliver()`; runs no
  validators and does not persist.
- `spawn_actor(id, config)` — delegates registration to
  `inner.spawn_actor()`, returns `(ActorStore, Mailbox)` instead of just
  `Mailbox`.
- `unstable` feature: `actor_meta`, `event_log`, `actor_cursors`,
  `spawn_actor_with_permissions*`, scope-filtered delivery.

Actor isolation is **logical** — the `actor_id` column in `platform.db`
filters reads; `write_namespaces` gates writes at the `ActorStore`
boundary. No per-actor SQLite files.

## Key files

- `crates/core/ship-kernel/src/{router,mailbox,sink,validator}.rs` — pure abstractions.
- `crates/core/ship-substrate/src/events/kernel_router.rs` — production router wrapper.
- `crates/core/ship-substrate/src/persistence/actor_store.rs` — namespace-scoped handle.
- `crates/core/ship-substrate/src/persistence/event_store.rs` — `SqliteEventStore`, implements `ship_kernel::EventSink`.
