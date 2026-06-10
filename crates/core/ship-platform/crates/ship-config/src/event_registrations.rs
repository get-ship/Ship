//! Event registrations for the CONFIG watcher family.
//!
//! Registers the 8 `<domain>.changed` watcher-ping events with the global
//! event registry using the hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone-A Phase A bundle 3 (2026-06-10): the watcher-ping family is
//! emitted by ship-config's watcher (L1) and belongs here. These registrations
//! moved from `ship_substrate::events::payload_registry`.
//!
//! Payload structs (ConfigChanged, ConfigFileChanged) STAY in ship_types::payload
//! — ConfigChanged is a shared type used by all watcher events; ConfigFileChanged
//! has no owner-specific reason to move.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── config.changed / config.file_changed ────────────────────────────────────
// Machine/daemon runtime settings (daemon.port, terminal.program, watcher.*,
// isolation tiers) — per-machine, never synced. `set_setting_and_emit` mints
// daemon-internal with no space; Runtime is the correct partition.
register_event!("config.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigChanged", ship_types::payload::ConfigChanged);
register_event!("config.file_changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigFileChanged", ship_types::payload::ConfigFileChanged);

// ── <domain>.changed watcher-ping family ────────────────────────────────────
// The coarse-grain watcher pings: emitted by ship-config's domain watcher
// (`author_change`) when a `.ship/<domain>/` file changes.
// All share the ConfigChanged payload.
register_event!("agent.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentChanged", ship_types::payload::ConfigChanged);
register_event!("collection.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigChanged", ship_types::payload::ConfigChanged);
register_event!("pipeline.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineChanged", ship_types::payload::ConfigChanged);
register_event!("rule.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "RuleChanged", ship_types::payload::ConfigChanged);
register_event!("skill.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SkillChanged", ship_types::payload::ConfigChanged);
register_event!("view.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "ViewChanged", ship_types::payload::ConfigChanged);
