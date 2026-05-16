//! View-authoring SSE event formatters + publish helpers.
//!
//! The per-view SSE channel (`/_sui/views/{name}/sse`) already carries
//! `patch-signals` and `patch-elements` events; the view-authoring
//! contract extends it with four agent-facing event types:
//!
//!   event: sui:view:annotation         — fired by `view.annotate`
//!   event: sui:view:annotation:clear   — fired after the post-write delay
//!   event: sui:view:compiled           — fired by bundler module_changed
//!   event: sui:view:errored            — fired by bundler compile_error
//!
//! Publish goes through [`sui_patch_channel::publish`] — pre-encoded SSE
//! frames are fanned out to every open SSE connection for the view.

use serde::Serialize;

use super::sui_patch_channel;

/// Wire payload for `sui:view:annotation`. Mirrors the contract block in
/// `view-authoring-contract.md`.
#[derive(Debug, Clone, Serialize)]
pub struct AnnotationPayload {
    pub annotation_id: String,
    pub view_name: String,
    pub selector: String,
    pub note: String,
    pub kind: String,
    pub turn_id: String,
    pub timestamp: String,
}

/// Wire payload for `sui:view:annotation:clear`.
/// `annotation_id` omitted clears every pending pin for the view.
#[derive(Debug, Clone, Serialize)]
pub struct AnnotationClearPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<String>,
    pub view_name: String,
}

/// Wire payload for `sui:view:compiled` and `sui:view:errored`.
#[derive(Debug, Clone, Serialize)]
pub struct CompileEventPayload {
    pub view_name: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

fn frame(event: &str, payload: &serde_json::Value) -> String {
    format!(
        "event: {event}\ndata: {}\n\n",
        serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()),
    )
}

/// Format + publish an annotation pin to the per-view SSE channel.
/// Returns the receiver count from the underlying broadcast.
pub async fn publish_annotation(view_name: &str, payload: &AnnotationPayload) -> usize {
    let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let frame = frame("sui:view:annotation", &value);
    sui_patch_channel::publish(view_name, frame).await
}

/// Publish an `annotation:clear` event. Omitting `annotation_id` clears
/// every pending pin on the view.
pub async fn publish_annotation_clear(
    view_name: &str,
    payload: &AnnotationClearPayload,
) -> usize {
    let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let frame = frame("sui:view:annotation:clear", &value);
    sui_patch_channel::publish(view_name, frame).await
}

/// Publish a successful-compile event.
pub async fn publish_compiled(view_name: &str, payload: &CompileEventPayload) -> usize {
    let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let frame = frame("sui:view:compiled", &value);
    sui_patch_channel::publish(view_name, frame).await
}

/// Publish a compile-failure event.
pub async fn publish_errored(view_name: &str, payload: &CompileEventPayload) -> usize {
    let value = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
    let frame = frame("sui:view:errored", &value);
    sui_patch_channel::publish(view_name, frame).await
}

/// Try to project a bundler target name (`views/<name>/view`) onto the
/// view name. Returns `None` for non-view targets (`sui-runtime`, etc.).
pub fn view_name_from_bundler_target(target: &str) -> Option<&str> {
    let rest = target.strip_prefix("views/")?;
    let name = rest.strip_suffix("/view")?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_event_and_data(frame: &str) -> (String, serde_json::Value) {
        let event_line = frame.lines().find(|l| l.starts_with("event: ")).unwrap();
        let data_line = frame.lines().find(|l| l.starts_with("data: ")).unwrap();
        let event = event_line["event: ".len()..].to_string();
        let data: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).unwrap();
        (event, data)
    }

    #[tokio::test]
    async fn annotation_frame_round_trips() {
        let payload = AnnotationPayload {
            annotation_id: "a1".into(),
            view_name: "dashboard".into(),
            selector: ".btn".into(),
            note: "rename me".into(),
            kind: "change".into(),
            turn_id: "t1".into(),
            timestamp: "2026-05-16T00:00:00Z".into(),
        };
        let value = serde_json::to_value(&payload).unwrap();
        let f = frame("sui:view:annotation", &value);
        let (e, d) = extract_event_and_data(&f);
        assert_eq!(e, "sui:view:annotation");
        assert_eq!(d["annotation_id"], "a1");
        assert_eq!(d["kind"], "change");
    }

    #[tokio::test]
    async fn annotation_clear_omits_id_when_none() {
        let payload = AnnotationClearPayload {
            annotation_id: None,
            view_name: "dashboard".into(),
        };
        let value = serde_json::to_value(&payload).unwrap();
        let f = frame("sui:view:annotation:clear", &value);
        let (_, d) = extract_event_and_data(&f);
        assert!(d.get("annotation_id").is_none());
        assert_eq!(d["view_name"], "dashboard");
    }

    #[test]
    fn bundler_target_extraction() {
        assert_eq!(view_name_from_bundler_target("views/dashboard/view"), Some("dashboard"));
        assert_eq!(view_name_from_bundler_target("views/my-view/view"), Some("my-view"));
        assert_eq!(view_name_from_bundler_target("sui-runtime"), None);
        assert_eq!(view_name_from_bundler_target("views//view"), None);
        assert_eq!(view_name_from_bundler_target("views/dashboard/index.html"), None);
    }

    /// End-to-end: subscribe to the per-view channel, publish a compile
    /// event, verify the subscriber receives a well-formed frame.
    #[tokio::test]
    async fn publish_compiled_reaches_per_view_subscriber() {
        let mut rx = sui_patch_channel::subscribe("vae-publish-test").await;
        let payload = CompileEventPayload {
            view_name: "vae-publish-test".into(),
            timestamp: "2026-05-16T00:00:00Z".into(),
            message: None,
            line: None,
            col: None,
            file: None,
        };
        let n = publish_compiled("vae-publish-test", &payload).await;
        assert_eq!(n, 1);
        let frame = rx.recv().await.unwrap();
        let (e, d) = extract_event_and_data(&frame);
        assert_eq!(e, "sui:view:compiled");
        assert_eq!(d["view_name"], "vae-publish-test");
    }
}
