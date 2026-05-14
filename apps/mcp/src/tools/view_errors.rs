//! MCP tool: retrieve client-side view errors from the daemon's ring buffer.
//!
//! The daemon captures browser-side errors posted by the shell (JS errors,
//! unhandled rejections, Datastar fetch failures) into an in-memory ring
//! buffer. This tool lets an iterating agent read that buffer instead of
//! asking the user to copy their browser console.

use anyhow::{Result, anyhow};

use crate::requests::ViewErrorsRequest;

/// GET + optional DELETE against `/api/v1/view-errors`.
///
/// Returns a readable text summary — the MCP contract is stringly typed.
pub async fn view_errors(req: ViewErrorsRequest) -> String {
    match fetch_and_format(req).await {
        Ok(out) => out,
        Err(e) => format!("Error: {e}"),
    }
}

async fn fetch_and_format(req: ViewErrorsRequest) -> Result<String> {
    let errors_json = http_get_errors(req.since, req.view.as_deref(), req.limit).await?;
    let parsed: serde_json::Value = serde_json::from_str(&errors_json)
        .map_err(|e| anyhow!("invalid response from daemon: {e}"))?;
    let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    let errs = parsed
        .get("errors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = String::new();
    if errs.is_empty() {
        out.push_str("No client-side view errors captured.\n");
        out.push_str(&format!("(buffer total: {total})\n"));
    } else {
        out.push_str(&format!(
            "{} error(s) returned (buffer total: {total})\n\n",
            errs.len(),
        ));
        for e in &errs {
            let id = e.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let kind = e.get("type").and_then(|v| v.as_str()).unwrap_or("?");
            let view = e.get("view").and_then(|v| v.as_str()).unwrap_or("-");
            let url = e.get("url").and_then(|v| v.as_str()).unwrap_or("-");
            let ts = e
                .get("timestamp_client")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let msg = e.get("message").and_then(|v| v.as_str()).unwrap_or("");
            out.push_str(&format!(
                "#{id} [{kind}] view={view} url={url} at {ts}\n  {msg}\n"
            ));
            if let Some(file) = e.get("filename").and_then(|v| v.as_str()) {
                let line = e.get("line").and_then(|v| v.as_u64()).unwrap_or(0);
                let col = e.get("column").and_then(|v| v.as_u64()).unwrap_or(0);
                out.push_str(&format!("  at {file}:{line}:{col}\n"));
            }
            if let Some(stack) = e.get("stack").and_then(|v| v.as_str())
                && !stack.is_empty()
            {
                // First few stack frames only — full stack stays in the
                // JSON response for callers that want the raw record.
                for frame in stack.lines().take(5) {
                    out.push_str(&format!("  {frame}\n"));
                }
            }
            if let Some(detail) = e.get("detail")
                && !detail.is_null()
            {
                out.push_str(&format!("  detail: {detail}\n"));
            }
            out.push('\n');
        }
    }

    if req.clear_after.unwrap_or(false) {
        match http_delete_errors().await {
            Ok(cleared) => out.push_str(&format!("(cleared {cleared} from buffer)\n")),
            Err(e) => out.push_str(&format!("(clear failed: {e})\n")),
        }
    }

    Ok(out)
}

async fn http_get_errors(
    since: Option<u64>,
    view: Option<&str>,
    limit: Option<usize>,
) -> Result<String> {
    let global_dir = ship_substrate::project::get_global_dir()?;
    let port = read_port(&global_dir)?;
    let token = read_token(&global_dir)?;
    let mut url = format!("http://127.0.0.1:{port}/api/v1/view-errors");
    let mut sep = '?';
    if let Some(s) = since {
        url.push_str(&format!("{sep}since={s}"));
        sep = '&';
    }
    if let Some(v) = view {
        url.push_str(&format!("{sep}view={v}"));
        sep = '&';
    }
    if let Some(l) = limit {
        url.push_str(&format!("{sep}limit={l}"));
        let _ = sep;
    }
    let resp = reqwest::Client::new()
        .get(&url)
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

async fn http_delete_errors() -> Result<u64> {
    let global_dir = ship_substrate::project::get_global_dir()?;
    let port = read_port(&global_dir)?;
    let token = read_token(&global_dir)?;
    let url = format!("http://127.0.0.1:{port}/api/v1/view-errors");
    let resp = reqwest::Client::new()
        .delete(&url)
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
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("invalid clear response: {e}"))?;
    Ok(parsed.get("cleared").and_then(|v| v.as_u64()).unwrap_or(0))
}

fn read_port(_global_dir: &std::path::Path) -> Result<u16> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("cwd unavailable: {e}"))?;
    ship_substrate::home::discover_running_daemon(&cwd)
        .map(|h| h.port)
        .ok_or_else(|| anyhow!("ship daemon not running"))
}

fn read_token(_global_dir: &std::path::Path) -> Result<String> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("cwd unavailable: {e}"))?;
    ship_substrate::home::discover_running_daemon(&cwd)
        .map(|h| h.token)
        .ok_or_else(|| anyhow!("daemon token missing"))
}
