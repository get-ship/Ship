use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

use super::handle::{ExecHandle, SandboxHandle};
use super::policy::SandboxPolicy;

#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub bin: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
}

#[derive(Debug, Error)]
pub enum IsolatorError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("policy rejected: {0}")]
    PolicyRejected(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("sidecar error: {0}")]
    Sidecar(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
pub trait Isolator: Send + Sync {
    fn name(&self) -> &'static str;

    async fn available(&self) -> Result<(), IsolatorError>;

    async fn prepare(&self, policy: &SandboxPolicy) -> Result<SandboxHandle, IsolatorError>;

    async fn exec(
        &self,
        sandbox: &SandboxHandle,
        cmd: LaunchCommand,
    ) -> Result<ExecHandle, IsolatorError>;

    async fn shutdown(&self, sandbox: SandboxHandle) -> Result<(), IsolatorError>;

    /// Argv-rewrite hook for supervisors that own their own PTY/process
    /// lifecycle (ship-supervisor's ProcessExecutor). Returns the
    /// rewritten `(bin, args)` that, when spawned, runs the original
    /// command inside the already-prepared sandbox.
    ///
    /// Backends that don't support argv-rewrite (or that don't make
    /// sense without their own exec channel — sidecar-based, cloud,
    /// etc.) return `None`. The supervisor then either falls back to
    /// `exec()` (its own PTY discarded) or refuses to enforce.
    ///
    /// Default returns `None` — backends opt in.
    fn wrap_for_external_spawn(
        &self,
        _sandbox: &SandboxHandle,
        _bin: &std::path::Path,
        _args: &[String],
    ) -> Option<(std::path::PathBuf, Vec<String>)> {
        None
    }
}
