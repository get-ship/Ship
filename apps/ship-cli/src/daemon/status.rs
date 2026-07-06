use anyhow::Result;
use crate::daemon::ensure::daemon_info_if_healthy;
use crate::daemon::supervision::{select, AdapterKind, SupervisionStatus};

pub(super) fn daemon_status() -> Result<()> {
    let daemon_dir = ship_substrate::home::current_daemon_dir();
    let adapter = select();
    let supervision_status = adapter.status(&daemon_dir)?;

    // Try the live REST endpoint first — that's the daemon's own status,
    // richer than what a supervisor adapter can see from outside.
    if let Some((port, token)) = daemon_info_if_healthy() {
        return print_live_status(port, &token, &supervision_status);
    }

    print_supervision_only(&supervision_status);
    Ok(())
}

pub(super) fn print_supervision_only(s: &SupervisionStatus) {
    if !s.running {
        match s.pid {
            Some(pid) => println!(
                "shipd not running (stale pidfile from pid {pid})"
            ),
            None => println!("shipd not running"),
        }
        println!("  supervised by: {}", s.adapter.label());
        return;
    }
    println!("shipd supervisor running — daemon unreachable");
    println!("  supervised by: {}", s.adapter.label());
    if let Some(pid) = s.pid {
        println!("  pid:           {pid}");
    }
    if let Some(v) = &s.version {
        println!("  version:       {v}");
    }
    if let Some(u) = s.uptime_secs {
        println!("  uptime:        {}", format_uptime(u));
    }
    if s.adapter == AdapterKind::Systemd {
        print_journal_tail(s);
    }
}

pub(super) fn print_live_status(port: u16, token: &str, supervision: &SupervisionStatus) -> Result<()> {
    let supervisor_pid = supervision.pid;
    // /api/health carries the full status payload (pid/version/uptime/jobs/
    // sweep) — the former authenticated /daemon/status route folded into it.
    let url = format!("http://127.0.0.1:{port}/api/health");
    let resp = crate::http::daemon_client()
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| anyhow::anyhow!("status request failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let value: serde_json::Value = resp
        .json()
        .map_err(|e| anyhow::anyhow!("status: malformed response body: {e}"))?;

    // `healthy: false` means a supervised background task died — still a
    // running daemon, so print the status plus a degraded marker.
    let degraded = !value["healthy"].as_bool().unwrap_or(true);
    let d = &value;
    let pid = d["pid"].as_u64().unwrap_or(0);
    let version = d["version"].as_str().unwrap_or("?");
    let uptime = format_uptime(d["uptime_secs"].as_u64().unwrap_or(0));
    let total = d["jobs_total"].as_u64().unwrap_or(0);
    let by_status = d["jobs_by_status"].as_object().cloned().unwrap_or_default();
    let by_status_summary: Vec<String> = by_status
        .iter()
        .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n)))
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| format!("{n} {k}"))
        .collect();
    let last_sweep = d["last_sweep_at"]
        .as_str()
        .map(format_relative_time)
        .unwrap_or_else(|| "never".to_string());
    let drift = d["last_sweep_drift"].as_u64().unwrap_or(0);

    if degraded {
        println!("shipd running (DEGRADED — a background task died; see /api/health tasks)");
    } else {
        println!("shipd running");
    }
    println!("  supervised by:  {}", supervision.adapter.label());
    println!("  pid:            {pid}");
    if let Some(s) = supervisor_pid
        && (s as u64) != pid {
            println!("  supervisor pid: {s}");
        }
    println!("  version:        {version}");
    println!("  port:           {port}");
    println!("  uptime:         {uptime}");
    if by_status_summary.is_empty() {
        println!("  jobs:           {total} total");
    } else {
        println!(
            "  jobs:           {total} total ({})",
            by_status_summary.join(", ")
        );
    }
    println!("  last sweep:     {last_sweep}");
    println!("  drift detected: {drift}");
    if supervision.adapter == AdapterKind::Systemd {
        print_journal_tail(supervision);
    }
    Ok(())
}

pub(super) fn print_journal_tail(s: &SupervisionStatus) {
    if let Some(tail) = s.extras.iter().find(|(k, _)| k == "journal_tail") {
        println!("  journal tail:");
        for line in tail.1.lines() {
            println!("    {line}");
        }
    }
}

/// Render seconds as `2h 14m`, `34m 12s`, or `47s` — keeps the longest
/// two units that still fit in 8 chars.
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else if mins > 0 {
        format!("{mins}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Render an RFC3339 timestamp as `8s ago`, `12m ago`, `2h ago`, etc.
pub fn format_relative_time(ts: &str) -> String {
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return ts.to_string();
    };
    let secs = (chrono::Utc::now() - dt.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}
