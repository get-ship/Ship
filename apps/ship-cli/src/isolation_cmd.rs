//! `ship isolation status` / `rescan` — surfaces capability probe +
//! recommender result for human and JSON consumption.
//!
//! Talks to the platform's `isolation::decide()` / `rescan_capabilities()`
//! directly — no daemon round-trip. That keeps this command available
//! even when the daemon is down.

use anyhow::Result;
use ship_isolation::capabilities::{Capabilities, HostOs};
use ship_isolation::recommender::{ResolutionReason, ResolvedIsolation};
use ship_platform::isolation;

use crate::commands::IsolationCommands;

pub fn run(action: IsolationCommands) -> Result<()> {
    match action {
        IsolationCommands::Status { json } => render(false, json),
        IsolationCommands::Rescan { json } => render(true, json),
    }
}

fn render(rescan: bool, as_json: bool) -> Result<()> {
    let caps = if rescan {
        isolation::rescan_capabilities()?
    } else {
        isolation::cached_capabilities()?
    };

    let decision = isolation::decide(None, None);

    if as_json {
        let payload = serde_json::json!({
            "capabilities": &caps,
            "decision": match &decision {
                Ok(d) => serde_json::to_value(d)?,
                Err(e) => serde_json::json!({ "error": e.to_string() }),
            },
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    print_human(&caps, decision.as_ref().ok());
    if let Err(e) = decision {
        println!();
        println!("decision: {e}");
    }
    Ok(())
}

fn print_human(caps: &Capabilities, decision: Option<&isolation::IsolationDecision>) {
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
        ResolutionReason::Downgrade { from, why } => {
            format!("downgraded from {from}: {why}")
        }
        ResolutionReason::AlreadyInSandbox => {
            "already inside someone else's sandbox; not double-wrapping".into()
        }
    }
}
