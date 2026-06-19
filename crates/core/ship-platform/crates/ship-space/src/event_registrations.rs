//! Event registrations for the SPACE domain.
//!
//! Registers the project.* / space.setting.* / workspace.* events with the
//! global event registry using the hoisted `register_event!` macro from
//! `ship_types`. The legacy project-log daemon-wide audit stream was deleted
//! (2026-06-19) — the writer API had no callers and the read path was
//! unreferenced; the corresponding payload and event-name const in
//! ship-types were removed as well.
//!
//! Keystone-A Phase A bundle 4 (2026-06-10): the SPACE domain owns its events
//! at L1. These registrations moved from `ship_substrate::events::payload_registry`.
//!
//! Payload structs STAY in ship_types::payload — WorkspaceActivated is
//! consumed by L0 (tests_actor.rs); SpaceSettingChanged is consumed by
//! ship-config's SpaceSettingsProjection.

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
register_event!("project.archived", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectArchived", ship_types::payload::ProjectArchived);
register_event!("project.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectRegistered", ship_types::payload::ProjectRegistered);
register_event!("project.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectSeen", ship_types::payload::ProjectSeen);
register_event!("project.switched", [EventScope::Space], Retention::Durable, Stability::Stable, "ProjectSwitched", ship_types::payload::ProjectSwitched);

// ── space.setting.* ─────────────────────────────────────────────────────────
register_event!("space.setting.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "SpaceSettingChanged", ship_types::payload::SpaceSettingChanged);

// ── workspace.* (deprecated) ────────────────────────────────────────────────
register_event!("workspace.created", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceCreated", ship_types::payload::WorkspaceCreated);
register_event!("workspace.activated", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceActivated", ship_types::payload::WorkspaceActivated);
register_event!("workspace.archived", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceArchived", ship_types::payload::WorkspaceArchived);
