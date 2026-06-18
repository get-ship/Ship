//! Client-side error capture ring buffer for Ship views.
//!
//! Relocated from `apps/shipd/src/routes/view_errors_api.rs` (Batch C routes-cull).
//!
//! Stores reports in a bounded in-memory ring buffer so agents iterating on
//! views can retrieve browser-side Datastar/JS errors via an MCP tool instead
//! of asking the user to copy the console.
//!
//! Not persisted — buffer resets on daemon restart. No per-user scoping;
//! auth is the daemon bearer token.
//!
//! # Capability gap
//! There is no Connect wire for these endpoints — REST-only surface for now.
//! The ring-buffer logic lives here so a future `ViewErrorsService` proto can
//! call `record`/`query`/`clear` without touching shipd. Tracked in the
//! routes-cull plan (§view_errors disposition).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const RING_CAP: usize = 200;

fn buffer() -> &'static Mutex<VecDeque<StoredError>> {
    static BUF: OnceLock<Mutex<VecDeque<StoredError>>> = OnceLock::new();
    BUF.get_or_init(|| Mutex::new(VecDeque::with_capacity(RING_CAP)))
}

fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(1);
    SEQ.fetch_add(1, Ordering::Relaxed)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Transport-agnostic incoming error report. The REST shell and any future
/// Connect wire both map onto this before calling `record`.
pub struct ErrorReport {
    pub view: Option<String>,
    pub url: Option<String>,
    pub kind: Option<String>,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
}

/// Append a report to the ring, evicting the oldest past `RING_CAP`. Returns the
/// assigned id.
pub fn record(r: ErrorReport) -> u64 {
    let id = next_id();
    let stored = StoredError {
        id,
        view: r.view,
        url: r.url,
        kind: r.kind.unwrap_or_else(|| "unknown".to_string()),
        message: r.message,
        stack: r.stack,
        filename: r.filename,
        line: r.line,
        column: r.column,
        detail: r.detail,
        user_agent: r.user_agent,
        timestamp_client: r.timestamp_client,
        timestamp_server_ms: now_ms(),
    };
    let mut buf = buffer().lock().expect("view-errors buffer poisoned");
    if buf.len() >= RING_CAP {
        buf.pop_front();
    }
    buf.push_back(stored);
    id
}

/// Newest-first query with `since` (id >) + `view` filters, capped at `limit`.
/// Returns `(matched, total_in_buffer)`.
pub fn query(
    since: Option<u64>,
    view: Option<&str>,
    limit: usize,
) -> (Vec<StoredError>, usize) {
    let buf = buffer().lock().expect("view-errors buffer poisoned");
    let total = buf.len();
    let matched = buf
        .iter()
        .filter(|e| since.map(|s| e.id > s).unwrap_or(true))
        .filter(|e| match (view, &e.view) {
            (Some(want), Some(have)) => want == have,
            (Some(_), None) => false,
            (None, _) => true,
        })
        .rev()
        .take(limit)
        .cloned()
        .collect();
    (matched, total)
}

/// Drain the ring; returns how many were cleared.
pub fn clear() -> usize {
    let mut buf = buffer().lock().expect("view-errors buffer poisoned");
    let n = buf.len();
    buf.clear();
    n
}

// ── REST transport types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PostErrorReq {
    pub view: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
}

impl From<PostErrorReq> for ErrorReport {
    fn from(r: PostErrorReq) -> Self {
        Self {
            view: r.view,
            url: r.url,
            kind: r.kind,
            message: r.message,
            stack: r.stack,
            filename: r.filename,
            line: r.line,
            column: r.column,
            detail: r.detail,
            user_agent: r.user_agent,
            timestamp_client: r.timestamp_client,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredError {
    pub id: u64,
    pub view: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
    pub stack: Option<String>,
    pub filename: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub detail: Option<serde_json::Value>,
    pub user_agent: Option<String>,
    pub timestamp_client: Option<String>,
    pub timestamp_server_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub since: Option<u64>,
    pub view: Option<String>,
    pub limit: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        buffer().lock().unwrap().clear();
    }

    #[test]
    fn ring_buffer_caps_at_200() {
        reset();
        for i in 0..250u32 {
            let mut buf = buffer().lock().unwrap();
            if buf.len() >= RING_CAP {
                buf.pop_front();
            }
            buf.push_back(StoredError {
                id: i as u64,
                view: None,
                url: None,
                kind: "t".into(),
                message: format!("m{i}"),
                stack: None,
                filename: None,
                line: None,
                column: None,
                detail: None,
                user_agent: None,
                timestamp_client: None,
                timestamp_server_ms: 0,
            });
        }
        let buf = buffer().lock().unwrap();
        assert_eq!(buf.len(), RING_CAP);
        assert_eq!(buf.front().unwrap().id, 50);
    }

    #[test]
    fn record_and_query_returns_newest_first() {
        reset();
        let id1 = record(ErrorReport {
            view: Some("my-view".into()),
            url: None,
            kind: Some("js-error".into()),
            message: "first".into(),
            stack: None,
            filename: None,
            line: None,
            column: None,
            detail: None,
            user_agent: None,
            timestamp_client: None,
        });
        let id2 = record(ErrorReport {
            view: Some("my-view".into()),
            url: None,
            kind: Some("js-error".into()),
            message: "second".into(),
            stack: None,
            filename: None,
            line: None,
            column: None,
            detail: None,
            user_agent: None,
            timestamp_client: None,
        });
        assert!(id2 > id1);
        let (results, _) = query(None, Some("my-view"), 10);
        assert_eq!(results[0].id, id2, "newest should be first");
    }

    #[test]
    fn clear_drains_buffer() {
        reset();
        record(ErrorReport {
            view: None,
            url: None,
            kind: None,
            message: "x".into(),
            stack: None,
            filename: None,
            line: None,
            column: None,
            detail: None,
            user_agent: None,
            timestamp_client: None,
        });
        let cleared = clear();
        assert!(cleared >= 1);
        let (results, total) = query(None, None, 10);
        assert_eq!(total, 0);
        assert!(results.is_empty());
    }
}
