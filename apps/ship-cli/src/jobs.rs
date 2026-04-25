//! `ship jobs` — list current jobs with projected status, current step,
//! age, live process state, and notes. The daemon does the projection +
//! drift correlation; the CLI just renders.

use anyhow::{Result, anyhow};
use chrono::Utc;

/// GET /api/v1/jobs and render either as a table or as JSON.
pub fn run(json: bool) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let url = format!("http://127.0.0.1:{port}/api/v1/jobs");
    let mut resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| anyhow!("jobs request failed: {e}"))?;

    let value: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| anyhow!("jobs: malformed response body: {e}"))?;

    if !value["ok"].as_bool().unwrap_or(false) {
        return Err(anyhow!(
            "daemon returned error: {}",
            value["data"].as_str().unwrap_or("unknown")
        ));
    }

    let mut jobs: Vec<serde_json::Value> = value["data"]["jobs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    jobs.sort_by(|a, b| {
        b["updated_at"]
            .as_str()
            .unwrap_or("")
            .cmp(a["updated_at"].as_str().unwrap_or(""))
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&jobs)?);
        return Ok(());
    }

    if jobs.is_empty() {
        println!("No jobs.");
        return Ok(());
    }

    println!(
        "{:<22} {:<12} {:<10} {:<6} {:<8} NOTES",
        "SLUG", "STATUS", "STEP", "AGE", "PROCESS"
    );
    println!("{}", "-".repeat(80));

    for j in &jobs {
        let slug = j["slug"].as_str().unwrap_or("");
        let status = j["status"].as_str().unwrap_or("");
        let step = j["current_step"].as_str().unwrap_or("-");
        let age = format_age(j["updated_at"].as_str().unwrap_or(""));
        let proc_state = render_process(j["process_state"].as_str().unwrap_or("none"));
        let notes = j["notes"].as_str().unwrap_or("-");
        println!(
            "{:<22} {:<12} {:<10} {:<6} {:<8} {}",
            truncate(slug, 22),
            truncate(status, 12),
            truncate(step, 10),
            age,
            proc_state,
            notes,
        );
    }
    Ok(())
}

fn render_process(s: &str) -> String {
    match s {
        "alive" => "alive".to_string(),
        "missing" => "MISSING".to_string(),
        "terminal" => "-".to_string(),
        _ => "-".to_string(),
    }
}

fn truncate(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        s.to_string()
    } else {
        chars[..n.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}

/// Render age relative to `now` as the largest unit that fits in 5 chars.
/// Examples: 4s, 12m, 2h, 1d.
fn format_age(updated_at: &str) -> String {
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) else {
        return "?".into();
    };
    let secs = (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_process_states() {
        assert_eq!(render_process("alive"), "alive");
        assert_eq!(render_process("missing"), "MISSING");
        assert_eq!(render_process("terminal"), "-");
        assert_eq!(render_process("none"), "-");
        assert_eq!(render_process("bogus"), "-");
    }

    #[test]
    fn truncate_with_ellipsis() {
        assert_eq!(truncate("short", 10), "short");
        let long = truncate("abcdefghij", 5);
        assert_eq!(long.chars().count(), 5);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn format_age_buckets() {
        let now = Utc::now();
        let mk = |secs: i64| (now - chrono::Duration::seconds(secs)).to_rfc3339();
        assert_eq!(format_age(&mk(5)), "5s");
        let m = format_age(&mk(120));
        assert!(m.ends_with('m'));
        let h = format_age(&mk(3600 * 2));
        assert!(h.ends_with('h'));
        let d = format_age(&mk(86400));
        assert!(d.ends_with('d'));
    }

    #[test]
    fn format_age_handles_invalid_input() {
        assert_eq!(format_age("not-a-date"), "?");
    }
}
