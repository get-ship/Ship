//! systemd [`SupervisionAdapter`]. Shells out to `systemctl --user` and
//! `journalctl --user`; no D-Bus dep (would nearly double the binary).

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use super::supervision::{AdapterKind, StartContext, SupervisionAdapter, SupervisionStatus};

const UNIT_NAME: &str = "shipd.service";

pub struct SystemdAdapter;

impl SystemdAdapter {
    /// True when `shipd.service` is registered with the user systemd instance.
    /// Covers "enabled", "static", "alias" — anything `is-enabled` accepts.
    pub fn is_managing_shipd() -> bool {
        is_managing_unit(UNIT_NAME)
    }
}

impl SupervisionAdapter for SystemdAdapter {
    fn name(&self) -> AdapterKind {
        AdapterKind::Systemd
    }

    fn status(&self, _global_dir: &Path) -> Result<SupervisionStatus> {
        let props = query_show(UNIT_NAME)?;
        let active_state = props.get("ActiveState").map(String::as_str).unwrap_or("");
        let running = active_state == "active";
        let pid = props
            .get("MainPID")
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|p| *p != 0);
        let uptime_secs = compute_uptime(props.get("ActiveEnterTimestampMonotonic"));

        let mut extras = Vec::new();
        if !active_state.is_empty() {
            extras.push(("active_state".to_string(), active_state.to_string()));
        }
        if let Some(sub) = props.get("SubState")
            && !sub.is_empty()
        {
            extras.push(("sub_state".to_string(), sub.clone()));
        }
        if let Some(nr) = props.get("NRestarts") {
            extras.push(("n_restarts".to_string(), nr.clone()));
        }
        if let Some(ts) = props.get("ExecMainStartTimestamp")
            && !ts.is_empty()
        {
            extras.push(("started_at".to_string(), ts.clone()));
        }
        if let Some(tail) = journal_tail(UNIT_NAME, 5) {
            extras.push(("journal_tail".to_string(), tail));
        }

        Ok(SupervisionStatus {
            adapter: AdapterKind::Systemd,
            running,
            pid,
            version: None,
            uptime_secs,
            extras,
        })
    }

    fn start(&self, _ctx: &StartContext) -> Result<()> {
        systemctl(&["start", UNIT_NAME])
    }

    fn stop(&self) -> Result<()> {
        systemctl(&["stop", UNIT_NAME])
    }

    fn restart(&self, _ctx: &StartContext) -> Result<()> {
        systemctl(&["restart", UNIT_NAME])
    }
}

/// Run `systemctl --user <args...>` and surface a useful error on failure.
///
/// We inherit stdin but capture stdout/stderr so the caller sees
/// systemd's real complaint (e.g. "Unit shipd.service not found") rather
/// than a generic non-zero exit.
fn systemctl(args: &[&str]) -> Result<()> {
    let mut full_args: Vec<&str> = vec!["--user"];
    full_args.extend_from_slice(args);
    let output = Command::new("systemctl")
        .args(&full_args)
        .output()
        .with_context(|| format!("cannot invoke systemctl {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        if msg.is_empty() {
            anyhow::bail!(
                "systemctl --user {} failed (exit {})",
                args.join(" "),
                output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".to_string())
            );
        }
        anyhow::bail!("systemctl --user {} failed: {msg}", args.join(" "));
    }
    Ok(())
}

/// Run `systemctl --user is-enabled <unit>` silently. Exit 0 ⇒ managed.
fn is_managing_unit(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["--user", "is-enabled", unit])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Parse `Key=Value` lines from `systemctl --user show <unit> --property=...`.
fn query_show(unit: &str) -> Result<std::collections::HashMap<String, String>> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "show",
            unit,
            "--property=ActiveState,SubState,MainPID,NRestarts,\
             ExecMainStartTimestamp,ActiveEnterTimestampMonotonic",
        ])
        .output()
        .with_context(|| format!("cannot invoke systemctl show {unit}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "systemctl show {unit} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let mut map = std::collections::HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

/// uptime = proc_uptime - active_enter_monotonic; best-effort.
fn compute_uptime(active_enter_us: Option<&String>) -> Option<u64> {
    let enter_us: u64 = active_enter_us?.parse().ok()?;
    if enter_us == 0 {
        return None;
    }
    let proc_uptime = std::fs::read_to_string("/proc/uptime").ok()?;
    let boot_secs: f64 = proc_uptime.split_whitespace().next()?.parse().ok()?;
    let now_us = (boot_secs * 1_000_000.0) as u64;
    Some(now_us.saturating_sub(enter_us) / 1_000_000)
}

/// `journalctl --user -u <unit> -n <lines> --no-pager -o cat`, best-effort.
fn journal_tail(unit: &str, lines: u32) -> Option<String> {
    let output = Command::new("journalctl")
        .args([
            "--user",
            "-u",
            unit,
            "-n",
            &lines.to_string(),
            "--no-pager",
            "-o",
            "cat",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
pub(crate) fn is_managing_unit_for_tests(unit: &str) -> bool {
    is_managing_unit(unit)
}
