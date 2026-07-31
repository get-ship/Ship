//! Event registrations for the CONFIG family.
//!
//! Registers the watcher-ping events (`config.changed`, `config.file_changed`,
//! `<domain>.changed` × 6) with the global event registry via the hoisted
//! `register_event!` macro from `ship_types`. These are EMITTED by ship-settings
//! (the watcher and the config service), so the registration is link-time
//! present here.
//!
//! `agent.def.updated` is NOT registered here — it's owned by the agent-def
//! domain (`agent-def/src/event_registrations.rs`). ship-settings still emits it
//! from the gated def-write path and links agent-def as a dev-dependency so its
//! tests carry the registration; the daemon links both in production.
//!
//! Payload structs (`ConfigChanged`, `ConfigFileChanged`) STAY in
//! `ship_types::payload` — `ConfigChanged` is the shared watcher-event type.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── config.changed / config.file_changed ────────────────────────────────────
// Machine/daemon runtime settings (daemon.port, terminal.program, watcher.*,
// isolation tiers) — per-machine, never synced. `set_setting_and_emit` mints
// `config.changed` is a daemon-settings mutation — the platform record
// of "this setting was set to this value at this time". It is a Durable
// fact (the settings are the source of truth on disk; the event log is
// the journal of how they got there). Reclassified from Ephemeral to
// Durable in job/transient-events so the kernel's `route` honors
// retention (Ephemeral now means "deliver to live subscribers only").
register_event!(
    "config.changed",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "ConfigChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "config.file_changed",
    [EventScope::Runtime],
    Retention::Ephemeral,
    Stability::Stable,
    "ConfigFileChanged",
    ship_types::payload::ConfigFileChanged
);

// ── <domain>.changed watcher-ping family ────────────────────────────────────
// The coarse-grain watcher pings: emitted by ship-settings's domain watcher
// (`author_change`) when a `.ship/<domain>/` file changes.
// All share the ConfigChanged payload.
register_event!(
    "agent.changed",
    [EventScope::Space],
    Retention::Ephemeral,
    Stability::Stable,
    "AgentChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "collection.changed",
    [EventScope::Runtime],
    Retention::Ephemeral,
    Stability::Stable,
    "ConfigChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "pipeline.changed",
    [EventScope::Runtime],
    Retention::Ephemeral,
    Stability::Stable,
    "PipelineChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "rule.changed",
    [EventScope::Runtime],
    Retention::Ephemeral,
    Stability::Stable,
    "RuleChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "skill.changed",
    [EventScope::Runtime],
    Retention::Ephemeral,
    Stability::Stable,
    "SkillChanged",
    ship_types::payload::ConfigChanged
);
register_event!(
    "view.changed",
    [EventScope::Space],
    Retention::Ephemeral,
    Stability::Stable,
    "ViewChanged",
    ship_types::payload::ConfigChanged
);
