//! Event registry — compile-time-collected metadata for every annotated event payload.
//!
//! Each payload struct annotated with `#[derive(EventPayload)] #[event(...)]` submits
//! an `EventRegistration` via `inventory::submit!`. The `registry()` function assembles
//! all registrations at runtime (no hand-maintained list; drift is impossible).
//!
//! ## Naming
//!
//! `EventScope` (not `Scope`) avoids collision with `ship_types::scope::Scope`
//! (the authoring-scope enum for vector directories).

/// Which log the event rides → what it syncs with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventScope {
    /// Event is local to the device; never synced to the account or space log.
    Device,
    /// Event belongs to the user's account; synced across devices.
    Account,
    /// Event belongs to a specific space (project); replayed during space restoration.
    Space,
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
    pub scope: EventScope,
    pub retention: Retention,
    pub stability: Stability,
    /// First version in which this event was emitted, if known.
    pub since_version: Option<&'static str>,
    /// `std::any::type_name`-like string for the payload struct.
    pub payload_type_name: &'static str,
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
