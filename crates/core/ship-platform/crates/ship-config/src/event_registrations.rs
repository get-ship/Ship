//! Event registrations for the CONFIG family.
//!
//! Registers the watcher-ping events (`config.changed`, `config.file_changed`,
//! `<domain>.changed` × 6) and the `agent.def.updated` Format-2 def
//! notification with the global event registry using the hoisted
//! `register_event!` macro from `ship_types`. All of these are EMITTED by
//! ship-config (the watcher, the gated def write door, and the config
//! service), so the registration must be link-time present here.
//!
//! Keystone-A Phase A bundle 3 (2026-06-10): the watcher-ping family moved
//! from `ship_substrate::events::payload_registry`. The `agent.def.updated`
//! registration moved from `agent-def/src/event_registrations.rs` in the
//! event-routing correctness fix (2026-06-19): the emitter is ship-config's
//! watcher / agents_write path, not agent-def.
//!
//! Payload structs (`ConfigChanged`, `ConfigFileChanged`, `AgentDefUpdated`)
//! STAY in `ship_types::payload` — `ConfigChanged` is the shared type used
//! by all watcher events; the others have no owner-specific reason to move.

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

// ── agent.def.updated (Format-2 def notification) ───────────────────────────
// Fine-grained, space-stamped, content-hashed notification that an
// `.ship/agents/<id>.jsonc` was written. Emitted by the gated write path
// (`agents_write::save_agents_config`) and the watcher (`watcher.rs`); the
// notification carries ONLY {path, content_hash} — never file content. The
// read-only def index re-reads the file on the ping. Space-scoped because the
// def lives in a space's `.ship`.
//
// Lives here (not agent-def) because this is the EMITTER's crate — a test
// binary linking ship-config without agent-def still needs the registration
// to be link-time present for the kernel to route the event.
register_event!("agent.def.updated", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentDefUpdated", ship_types::payload::AgentDefUpdated);
