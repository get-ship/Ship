//! Feedback domain logic — validation, rate-limiting, deduplication, kernel emit.
//!
//! Moved from `apps/shipd/src/routes/feedback_api.rs` (Batch A routes-cull).
//! The `ViewBroadcastPort` trait abstracts view-session broadcast so the logic
//! has no compile-time dependency on the shipd `ViewSessionRegistry`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::feedback::{AuthorKind, FeedbackEvent, FeedbackKind};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::KernelLog;
use thiserror::Error;
use tokio::sync::Mutex;
use ulid::Ulid;

// ── Port trait ─────────────────────────────────────────────────────────────────

/// Shipd implements this; ship-agents calls it for view-session fan-out.
#[async_trait]
pub trait ViewBroadcastPort: Send + Sync {
    /// Broadcast a feedback payload to all subscribers of `view_id`.
    /// Returns `Err(ViewNotFound)` when the view_id is unknown or closed.
    async fn broadcast_feedback(
        &self,
        view_id: &str,
        payload: &serde_json::Value,
    ) -> Result<(), ViewBroadcastError>;
}

#[derive(Debug, Error)]
pub enum ViewBroadcastError {
    #[error("view session not found: {0}")]
    NotFound(String),
    #[error("view session error: {0}")]
    Other(String),
}

// ── Domain error ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum FeedbackError {
    #[error("invalid_id: id must be a ULID")]
    InvalidId,
    #[error("selector_required: kind=point requires target.selector")]
    SelectorRequired,
    #[error("rect_forbidden: kind=point must not carry target.rect")]
    RectForbidden,
    #[error("rect_required: kind=region requires target.rect")]
    RectRequired,
    #[error("rect_invalid: target.rect floats must be finite and ≥ 0")]
    RectInvalid,
    #[error("note_too_large: note exceeds 10 KiB")]
    NoteTooLarge,
    #[error("selection_too_large: selection.text exceeds 20 KiB")]
    SelectionTooLarge,
    #[error("rate_limited: agent feedback rate limit exceeded (10/sec)")]
    RateLimited,
    #[error("duplicate_id: feedback id already seen within 60s window")]
    DuplicateId,
    #[error("view not found: {0}")]
    ViewNotFound(String),
    #[error("serialize failed: {0}")]
    SerializeFailed(String),
    #[error("emit failed: {0}")]
    EmitFailed(String),
}

// ── Rate-limit / dedupe guard ─────────────────────────────────────────────────

const DEDUPE_WINDOW: Duration = Duration::from_secs(60);
const AGENT_CAPACITY: f64 = 10.0;
const AGENT_REFILL_PER_SEC: f64 = 10.0;
const NOTE_MAX_BYTES: usize = 10 * 1024;
const SELECTION_TEXT_MAX_BYTES: usize = 20 * 1024;

/// Daemon-scoped state for feedback ingress. Holds agent rate-limit buckets and
/// a sliding-window dedupe cache. One instance per daemon process.
#[derive(Default)]
pub struct FeedbackGuard {
    inner: Mutex<GuardInner>,
}

#[derive(Default)]
struct GuardInner {
    /// One token bucket per agent caller.
    buckets: HashMap<String, (f64, Instant)>,
    /// id → accepted-at. Entries older than DEDUPE_WINDOW are GC'd lazily.
    seen_ids: HashMap<String, Instant>,
}

impl FeedbackGuard {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    async fn check_and_record_id(&self, id: &str) -> Result<(), FeedbackError> {
        let mut g = self.inner.lock().await;
        let now = Instant::now();
        g.seen_ids.retain(|_, t| now.duration_since(*t) < DEDUPE_WINDOW);
        if g.seen_ids.contains_key(id) {
            return Err(FeedbackError::DuplicateId);
        }
        g.seen_ids.insert(id.to_string(), now);
        Ok(())
    }

    /// Try to consume a token for `agent`. Returns `false` when the bucket is empty.
    pub async fn try_consume_agent(&self, agent: &str) -> bool {
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

/// Process-wide shared guard for callers that don't carry a per-request guard.
static FEEDBACK_GUARD: std::sync::LazyLock<Arc<FeedbackGuard>> =
    std::sync::LazyLock::new(FeedbackGuard::new);

pub fn shared_guard() -> &'static FeedbackGuard {
    &FEEDBACK_GUARD
}

// ── Domain fn ─────────────────────────────────────────────────────────────────

/// Core feedback processing — validation, rate-limiting, dedupe, kernel emit.
///
/// `agent_id`: the authenticated agent's id, if any (for rate-limiting).
/// `view_broadcast`: impl of `ViewBroadcastPort` — injected by shipd.
pub async fn post_feedback(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    view_broadcast: &dyn ViewBroadcastPort,
    guard: &FeedbackGuard,
    agent_id: Option<&str>,
    mut ev: FeedbackEvent,
) -> Result<String, FeedbackError> {
    // Server-side id generation for human action feedback where the view
    // template can't produce a ULID.
    if ev.author_kind == AuthorKind::Human
        && ev.kind == FeedbackKind::Action
        && Ulid::from_string(&ev.id).is_err()
    {
        ev.id = Ulid::new().to_string();
    }
    validate(&ev)?;

    if ev.author_kind == AuthorKind::Agent {
        let key = agent_id.unwrap_or("anonymous-agent");
        if !guard.try_consume_agent(key).await {
            return Err(FeedbackError::RateLimited);
        }
    }

    guard.check_and_record_id(&ev.id).await?;

    if let Some(view_id) = ev.target.view_id.clone() {
        let payload = serde_json::to_value(&ev)
            .map_err(|e| FeedbackError::SerializeFailed(e.to_string()))?;
        view_broadcast
            .broadcast_feedback(&view_id, &payload)
            .await
            .map_err(|e| match e {
                ViewBroadcastError::NotFound(id) => FeedbackError::ViewNotFound(id),
                ViewBroadcastError::Other(msg) => FeedbackError::EmitFailed(msg),
            })?;
        emit_kernel(kernel, &emit_ctx, &ev, &view_id).await?;
    } else {
        emit_kernel(kernel, &emit_ctx, &ev, &ev.id.clone()).await?;
    }

    Ok(ev.id.clone())
}

async fn emit_kernel(
    kernel: &KernelLog,
    ctx: &EmitContext,
    ev: &FeedbackEvent,
    entity_id: &str,
) -> Result<(), FeedbackError> {
    let payload = serde_json::to_value(ev)
        .map_err(|e| FeedbackError::SerializeFailed(e.to_string()))?;
    let envelope = EventEnvelope::new("studio.feedback.created", entity_id, &payload)
        .map_err(|e| FeedbackError::EmitFailed(e.to_string()))?;
    kernel
        .emit(envelope, ctx)
        .await
        .map_err(|e| FeedbackError::EmitFailed(e.to_string()))?;
    Ok(())
}

// ── Validation ────────────────────────────────────────────────────────────────

fn validate(ev: &FeedbackEvent) -> Result<(), FeedbackError> {
    if Ulid::from_string(&ev.id).is_err() {
        return Err(FeedbackError::InvalidId);
    }
    match ev.kind {
        FeedbackKind::Point => {
            if ev.target.selector.is_none() {
                return Err(FeedbackError::SelectorRequired);
            }
            if ev.target.rect.is_some() {
                return Err(FeedbackError::RectForbidden);
            }
        }
        FeedbackKind::Region => match ev.target.rect {
            None => return Err(FeedbackError::RectRequired),
            Some(rect) => {
                if rect.iter().any(|v| !v.is_finite() || *v < 0.0) {
                    return Err(FeedbackError::RectInvalid);
                }
            }
        },
        FeedbackKind::Action => {}
    }
    if let Some(note) = &ev.note
        && note.len() > NOTE_MAX_BYTES
    {
        return Err(FeedbackError::NoteTooLarge);
    }
    if let Some(sel) = &ev.selection
        && let Some(text) = &sel.text
        && text.len() > SELECTION_TEXT_MAX_BYTES
    {
        return Err(FeedbackError::SelectionTooLarge);
    }
    Ok(())
}
