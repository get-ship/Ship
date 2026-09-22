//! Windows Task Scheduler registration for `shipd`.
//!
//! v1 deliberately uses a per-user logon-trigger task instead of a true
//! Windows Service. A Service requires admin rights at install time and
//! brings an extra dependency (service wrapper binary). Task Scheduler
//! is built-in, runs at every user logon, and needs no elevation.
//!
//! Task name: `ShipDaemon` (chosen to be unambiguous in `schtasks /query`).

use super::{InstallOptions, ServiceReport};
use anyhow::{Context, Result};
use std::path::Path;

const TASK_NAME: &str = "ShipDaemon";

pub fn install(shipd: &Path, opts: &InstallOptions) -> Result<ServiceReport> {
    // `/f` forces overwrite of any pre-existing task — idempotent.
    let tr = format!(
        "\"{}\" --host 127.0.0.1 --port 9315",
        shipd.display()
    );
    if opts.enable_autostart {
        let status = std::process::Command::new("schtasks")
            .args([
                "/Create",
                "/TN",
                TASK_NAME,
                "/TR",
                &tr,
                "/SC",
                "ONLOGON",
                "/RL",
                "LIMITED",
                "/F",
            ])
            .status()
            .context("exec schtasks /Create")?;
        anyhow::ensure!(
            status.success(),
            "schtasks /Create exited {status}"
        );
    }
    if opts.start_now {
        let _ = std::process::Command::new("schtasks")
            .args(["/Run", "/TN", TASK_NAME])
            .status();
    }
    Ok(ServiceReport::ok(format!(
        "shipd registered with Task Scheduler as '{}'",
        TASK_NAME
    )))
}

pub fn uninstall() -> Result<ServiceReport> {
    let _ = std::process::Command::new("schtasks")
        .args(["/End", "/TN", TASK_NAME])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let status = std::process::Command::new("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let message = match status {
        Ok(s) if s.success() => format!("removed scheduled task '{TASK_NAME}'"),
        _ => "shipd Task Scheduler entry was not registered".into(),
    };
    Ok(ServiceReport::ok(message))
}
