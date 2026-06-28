use chrono::{DateTime, Utc};
use specta::Type;

#[derive(Debug, Default, Clone, Type)]
pub struct EventFilter {
    pub entity_id: Option<String>,
    pub event_type: Option<String>,
    /// Partition axis to filter on (storage form: `device`/`account`/`space`/
    /// `system`). `None` matches any scope.
    pub scope: Option<String>,
    /// The id within `scope` — the unified replacement for the old `space_id`
    /// filter field. `None` matches any key.
    pub scope_key: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub elevated_only: bool,
    pub since: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
}
