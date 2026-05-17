---
group: Runtime
order: 6
title: Events
description: EventEnvelope, namespaces (kernel vs platform-only vs reserved), validators, and the typed event constants.
audience: internal
---

# Events

`crates/core/ship-substrate/src/events/`. Everything routable through the kernel is an `EventEnvelope`. The runtime's events module re-exports types from `ship-kernel` and `ship-types`, defines domain-specific payload types, runs validators, and exposes the global kernel router.

```
events/
  mod.rs                  — re-exports, module tree
  annotation.rs           — annotation payload types + event consts
  daemon.rs               — daemon lifecycle payload types + consts
  executor.rs             — executor session/window payload types + consts
  feedback.rs             — feedback payload types + FeedbackEvent DSL
  filter.rs               — EventFilter (query DSL)
  global_kernel_router.rs — process-global router initialization + access
  job.rs                  — job payload types + consts + Pipeline DAG
  kernel_lifecycle.rs     — spawn/stop actor helpers
  kernel_router.rs        — runtime's KernelRouter wrapper
  kernel_security.rs      — actor_meta + scope-aware delivery (unstable)
  payload_schema.rs       — PayloadSchemaValidator
  permissions.rs          — permission-aware emit checks (unstable)
  process.rs              — process lifecycle payload types + consts
  supervisor.rs           — supervisor lifecycle payload types + consts
  validator.rs            — EventValidator trait + built-in validators + namespace lists
```

## EventEnvelope

Defined in `ship-types::envelope::EventEnvelope`, re-exported here. The universal event record:

| Field | Type | Purpose |
|---|---|---|
| `id` | String | Monotonic ULID. Encodes insertion time; query sorts use `id ASC`. |
| `event_type` | String | Dot-namespaced (`workspace.created`, `job.step_completed`). |
| `entity_id` | String | The entity this event describes. |
| `actor` | String | Human-readable emitter (defaults to `"ship"`). |
| `payload_json` | String | Serialized payload. |
| `version` | u32 | Schema version. |
| `causation_id` | Option<String> | The event that caused this one. |
| `workspace_id` | Option<String> | Scoping context. |
| `session_id` | Option<String> | Scoping context. |
| `actor_id` | Option<String> | Kernel actor that emitted. Stamped by `ActorStore::append`. |
| `parent_actor_id` | Option<String> | Parent in actor hierarchy. |
| `target_actor_id` | Option<String> | Direct-delivery target (bypasses namespace fanout). |
| `elevated` | bool | Platform-scope (visible to sync). |
| `created_at` | DateTime<Utc> | RFC3339. |

Builder methods on the envelope: `with_causation`, `with_context`, `with_actor_id`, `with_parent_actor_id`, `with_target`, `elevate`.

Constructor: `EventEnvelope::new(event_type, entity_id, &payload)` where payload is anything `Serialize`-able.

## Event type constants

~83 constants defined in `ship-types::event_types`, re-exported through `ship_substrate::events::types::event_types`. Callers reference `event_types::PIPELINE_RUN_STARTED` rather than writing `"pipeline.run.started"` literally — the constant catches typos at compile time.

Complete list grouped by domain:

**Workspace (3):** `WORKSPACE_CREATED`, `_ACTIVATED`, `_ARCHIVED`. (The `workspace` runtime primitive was retired; only these three constants remain.)

**Session (11):** `SESSION_STARTED`, `_PROGRESS`, `_ENDED`, `_DRAIN_STARTED`, `_DRAIN_COMPLETED`, `_DRAIN_ABORTED`, `_TOOL_COUNT_INCREMENTED`, `_ATTACHED`, `_HEARTBEAT`, `_STATUS_CHANGED`, `_TURN_FAILED`. There is no `SESSION_RECORDED` — end-of-session telemetry lands inline on `session.ended`.

**Actor (5):** `ACTOR_CREATED`, `_WOKE`, `_SLEPT`, `_STOPPED`, `_CRASHED`.

**Gate (2):** `GATE_PASSED`, `GATE_FAILED`.

**Job (10):** `PIPELINE_RUN_STARTED`, `_COMPLETED`, `_FAILED`, `_DISPATCHED`, `_UPDATE`, `_STEP_COMPLETED`, `_STEP_FAILED`, `_CANCELLED`, `_MERGED`, `_SPAWN_FAILED`.

**Skill (7):** `SKILL_STARTED`, `_COMPLETED`, `_FAILED`, `_ATTACHED_TO_SESSION`, `_CREATED`, `_UPDATED`, `_DELETED`.

**Supervisor (5):** `SUPERVISOR_SHUTTING_DOWN`, `_SWEEP_STARTED`, `_SWEEP_COMPLETED`, `_DRIFT_DETECTED`, `_COHERENCE_RESTORED`.

**Process (5):** `PROCESS_SPAWNED`, `_EXITED`, `_SIGNAL_SENT`, `_ORPHANED`, `_KILLED`.

**Executor (2):** `EXECUTOR_SESSION_DESTROYED`, `_WINDOW_DESTROYED`.

**Daemon (6):** `DAEMON_STARTUP_COMPLETED`, `_SHUTDOWN_COMPLETED`, `_CRASHED`, `_PORT_BIND_FAILED`, `_PIDFILE_ACQUIRED`, `_PIDFILE_STALE_TAKEN`.

**Artifact (4):** `ARTIFACT_CREATED`, `_UPDATED`, `_MOVED`, `_DELETED`.

**Change events (8):** `CONFIG_CHANGED`, `AGENT_CHANGED`, `SKILL_CHANGED`, `VIEW_CHANGED`, `PIPELINE_CHANGED`, `RULE_CHANGED`, `PROJECT_CHANGED`, `PROJECT_LOG`.

**Project registry (3):** `PROJECT_REGISTERED`, `PROJECT_SEEN`, `PROJECT_ARCHIVED`.

**Auth (3):** `AUTH_SESSION_CREATED`, `_REVOKED`, `_EXPIRED`.

**Daemon owner (2):** `DAEMON_OWNER_BOUND`, `_UNBOUND`.

**Rule (3):** `RULE_CREATED`, `_UPDATED`, `_DELETED`.

**Studio (2):** `STUDIO_CHAT_TURN`, `STUDIO_REPLY`.

**Other:** `COMPILE_COMPLETED`, `PERMISSIONS_UPDATED`.

A sentinel `ALL: &[&str]` enumerates every constant. One test enforces uniqueness (`event_type_constants_are_unique`). There is no count-tracking test — adding a constant requires appending it to `ALL`.

## Namespaces

Three lists maintained in `validator.rs`:

**`KERNEL_NAMESPACES`** (13) — events routed to actor mailboxes. Actors subscribe to these prefixes.

```
actor. collection. config. daemon. gate. job. mesh.
runtime. session. studio. supervisor. sync. workspace.
```

**`PLATFORM_ONLY_NAMESPACES`** (2) — persisted in `platform.db` but not routed to actors. UI/config events actors don't need.

```
project. skill.
```

**`RESERVED_NAMESPACES`** — the union. Untrusted callers cannot emit events with these prefixes. Enforced by the `ReservedNamespaceValidator`. A test (`namespace_lists_are_aligned`) asserts `RESERVED = KERNEL ∪ PLATFORM_ONLY`.

Everything not in these lists is open — skills emit events in their own skill-id-prefixed namespaces (e.g. `my-skill.tool_run`), and other application code uses custom namespaces freely.

## Validators

`EventValidator` trait (from `ship-kernel`, re-exported):

```rust
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &EventEnvelope) -> Result<(), KernelError>;
}
```

Validators run synchronously in the kernel's `route()` before persist. Failing validation discards the event entirely — not persisted, not delivered.

Built-in validators registered with the runtime's router:

**`NamespaceValidator`** — skill events must match the skill's own namespace. If caller is a skill with id `my-skill`, emitted event types must start with `my-skill.` or be outside the reserved lists.

**`ReservedNamespaceValidator`** — blocks untrusted callers from emitting reserved namespaces. Trust derived from `CallerKind`:

- **Trusted**: `CallerKind::Runtime`, `CallerKind::Cli`.
- **Untrusted**: `CallerKind::Mcp`, `CallerKind::Sdk`, `CallerKind::CloudSync`.

**`ElevatedValidator`** — callers cannot set `elevated = true` unless trusted.

**`PayloadSchemaValidator`** (in `payload_schema.rs`) — JSON schema enforcement for payloads when a schema is registered for the event type. Not all types have schemas registered today.

## EmitContext

```rust
pub struct EmitContext {
    pub caller_kind: CallerKind,
    pub skill_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
}
```

Passed alongside the envelope to `KernelRouter::emit(event, &ctx)`. Validators consume it to determine trust level and namespace scope. Constructors for common contexts live in `validator.rs`.

## EventFilter

Query DSL for `EventStore::query`:

```rust
pub struct EventFilter {
    pub entity_id: Option<String>,
    pub event_type: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub elevated_only: bool,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
```

All fields optional. Unset fields are ignored by the query builder. Results always ordered `id ASC` (insertion order).

## Global kernel router

`events/global_kernel_router.rs` exposes a process-wide router:

- `init_kernel_router(ship_dir) -> Arc<Mutex<KernelRouter>>` — initializes the global router against `{ship_dir}/platform.db`. Called by the daemon at startup.
- `kernel_router() -> Arc<Mutex<KernelRouter>>` — accesses the initialized global router. Panics if not initialized (daemon invariant).
- `new_isolated_for_test(ship_dir)` — test helper that constructs a fresh router without touching the global. Each test builds its own isolated router.

Most production code accesses the router via the daemon's `ShipRuntime::kernel` field rather than through the global. The global exists for call sites that don't have the runtime in scope.

## Domain payload modules

Each major event family has a module defining its payload types and sometimes event constants local to the domain:

- `job.rs` — `PipelineRunStartedPayload`, `Pipeline`, `PipelineStep`, `StepType`.
- `process.rs` — `ProcessSpawnedPayload`, `ProcessExitedPayload`, etc.
- `supervisor.rs` — `SupervisorSweepPayload`, drift detection payloads.
- `executor.rs` — session/window lifecycle payloads.
- `daemon.rs` — startup, shutdown, crash payloads.
- `annotation.rs` — `AnnotationEvent` for canvas annotations.
- `feedback.rs` — `FeedbackEvent`, `FeedbackKind`, `FeedbackTarget`, `AuthorKind`, `Selection`.

Each module deserializes its payloads from `envelope.payload_json` via `serde_json::from_str`. Handlers that consume events pull the matching payload type by event_type.

## Kernel lifecycle

`events/kernel_lifecycle.rs` wraps `spawn_actor` / `stop_actor` with runtime-side bookkeeping. Actor-metadata tracking (the `actor_meta` HashMap on the wrapper) lives here. Used by supervisors that spin up and tear down service actors.

## Permissions + kernel_security (unstable)

Under `#[cfg(feature = "unstable")]`:

- `permissions.rs` — permission-aware emit checks. Integrates `ship-types::Permissions` with validator chain.
- `kernel_security.rs` — `ActorMeta`, scope-filtered delivery, the `spawn_actor_with_permissions*` helpers, and the `unstable_senders` channel map used by permissions-based actors. Lets the wrapper distinguish permissions-tracked actors from legacy `spawn_actor` actors when deciding whether a full-mailbox error should propagate or silently drop.

## Related

- [Kernel](/architecture/kernel/) — routing contract, validate → persist → deliver.
- [Persistence](/runtime/persistence/) — `EventStore`, `SqliteEventStore`, `ActorStore`.
- [Platform DB](/runtime/platform-db/) — the `events` table schema.
- [Projections](/runtime/projections/) — read models derived from the event log.
