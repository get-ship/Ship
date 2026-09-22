//! `agent-def` (L1) — the unified agent-definition domain.
//!
//! The agent definition + its facets (config, harnesses, rules, permissions,
//! skills, imports). Both runtimes — Floki (embedded code-mode) and ship-code
//! (third-party-harness drivers) — read this ONE definition; the control plane
//! and shipd depend on it for config CRUD without pulling either runtime.
//!
//! (Floki carve D2a: relocation only — same types, new crate boundary. The
//! schema unification into a single `Agent` + runtime discriminant is D2b.)

// The stack->harness compile driver — third-party-harness territory
// (emits via harness-config-compiler). Explicitly gated: floki and the standard
// build never carry it.
#[cfg(feature = "bridge")]
pub mod compile;
pub mod emit;
pub mod file_changed_consumer;
pub mod skill;
pub mod skill_paths;
// KV-backed skill vars need sqlx — gated so Yoke/floki can depend on the thin
// Agent + session-event surface without sqlite.
mod event_registrations;
#[cfg(feature = "kv")]
pub mod skill_vars;
#[cfg(feature = "kv")]
pub mod skill_vars_store;
pub use emit::SessionEventSink;

#[cfg(test)]
pub mod tests;
#[cfg(test)]
mod testutil;
