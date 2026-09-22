//! Event registrations for the SPACE domain.
//!
//! Registers the `space.*` Runtime-scoped events with the global event
//! registry using the hoisted `register_event!` macro from `ship_types`.
//!
//! Identity lives in the `spaces` Runtime-scoped projection (folded from
//! these events by `SpacesProjection`); identity is NEVER minted from the
//! path — see `crate::identity::ensure_identity_for_root`, the single
//! accessor the future fenced space-config session may relocate.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── space.* (Runtime-scoped — the `spaces` projection) ──────────────────────
// The runtime emits `space.opened` for every space it opens (mint-once at
// creation) and `space.seen` for every re-open (bumps last_seen_at). The
// projection is Runtime-scoped because the table is device-local (never
// syncs — location facts belong to the device, not to portable history).
register_event!("space.opened", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SpaceOpened", ship_types::payload::SpaceOpened);
register_event!("space.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SpaceSeen", ship_types::payload::SpaceSeen);
// Stack membership (`entity_id` = space_id). Runtime-scoped: rows carry
// paths, and location facts belong to the device.
register_event!("stack.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "StackRegistered", ship_types::payload::StackRegistered);

// Space-scoped settings changes ride the unified `settings.changed` event
// (SettingsService.Set, EventScope::Space). Per-space settings are read +
// written through the unified `settings_overlay` table.
