//! Dot-path filter matching for hook triggers.

use serde_json::Value;

/// Evaluate `filter` against a context built from the event envelope.
///
/// Every key in `filter` is a dot-path into `ctx`. The hook matches when every
/// path resolves and every value matches exactly. An absent filter matches.
pub fn matches(filter: Option<&Value>, ctx: &Value) -> bool {
    let Some(filter) = filter else { return true };
    let Some(obj) = filter.as_object() else {
        return false;
    };
    obj.iter().all(|(path, expected)| {
        resolve_path(ctx, path)
            .map(|found| values_match(found, expected))
            .unwrap_or(false)
    })
}

/// Build the context an event is matched/rendered against. Mirrors the
/// envelope surface so hook authors can write `event.payload.status`,
/// `event.entity_id`, etc.
pub fn build_event_context(envelope: &ship_substrate::events::EventEnvelope) -> Value {
    let payload: Value = serde_json::from_str(&envelope.payload_json).unwrap_or(Value::Null);
    serde_json::json!({
        "id": envelope.id,
        "event_type": envelope.event_type,
        "entity_id": envelope.entity_id,
        "actor": envelope.actor,
        "actor_id": envelope.actor_id,
        "scope": envelope.scope.as_str(),
        "scope_key": envelope.scope_key,
        // Legacy accessor: the space id when space-scoped, else null. Kept so
        // existing `event.space_id` hook expressions keep resolving.
        "space_id": envelope.space_id(),
        "session_id": envelope.session_id,
        "causation_id": envelope.causation_id,
        "payload": payload,
    })
}

fn resolve_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(segment)?,
            Value::Array(arr) => {
                let idx: usize = segment.parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }
    Some(current)
}

fn values_match(found: &Value, expected: &Value) -> bool {
    found == expected
}

