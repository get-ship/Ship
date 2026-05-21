use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

impl SandboxId {
    pub fn new() -> Self {
        Self(nanoid::nanoid!(12))
    }
}

impl Default for SandboxId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SandboxHandle {
    pub id: SandboxId,
    pub backend: &'static str,
    pub root: PathBuf,
    pub state: Arc<dyn std::any::Any + Send + Sync>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_id_is_non_empty_and_unique_enough() {
        let a = SandboxId::new();
        let b = SandboxId::new();
        assert!(!a.0.is_empty());
        assert_ne!(a, b);
    }
}
