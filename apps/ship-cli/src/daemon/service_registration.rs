//! Cross-platform service registration for `shipd`.
//!
//! One Rust entry point (`install_service` / `uninstall_service`) backed
//! by platform-specific submodules that write the OS-native service
//! description (systemd user unit / launchd user agent / Windows Task
//! Scheduler task), load/start it, and undo the same on uninstall.
//!
//! Called from:
//!   * `ship daemon install-service` / `ship daemon uninstall-service`
//!   * installer post-install hooks (.deb postinst, .rpm post, .pkg
//!     postinstall, .msi custom action) via `ship daemon install-service`
//!   * the legacy `scripts/install.sh` dev fallback
//!
//! Idempotent: re-running on an already-registered system silently
//! replaces the existing service file. Uninstall is a no-op when the
//! service isn't registered.

use anyhow::{Context, Result};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Options controlling what happens after the service description lands
/// on disk. Callers pass `InstallOptions::default()` for the canonical
/// installer flow (drop file + start it).
#[derive(Clone, Debug)]
pub struct InstallOptions {
    /// Start the service immediately after registering it.
    pub start_now: bool,
    /// Enable the service to run on every login (autostart).
    pub enable_autostart: bool,
    /// Absolute path to the `shipd` binary. Detected automatically when
    /// `None` — walks `$SHIPD_PATH`, the sibling of the running `ship`,
    /// and the canonical platform install paths.
    pub shipd_path: Option<PathBuf>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            start_now: true,
            enable_autostart: true,
            shipd_path: None,
        }
    }
}

/// Report surfaced to the CLI on install/uninstall. Human-readable; the
/// CLI prints `message` and returns the exit code from the underlying
/// OS call (0 on success).
#[derive(Clone, Debug)]
pub struct ServiceReport {
    pub message: String,
}

impl ServiceReport {
    fn ok(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

/// Register `shipd` with the OS service manager.
pub fn install_service(opts: InstallOptions) -> Result<ServiceReport> {
    let shipd = resolve_shipd_path(opts.shipd_path.as_deref())
        .context("locate shipd binary")?;
    #[cfg(target_os = "linux")]
    {
        return linux::install(&shipd, &opts);
    }
    #[cfg(target_os = "macos")]
    {
        return macos::install(&shipd, &opts);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::install(&shipd, &opts);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = opts;
        let _ = shipd;
        anyhow::bail!("service registration is not supported on this platform")
    }
}

/// Unregister `shipd` from the OS service manager. No-op when nothing
/// is registered.
pub fn uninstall_service() -> Result<ServiceReport> {
    #[cfg(target_os = "linux")]
    {
        return linux::uninstall();
    }
    #[cfg(target_os = "macos")]
    {
        return macos::uninstall();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::uninstall();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("service registration is not supported on this platform")
    }
}

/// Resolve the absolute path to `shipd`. Priority:
///   1. Explicit override (CLI `--shipd-path` / `SHIPD_PATH` env).
///   2. Sibling of the currently-running `ship` executable.
///   3. `which shipd` — wherever the shell would pick it up.
///   4. Canonical platform install path (`/usr/bin/shipd` on Linux,
///      `/usr/local/bin/shipd` on macOS, `shipd.exe` in the same dir as
///      `ship.exe` on Windows).
fn resolve_shipd_path(explicit: Option<&std::path::Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }
    if let Ok(env) = ship_substrate::env::read_raw(ship_substrate::env::Var::DaemonPath)
        && !env.is_empty()
    {
        return Ok(PathBuf::from(env));
    }
    if let Ok(self_exe) = std::env::current_exe()
        && let Some(parent) = self_exe.parent()
    {
        let candidate = parent.join(shipd_filename());
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    if let Some(found) = which_shipd() {
        return Ok(found);
    }
    for fallback in default_paths() {
        if fallback.exists() {
            return Ok(fallback);
        }
    }
    anyhow::bail!("could not locate the shipd binary; set SHIPD_PATH or reinstall Ship")
}

fn shipd_filename() -> &'static str {
    if cfg!(target_os = "windows") { "shipd.exe" } else { "shipd" }
}

fn which_shipd() -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let separator = if cfg!(target_os = "windows") { ';' } else { ':' };
    for entry in path_var.split(separator) {
        if entry.is_empty() {
            continue;
        }
        let candidate = std::path::Path::new(entry).join(shipd_filename());
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn default_paths() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        vec![
            PathBuf::from("/usr/bin/shipd"),
            PathBuf::from("/usr/local/bin/shipd"),
            dirs::home_dir()
                .map(|h| ship_substrate::ShipHome::project_ship_dir(h).join("bin").join("shipd"))
                .unwrap_or_default(),
        ]
    }
    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("/usr/local/bin/shipd"),
            PathBuf::from("/opt/homebrew/bin/shipd"),
            dirs::home_dir()
                .map(|h| ship_substrate::ShipHome::project_ship_dir(h).join("bin").join("shipd"))
                .unwrap_or_default(),
        ]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from(r"C:\Program Files\Ship\shipd.exe"),
            dirs::home_dir()
                .map(|h| ship_substrate::ShipHome::project_ship_dir(h).join("bin").join("shipd.exe"))
                .unwrap_or_default(),
        ]
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_shipd_accepts_explicit_override() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let resolved = resolve_shipd_path(Some(&path)).unwrap();
        assert_eq!(resolved, path);
    }

    #[test]
    fn shipd_filename_is_platform_appropriate() {
        let name = shipd_filename();
        if cfg!(windows) {
            assert_eq!(name, "shipd.exe");
        } else {
            assert_eq!(name, "shipd");
        }
    }
}
