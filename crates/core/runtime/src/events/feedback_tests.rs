//! Serialization round-trips for the FeedbackEvent wire format.

use super::*;
use chrono::TimeZone;

fn sample() -> FeedbackEvent {
    FeedbackEvent {
        id: "01HZZ".into(),
        kind: FeedbackKind::Region,
        target: FeedbackTarget {
            view_id: Some("canvas-42".into()),
            selector: Some("#root .cell[data-id=7]".into()),
            rect: Some([10.0, 20.0, 100.0, 50.0]),
            viewport: Some([1440.0, 900.0]),
            ..Default::default()
        },
        intent: Some(FeedbackIntent::RequestChange),
        note: Some("Rename this to `total`".into()),
        selection: Some(Selection {
            kind: "text".into(),
            text: Some("amount".into()),
        }),
        payload: None,
        created_at: Utc.with_ymd_and_hms(2026, 4, 13, 12, 0, 0).unwrap(),
        author_kind: AuthorKind::Human,
    }
}

#[test]
fn feedback_event_roundtrip() {
    let ev = sample();
    let json = serde_json::to_string(&ev).unwrap();
    let back: FeedbackEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, back);
}

#[test]
fn optional_fields_default_on_absent_input() {
    let src = serde_json::json!({
        "id": "x",
        "kind": "point",
        "target": {},
        "created_at": "2026-04-13T12:00:00Z",
        "author_kind": "agent",
    });
    let parsed: FeedbackEvent = serde_json::from_value(src).unwrap();
    assert_eq!(parsed.intent, None);
    assert_eq!(parsed.note, None);
    assert_eq!(parsed.selection, None);
    assert_eq!(parsed.target.view_id, None);
}

#[test]
fn intent_enum_serializes_snake_case() {
    let ev = FeedbackEvent { intent: Some(FeedbackIntent::RequestChange), ..sample() };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("\"request_change\""), "got: {json}");
}

#[test]
fn author_kind_agent_roundtrip() {
    let ev = FeedbackEvent { author_kind: AuthorKind::Agent, ..sample() };
    let json = serde_json::to_string(&ev).unwrap();
    let back: FeedbackEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.author_kind, AuthorKind::Agent);
}
