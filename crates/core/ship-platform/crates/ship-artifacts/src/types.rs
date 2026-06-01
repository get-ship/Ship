/// Three-scope model — run artifacts are ephemeral, project are persistent,
/// user are personal across workspaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactScope {
    Run { space_id: String, run_id: String },
    Project { space_id: String },
    User { user_id: String },
}

impl ArtifactScope {
    pub fn kind_str(&self) -> &'static str {
        match self {
            ArtifactScope::Run { .. } => "run",
            ArtifactScope::Project { .. } => "project",
            ArtifactScope::User { .. } => "user",
        }
    }

    pub fn space_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::Run { space_id, .. } | ArtifactScope::Project { space_id } => {
                Some(space_id.as_str())
            }
            ArtifactScope::User { .. } => None,
        }
    }

    pub fn run_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::Run { run_id, .. } => Some(run_id.as_str()),
            _ => None,
        }
    }

    pub fn user_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::User { user_id } => Some(user_id.as_str()),
            _ => None,
        }
    }

    pub fn lock_key(&self) -> String {
        match self {
            ArtifactScope::Run { space_id, run_id } => format!("run:{space_id}:{run_id}"),
            ArtifactScope::Project { space_id } => format!("project:{space_id}"),
            ArtifactScope::User { user_id } => format!("user:{user_id}"),
        }
    }
}

/// Metadata record returned by service methods.
#[derive(Debug, Clone)]
pub struct ArtifactRecord {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    /// Relative blob path (from global_dir). Internal — needed to locate bytes.
    pub path: String,
    pub scope: String,
    pub space_id: Option<String>,
    pub run_id: Option<String>,
    pub user_id: Option<String>,
    pub content_hash: Option<String>,
    pub expires_at: Option<String>,
    pub owner_actor: Option<String>,
    /// Plain typed string — no ship-collection dependency.
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub created_at: String,
}
