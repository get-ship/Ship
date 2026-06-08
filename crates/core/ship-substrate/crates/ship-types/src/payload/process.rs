// -- Process events ----------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessOrphaned {
    pub pid: u32,
    pub cmd: String,
    pub last_known_slug: Option<String>,
    pub detected_by: String,
}
