//! Detect what's supervising shipd and delegate health queries.
//!
//! Linux users who opt into `contrib/shipd.service` let `systemd --user`
//! own restart policy, crash detection, and unit lifecycle; this module
//! reports its view. Everyone else falls back to the pidfile path from
//! slice 1. The adapter returns the same shape regardless, so
//! `ship daemon status` doesn't branch on implementation.

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::supervision_pidfile::PidfileAdapter;
use super::supervision_systemd::SystemdAdapter;

/// Reportable supervision state. Same shape regardless of adapter.
#[derive(Debug, Clone)]
pub struct SupervisionStatus {
    /// Which adapter answered.
    pub adapter: AdapterKind,
    /// Process is currently up (supervisor holds the lock / unit is active).
    pub running: bool,
    /// Supervisor pid (the process holding the lock or unit MainPID).
    pub pid: Option<u32>,
    /// Version reported by the supervisor (from pidfile or unit).
    pub version: Option<String>,
    /// Uptime if the supervisor can compute it.
    pub uptime_secs: Option<u64>,
    /// Adapter-specific extras (systemd: ActiveState, NRestarts, journal
    /// tail; pidfile: started_at, role).
    pub extras: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    Systemd,
    Pidfile,
}

impl AdapterKind {
    pub fn label(self) -> &'static str {
        match self {
            AdapterKind::Systemd => "systemd",
            AdapterKind::Pidfile => "ship",
        }
    }
}

/// Inputs the pidfile adapter needs to spawn shipd, plus the global
/// dir so callers (e.g. the version-mismatch guard) can resolve
/// `~/.ship/` without redoing the lookup. The systemd adapter largely
/// ignores host/dev — those live in the unit file — but still receives
/// the context so both adapters present the same surface.
#[derive(Debug, Clone)]
pub struct StartContext {
    pub host: String,
    pub dev: bool,
    pub global_dir: PathBuf,
}

pub trait SupervisionAdapter: Send + Sync {
    #[cfg_attr(not(test), allow(dead_code))]
    fn name(&self) -> AdapterKind;
    fn status(&self, global_dir: &Path) -> Result<SupervisionStatus>;
    fn start(&self, ctx: &StartContext) -> Result<()>;
    fn stop(&self) -> Result<()>;
    /// Default: stop then start. Systemd overrides with a single
    /// `systemctl --user restart` so the unit never transitions through
    /// `inactive` (keeps ExecStopPost, dependencies, etc. coherent).
    fn restart(&self, ctx: &StartContext) -> Result<()> {
        self.stop()?;
        self.start(ctx)
    }
}

/// Pick the right adapter for this machine. Systemd takes precedence
/// when `shipd.service` is registered; pidfile is the always-available
/// fallback.
pub fn select() -> Box<dyn SupervisionAdapter> {
    select_with(SystemdAdapter::is_managing_shipd())
}

/// Selection entry point that accepts the probe result, exposed for tests
/// so they don't need a live systemd user bus to verify routing.
pub(crate) fn select_with(systemd_active: bool) -> Box<dyn SupervisionAdapter> {
    if systemd_active {
        Box::new(SystemdAdapter)
    } else {
        Box::new(PidfileAdapter)
    }
}
