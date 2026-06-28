use ship_substrate_core::events::validator::EmitContext;

/// `artifact.*` events are Space-scoped; the kernel fail-closes a space-unbound
/// caller (`ScopeViolation::NoSpaceForSpaceEvent`). Bind the artifact scope's
/// space (Run/Project carry one; User scope is space-less and stays unbound).
pub(crate) fn emit_ctx(space_id: Option<&str>) -> EmitContext {
    EmitContext::daemon_internal(space_id.map(ship_types::SpaceId::new))
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
}

