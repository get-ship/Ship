use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor: String,
    pub payload_json: String,
    pub version: u32,
    pub causation_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub target_actor_id: Option<String>,
    pub elevated: bool,
    pub created_at: DateTime<Utc>,
}

// impl EventEnvelope — not yet implemented; tests below define the contract.

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
