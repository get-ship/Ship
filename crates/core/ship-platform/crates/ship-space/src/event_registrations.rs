//! Event registrations for the SPACE domain.
//! Registers the project.* / space.setting.* events with the global event
//! registry using the hoisted `register_event!` macro from `ship_types`.
//! workspace.* (deprecated) was deleted (deadlayer-cut, 2026-06-19) — the
//! events had no live emitters and only test-only consumers.
//!
//! Keystone-A Phase A bundle 4 (2026-06-10): the SPACE domain owns its events
//! at L1. These registrations moved from `ship_substrate::events::payload_registry`.
//!
//! Payload structs STAY in ship_types::payload — SpaceSettingChanged is
//! consumed by ship-config's SpaceSettingsProjection.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── project.* ───────────────────────────────────────────────────────────────
// The `projects` registry is the daemon's machine-wide record of known project
// paths, read by id/path with no space in hand — and the source the space_id
// resolver itself consults. So the registry events are Runtime-owned (Phase 4d
// §6.1): a space-partitioned registry could not bootstrap space-id resolution,
// and routing them to space.db while reading daemon-wide would force fan-out.
// Space. (The legacy project-log daemon-wide audit stream is GONE — see the
// module doc.)
register_event!("project.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectRegistered", ship_types::payload::ProjectRegistered);
register_event!("project.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectSeen", ship_types::payload::ProjectSeen);
register_event!("project.switched", [EventScope::Space], Retention::Durable, Stability::Stable, "ProjectSwitched", ship_types::payload::ProjectSwitched);

// Space-scoped settings changes ride the unified `settings.changed` event
// (SettingsService.Set, EventScope::Space). Per-space settings are read +
// written through the unified `settings_overlay` table.
