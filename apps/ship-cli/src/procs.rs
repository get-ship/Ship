//! `ship procs [--slug X]` — list process records from the
//! ProcessRecord projection. Read-only view of process lifecycle.

use anyhow::{Result, anyhow};

pub fn run(slug: Option<&str>, limit: Option<usize>, json: bool) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let mut url = format!("http://127.0.0.1:{port}/api/v1/procs");
    let mut q: Vec<String> = vec![];
    if let Some(s) = slug {
        q.push(format!("slug={}", urlencode(s)));
    }
    if let Some(l) = limit {
        q.push(format!("limit={l}"));
    }
    if !q.is_empty() {
        url.push('?');
        url.push_str(&q.join("&"));
    }

    let mut resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| anyhow!("procs request failed: {e}"))?;
    let value: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| anyhow!("procs: malformed response body: {e}"))?;

    if !value["ok"].as_bool().unwrap_or(false) {
        return Err(anyhow!(
            "daemon returned error: {}",
            value["data"].as_str().unwrap_or("unknown")
        ));
    }

    let procs: Vec<serde_json::Value> = value["data"]["procs"]
        .as_array()
        .cloned()
        .unwrap_or_default();

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

/// Minimal URL component encoder — only escapes characters that would break
/// query parsing. We deliberately avoid pulling in a crate for one call.
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_passes_unreserved() {
        assert_eq!(urlencode("simple-slug_01"), "simple-slug_01");
    }

    #[test]
    fn urlencode_escapes_reserved() {
        assert_eq!(urlencode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn fmt_ts_round_trips_iso() {
        let s = "2026-04-15T21:39:50Z";
        assert_eq!(fmt_ts(s), "2026-04-15 21:39:50");
    }

    #[test]
    fn fmt_ts_returns_input_for_invalid() {
        assert_eq!(fmt_ts("garbage"), "garbage");
    }
}
