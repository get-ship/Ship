//! Event registrations for the CLOUD sync domain.
//!
//! Registers the 4 sync.* events with the global event registry using the
//! hoisted `register_event!` macro from `ship_types`, making them available
//! to downstream code that calls `event_registry::registry()`.
//!
//! Substrate-cohesion de-bucket (2026-06-11): the CLOUD/SYNC domain owns its
//! events at L1 in this crate — the sole emitter is
//! `ship_cloud::sync_service::SyncServiceHandler`. These registrations moved
//! from `ship_substrate::events::payload_registry` so registrations co-locate
//! with their domain owner (mirrors the agent-def, ship-auth, ship-space …
//! registration module pattern from keystone-A Phase A).
//!
//! Payload structs remain in `ship_types::payload` (the leaf-layer types
//! crate); only the registrations — ownership of name, scope, retention, and
//! the carried validator — live with the domain.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── sync.* ──────────────────────────────────────────────────────────────────
// Runtime-scoped daemon-service events: no space binding (the service task
// has no session). sync.push.completed is Windowed (high-frequency progress
// tick — the recent push record matters, full history does not; swept past
// 7 days); the three state-change events are Durable (started / stopped =
// lifecycle facts, push.failed = error record).
register_event!("sync.push.completed", [EventScope::Runtime], Retention::Windowed { ttl_days: 7 }, Stability::Stable, "SyncPushCompleted", ship_types::payload::SyncPushCompleted);
register_event!("sync.push.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncPushFailed", ship_types::payload::SyncPushFailed);
register_event!("sync.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStarted", ship_types::payload::SyncStarted);
register_event!("sync.stopped", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStopped", ship_types::payload::SyncStopped);
