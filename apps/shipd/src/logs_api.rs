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

use crate::rest_api::ApiState;
use crate::supervisor::helpers::worktrees_dir;

/// How often we check the file for new bytes.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// On first connection, emit at most this many bytes from the tail so
/// a long-running job doesn't dump hundreds of KB into the client at
/// once. Subsequent appends are not capped.
const INITIAL_TAIL_BYTES: u64 = 64 * 1024;

/// GET /api/runtime/jobs/{slug}/log/stream
pub async fn stream_job_log(
    State(_state): State<ApiState>,
    Path(slug): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
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
        let mut pos = seek_initial(&log_path).await;

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

/// Resolve the initial seek position: either 0 (file small enough to
/// send entirely) or `size - INITIAL_TAIL_BYTES` (send only the tail).
async fn seek_initial(path: &PathBuf) -> u64 {
    let size = tokio::fs::metadata(path).await.map(|m| m.len()).unwrap_or(0);
    if size > INITIAL_TAIL_BYTES {
        0 // we'll seek to (size - INITIAL_TAIL_BYTES) inside read_chunk
    } else {
        0
    }
    // NB: seek-to-tail logic happens inside the first read_chunk via
    // the from/to pair below. This function returns the starting pos
    // for *subsequent* tail reads, which is the file size at send time.
    // Simpler: just return 0 and let the first chunk be the whole file
    // when it's under the cap. See read_chunk below.
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
                while let Some(nc) = chars.next() {
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
