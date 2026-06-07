//! HTTP sync client — pushes local events to cloud, pulls remote events down.

use anyhow::{Context, Result};

use ship_substrate::events::EventEnvelope;
use crate::types::{PullResponse, PushRequest, PushResponse};

pub struct SyncClient {
    base_url: String,
    agent: ureq::Agent,
}

impl SyncClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    /// Return the normalised base URL. Test-only — surfaces the private field
    /// so integration tests outside this file can assert URL shaping without
    /// Return the normalised base URL. Test-only — surfaces the private field
    /// so the integration tests in `tests/integration_client.rs` can assert
    /// URL shaping without making live HTTP calls. `pub` rather than
    /// `#[cfg(test)]` because integration tests live in a separate crate;
    /// keeping the doc comment signals intent to human readers.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Push platform events (workspace.*, job.*, session.*, gate.*) to cloud.
    pub fn push_platform_events(
        &self,
        space_id: &str,
        events: &[EventEnvelope],
    ) -> Result<PushResponse> {
        let url = format!("{}/api/sync/platform/push", self.base_url);
        let body = PushRequest {
            events: events.to_vec(),
        };
        let resp: PushResponse = self
            .agent
            .post(&url)
            .header("x-project-id", space_id)
            .send_json(&body)
            .context("sync push request failed")?
            .body_mut()
            .read_json()
            .context("sync push response parse failed")?;
        Ok(resp)
    }

    /// Pull platform events from cloud since cursor.
    pub fn pull_platform_events(
        &self,
        space_id: &str,
        cursor: Option<&str>,
    ) -> Result<PullResponse> {
        let mut url = format!("{}/api/sync/platform/pull", self.base_url);
        if let Some(c) = cursor {
            url.push_str(&format!("?since={c}"));
        }
        let resp: PullResponse = self
            .agent
            .get(&url)
            .header("x-project-id", space_id)
            .call()
            .context("sync pull request failed")?
            .body_mut()
            .read_json()
            .context("sync pull response parse failed")?;
        Ok(resp)
    }

    /// Push workspace-scoped events to cloud.
    pub fn push_workspace_events(
        &self,
        workspace_id: &str,
        space_id: &str,
        events: &[EventEnvelope],
    ) -> Result<PushResponse> {
        let url = format!(
            "{}/api/sync/workspace/{workspace_id}/push",
            self.base_url
        );
        let body = PushRequest {
            events: events.to_vec(),
        };
        let resp: PushResponse = self
            .agent
            .post(&url)
            .header("x-project-id", space_id)
            .send_json(&body)
            .context("workspace sync push failed")?
            .body_mut()
            .read_json()
            .context("workspace sync push response parse failed")?;
        Ok(resp)
    }

    /// Pull workspace-scoped events from cloud since cursor.
    pub fn pull_workspace_events(
        &self,
        workspace_id: &str,
        space_id: &str,
        cursor: Option<&str>,
    ) -> Result<PullResponse> {
        let mut url = format!(
            "{}/api/sync/workspace/{workspace_id}/pull",
            self.base_url
        );
        if let Some(c) = cursor {
            url.push_str(&format!("?since={c}"));
        }
        let resp: PullResponse = self
            .agent
            .get(&url)
            .header("x-project-id", space_id)
            .call()
            .context("workspace sync pull failed")?
            .body_mut()
            .read_json()
            .context("workspace sync pull response parse failed")?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_events() -> Vec<EventEnvelope> {
        vec![EventEnvelope {
            id: "01J0000000000000000000000A".to_string(),
            event_type: "workspace.created".to_string(),
            entity_id: "ws-1".to_string(),
            actor: "ship".to_string(),
            payload_json: "{}".to_string(),
            version: 1,
            causation_id: None,
            scope: ship_substrate::events::EventScope::Space,
            scope_key: Some("ws-1".to_string()),
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: true,
            created_at: Utc::now(),
            author_account_id: None,
            author_device_id: None,
            signature: None,
        }]
    }

    #[test]
    fn push_request_body_has_no_space_id() {
        let events = sample_events();
        let body = PushRequest { events };
        let json = serde_json::to_value(&body).unwrap();
        assert!(json.get("space_id").is_none(), "space_id must be in header, not body");
        assert_eq!(json["events"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn client_constructs_with_trailing_slash() {
        let client = SyncClient::new("https://api.example.com/");
        assert_eq!(client.base_url, "https://api.example.com");
    }

    #[test]
    fn client_constructs_without_trailing_slash() {
        let client = SyncClient::new("https://api.example.com");
        assert_eq!(client.base_url, "https://api.example.com");
    }
}
