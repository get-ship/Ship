//! Live agent-log streaming — HTTP SSE transport adapter.
//!
//! Domain logic (slug validation, file discovery, byte-range reads, ANSI
//! stripping) lives in `ship_pipeline_runtime::log_stream`. This module
//! is the thin Axum SSE wrapper that calls into that crate via
//! `WorktreesDirPort`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use ship_pipeline_runtime::log_stream::{POLL_INTERVAL, read_chunk, resolve_log_path, strip_ansi};

use crate::pipeline_ports::WorktreesDirAdapter;
use crate::routes::rest_api::ApiState;

/// GET /api/v1/agent_sessions/{slug}/log/stream
pub async fn stream_job_log(
    State(_state): State<ApiState>,
    Path(slug): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let port = WorktreesDirAdapter;
    let log_path = resolve_log_path(&port, &slug).map_err(|msg| {
        if msg.contains("invalid slug") {
            (StatusCode::BAD_REQUEST, msg)
        } else {
            (StatusCode::NOT_FOUND, msg)
        }
    })?;

    let stream = async_stream::stream! {
        let mut pos = 0u64;

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

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(30))))
}
