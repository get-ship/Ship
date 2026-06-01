use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::CallerKind;

pub(crate) fn emit_ctx() -> EmitContext {
    EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, space_id: None, session_id: None }
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub(crate) fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
}

