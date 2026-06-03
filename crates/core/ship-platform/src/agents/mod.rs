pub mod config;
pub mod import;
pub mod permissions;
pub mod harnesses;
pub mod rule;
pub mod skill;
pub mod skill_paths;
pub mod skill_vars;

#[cfg(test)]
pub mod tests;

/// Open a Runtime-scope store for the agent-file audit emits
/// (`rule.* / permissions.* / skill.*`).
///
/// These CRUD helpers take a `ship_dir` but no `space_id` and have no
/// read-back projection; per the Phase-4d resolution of the §6.3 question
/// they are emitted as a Runtime audit stream in `runtime.db` (the families
/// were reclassified to `{Runtime}` to match — see `payload_registry.rs`).
pub(crate) fn runtime_audit_store(
) -> anyhow::Result<ship_substrate::persistence::event_store::SqliteEventStore> {
    ship_substrate::persistence::event_store::SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
}

/// The audit emits' writer binding. `rule.* / permissions.* / skill.*` are
/// Runtime-scoped daemon audit events with no space at the call site, so the
/// daemon-internal binding (device id, no account, no space) is the
/// authoritative non-space binding `enforce_scope` stamps the Runtime key from.
pub(crate) fn runtime_audit_binding() -> ship_types::ScopeBinding {
    ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk)
}
