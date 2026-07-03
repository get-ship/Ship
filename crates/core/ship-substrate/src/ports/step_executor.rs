//! StepExecutor — L0 port for pipeline step command execution.
//!
//! "Execute a pipeline step's shell command in a working directory and give
//! me the result."  The port owns the spawn mechanism (via `spawn_bounded`)
//! and enforces resource/time caps so no step command can unboundedly
//! consume memory or wall-clock time.
//!
//! Callers declare intent:
//!   `step_executor.run(command, work_dir).await?`
//!
//! The port owns the mechanism: `sh -c <command>` via `spawn_bounded::Cmd`,
//! with the 2 GB virtual-memory cap and 30 s timeout that `spawn_bounded`
//! provides by default.

use std::path::Path;
use thiserror::Error;

/// The output captured from a pipeline step command.
#[derive(Debug, Clone)]
pub struct StepOutput {
    /// True when the process exited with status 0.
    pub success: bool,
    /// The process exit code, or -1 if the OS did not report one.
    pub exit_code: i32,
    /// Captured stdout (empty string when the command produced none).
    pub stdout: String,
    /// Captured stderr (empty string when the command produced none).
    pub stderr: String,
}

/// Errors that `StepExecutor::run` can surface.
#[derive(Debug, Error)]
pub enum StepExecutorError {
    /// The step command exceeded the wall-clock time cap.
    #[error("step command timed out after {0}s")]
    TimedOut(u64),
    /// The step command was killed for exceeding the virtual-memory cap.
    #[error("step command exceeded memory cap ({0} MB virtual)")]
    MemoryCapped(u64),
    /// An I/O error prevented the command from being spawned or read.
    #[error("step command spawn error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<crate::spawn_bounded::BoundedError> for StepExecutorError {
    fn from(e: crate::spawn_bounded::BoundedError) -> Self {
        match e {
            crate::spawn_bounded::BoundedError::TimedOut(d) => {
                StepExecutorError::TimedOut(d.as_secs())
            }
            crate::spawn_bounded::BoundedError::OomCapped(mb) => {
                StepExecutorError::MemoryCapped(mb)
            }
            crate::spawn_bounded::BoundedError::Io(e) => StepExecutorError::Io(e),
        }
    }
}

/// L0 port: execute a pipeline step's shell command with bounded resources.
///
/// Constructed once per step dispatch; not a long-lived service.
/// All resource caps are owned here — callers never touch spawn primitives.
pub struct StepExecutor;

impl StepExecutor {
    /// Execute `command` (interpreted by `sh -c`) in `work_dir`.
    ///
    /// Returns `StepOutput` on any process exit (including non-zero).
    /// Returns `Err` only when the command could not be spawned at all,
    /// timed out, or was killed by the memory cap.
    pub async fn run(
        &self,
        command: &str,
        work_dir: &Path,
    ) -> Result<StepOutput, StepExecutorError> {
        use crate::spawn_bounded::Cmd;

        let out = Cmd::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(work_dir)
            .output_async()
            .await?;

        let exit_code = out.status.code().unwrap_or(-1);
        Ok(StepOutput {
            success: out.status.success(),
            exit_code,
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp() -> PathBuf {
        std::env::temp_dir()
    }

    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn run_successful_command() {
        let ex = StepExecutor;
        let out = ex.run("printf hello", &tmp()).await.expect("ok");
        assert!(out.success);
        assert_eq!(out.exit_code, 0);
        assert_eq!(out.stdout, "hello");
        assert!(out.stderr.is_empty());
    }

    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn run_failing_command() {
        let ex = StepExecutor;
        let out = ex.run("exit 42", &tmp()).await.expect("spawned");
        assert!(!out.success);
        assert_eq!(out.exit_code, 42);
    }

    #[tokio::test]
    #[cfg_attr(not(unix), ignore)]
    async fn run_captures_stderr() {
        let ex = StepExecutor;
        let out = ex.run("echo err >&2", &tmp()).await.expect("ok");
        assert!(out.success);
        assert!(out.stderr.contains("err"), "stderr: {:?}", out.stderr);
    }
}
