//! Pure encoding helpers for SUI SSE patch events.
//!
//! Mirrors the Datastar event-name + line-prefix discipline so the wire
//! format stays compatible and v2 `patch-elements` can extend it cleanly.
//!
//! Wire format (per spec):
//!   event: patch-signals
//!   data: {"name":"<name>","value":<LiveState JSON>}
//!
//! One frame per data source per update. The runtime routes by `name` to
//! the matching LiveSignal and replaces its value with `value`.

/// Format one `patch-signals` SSE frame for a named data source.
///
/// `name` is the `<data name="...">` declaration's name; `value` is the
/// `LiveState<T>` JSON value to apply (e.g. `{"kind":"ok","value":{...}}`).
pub fn format_patch_signals(name: &str, value: &serde_json::Value) -> String {
    let payload = serde_json::json!({
        "name": name,
        "value": value,
    });
    format!(
        "event: patch-signals\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    )
}

/// Format a keepalive comment frame. Sent periodically to prevent proxies
/// from closing idle SSE connections.
pub fn format_keepalive() -> &'static str {
    ": ship-keepalive\n\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn patch_signals_frame_has_correct_event_name() {
        let frame = format_patch_signals("overview", &json!({"kind": "loading"}));
        assert!(frame.starts_with("event: patch-signals\n"), "got: {frame}");
    }

    #[test]
    fn patch_signals_frame_encodes_name_and_value() {
        let value = json!({"kind": "ok", "value": {"branch": "main"}});
        let frame = format_patch_signals("overview", &value);
        assert!(frame.contains(r#""name":"overview""#), "got: {frame}");
        assert!(frame.contains(r#""branch":"main""#), "got: {frame}");
    }

    #[test]
    fn patch_signals_frame_ends_with_double_newline() {
        let frame = format_patch_signals("x", &json!(null));
        assert!(frame.ends_with("\n\n"), "SSE frames must end with \\n\\n; got: {frame:?}");
    }

    #[test]
    fn keepalive_is_sse_comment() {
        let ka = format_keepalive();
        assert!(ka.starts_with(':'), "SSE comments start with colon; got: {ka:?}");
    }

    #[test]
    fn loading_envelope_round_trips() {
        let value = json!({"kind": "loading"});
        let frame = format_patch_signals("agents", &value);
        // Extract the data line and parse it back.
        let data_line = frame
            .lines()
            .find(|l| l.starts_with("data: "))
            .expect("data line");
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).expect("valid JSON");
        assert_eq!(parsed["name"], "agents");
        assert_eq!(parsed["value"]["kind"], "loading");
    }

    #[test]
    fn ok_envelope_round_trips() {
        let value = json!({"kind": "ok", "value": {"commits": 3}});
        let frame = format_patch_signals("log", &value);
        let data_line = frame
            .lines()
            .find(|l| l.starts_with("data: "))
            .expect("data line");
        let parsed: serde_json::Value =
            serde_json::from_str(&data_line["data: ".len()..]).expect("valid JSON");
        assert_eq!(parsed["value"]["kind"], "ok");
        assert_eq!(parsed["value"]["value"]["commits"], 3);
    }
}
