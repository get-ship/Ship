//! `ship isolation …` — operator CLI for the capability probe, recommender,
//! and isolation-settings init.
//!
//! Runs the reads in-process via `ship_platform::isolation` /
//! `isolation_ops` (they share `daemon_settings`, so the outcome matches the
//! daemon's). The isolation reads are no longer mirrored as REST routes — the
//! daemon serves them on the proto IsolationService (`ship ops call
//! isolation.*`), so the CLI uses the logic layer directly rather than a
//! parallel REST path. `ship isolation sandboxes` still calls the daemon
//! (`/api/v1/sandboxes`) since live sandbox state only exists there.

use anyhow::Result;
use ship_isolation::capabilities::{Capabilities, HostOs};
use ship_isolation::recommender::{ResolutionReason, ResolvedIsolation};
use ship_platform::{isolation, isolation_ops};

use crate::commands::IsolationCommands;

pub fn run(action: IsolationCommands) -> Result<()> {
    match action {
        IsolationCommands::Status { json } => render_status(false, json),
        IsolationCommands::Rescan { json } => render_status(true, json),
        IsolationCommands::Verify { json } => render_verify(json),
        IsolationCommands::Recommendations { json } => render_recommendations(json),
        IsolationCommands::Preview { tier, strategy, json } => {
            render_preview(tier.as_deref(), strategy.as_deref(), json)
        }
        IsolationCommands::Init { tier, strategy, enforce, json } => {
            render_init(tier.as_deref(), strategy.as_deref(), enforce, json)
        }
        IsolationCommands::Sandboxes { json } => render_sandboxes(json),
    }
}

// ── sandboxes ────────────────────────────────────────────────────────────────

fn render_sandboxes(as_json: bool) -> Result<()> {
    let Some((port, token)) = crate::daemon::daemon_info_if_healthy() else {
        anyhow::bail!(
            "ship sandbox list requires the daemon to be running — \
             start it with: ship daemon start"
        );
    };
    let payload = call_daemon(port, &token, "/api/v1/sandboxes")?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    print_sandboxes_human(&payload);
    Ok(())
}

fn print_sandboxes_human(payload: &serde_json::Value) {
    let empty = Vec::new();
    let sandboxes = payload["data"]["sandboxes"].as_array().unwrap_or(&empty);
    if sandboxes.is_empty() {
        println!("No sandboxes registered. Sandboxes are created lazily on first actor spawn.");
        return;
    }
    println!("{:<32} {:<16} {:<10} {:<12} {:<6} ACTORS",
        "ID", "PROJECT", "NAME", "STATE", "OWNER");
    for s in sandboxes {
        let id = s["id"].as_str().unwrap_or("?");
        let project = s["project"].as_str().unwrap_or("?");
        let name = s["name"].as_str().unwrap_or("?");
        let state = s["state"].as_str().unwrap_or("?");
        let owner = if s["has_owner"].as_bool().unwrap_or(false) { "yes" } else { "no" };
        let n_actors = s["actors"].as_array().map(|a| a.len()).unwrap_or(0);
        // Truncate project to fit; users with long paths can pass --json
        let project_disp = if project.len() > 14 {
            format!("…{}", &project[project.len() - 13..])
        } else {
            project.to_string()
        };
        println!("{:<32} {:<16} {:<10} {:<12} {:<6} {}",
            id, project_disp, name, state, owner, n_actors);
    }
}

// ── status / rescan ──────────────────────────────────────────────────────────

fn render_status(rescan: bool, as_json: bool) -> Result<()> {
    let payload = in_process_status(rescan)?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    print_status_human(&payload);
    Ok(())
}

fn in_process_status(rescan: bool) -> Result<serde_json::Value> {
    let caps = if rescan {
        isolation::rescan_capabilities()?
    } else {
        isolation::cached_capabilities()?
    };
    let decision = isolation::decide(None, None);
    Ok(serde_json::json!({
        "capabilities": &caps,
        "decision": match &decision {
            Ok(d) => serde_json::to_value(d)?,
            Err(e) => serde_json::json!({ "error": e.to_string() }),
        },
    }))
}

// ── verify / recommendations / preview / init ────────────────────────────────

fn render_verify(as_json: bool) -> Result<()> {
    let v = serde_json::to_value(&isolation_ops::verify_decision())?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(());
    }
    print_verify_human(&v);
    Ok(())
}

fn render_recommendations(as_json: bool) -> Result<()> {
    let v = serde_json::to_value(&isolation_ops::recommendations()?)?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(());
    }
    print_recommendations_human(&v);
    Ok(())
}

fn render_preview(tier: Option<&str>, strategy: Option<&str>, _as_json: bool) -> Result<()> {
    let v = preview_in_process(tier, strategy)?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

fn preview_in_process(tier: Option<&str>, strategy: Option<&str>) -> Result<serde_json::Value> {
    let profile = ship_isolation::tier::IsolationProfile {
        tier: tier.and_then(ship_isolation::tier::IsolationTier::from_id),
        strategy: strategy.and_then(ship_isolation::tier::IsolationStrategy::from_id),
        policy: None,
    };
    let decision = isolation_ops::preview(profile)?;
    Ok(serde_json::to_value(&decision)?)
}

fn render_init(
    tier: Option<&str>,
    strategy: Option<&str>,
    enforce: bool,
    as_json: bool,
) -> Result<()> {
    let v = init_in_process(tier, strategy, enforce)?;
    if as_json {
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(());
    }
    print_init_human(&v);
    Ok(())
}

fn init_in_process(
    tier: Option<&str>,
    strategy: Option<&str>,
    enforce: bool,
) -> Result<serde_json::Value> {
    let req = isolation_ops::InitRequest {
        tier: tier.map(str::to_string),
        strategy: strategy.map(str::to_string),
        enforce: Some(enforce),
    };
    let result = isolation_ops::init(req)?;
    Ok(serde_json::to_value(&result)?)
}

// ── daemon HTTP client ───────────────────────────────────────────────────────

fn call_daemon(port: u16, token: &str, path: &str) -> Result<serde_json::Value> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let resp = crate::http::daemon_client()
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .timeout(std::time::Duration::from_secs(5))
        .send()?;
    if !resp.status().is_success() {
        anyhow::bail!("daemon returned {}", resp.status());
    }
    let envelope: serde_json::Value = resp.json()?;
    // Shipd wraps responses in MeshResponse { ok, data }. Unwrap data.
    Ok(envelope
        .get("data")
        .cloned()
        .unwrap_or(serde_json::Value::Null))
}

// ── rendering ────────────────────────────────────────────────────────────────

fn print_status_human(payload: &serde_json::Value) {
    let caps: Option<Capabilities> = payload
        .get("capabilities")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let decision: Option<isolation::IsolationDecision> = payload
        .get("decision")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let Some(caps) = caps else {
        println!("{}", serde_json::to_string_pretty(payload).unwrap_or_default());
        return;
    };
    println!("Host OS: {}", os_label(caps.os));
    println!("Inside a sandbox already: {}", yes_no(caps.in_sandbox));
    println!();
    println!("Available primitives:");
    primitive(caps.docker, "docker");
    primitive(caps.podman, "podman");
    primitive(caps.youki.is_some(), "youki binary");
    primitive(caps.youki_rootfs.is_some(), "youki rootfs");
    primitive(caps.lima.is_some(), "lima");
    primitive(caps.tart.is_some(), "tart");
    primitive(caps.wsl2, "WSL2");
    primitive(caps.srt, "sandbox-runtime (srt)");
    primitive(caps.sandbox_exec, "sandbox-exec (macOS)");
    primitive(caps.vz_framework, "Vz framework (macOS arm64)");
    primitive(caps.fly_cli, "fly CLI");
    primitive(caps.fly_authed, "fly authenticated");

    println!();
    println!("Available strategies:");
    for s in caps.available_strategies() {
        println!("  - {s}");
    }

    if let Some(d) = decision {
        println!();
        println!("Recommendation for next agent run:");
        println!("  strategy:   {}", d.resolved.strategy);
        println!("  tier:       {} ({})", d.resolved.tier, d.resolved.tier.label());
        println!("  reason:     {}", reason_label(&d.resolved));
        if d.resolved.downgraded {
            println!("  warning:    DOWNGRADED — preferred isolation unavailable");
        }
    }
}

fn print_verify_human(payload: &serde_json::Value) {
    let ok = payload.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    println!("{} overall", if ok { "✓" } else { "✗" });
    if let Some(steps) = payload.get("steps").and_then(|s| s.as_array()) {
        for step in steps {
            let name = step.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let ok = step.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            let detail = step.get("detail").and_then(|v| v.as_str()).unwrap_or("");
            println!("  {} {name}: {detail}", if ok { "✓" } else { "✗" });
        }
    }
}

fn print_recommendations_human(payload: &serde_json::Value) {
    let host_os = payload
        .get("host_os")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let default_tier = payload
        .get("os_default_tier")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    println!("Host OS: {host_os}");
    println!("Per-OS default tier: {default_tier}");
    println!();
    if let Some(tiers) = payload.get("tiers").and_then(|t| t.as_array()) {
        for tier in tiers {
            let id = tier.get("tier").and_then(|v| v.as_str()).unwrap_or("?");
            let label = tier.get("label").and_then(|v| v.as_str()).unwrap_or("?");
            let recommended = tier.get("recommended").and_then(|v| v.as_bool()).unwrap_or(false);
            let marker = if recommended { " (recommended)" } else { "" };
            println!("{id}: {label}{marker}");
            if let Some(strategies) = tier.get("strategies").and_then(|s| s.as_array()) {
                for strat in strategies {
                    let s_id = strat.get("strategy").and_then(|v| v.as_str()).unwrap_or("?");
                    let avail = strat.get("available").and_then(|v| v.as_bool()).unwrap_or(false);
                    let dot = if avail { "✓" } else { "✗" };
                    let reason = strat
                        .get("unavailable_reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if reason.is_empty() {
                        println!("  {dot} {s_id}");
                    } else {
                        println!("  {dot} {s_id} — {reason}");
                    }
                }
            }
            println!();
        }
    }
}

fn print_init_human(payload: &serde_json::Value) {
    let ok = payload.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    println!("{} {}", if ok { "✓" } else { "✗" }, if ok { "applied" } else { "refused" });
    if let Some(applied) = payload.get("applied").and_then(|a| a.as_array()) {
        for entry in applied {
            if let Some(s) = entry.as_str() {
                println!("  + {s}");
            }
        }
    }
    if let Some(verify) = payload.get("verify") {
        println!();
        println!("verify:");
        print_verify_human(verify);
    }
}

fn primitive(present: bool, name: &str) {
    let mark = if present { "yes" } else { "no" };
    println!("  - {name}: {mark}");
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

fn os_label(os: HostOs) -> &'static str {
    match os {
        HostOs::Linux => "Linux",
        HostOs::MacosArm => "macOS (Apple Silicon)",
        HostOs::MacosIntel => "macOS (Intel)",
        HostOs::Windows => "Windows",
        HostOs::Wsl2 => "WSL2",
        HostOs::Unknown => "unknown",
    }
}

fn reason_label(r: &ResolvedIsolation) -> String {
    match &r.reason {
        ResolutionReason::RunOverride => "explicit per-run override".into(),
        ResolutionReason::AgentOverride => "set in the agent profile".into(),
        ResolutionReason::UserDefault => "your daemon default".into(),
        ResolutionReason::OsDefault { os } => format!("OS default for {}", os_label(*os)),
        ResolutionReason::Downgrade { from, why } => format!("downgraded from {from}: {why}"),
        ResolutionReason::AlreadyInSandbox => {
            "already inside someone else's sandbox; not double-wrapping".into()
        }
    }
}
