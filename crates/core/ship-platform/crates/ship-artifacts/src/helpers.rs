use ship_substrate::events::validator::EmitContext;

pub(crate) fn emit_ctx() -> EmitContext {
    // Behavior-preserving: this service emitted with no space today; the
    // daemon-internal mint keeps the keyless shape while carrying a binding.
    EmitContext::daemon_internal(None)
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
}

