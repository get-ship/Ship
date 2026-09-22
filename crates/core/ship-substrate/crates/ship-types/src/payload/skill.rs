// -- Skill aggregate (runtime events) ------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillFailed {
    pub skill_id: String,
    pub error: String,
}
