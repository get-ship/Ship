//! `ship tail <slug>` — stream events for a single slug.
//!
//! Implementation: 1-second polling via `EventsService.Poll` (Connect).
//! SSE would be cleaner, but the kernel mailbox stream only delivers future
//! events — it has no replay cursor. Polling lets the caller resume cleanly
//! across reconnects with a ULID `since_id`.

use anyhow::{Context, Result, anyhow};
use std::time::Duration;

pub fn run(slug: &str, namespace: Option<&str>) -> Result<()> {
    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::{EventsPollReq, EventsServiceClient};
    let (port, token) = crate::daemon::ensure_daemon()?;
    let uri: http::Uri =
        format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = EventsServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;
    let mut since: Option<String> = None;

    println!("Tailing events for slug='{slug}' (Ctrl+C to stop)");
    loop {
        let req = EventsPollReq {
            slug: slug.to_string(),
            since_id: since.clone().unwrap_or_default(),
            namespace: namespace.unwrap_or_default().to_string(),
            ..Default::default()
        };
        let resp = rt
            .block_on(client.poll(req))
            .map_err(|e| anyhow!("tail poll failed: {e}"))?;
        let value: serde_json::Value = serde_json::from_str(&resp.view().json)
            .map_err(|e| anyhow!("tail: malformed response body: {e}"))?;

        let events: Vec<serde_json::Value> =
            value["events"].as_array().cloned().unwrap_or_default();
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
}
