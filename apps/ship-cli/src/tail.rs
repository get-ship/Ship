//! `ship tail <slug>` — stream events for a single slug.
//!
//! Implementation: 1-second polling against `/api/v1/events/poll`.
//! SSE would be cleaner, but the existing /api/v1/events/stream SSE only
//! delivers future events from the kernel mailbox — it has no replay
//! cursor. Polling lets the caller resume cleanly across reconnects with
//! a ULID `since_id`.

use anyhow::{Result, anyhow};
use std::time::Duration;

pub fn run(slug: &str, namespace: Option<&str>) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let mut since: Option<String> = None;
    let bearer = format!("Bearer {token}");

    println!("Tailing events for slug='{slug}' (Ctrl+C to stop)");
    loop {
        let mut url = format!(
            "http://127.0.0.1:{port}/api/v1/events/poll?slug={}",
            urlencode(slug),
        );
        if let Some(ref c) = since {
            url.push_str(&format!("&since_id={}", urlencode(c)));
        }
        if let Some(ns) = namespace {
            url.push_str(&format!("&namespace={ns}"));
        }

        let mut resp = ureq::get(&url)
            .header("Authorization", &bearer)
            .config()
            .http_status_as_error(false)
            .build()
            .call()
            .map_err(|e| anyhow!("tail poll failed: {e}"))?;
        let value: serde_json::Value = resp
            .body_mut()
            .read_json()
            .map_err(|e| anyhow!("tail: malformed response body: {e}"))?;

        if !value["ok"].as_bool().unwrap_or(false) {
            return Err(anyhow!(
                "daemon returned error: {}",
                value["data"].as_str().unwrap_or("unknown")
            ));
        }

        let events: Vec<serde_json::Value> = value["data"]["events"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for e in &events {
            print_event(e);
            if let Some(id) = e["id"].as_str() {
                since = Some(id.to_string());
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn print_event(e: &serde_json::Value) {
    let ts = e["created_at"].as_str().unwrap_or("");
    let formatted_ts = chrono::DateTime::parse_from_rfc3339(ts)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|_| ts.to_string());
    let evt_type = e["event_type"].as_str().unwrap_or("");
    let entity = e["entity_id"].as_str().unwrap_or("");

    let payload_summary = e["payload_json"]
        .as_str()
        .and_then(|j| serde_json::from_str::<serde_json::Value>(j).ok())
        .map(summarize_payload)
        .unwrap_or_default();
    println!("{formatted_ts}  {evt_type:<28} entity={entity}  {payload_summary}");
}

fn summarize_payload(v: serde_json::Value) -> String {
    // A few common keys we want highlighted; otherwise the truncated
    // payload itself.
    let keys = [
        "slug", "step_id", "step_name", "pid", "verdict", "trigger", "reason", "error",
    ];
    let parts: Vec<String> = keys
        .iter()
        .filter_map(|k| v.get(*k).and_then(|val| {
            let s = match val {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if s.is_empty() || s == "null" {
                None
            } else {
                Some(format!("{k}={s}"))
            }
        }))
        .collect();
    if parts.is_empty() {
        let mut s = v.to_string();
        if s.len() > 80 {
            let boundary = s.char_indices().take_while(|(i, _)| *i < 79).last().map(|(i, c)| i + c.len_utf8()).unwrap_or(0);
            s.truncate(boundary);
            s.push('…');
        }
        s
    } else {
        parts.join(" ")
    }
}

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
    fn summarize_picks_known_keys() {
        let v = serde_json::json!({"slug": "x-1", "pid": 42, "noise": "ignored"});
        let s = summarize_payload(v);
        assert!(s.contains("slug=x-1"));
        assert!(s.contains("pid=42"));
        assert!(!s.contains("noise"));
    }

    #[test]
    fn summarize_falls_back_to_payload_when_no_known_keys() {
        let v = serde_json::json!({"weird": "data"});
        let s = summarize_payload(v);
        assert!(s.contains("weird"));
    }

    #[test]
    fn urlencode_basic() {
        assert_eq!(urlencode("abc"), "abc");
        assert_eq!(urlencode("a/b"), "a%2Fb");
    }
}
