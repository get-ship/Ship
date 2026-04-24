//! MCP tool: fuzzy-search Ship's SDK surface via the daemon's
//! `GET /api/sdk/query` endpoint.
//!
//! The daemon owns the canonical SDK manifest (macros, REST routes, view
//! manifests, collection and trigger schemas) — this tool just formats the
//! hits for an agent author looking for the right primitive.

use anyhow::{Result, anyhow};

use crate::requests::SdkQueryRequest;

/// Dispatch a query against the daemon and format the hits.
pub async fn sdk_query(req: SdkQueryRequest) -> String {
    match fetch_and_format(req).await {
        Ok(out) => out,
        Err(e) => format!("Error: {e}"),
    }
}

async fn fetch_and_format(req: SdkQueryRequest) -> Result<String> {
    let query = req.query.trim().to_string();
    if query.is_empty() {
        return Err(anyhow!("query must not be empty"));
    }
    let limit = req.limit.unwrap_or(20).max(1);
    let kind_filter = req.kind.as_deref().map(|s| s.to_ascii_lowercase());

    let body = http_get_query(&query).await?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| anyhow!("invalid response from daemon: {e}"))?;

    let hits = parsed
        .get("hits")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut filtered: Vec<&serde_json::Value> = hits
        .iter()
        .filter(|h| match &kind_filter {
            Some(k) => h.get("kind").and_then(|v| v.as_str()) == Some(k.as_str()),
            None => true,
        })
        .collect();
    filtered.truncate(limit);

    if filtered.is_empty() {
        return Ok(format!(
            "no matches for \"{query}\"; try a broader term"
        ));
    }

    let total = hits.len();
    let mut out = String::new();
    out.push_str(&format!(
        "{} hit(s) shown (of {} matched) for \"{query}\"\n",
        filtered.len(),
        total,
    ));
    for h in filtered {
        let kind = h.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
        let id = h.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let summary_raw = h.get("summary").and_then(|v| v.as_str()).unwrap_or("");
        let summary = truncate(summary_raw, 80);
        out.push_str(&format!("[{kind}] {id} — {summary}\n"));
    }
    Ok(out)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

async fn http_get_query(query: &str) -> Result<String> {
    let global_dir = runtime::project::get_global_dir()?;
    let port = read_port(&global_dir)?;
    let token = read_token(&global_dir)?;
    // Percent-encode the query — reqwest's query() handles it for us.
    let url = format!("http://127.0.0.1:{port}/api/sdk/query");
    let resp = reqwest::Client::new()
        .get(&url)
        .query(&[("q", query)])
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("daemon error ({status}): {text}"));
    }
    Ok(text)
}

fn read_port(global_dir: &std::path::Path) -> Result<u16> {
    let raw = std::fs::read_to_string(global_dir.join("daemon.port"))
        .map_err(|_| anyhow!("ship daemon not running"))?;
    raw.trim()
        .parse::<u16>()
        .map_err(|e| anyhow!("invalid daemon.port: {e}"))
}

fn read_token(global_dir: &std::path::Path) -> Result<String> {
    let raw = std::fs::read_to_string(global_dir.join("daemon.token"))
        .map_err(|_| anyhow!("daemon.token missing"))?;
    Ok(raw.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_summary() {
        let s = "a".repeat(100);
        let t = truncate(&s, 80);
        assert_eq!(t.chars().count(), 80);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn short_summary_unchanged() {
        assert_eq!(truncate("short", 80), "short");
    }
}
