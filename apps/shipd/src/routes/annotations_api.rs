//! REST handler for `POST /api/v1/annotations`.
//!
//! Accepts an `AnnotationEvent`, persists it to the built-in `annotations`
//! collection (which fires `collection.item_created`), then emits the
//! canonical `studio.annotation.created` event so subscribed peers receive
//! the wire-format payload directly. View-session-targeted annotations are
//! also broadcast as a Datastar `annotation` signal so co-watchers update
//! without an extra subscribe.

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ship_platform::collections::builtin::{annotations_schema, ANNOTATIONS_ID};
use ship_platform::collections::service as collection_service;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::annotation::AnnotationEvent;
use ulid::Ulid;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;

const NOTE_MAX_BYTES: usize = 10 * 1024;
const SELECTION_TEXT_MAX_BYTES: usize = 20 * 1024;

#[utoipa::path(
    operation_id = "postAnnotation",
    post,
    path = "/api/v1/annotations",
    request_body(content_type = "application/json", description = "AnnotationEvent payload — target, selector, intent, note"),
    responses(
        (status = 200, description = "Annotation persisted and broadcast"),
        (status = 400, description = "Invalid annotation payload", body = crate::routes::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::routes::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::routes::openapi_types::ErrorResponse),
    ),
    tag = "runtime"
)]
pub async fn post_annotation(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(mut ev): Json<AnnotationEvent>,
) -> Response {
    if Ulid::from_string(&ev.id).is_err() {
        ev.id = Ulid::new().to_string();
    }
    if let Err(resp) = validate(&ev) {
        return resp;
    }

    let payload_json = match serde_json::to_string(&ev) {
        Ok(s) => s,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "serialize_failed",
                &e.to_string(),
            );
        }
    };

    let row = serde_json::json!({
        "kind": annotation_kind_str(&ev),
        "view_id": ev.target.view_id.clone(),
        "file_path": ev.target.file_path.clone(),
        "selector": ev.target.selector.clone(),
        "intent": ev.intent.map(|i| serde_json::to_value(i).ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()),
        "note": ev.note.clone(),
        "payload": payload_json,
        "author_kind": match ev.author_kind {
            ship_substrate::events::annotation::AuthorKind::Human => "human",
            ship_substrate::events::annotation::AuthorKind::Agent => "agent",
        },
    });

    let mut data_guard = match state.data_service().await {
        Ok(g) => g,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "data_unavailable",
                &e.to_string(),
            );
        }
    };
    let data = data_guard.as_mut().expect("data service initialized");
    let schema = annotations_schema();
    if let Err(e) = collection_service::create_item(
        data,
        state.kernel(),
        &schema,
        &ev.id,
        &row,
        auth.agent_id.as_deref(),
    )
    .await
    {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "persist_failed",
            &e.to_string(),
        );
    }
    drop(data_guard);

    // Mirror the canonical wire-format event so EventRelay delivers the
    // full `AnnotationEvent` payload to subscribed peers (collection.* fires
    // a different shape and is consumed by view rails, not agents).
    let payload_value = match serde_json::to_value(&ev) {
        Ok(v) => v,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "serialize_failed",
                &e.to_string(),
            );
        }
    };
    let envelope = match EventEnvelope::new("studio.annotation.created", &ev.id, &payload_value) {
        Ok(e) => e,
        Err(e) => {
            return err(
                StatusCode::BAD_REQUEST,
                "envelope_invalid",
                &e.to_string(),
            );
        }
    };
    let ctx = auth.to_daemon_emit_context();
    if let Err(e) = state.kernel().lock().await.route(envelope, &ctx).await {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "route_failed",
            &e.to_string(),
        );
    }

    if let Some(view_id) = ev.target.view_id.as_ref() {
        let signal = serde_json::json!({ "annotation": payload_value });
        let _ = state.view_sessions().broadcast_feedback(view_id, &signal).await;
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": ev.id })),
    )
        .into_response()
}

fn annotation_kind_str(ev: &AnnotationEvent) -> &'static str {
    use ship_substrate::events::annotation::AnnotationKind;
    match ev.kind {
        AnnotationKind::Point => "point",
        AnnotationKind::Region => "region",
        AnnotationKind::Action => "action",
    }
}

fn validate(ev: &AnnotationEvent) -> Result<(), Response> {
    use ship_substrate::events::annotation::AnnotationKind;
    match ev.kind {
        AnnotationKind::Point => {
            if ev.target.selector.is_none() {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "selector_required",
                    "kind=point requires target.selector",
                ));
            }
        }
        AnnotationKind::Region => {
            if let Some(rect) = ev.target.rect
                && rect.iter().any(|v| !v.is_finite() || *v < 0.0)
            {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "rect_invalid",
                    "target.rect floats must be finite and ≥ 0",
                ));
            }
        }
        AnnotationKind::Action => {}
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

#[cfg(test)]
#[path = "annotations_api_tests.rs"]
mod tests;
