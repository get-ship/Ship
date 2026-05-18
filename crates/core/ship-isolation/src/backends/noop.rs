use async_trait::async_trait;
use std::sync::Arc;
use tokio::process::Command;

use crate::handle::{ExecHandle, SandboxHandle, SandboxId};
use crate::policy::SandboxPolicy;
use crate::trait_def::{Isolator, IsolatorError, LaunchCommand};

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

    async fn exec(
        &self,
        sandbox: &SandboxHandle,
        cmd: LaunchCommand,
    ) -> Result<ExecHandle, IsolatorError> {
        let mut c = Command::new(&cmd.bin);
        c.args(&cmd.args).envs(&cmd.env).current_dir(&cmd.cwd);
        let child = c.spawn().map_err(IsolatorError::from)?;
        Ok(ExecHandle {
            sandbox: sandbox.id.clone(),
            pid: child.id(),
            started_at: std::time::SystemTime::now(),
        })
    }

    async fn shutdown(&self, _sandbox: SandboxHandle) -> Result<(), IsolatorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[tokio::test]
    async fn noop_always_available() {
        assert!(Noop.available().await.is_ok());
    }

    #[tokio::test]
    async fn noop_exec_runs_true() {
        let pol = SandboxPolicy::workspace_default(std::env::temp_dir());
        let h = Noop.prepare(&pol).await.unwrap();
        let bin = if cfg!(windows) { "cmd" } else { "true" };
        let args = if cfg!(windows) {
            vec!["/C".into(), "exit 0".into()]
        } else {
            Vec::new()
        };
        let cmd = LaunchCommand {
            bin: PathBuf::from(bin),
            args,
            env: HashMap::new(),
            cwd: std::env::temp_dir(),
        };
        let eh = Noop.exec(&h, cmd).await.unwrap();
        assert!(eh.pid.is_some());
        Noop.shutdown(h).await.unwrap();
    }
}
