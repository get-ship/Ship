//! The two-rail tool model the Ship exploration forced on us.
//!
//! Rail A — structured/daemon ops  -> `OpsTransport` (see transport.rs)
//! Rail B — shell / exec / fs write -> sidecar Processes/Run  (Sandbox, below)
//!
//! Rail A now lives in `transport.rs` (the agent↔shipd channel + AuthContext).
//! This file owns Rail B only: shell execution, which is refused on the host by
//! construction and routed to the sidecar in production.

use std::process::Command;

// ---------------------------------------------------------------------------
// Rail B: shell / exec -> sandbox (sidecar in prod)
// ---------------------------------------------------------------------------

pub struct ExecOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait Sandbox: Send + Sync {
    /// Run a command inside the isolation boundary. Returns the *real* exit code
    /// so the verify gate can refuse a silent failure.
    fn run(&self, cmd: &str) -> Result<ExecOutput, String>;
    fn label(&self) -> &'static str;
}

/// Default: there is no sandbox attached, so there is nowhere safe to run. We
/// refuse rather than fall back to host exec. "bash on the host is insanity."
pub struct RefuseHostSandbox;
impl Sandbox for RefuseHostSandbox {
    fn run(&self, _cmd: &str) -> Result<ExecOutput, String> {
        Err("no sidecar attached — refusing to exec on host. \
             Attach a Ship isolation tier (ship-isolation) and route through \
             the sidecar Processes/Run API."
            .into())
    }
    fn label(&self) -> &'static str {
        "refuse-host"
    }
}

/// Spike-only escape hatch (env SHIP_SPIKE_ALLOW_HOST_BASH=1) so the loop can be
/// exercised end-to-end on a dev box. Stands in for the sidecar client. NEVER
/// ships — its existence behind an explicit env flag is the whole point.
pub struct LocalSandbox;
impl Sandbox for LocalSandbox {
    fn run(&self, cmd: &str) -> Result<ExecOutput, String> {
        let out = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(ExecOutput {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
    fn label(&self) -> &'static str {
        "LOCAL-HOST-SPIKE-ONLY"
    }
}

pub fn sandbox_from_env() -> Box<dyn Sandbox> {
    match std::env::var("SHIP_SPIKE_ALLOW_HOST_BASH").as_deref() {
        Ok("1") => Box::new(LocalSandbox),
        _ => Box::new(RefuseHostSandbox),
    }
}
