//! `ship procs [--slug X] [--kind K]` — list process records from the
//! ProcessRecord projection. Read-only view of process lifecycle.

use anyhow::{Context, Result, anyhow};

/// Valid `--kind` values, mirroring `SessionKind` in
/// `crates/core/ship-platform/src/services/sessions/types.rs`.
pub(crate) const VALID_KINDS: &[&str] = &["agent", "pipeline-step", "shell"];

pub fn run(slug: Option<&str>, kind: Option<&str>, limit: Option<usize>, json: bool) -> Result<()> {
    if let Some(k) = kind {
        validate_kind(k)?;
    }
    let (port, token) = crate::daemon::ensure_daemon()?;
    // RuntimeService.ListProcs (Connect) replaces GET /api/v1/procs. The typed
    // RuntimeProcess messages are serde-Serializable, so re-encode to the Value
    // shape the printer already consumes (snake_case fields).
    let procs: Vec<serde_json::Value> = {
        use connectrpc::client::{ClientConfig, HttpClient};
        use ship_ops_rpc::{ListProcsRequest, RuntimeServiceClient};
        let uri: http::Uri =
            format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
        let config =
            ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
        let client = RuntimeServiceClient::new(HttpClient::plaintext(), config);
        let req = ListProcsRequest {
            slug: slug.map(str::to_string),
            kind: kind.map(str::to_string),
            limit: limit.map(|l| l as u32),
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
        let resp = rt
            .block_on(client.list_procs(req))
            .map_err(|e| anyhow!("procs call failed: {e}"))?;
        resp.view()
            .processes
            .iter()
            .map(|p| {
                serde_json::json!({
                    "pid": p.pid,
                    "slug": p.slug,
                    "step_id": p.step_id,
                    "cmd": p.cmd,
                    "status": p.status,
                    "spawned_at": p.spawned_at,
                    "exited_at": p.exited_at,
                })
            })
            .collect()
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&procs)?);
        return Ok(());
    }

    if procs.is_empty() {
        println!("No processes recorded.");
        return Ok(());
    }

    println!(
        "{:<8} {:<22} {:<10} {:<10} {:<8} {:<22} EXITED",
        "PID", "SLUG", "STEP", "CMD", "STATUS", "SPAWNED"
    );
    println!("{}", "-".repeat(110));

    for p in &procs {
        let pid = p["pid"].as_u64().map(|n| n.to_string()).unwrap_or("-".into());
        let s = p["slug"].as_str().unwrap_or("");
        let step = p["step_id"].as_str().unwrap_or("-");
        let cmd = p["cmd"].as_str().unwrap_or("");
        let status = p["status"].as_str().unwrap_or("");
        let spawned = fmt_ts(p["spawned_at"].as_str().unwrap_or(""));
        let exited = p["exited_at"]
            .as_str()
            .map(fmt_ts)
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<8} {:<22} {:<10} {:<10} {:<8} {:<22} {}",
            truncate(&pid, 8),
            truncate(s, 22),
            truncate(step, 10),
            truncate(cmd, 10),
            truncate(status, 8),
            spawned,
            exited,
        );
    }
    Ok(())
}

pub(crate) fn validate_kind(kind: &str) -> Result<()> {
    if VALID_KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(anyhow!(
            "invalid --kind value '{kind}'. Valid values: {}",
            VALID_KINDS.join(", ")
        ))
    }
}

fn fmt_ts(s: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|_| s.to_string())
}

fn truncate(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        chars[..n.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_ts_round_trips_iso() {
        let s = "2026-04-15T21:39:50Z";
        assert_eq!(fmt_ts(s), "2026-04-15 21:39:50");
    }

    #[test]
    fn fmt_ts_returns_input_for_invalid() {
        assert_eq!(fmt_ts("garbage"), "garbage");
    }

    #[test]
    fn validate_kind_accepts_session_kinds() {
        for k in VALID_KINDS {
            assert!(validate_kind(k).is_ok(), "{k} should be valid");
        }
    }

    #[test]
    fn validate_kind_rejects_unknown_value() {
        let err = validate_kind("widget").unwrap_err().to_string();
        assert!(err.contains("widget"), "error should name the bad value: {err}");
        for k in VALID_KINDS {
            assert!(err.contains(k), "error should list valid value {k}: {err}");
        }
    }
}
