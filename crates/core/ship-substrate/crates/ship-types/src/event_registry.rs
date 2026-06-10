//! Event registry — compile-time-collected metadata for every registered event payload.
//!
//! Registrations are submitted via `inventory::submit!` — in practice through
//! hand-written `register_event!` lists in `payload_registry.rs` and
//! `agent-def`'s `event_registrations.rs` (the `#[derive(EventPayload)]` derive
//! exists but currently has no users). `registry()` assembles all submissions at
//! link time, so there is no central aggregation list — but registration entries
//! themselves are hand-maintained and can drift from their payload structs.
//!
//! ## Naming
//!
//! `EventScope` (not `Scope`) avoids collision with `ship_types::scope::Scope`
//! (the authoring-scope enum for vector directories).

/// Which partition the event rides → which `(scope, scope_key)` it belongs to
/// on the envelope, and what it syncs with. The variant IS the sync boundary:
/// who OWNS (writes the authoritative truth) defines the partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum EventScope {
    /// Execution state owned per-machine — sessions, pipelines, steps,
    /// processes, actors, windows, daemon lifecycle, machine config. Snapshot/
    /// restorable, NOT continuously synced. `scope_key` = the machine/device id.
    /// Fail-closed default for unregistered/legacy events.
    #[default]
    Runtime,
    /// Event belongs to the user's account; live N-device (one user) sync.
    /// `scope_key` = the account id.
    Account,
    /// Collaborative space work; live N-collaborator sync. Replayed during
    /// space restoration. `scope_key` = the space id.
    Space,
}

impl EventScope {
    /// Stable lowercase wire/storage form. Persisted in the `events.scope`
    /// column and carried on the serialized envelope.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventScope::Runtime => "runtime",
            EventScope::Account => "account",
            EventScope::Space => "space",
        }
    }

    /// Parse the storage/wire form back to a variant. Returns `None` for any
    /// unrecognized string (callers decide the fail-closed default).
    pub fn from_storage(s: &str) -> Option<Self> {
        match s {
            "runtime" => Some(EventScope::Runtime),
            "account" => Some(EventScope::Account),
            "space" => Some(EventScope::Space),
            _ => None,
        }
    }
}

impl std::fmt::Display for EventScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether the event is kept in full history or windowed/dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Retention {
    /// Persisted forever; replayed during projection bootstrap.
    Durable,
    /// Windowed; only the last N or a recent time window is retained.
    Ephemeral,
}

/// Maturity of the event's schema contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stability {
    /// Schema is stable; breaking changes require a major-version bump.
    Stable,
    /// Schema may change without notice.
    Experimental,
    /// Event is retained for compat but should not be used in new code.
    Deprecated,
}

/// Complete metadata for one event type. Statically allocated.
pub struct EventDef {
    /// Dot-separated event name, e.g. `"actor.created"`.
    pub name: &'static str,
    /// The SET of scopes this event type is allowed to ride (Model C). The
    /// emission's binding picks one of these; `enforce_scope` validates the
    /// chosen scope is in this set AND that the binding is authorized for it.
    /// The first element is the **primary** (default) scope — stamped at
    /// `EventEnvelope::new`. A size-1 set behaves exactly as the old
    /// single-scope model.
    pub allowed_scopes: &'static [EventScope],
    pub retention: Retention,
    pub stability: Stability,
    /// First version in which this event was emitted, if known.
    pub since_version: Option<&'static str>,
    /// `std::any::type_name`-like string for the payload struct.
    pub payload_type_name: &'static str,
    /// Payload schema validator CARRIED BY the registration (Phase 4a,
    /// keystone A: each domain owns its events' validation; the substrate
    /// validates whatever is registered in the linked binary). Deserializes
    /// the JSON payload into the declared payload struct; `Err` is the
    /// serde message. Mandatory — a registration without a real payload
    /// type cannot exist, which is what retired the hand-maintained L0
    /// `schema_for_event_type` match (it silently skipped 26 registered
    /// events and could never see L1 payloads).
    pub validate_payload: fn(&str) -> Result<(), String>,
}

impl EventDef {
    /// The default scope stamped at `EventEnvelope::new` — the first element of
    /// the allowed-set. Every registered event has at least one scope (the
    /// `register_event!` macro and `#[event(...)]` derive both require one).
    pub fn primary_scope(&self) -> EventScope {
        // SAFETY-of-invariant: the registration macros never produce an empty
        // set. Fail-closed to Runtime if a malformed (empty) set ever appears.
        self.allowed_scopes.first().copied().unwrap_or(EventScope::Runtime)
    }
}

/// Trait implemented by the `#[derive(EventPayload)]` macro on each payload struct.
pub trait EventDefSource {
    fn event_def() -> &'static EventDef;
}

/// Registration record submitted via `inventory::submit!`.
pub struct EventRegistration {
    /// Function pointer returning the static `EventDef` for this event.
    pub def: fn() -> &'static EventDef,
}

inventory::collect!(EventRegistration);

/// Return all registered `EventDef`s, collected at link time from every crate
/// that annotated a payload struct with `#[derive(EventPayload)]`.
///
/// Order is not guaranteed (determined by linker section layout).
pub fn registry() -> impl Iterator<Item = &'static EventDef> {
    inventory::iter::<EventRegistration>.into_iter().map(|r| (r.def)())
}

/// Look up a single event by name.
pub fn lookup(name: &str) -> Option<&'static EventDef> {
    registry().find(|d| d.name == name)
}

/// Resolve the **primary** partition scope for an event type from the registry.
/// Returns `None` when the type is unregistered (ad-hoc / test events); callers
/// apply the fail-closed default. This is the source consulted by
/// `EventEnvelope::new` to stamp the envelope's default `scope` (the first
/// element of the allowed-set).
pub fn scope_for(name: &str) -> Option<EventScope> {
    lookup(name).map(|d| d.primary_scope())
}

/// The full allowed-set of scopes for an event type (Model C). Returns an empty
/// slice when the type is unregistered — callers treat "not in any allowed-set"
/// per their own fail-closed rule. `enforce_scope` consults this to validate the
/// chosen scope is permitted for the event type.
pub fn allowed_scopes(name: &str) -> &'static [EventScope] {
    lookup(name).map(|d| d.allowed_scopes).unwrap_or(&[])
}

/// Returns all event names in the registry (sorted for stable output).
pub fn all_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = registry().map(|d| d.name).collect();
    names.sort_unstable();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_are_unique() {
        let names = all_names();
        let mut set = std::collections::HashSet::new();
        for name in &names {
            assert!(
                set.insert(*name),
                "duplicate event name in registry: {name}"
            );
        }
    }
}
