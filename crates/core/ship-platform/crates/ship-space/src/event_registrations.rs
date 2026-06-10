//! Event registrations for the SPACE domain.
//!
//! Registers the 9 project.* / workspace.* / space.setting.changed events
//! with the global event registry using the hoisted `register_event!` macro
//! from `ship_types`.
//!
//! Keystone-A Phase A bundle 4 (2026-06-10): the SPACE domain owns its events
//! at L1. These registrations moved from `ship_substrate::events::payload_registry`.
//!
//! Payload structs STAY in ship_types::payload — several are consumed by L0
//! (ProjectLog by ship-substrate/src/log.rs; WorkspaceActivated by tests_actor.rs)
//! and by ship-config's SpaceSettingsProjection (SpaceSettingChanged).

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── project.* ───────────────────────────────────────────────────────────────
// The `projects` registry is the daemon's machine-wide record of known project
// paths, read by id/path with no space in hand — and the source the space_id
// resolver itself consults. So the registry events are Runtime-owned (Phase 4d
// §6.1): a space-partitioned registry could not bootstrap space-id resolution,
// and routing them to space.db while reading daemon-wide would force fan-out.
// `project.log` is a daemon-wide audit stream (fixed "project" entity, no
// space) → also Runtime. `project.switched` carries a space binding through
// the kernel and stays Space.
register_event!("project.archived", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectArchived", ship_types::payload::ProjectArchived);
register_event!("project.log", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectLog", ship_types::payload::ProjectLog);
register_event!("project.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectRegistered", ship_types::payload::ProjectRegistered);
register_event!("project.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectSeen", ship_types::payload::ProjectSeen);
register_event!("project.switched", [EventScope::Space], Retention::Durable, Stability::Stable, "ProjectSwitched", ship_types::payload::ProjectSwitched);

// ── space.setting.* ─────────────────────────────────────────────────────────
register_event!("space.setting.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "SpaceSettingChanged", ship_types::payload::SpaceSettingChanged);

// ── workspace.* (deprecated) ────────────────────────────────────────────────
register_event!("workspace.created", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceCreated", ship_types::payload::WorkspaceCreated);
register_event!("workspace.activated", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceActivated", ship_types::payload::WorkspaceActivated);
register_event!("workspace.archived", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceArchived", ship_types::payload::WorkspaceArchived);
