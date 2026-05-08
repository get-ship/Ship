---
group: Runtime
order: 10
title: Mesh
description: Agent-to-agent registration, directed messaging, broadcast, and discovery. Thin validator and address resolver over the kernel.
audience: internal
---

# Mesh

`crates/core/runtime/src/services/mesh.rs`. A registry + validator for inter-agent communication. The mesh does not route events itself — it validates addresses, constructs `EventEnvelope`s with `target_actor_id` set (or the broadcast set), and pushes them to an outbox channel. The caller drains the outbox through the kernel, so mesh messages go through persist-before-deliver like any other event.

The daemon runs `MeshService` as a service actor subscribed to `mesh.*`. See the daemon startup code for wiring.

## Types

```rust
pub enum AgentStatus { Active, Busy, Idle }

pub struct MeshEntry {
    pub agent_id: String,
    pub label: String,
    pub capabilities: Vec<String>,
    pub registered_at: DateTime<Utc>,
    pub status: AgentStatus,
}

pub type SharedMeshRegistry = Arc<RwLock<HashMap<String, MeshEntry>>>;

pub struct MeshService {
    agents: HashMap<String, MeshEntry>,
    shared_registry: SharedMeshRegistry,
    outbox: mpsc::UnboundedSender<EventEnvelope>,
}
```

`SharedMeshRegistry` is the read view — REST handlers read this under an `RwLock` without going through the actor. Writes always go through the actor's event loop so the registry stays in sync with the event log.

## Constructors

```rust
MeshService::new(outbox)                           // fresh registry
MeshService::with_shared_registry(outbox, shared)  // share the registry with REST read path
```

The daemon creates a `SharedMeshRegistry`, wraps the mesh with `with_shared_registry`, and hands the same `Arc` to the REST API. Reads skip the actor entirely; writes flow through it.

## Event types

`MeshService` implements `ServiceHandler::handle(event, store)`. Dispatches by `event.event_type`:

| Event | Handler | Behavior |
|---|---|---|
| `mesh.register` | `handle_register` | Add entry with `AgentStatus::Active`. Payload carries id, label, capabilities. Duplicate register updates the existing entry. |
| `mesh.deregister` | `handle_deregister` | Remove entry. No error if not present. |
| `mesh.send` | `handle_send` | Validate `to` exists. Emit `mesh.message` with `target_actor_id = to`. If target unknown, emit `mesh.send.failed` back to sender. |
| `mesh.broadcast` | `handle_broadcast` | Emit `mesh.message` for every agent (except sender). Optional `capability` filter in payload. |
| `mesh.discover.request` | `handle_discover` | Emit `mesh.discover.response` with matching agents. Filterable by `capability` and `status`. |
| `mesh.status` | `handle_status` | Update an agent's `AgentStatus`. |

Anything else the actor sees (non-`mesh.*`) is ignored — the subscription is scoped to `mesh.`.

Events emitted *by* the mesh:

| Event | Emitted from | Purpose |
|---|---|---|
| `mesh.message` | `send` / `broadcast` | The actual message. `target_actor_id` set for direct; for broadcast, one envelope per recipient. |
| `mesh.send.failed` | `send` when target unknown | Sender gets notified their target isn't registered. |
| `mesh.discover.response` | `discover.request` | Reply payload with matching entries. |

## Address resolution

The sender is pulled from `event.actor_id`, falling back to `payload["from"]`, defaulting to `"unknown"`. For broadcast, the resolved sender is excluded from recipients (no self-broadcasts).

Targets are looked up by `agent_id` in the registry. No pattern matching, no wildcards. If you want pattern-based routing, subscribe to a namespace at the kernel level directly rather than using mesh.

## Outbox

The mesh doesn't call `kernel.emit` directly. Why:

- Emission needs an async kernel lock, but `ServiceHandler::handle` is sync.
- Some handlers produce multiple events (broadcast → one per recipient); mixing async emission into a sync handler is awkward.

Instead: the service actor has an `mpsc::UnboundedSender<EventEnvelope>` outbox. Handlers push events. A separate tokio task drains the outbox and calls `kernel.emit` for each. This also means mesh emissions go through the same validate → persist → deliver contract as everything else; the mesh has no backdoor.

## Persist-before-deliver still holds

A `mesh.send` emitted by agent A:

1. A emits `mesh.send { to: B, body: ... }` via kernel.
2. Kernel persists → mesh actor's mailbox.
3. Mesh's handler validates, constructs `mesh.message { ..., target_actor_id: B }`, pushes to outbox.
4. Outbox drainer calls `kernel.emit(mesh.message)`.
5. Kernel persists → targets only B's mailbox (direct delivery via `target_actor_id`).
6. B receives it.

Two events in the log for one logical message (the `send` intent and the `message` delivery). Downstream consumers (audit, sync) have both.

## Capabilities

`MeshEntry.capabilities: Vec<String>`. Free-form strings declared at register time. Filters on broadcast and discover use prefix/substring match depending on payload shape. Mesh doesn't enforce capability semantics — what `"code"` or `"gate-review"` means is up to the agents that emit/consume them.

## Tests

Co-located in `runtime/src/services/mesh_tests.rs` and `runtime/src/services/integration_tests.rs`. Pattern: spin up a mesh with a stubbed outbox receiver, fire events, assert what lands on the outbox. Uses `MeshService::new(outbox)` directly — no need for a real kernel in unit tests.

## Invariants

- Writes to the registry go only through the actor. REST handlers read the `SharedMeshRegistry` but never mutate it.
- Every emitted mesh envelope has `target_actor_id` set (or is part of a broadcast expansion). No accidental namespace fanout.
- `mesh.message` delivery uses `target_actor_id` — bypasses subscription matching. A recipient doesn't need to subscribe to `mesh.` to get a direct message.
- Sender is excluded from broadcast recipients.

## Related

- [Kernel](/architecture/kernel/) — routing contract; mesh envelopes go through it.
- [Events](/runtime/events/) — `mesh.` namespace sits in `KERNEL_NAMESPACES`.
- [Orchestrator](/runtime/orchestrator/) — another service actor, same `ServiceHandler` pattern.
