//! Data helpers for the `chat` daemon-rendered view.
//!
//! Converts event envelopes into conversation messages, mirroring the
//! React `ChatTab.tsx` shaping logic (user vs agent vs system, session
//! progress, job transitions). The heavy lifting stays in the template —
//! this module only reshapes envelopes into a serializable row.

use serde::Serialize;
use ship_substrate::events::EventEnvelope;
use std::path::Path;

pub const DEFAULT_LIMIT: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Sender {
    User,
    Agent,
    System,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChatMessage {
    pub id: String,
    pub sender: Sender,
    pub content: String,
    pub detail: Option<String>,
    pub timestamp: String,
    pub event_type: String,
    pub workspace_id: String,
}

fn payload_str(env: &EventEnvelope, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(&env.payload_json)
        .ok()
        .and_then(|v| v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string()))
}

fn to_message(env: &EventEnvelope) -> Option<ChatMessage> {
    let ts = env.created_at.format("%H:%M").to_string();
    let workspace_id = env.project_id.clone().unwrap_or_default();
    let base = |sender, content: String, detail: Option<String>| ChatMessage {
        id: env.id.clone(),
        sender,
        content,
        detail,
        timestamp: ts.clone(),
        event_type: env.event_type.clone(),
        workspace_id: workspace_id.clone(),
    };
    match env.event_type.as_str() {
        "studio.message" => {
            let text = payload_str(env, "summary")
                .or_else(|| payload_str(env, "text"))
                .unwrap_or_else(|| "Message sent".to_string());
            Some(base(Sender::User, text, None))
        }
        "session.progress" => {
            let msg = payload_str(env, "message")?;
            let escalation = payload_str(env, "escalation");
            Some(base(Sender::Agent, msg, escalation.map(|e| format!("[{e}]"))))
        }
        "session.started" => {
            let agent = payload_str(env, "agent_id").unwrap_or_default();
            let content = if agent.is_empty() {
                "Session started".into()
            } else {
                format!("Session started ({agent})")
            };
            Some(base(Sender::System, content, None))
        }
        "session.ended" => {
            let summary = payload_str(env, "summary").unwrap_or_else(|| "Session ended".into());
            Some(base(Sender::System, summary, None))
        }
        kind if kind.starts_with("pipeline.") => {
            let slug = payload_str(env, "slug").unwrap_or_else(|| env.entity_id.clone());
            let action = kind.trim_start_matches("pipeline.");
            Some(base(Sender::System, format!("Run {slug}: {action}"), None))
        }
        _ => None,
    }
}

/// Load recent messages for `workspace_id`, oldest first (chat order).
/// A `None` workspace disables scoping and returns all messages.
pub fn list_messages(
    ship_dir: &Path,
    workspace_id: Option<&str>,
    limit: usize,
) -> Vec<ChatMessage> {
    let envs = ship_substrate::db::events::list_recent_events_at(ship_dir, limit.max(1))
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "chat: list_recent_events_at failed");
            Vec::new()
        });
    envs.iter()
        .filter(|e| match (workspace_id, e.project_id.as_deref()) {
            (Some(w), Some(ew)) => w == ew,
            (Some(_), None) => false,
            (None, _) => true,
        })
        .filter_map(to_message)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn env(kind: &str, payload: serde_json::Value, ws: Option<&str>) -> EventEnvelope {
        EventEnvelope {
            id: "1".into(),
            event_type: kind.into(),
            entity_id: "e".into(),
            actor: "a".into(),
            payload_json: payload.to_string(),
            version: 1,
            causation_id: None,
            project_id: ws.map(String::from),
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn studio_message_becomes_user() {
        let m = to_message(&env(
            "studio.message",
            serde_json::json!({ "summary": "hi" }),
            None,
        ))
        .unwrap();
        assert_eq!(m.sender, Sender::User);
        assert_eq!(m.content, "hi");
    }

    #[test]
    fn session_progress_carries_escalation() {
        let m = to_message(&env(
            "session.progress",
            serde_json::json!({ "message": "working", "escalation": "needs_human" }),
            None,
        ))
        .unwrap();
        assert_eq!(m.sender, Sender::Agent);
        assert_eq!(m.detail.as_deref(), Some("[needs_human]"));
    }

    #[test]
    fn unknown_event_type_is_dropped() {
        assert!(to_message(&env("random.event", serde_json::json!({}), None)).is_none());
    }

    #[test]
    fn run_events_become_system_messages() {
        let m = to_message(&env(
            "pipeline.run.completed",
            serde_json::json!({ "slug": "my-job" }),
            None,
        ))
        .unwrap();
        assert_eq!(m.sender, Sender::System);
        assert!(m.content.contains("my-job"));
        assert!(m.content.contains("completed"));
    }
}
