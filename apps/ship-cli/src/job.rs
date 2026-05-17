//! `ship job ...` subcommand handlers.

use anyhow::{Result, anyhow};

/// POST /api/v1/agent_sessions/{slug}/cancel.
///
/// Prints a user-facing summary on success; treats 409 as a clean exit
/// (already terminal, idempotent) and surfaces other errors verbatim.
pub fn cancel(slug: &str, reason: &str) -> Result<()> {
    let (port, token) = crate::daemon::ensure_daemon()?;
    let url = format!("http://127.0.0.1:{port}/api/v1/agent_sessions/{slug}/cancel");
    let body = serde_json::json!({ "reason": reason });

    let mut resp = ureq::post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        // Non-2xx should not become transport errors — we want to
        // inspect 409 and print the current status.
        .config()
        .http_status_as_error(false)
        .build()
        .send_json(&body)
        .map_err(|e| anyhow!("cancel request failed: {e}"))?;

    let status = resp.status().as_u16();
    let value: serde_json::Value = resp.body_mut().read_json()
        .map_err(|e| anyhow!("cancel: malformed response body: {e}"))?;

    match status {
        200 => {
            println!("Cancelled {slug}");
            Ok(())
        }
        409 => {
            let current = value["data"]["status"].as_str().unwrap_or("terminal");
            println!("Job {slug} is already {current}; nothing to cancel.");
            Ok(())
        }
        404 => {
            Err(anyhow!("No job with slug '{slug}'"))
        }
        _ => {
            let msg = value["data"].as_str().map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            Err(anyhow!("cancel failed ({status}): {msg}"))
        }
    }
}
