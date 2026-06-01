use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::time::SystemTime;

use crate::event_registry::{self, EventScope};

thread_local! {
    static ULID_GEN: RefCell<ulid::Generator> = const { RefCell::new(ulid::Generator::new()) };
}

/// Monotonic ULID generator. `Generator::generate_from_datetime` fails only on
/// 64-bit randomness-counter exhaustion within the same millisecond (≈10^19
/// IDs/ms) or clock-rollback past 1970 — neither is recoverable here, so we
/// panic instead of poisoning every event-envelope caller with a Result.
#[allow(clippy::expect_used)]
fn monotonic_ulid() -> String {
    ULID_GEN.with(|g| {
        g.borrow_mut()
            .generate_from_datetime(SystemTime::now())
            .expect("ULID generation failed")
            .to_string()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EventEnvelope {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor: String,
    pub payload_json: String,
    pub version: u32,
    pub causation_id: Option<String>,
    /// Partition axis this event belongs to. Stamped at `new()` from the
    /// event-type registry (fail-closed `Device` for unregistered types).
    /// `#[serde(default)]` so envelopes serialized before the tuple existed
    /// still decode (→ Device).
    #[serde(default)]
    pub scope: EventScope,
    /// The id within `scope` — Space→space_id, Account→account_id,
    /// Device→device_id (or None), System→None. Together `(scope, scope_key)`
    /// is the unified replacement for the loose `space_id: Option<String>`:
    /// scope and its key always travel together.
    pub scope_key: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub target_actor_id: Option<String>,
    pub elevated: bool,
    pub created_at: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn new<P: Serialize>(
        event_type: &str,
        entity_id: &str,
        payload: &P,
    ) -> Result<Self, serde_json::Error> {
        let payload_json = serde_json::to_string(payload)?;
        Ok(Self {
            id: monotonic_ulid(),
            event_type: event_type.to_string(),
            entity_id: entity_id.to_string(),
            actor: "ship".to_string(),
            payload_json,
            version: 1,
            causation_id: None,
            scope: event_registry::scope_for(event_type).unwrap_or(EventScope::Device),
            scope_key: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: Utc::now(),
        })
    }

    pub fn with_causation(mut self, causation_id: &str) -> Self {
        self.causation_id = Some(causation_id.to_string());
        self
    }

    /// Attach the caller's space context + session. Behavior-preserving shim
    /// for the legacy `space_id`-keyed call sites: a present key means the event
    /// belongs to that space, so this stamps `(Space, key)`. A `None` key leaves
    /// the registry-inferred scope from `new()` untouched. Phase 1/2 replace
    /// this caller-supplied key with the authoritative substrate scope binding.
    pub fn with_context(mut self, space_id: Option<&str>, session_id: Option<&str>) -> Self {
        if let Some(key) = space_id {
            self.scope = EventScope::Space;
            self.scope_key = Some(key.to_string());
        }
        self.session_id = session_id.map(str::to_string);
        self
    }

    /// Set the full `(scope, scope_key)` tuple directly. The explicit form used
    /// by call sites that know the event's partition (Account/System/Device),
    /// not only Space.
    pub fn with_scope(mut self, scope: EventScope, scope_key: Option<&str>) -> Self {
        self.scope = scope;
        self.scope_key = scope_key.map(str::to_string);
        self
    }

    /// The space id this event belongs to, or `None` when it is not space-scoped.
    /// The legacy `space_id` accessor expressed over the new tuple — read sites
    /// that genuinely mean "which space" use this.
    pub fn space_id(&self) -> Option<&str> {
        match self.scope {
            EventScope::Space => self.scope_key.as_deref(),
            _ => None,
        }
    }

    pub fn with_actor_id(mut self, actor_id: &str) -> Self {
        self.actor_id = Some(actor_id.to_string());
        self
    }

    pub fn with_parent_actor_id(mut self, parent_actor_id: &str) -> Self {
        self.parent_actor_id = Some(parent_actor_id.to_string());
        self
    }

    pub fn with_target(mut self, target_actor_id: &str) -> Self {
        self.target_actor_id = Some(target_actor_id.to_string());
        self
    }

    pub fn elevate(mut self) -> Self {
        self.elevated = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_new_sets_required_fields() {
        let env = EventEnvelope::new("workspace.created", "ws-1", &serde_json::json!({}))
            .expect("new must succeed");
        assert_eq!(env.event_type, "workspace.created");
        assert_eq!(env.entity_id, "ws-1");
        assert_eq!(env.id.len(), 26, "id must be a 26-char ULID");
        assert_eq!(env.version, 1);
        assert!(!env.elevated);
    }

    #[test]
    fn event_envelope_builder_chain() {
        let env = EventEnvelope::new("workspace.created", "ws-1", &serde_json::json!({}))
            .unwrap()
            .with_causation("cause-1")
            .with_actor_id("agent.x")
            .with_target("agent.y")
            .elevate();
        assert_eq!(env.causation_id.as_deref(), Some("cause-1"));
        assert_eq!(env.actor_id.as_deref(), Some("agent.x"));
        assert_eq!(env.target_actor_id.as_deref(), Some("agent.y"));
        assert!(env.elevated);
    }

    #[test]
    fn event_envelope_roundtrips_serde() {
        let env = EventEnvelope::new("workspace.created", "ws-1", &serde_json::json!({})).unwrap();
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.event_type, env.event_type);
    }
}
