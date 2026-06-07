use async_trait::async_trait;
use std::sync::Arc;

use super::handle::{SandboxHandle, SandboxId};
use super::policy::SandboxPolicy;
use super::trait_def::{Isolator, IsolatorError};

#[derive(Debug, Default, Clone)]
pub struct Noop;

#[async_trait]
impl Isolator for Noop {
    fn name(&self) -> &'static str {
        "noop"
    }

    async fn available(&self) -> Result<(), IsolatorError> {
        Ok(())
    }

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError> {
        let root = policy
            .mounts
            .first()
            .map(|m| m.host_path.clone())
            .unwrap_or_else(std::env::temp_dir);
        Ok(SandboxHandle {
            id: SandboxId::new(),
            backend: "noop",
            root,
            state: Arc::new(()),
        })
    }

    async fn shutdown(&self, _sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_always_available() {
        assert!(Noop.available().await.is_ok());
    }

    #[tokio::test]
    async fn noop_prepare_yields_handle() {
        let pol = SandboxPolicy::workspace_default(std::env::temp_dir());
        let h = Noop.prepare(&pol).await.unwrap();
        assert_eq!(h.backend, "noop");
        Noop.shutdown(h).await.unwrap();
    }
}
