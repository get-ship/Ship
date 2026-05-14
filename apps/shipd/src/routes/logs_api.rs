//! Live agent-log streaming — tails `<worktree>/.artifacts/agent-<slug>.log`
//! as it's written and streams the bytes to an SSE consumer.
//!
//! Works for every provider and dispatch mode because every executor
//! ultimately teed PTY output into this file. No provider-specific
//! hooks; no tmux attachment required.
//!
//! SSE event shape:
//!   event: log
//!   data: <utf-8 bytes, ansi stripped, may contain newlines>
//! An initial `event: log` fires with the existing tail (last ~64 KiB),
//! then each poll tick yields only new bytes appended since the last
//! tick. A keepalive comment fires every 30s to keep proxies from
//! closing the stream during quiet periods.

use std::path::PathBuf;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::routes::rest_api::ApiState;
use crate::supervisor::helpers::worktrees_dir;

/// How often we check the file for new bytes.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// On first connection, emit at most this many bytes from the tail so
/// a long-running job doesn't dump hundreds of KB into the client at
/// once. Subsequent appends are not capped.
const INITIAL_TAIL_BYTES: u64 = 64 * 1024;

/// GET /api/v1/jobs/{slug}/log/stream
pub async fn stream_job_log(
    State(_state): State<ApiState>,
    Path(slug): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    if !is_valid_slug(&slug) {
        return Err((StatusCode::BAD_REQUEST, "invalid slug".to_string()));
    }
    let log_path = worktrees_dir()
        .join(&slug)
        .join(".artifacts")
        .join(format!("agent-{slug}.log"));

    if !log_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("no log for slug '{slug}' at {}", log_path.display()),
        ));
    }

    let stream = async_stream::stream! {
        let mut pos = 0u64;

        // Emit the initial tail.
        if let Some(chunk) = read_chunk(&log_path, 0, pos).await {
            let text = strip_ansi(&chunk);
            if !text.is_empty() {
                yield Ok::<_, std::convert::Infallible>(
                    Event::default().event("log").data(text)
                );
            }
        }

        let mut ticker = tokio::time::interval(POLL_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let Ok(meta) = tokio::fs::metadata(&log_path).await else { continue };
            let size = meta.len();
            if size <= pos { continue; }
            let new = match read_chunk(&log_path, pos, size).await {
                Some(b) => b,
                None => continue,
            };
            pos = size;
            let text = strip_ansi(&new);
            if text.is_empty() { continue; }
            yield Ok(Event::default().event("log").data(text));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30))))
}

/// Slug component must be a safe filename — alphanumeric/dash/underscore
/// only. Without this guard a caller could pass `../../etc/passwd` and
/// escape the worktrees dir via the `worktrees_dir().join(slug)` step.
fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 128
        && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Read bytes `[from, to)` from the file. Returns None on IO error or
/// empty range. Caps the initial read (when `from == 0`) at
/// `INITIAL_TAIL_BYTES` starting from the file end.
async fn read_chunk(path: &PathBuf, from: u64, to: u64) -> Option<Vec<u8>> {
    if to <= from {
        return None;
    }
    let mut file = tokio::fs::File::open(path).await.ok()?;

    // First read: if the file is larger than INITIAL_TAIL_BYTES,
    // seek forward to send only the tail.
    let start = if from == 0 && to > INITIAL_TAIL_BYTES {
        to - INITIAL_TAIL_BYTES
    } else {
        from
    };

    file.seek(std::io::SeekFrom::Start(start)).await.ok()?;
    let len = (to - start) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf).await.ok()?;
    Some(buf)
}

/// Strip ANSI CSI escape sequences from a byte buffer, returning a
/// UTF-8 string. Invalid bytes are lossy-converted.
fn strip_ansi(input: &[u8]) -> String {
    let s = String::from_utf8_lossy(input);
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if matches!(nc, '\x40'..='\x7E') { break; }
                }
                continue;
            }
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}
