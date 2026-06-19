// -- Project registry --------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRegistered {
    pub path: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSeen {
    pub seen_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectArchived {
    pub archived_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSwitched {
    pub space_id: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
    pub at: String,
}
