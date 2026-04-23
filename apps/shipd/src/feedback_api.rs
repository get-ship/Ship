//! REST handler for `POST /api/feedback`.
//!
//! Accepts a `FeedbackEvent` payload and routes it to the right subscriber:
//!
//! - `target.view_id` present and open → emit `studio.feedback.created`
//!   through the kernel (reaches the owning agent's actor mailbox) AND
//!   broadcast on the view session's SSE stream so co-watchers see it.
//! - `target.view_id` absent → workspace broadcast on the kernel event bus.
//! - `target.view_id` present but unknown / closed → 404.
//!
//! Validation, rate-limit (agent callers), and 60s id-dedupe are enforced
//! in this module. The wire format is `FeedbackEvent` — see
//! `crates/core/runtime/src/events/feedback.rs`.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use runtime::events::EventEnvelope;
use runtime::events::feedback::{AuthorKind, FeedbackEvent, FeedbackKind};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use ulid::Ulid;

use crate::auth::AuthContext;
use crate::rest_api::ApiState;
use crate::views::session::SessionError;

/// Daemon-scoped state for feedback ingress. Holds agent rate-limit
/// buckets and a sliding-window dedupe cache. One instance per daemon
/// (attached as an axum `Extension`).
#[derive(Default)]
pub struct FeedbackGuard {
    inner: Mutex<GuardInner>,
}

#[derive(Default)]
struct GuardInner {
    /// One token bucket per agent caller. Humans aren't tracked.
    buckets: HashMap<String, (f64, Instant)>,
    /// id → accepted-at. Entries older than DEDUPE_WINDOW are GC'd lazily.
    seen_ids: HashMap<String, Instant>,
}

const DEDUPE_WINDOW: Duration = Duration::from_secs(60);
/// Agent bucket: capacity 10, refill 10/sec → burst 10 then steady-state 10/s.
const AGENT_CAPACITY: f64 = 10.0;
const AGENT_REFILL_PER_SEC: f64 = 10.0;
const NOTE_MAX_BYTES: usize = 10 * 1024;
const SELECTION_TEXT_MAX_BYTES: usize = 20 * 1024;

impl FeedbackGuard {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Returns `Err` with an error code if the id has already been accepted
    /// inside the dedupe window. Otherwise records the id.
    async fn check_and_record_id(&self, id: &str) -> Result<(), &'static str> {
        let mut g = self.inner.lock().await;
        let now = Instant::now();
        g.seen_ids.retain(|_, t| now.duration_since(*t) < DEDUPE_WINDOW);
        if g.seen_ids.contains_key(id) {
            return Err("duplicate_id");
        }
        g.seen_ids.insert(id.to_string(), now);
        Ok(())
    }

    /// Try to consume a token for `agent`. Returns `false` if the bucket is
    /// empty. Humans should not call this.
    async fn try_consume_agent(&self, agent: &str) -> bool {
        let mut g = self.inner.lock().await;
        let now = Instant::now();
        let entry = g
            .buckets
            .entry(agent.to_string())
            .or_insert((AGENT_CAPACITY, now));
        let elapsed = now.duration_since(entry.1).as_secs_f64();
        entry.0 = (entry.0 + elapsed * AGENT_REFILL_PER_SEC).min(AGENT_CAPACITY);
        entry.1 = now;
        if entry.0 >= 1.0 {
            entry.0 -= 1.0;
            true
        } else {
            false
        }
    }
}

// ───────────────────────── Handler ─────────────────────────

pub async fn post_feedback(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Extension(guard): Extension<Arc<FeedbackGuard>>,
    Json(mut ev): Json<FeedbackEvent>,
) -> Response {
    // Allow server-side id generation for view-authored feedback where the
    // template can't easily produce a ULID. Empty string or any string that
    // isn't a valid ULID from a human-authored action = server generates.
    if ev.author_kind == AuthorKind::Human
        && ev.kind == FeedbackKind::Action
        && Ulid::from_string(&ev.id).is_err()
    {
        ev.id = Ulid::new().to_string();
    }
    if let Err(resp) = validate(&ev) {
        return resp;
    }

    // Agent-kind authorship is rate-limited per caller. Human authorship
    // isn't — humans don't burst. Rate-limit key prefers the authenticated
    // agent_id; fall back to `"anonymous-agent"` so rate-limit still binds.
    if ev.author_kind == AuthorKind::Agent {
        let key = auth.agent_id.clone().unwrap_or_else(|| "anonymous-agent".into());
        if !guard.try_consume_agent(&key).await {
            return err(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "agent feedback rate limit exceeded (10/sec)",
            );
        }
    }

    if let Err(code) = guard.check_and_record_id(&ev.id).await {
        return err(StatusCode::UNPROCESSABLE_ENTITY, code, "feedback id already seen within 60s window");
    }

    // View-session routing: present-and-known delivers twice, present-and-
    // unknown is a hard 404 (never silently broadcasts).
    if let Some(view_id) = ev.target.view_id.clone() {
        let payload = match serde_json::to_value(&ev) {
            Ok(v) => v,
            Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, "serialize_failed", &e.to_string()),
        };
        if let Err(e) = state.view_sessions().broadcast_feedback(&view_id, &payload).await {
            return session_err(e);
        }
        if let Err(resp) = emit_kernel(&state, &auth, &ev, &view_id).await {
            return resp;
        }
    } else {
        // Workspace broadcast path. Use the feedback id as the entity id so
        // every broadcast is independently addressable on the event log.
        if let Err(resp) = emit_kernel(&state, &auth, &ev, &ev.id).await {
            return resp;
        }
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": ev.id })),
    )
        .into_response()
}

async fn emit_kernel(
    state: &ApiState,
    auth: &AuthContext,
    ev: &FeedbackEvent,
    entity_id: &str,
) -> Result<(), Response> {
    let payload = serde_json::to_value(ev).map_err(|e| {
        err(StatusCode::INTERNAL_SERVER_ERROR, "serialize_failed", &e.to_string())
    })?;
    let envelope = EventEnvelope::new("studio.feedback.created", entity_id, &payload)
        .map_err(|e| err(StatusCode::BAD_REQUEST, "envelope_invalid", &e.to_string()))?;
    let ctx = auth.to_daemon_emit_context();
    state
        .kernel()
        .lock()
        .await
        .route(envelope, &ctx)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "route_failed", &e.to_string()))
}

// ───────────────────────── Validation ─────────────────────────

fn validate(ev: &FeedbackEvent) -> Result<(), Response> {
    if Ulid::from_string(&ev.id).is_err() {
        return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "invalid_id", "id must be a ULID"));
    }
    match ev.kind {
        FeedbackKind::Point => {
            if ev.target.selector.is_none() {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "selector_required",
                    "kind=point requires target.selector",
                ));
            }
            if ev.target.rect.is_some() {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "rect_forbidden",
                    "kind=point must not carry target.rect",
                ));
            }
        }
        FeedbackKind::Region => match ev.target.rect {
            None => {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "rect_required",
                    "kind=region requires target.rect",
                ));
            }
            Some(rect) => {
                if rect.iter().any(|v| !v.is_finite() || *v < 0.0) {
                    return Err(err(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "rect_invalid",
                        "target.rect floats must be finite and ≥ 0",
                    ));
                }
            }
        },
        FeedbackKind::Action => {}
    }
    if let Some(note) = &ev.note
        && note.len() > NOTE_MAX_BYTES
    {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "note_too_large",
            "note exceeds 10 KiB",
        ));
    }
    if let Some(sel) = &ev.selection
        && let Some(text) = &sel.text
        && text.len() > SELECTION_TEXT_MAX_BYTES
    {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "selection_too_large",
            "selection.text exceeds 20 KiB",
        ));
    }
    Ok(())
}

// ───────────────────────── Response helpers ─────────────────────────

fn err(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": { "code": code, "message": message }
        })),
    )
        .into_response()
}

fn session_err(e: SessionError) -> Response {
    match &e {
        SessionError::NotFound { .. } => err(
            StatusCode::NOT_FOUND,
            "view_not_found",
            "target.view_id refers to an unknown or closed view session",
        ),
        other => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "session_error",
            &serde_json::to_string(other).unwrap_or_default(),
        ),
    }
}

#[cfg(test)]
#[path = "feedback_api_tests.rs"]
mod tests;
