//! Split off from runtime_api.rs (mr-04 follow-up).
//!
//! `poll_events_core` lives in `ship_substrate::runtime_reads`; this module
//! re-exports it for backwards-compatibility and owns the SSE transport adapter.

use axum::{
    extract::State,
    http::StatusCode,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use ship_substrate::events::ActorConfig;
use serde::Deserialize;

use crate::routes::rest_api::ApiState;

// Re-export so `events_rpc.rs` and any existing callers continue to resolve.
pub use ship_substrate::runtime_reads::poll_events_core;

#[derive(Deserialize)]
pub struct EventsPollQuery {
    /// ULID cursor — return events with id > since_id, ordered ASC.
    pub since_id: Option<String>,
    /// If set, only return events whose envelope `entity_id` matches OR
    /// whose payload contains `"slug": "<slug>"`.
    pub slug: Option<String>,
    /// Optional event-namespace prefix filter ("job", "process", "supervisor").
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

/// GET /api/v1/runtime/events — SSE stream of all kernel events.
///
/// The SSE transport (kernel actor spawn, mailbox drain, ActorGuard cleanup)
/// is genuine daemon infrastructure — kept here as a thin transport wrapper.
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let actor_id = format!(
        "studio-events-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
        // SSE is the observability surface — clients opt in explicitly
        // when they want the full stream including directed events
        // (studio.chat_turn, hook.*, mesh.message). Without this flag,
        // the kernel's directed-event short-circuit hides those from
        // broadcast subscribers by design.
        see_directed_events: true,
    };

    let mailbox = {
        let mut kernel = state.kernel().lock().await;
        kernel.spawn_actor(&actor_id, config).map(|(_store, mb)| mb).map_err(|e| {
            tracing::warn!("event_stream: actor spawn failed: {e}");
            let status = match &e {
                ship_substrate::errors::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })?
    };

    let guard = crate::ship_runtime::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut mb = mailbox;
        while let Some(event) = mb.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default().event("ship.event").data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
