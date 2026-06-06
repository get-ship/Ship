//! Annotation domain logic — validation, persistence, kernel emit, view broadcast.
//!
//! Moved from `apps/shipd/src/routes/annotations_api.rs` (Batch A routes-cull).
//! The `CollectionPersistPort` and `ViewBroadcastPort` traits abstract the
//! shipd-specific services so ship-agents has no compile-time shipd dependency.

use async_trait::async_trait;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::annotation::AnnotationEvent;
use ship_substrate::events::validator::EmitContext;
use ship_substrate::ports::KernelLog;
use thiserror::Error;
use ulid::Ulid;

pub use super::feedback::ViewBroadcastPort;

// ── Port trait: collection persistence ───────────────────────────────────────

/// Shipd implements this for the built-in `annotations` collection.
#[async_trait]
pub trait CollectionPersistPort: Send + Sync {
    /// Ensure the `annotations` table exists and insert `row_bytes`.
    async fn persist_annotation(&self, row_bytes: &[u8]) -> Result<(), CollectionError>;
}

#[derive(Debug, Error)]
pub enum CollectionError {
    #[error("no active space: {0}")]
    NoActiveSpace(String),
    #[error("persist failed: {0}")]
    PersistFailed(String),
}

// ── Domain error ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AnnotationError {
    #[error("selector_required: kind=point requires target.selector")]
    SelectorRequired,
    #[error("rect_invalid: target.rect floats must be finite and ≥ 0")]
    RectInvalid,
    #[error("note_too_large: note exceeds 10 KiB")]
    NoteTooLarge,
    #[error("selection_too_large: selection.text exceeds 20 KiB")]
    SelectionTooLarge,
    #[error("collection error: {0}")]
    Collection(#[from] CollectionError),
    #[error("serialize failed: {0}")]
    SerializeFailed(String),
    #[error("emit failed: {0}")]
    EmitFailed(String),
}

const NOTE_MAX_BYTES: usize = 10 * 1024;
const SELECTION_TEXT_MAX_BYTES: usize = 20 * 1024;

// ── Domain fn ─────────────────────────────────────────────────────────────────

/// Core annotation processing — validation, persistence, kernel emit, view broadcast.
pub async fn post_annotation(
    kernel: &KernelLog,
    emit_ctx: EmitContext,
    view_broadcast: &dyn ViewBroadcastPort,
    collection: &dyn CollectionPersistPort,
    mut ev: AnnotationEvent,
) -> Result<String, AnnotationError> {
    if Ulid::from_string(&ev.id).is_err() {
        ev.id = Ulid::new().to_string();
    }
    validate(&ev)?;

    let row = serde_json::json!({
        "kind": annotation_kind_str(&ev),
        "view_id": ev.target.view_id.clone(),
        "file_path": ev.target.file_path.clone(),
        "selector": ev.target.selector.clone(),
        "intent": ev.intent.map(|i| serde_json::to_value(i).ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()),
        "note": ev.note.clone(),
        "payload": serde_json::to_string(&ev)
            .map_err(|e| AnnotationError::SerializeFailed(e.to_string()))?,
        "author_kind": match ev.author_kind {
            ship_substrate::events::annotation::AuthorKind::Human => "human",
            ship_substrate::events::annotation::AuthorKind::Agent => "agent",
        },
    });

    let row_bytes = serde_json::to_vec(&row)
        .map_err(|e| AnnotationError::SerializeFailed(e.to_string()))?;
    collection.persist_annotation(&row_bytes).await?;

    let payload_value = serde_json::to_value(&ev)
        .map_err(|e| AnnotationError::SerializeFailed(e.to_string()))?;
    let envelope = EventEnvelope::new("studio.annotation.created", &ev.id, &payload_value)
        .map_err(|e| AnnotationError::EmitFailed(e.to_string()))?;
    kernel
        .emit(envelope, &emit_ctx)
        .await
        .map_err(|e| AnnotationError::EmitFailed(e.to_string()))?;

    if let Some(view_id) = ev.target.view_id.as_ref() {
        let signal = serde_json::json!({ "annotation": payload_value });
        // Best-effort: view broadcast failure doesn't fail the annotation.
        let _ = view_broadcast.broadcast_feedback(view_id, &signal).await;
    }

    Ok(ev.id.clone())
}

fn annotation_kind_str(ev: &AnnotationEvent) -> &'static str {
    use ship_substrate::events::annotation::AnnotationKind;
    match ev.kind {
        AnnotationKind::Point => "point",
        AnnotationKind::Region => "region",
        AnnotationKind::Action => "action",
    }
}

fn validate(ev: &AnnotationEvent) -> Result<(), AnnotationError> {
    use ship_substrate::events::annotation::AnnotationKind;
    match ev.kind {
        AnnotationKind::Point => {
            if ev.target.selector.is_none() {
                return Err(AnnotationError::SelectorRequired);
            }
        }
        AnnotationKind::Region => {
            if let Some(rect) = ev.target.rect
                && rect.iter().any(|v| !v.is_finite() || *v < 0.0)
            {
                return Err(AnnotationError::RectInvalid);
            }
        }
        AnnotationKind::Action => {}
    }
    if let Some(note) = &ev.note
        && note.len() > NOTE_MAX_BYTES
    {
        return Err(AnnotationError::NoteTooLarge);
    }
    if let Some(sel) = &ev.selection
        && let Some(text) = &sel.text
        && text.len() > SELECTION_TEXT_MAX_BYTES
    {
        return Err(AnnotationError::SelectionTooLarge);
    }
    Ok(())
}
