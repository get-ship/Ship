use chrono::{DateTime, Utc};
use specta::Type;

#[derive(Debug, Default, Clone, Type)]
pub struct EventFilter {
    pub entity_id: Option<String>,
    pub event_type: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub elevated_only: bool,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
