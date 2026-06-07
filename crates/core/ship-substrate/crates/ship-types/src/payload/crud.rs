// -- Skill / Rule / Permission CRUD -----------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCreated {
    pub scope: String,
    pub name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillUpdated {
    pub scope: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillDeleted {
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleCreated {
    pub file_name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleUpdated {
    pub file_name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleDeleted {
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionsUpdated {
    pub scope: String,
    pub content_hash: String,
    pub permissions_json: String,
}
