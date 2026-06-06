pub mod config;
pub mod harnesses;
pub mod import;
pub mod permissions;
pub mod rule;
pub mod skill;
pub mod skill_paths;
pub mod skill_vars;

#[cfg(test)]
pub mod tests;

pub(crate) fn runtime_audit_store(
) -> anyhow::Result<ship_substrate::persistence::event_store::SqliteEventStore> {
    ship_substrate::persistence::event_store::SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
}

pub(crate) fn runtime_audit_binding() -> ship_types::ScopeBinding {
    ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk)
}
