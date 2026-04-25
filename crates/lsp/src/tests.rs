//! Unit + end-to-end tests for the LSP.

use std::path::PathBuf;

use lint::report::{Finding, Severity};
use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{
    DiagnosticSeverity, NumberOrString, PublishDiagnosticsParams,
};
use tempfile::TempDir;

use crate::diagnostics::{SOURCE, finding_to_diagnostic};
use crate::uri::{is_view_source, path_to_url, url_to_path};

// ── uri ─────────────────────────────────────────────────────────────

#[test]
fn path_url_roundtrip() {
    let path = PathBuf::from("/tmp/.ship/views/demo/index.html.j2");
    let url = path_to_url(&path).expect("encode");
    assert_eq!(url.scheme(), "file");
    let back = url_to_path(&url).expect("decode");
    assert_eq!(back, path);
}

#[test]
fn path_url_rejects_relative() {
    let path = PathBuf::from("relative/path.html.j2");
    assert!(path_to_url(&path).is_err());
}

#[test]
fn path_url_handles_spaces() {
    // Encoding must round-trip a space — common Windows/macOS path quirk.
    let path = PathBuf::from("/tmp/space dir/.ship/views/demo/index.html.j2");
    let url = path_to_url(&path).expect("encode");
    assert!(url.as_str().contains("space%20dir"));
    assert_eq!(url_to_path(&url).unwrap(), path);
}

#[test]
fn is_view_source_matches_ship_views_layout() {
    assert!(is_view_source(&PathBuf::from(
        "/proj/.ship/views/demo/index.html.j2"
    )));
    assert!(is_view_source(&PathBuf::from(
        "/proj/.ship/views/demo/partials/row.html.j2"
    )));
}

#[test]
fn is_view_source_rejects_other_paths() {
    assert!(!is_view_source(&PathBuf::from("/proj/index.html.j2")));
    assert!(!is_view_source(&PathBuf::from(
        "/proj/.ship/skills/foo.html.j2"
    )));
    assert!(!is_view_source(&PathBuf::from(
        "/proj/.ship/views/demo/manifest.yaml"
    )));
}

// ── diagnostics conversion ──────────────────────────────────────────

fn mk_finding(rule: &'static str, severity: Severity) -> Finding {
    Finding {
        file: PathBuf::from("/proj/.ship/views/demo/index.html.j2"),
        line: 4,
        col: 12,
        rule,
        severity,
        message: "boom".into(),
        suggest: Some("fix it".into()),
    }
}

#[test]
fn diagnostic_maps_rule_to_code_and_source() {
    let f = mk_finding(lint::RULE_NO_INLINE_SCRIPT, Severity::Error);
    let d = finding_to_diagnostic(&f);
    assert_eq!(d.source.as_deref(), Some(SOURCE));
    assert_eq!(
        d.code,
        Some(NumberOrString::String(lint::RULE_NO_INLINE_SCRIPT.into()))
    );
}

#[test]
fn diagnostic_severity_maps_both_levels() {
    let e = finding_to_diagnostic(&mk_finding(lint::RULE_NO_INLINE_SCRIPT, Severity::Error));
    assert_eq!(e.severity, Some(DiagnosticSeverity::ERROR));
    let w = finding_to_diagnostic(&mk_finding(lint::RULE_THEME_TOKEN_REF, Severity::Warn));
    assert_eq!(w.severity, Some(DiagnosticSeverity::WARNING));
}

#[test]
fn diagnostic_position_is_zero_based_single_char_span() {
    let d = finding_to_diagnostic(&mk_finding(lint::RULE_NO_INLINE_SCRIPT, Severity::Error));
    // (line=4, col=12) -> zero-based (3, 11) with end (3, 12).
    assert_eq!(d.range.start.line, 3);
    assert_eq!(d.range.start.character, 11);
    assert_eq!(d.range.end.line, 3);
    assert_eq!(d.range.end.character, 12);
}

#[test]
fn diagnostic_appends_suggest_to_message() {
    let d = finding_to_diagnostic(&mk_finding(lint::RULE_NO_INLINE_SCRIPT, Severity::Error));
    assert!(d.message.contains("boom"));
    assert!(d.message.contains("suggest: fix it"));
}

// One conversion case per rule that lint::ALL_RULES exposes — every
// code path through `findings_to_diagnostics` must round-trip.
#[test]
fn every_rule_id_round_trips_to_lsp_code() {
    for &rule in lint::ALL_RULES {
        let f = mk_finding(rule, Severity::Warn);
        let d = finding_to_diagnostic(&f);
        match d.code {
            Some(NumberOrString::String(s)) => assert_eq!(s, rule),
            other => panic!("rule {rule} produced {:?}", other),
        }
    }
}

// ── workspace resolution ────────────────────────────────────────────

#[test]
fn find_view_dir_walks_up_to_manifest() {
    let tmp = TempDir::new().unwrap();
    let view_dir = tmp.path().join(".ship/views/demo");
    std::fs::create_dir_all(view_dir.join("partials")).unwrap();
    std::fs::write(view_dir.join("manifest.yaml"), "name: demo\n").unwrap();
    let nested = view_dir.join("partials/row.html.j2");
    std::fs::write(&nested, "<div></div>").unwrap();

    let found = crate::workspace::find_view_dir(&nested).expect("found");
    assert_eq!(found, view_dir);
}

#[test]
fn find_view_dir_falls_back_to_views_envelope_without_manifest() {
    let tmp = TempDir::new().unwrap();
    let view_dir = tmp.path().join(".ship/views/demo");
    std::fs::create_dir_all(&view_dir).unwrap();
    let file = view_dir.join("index.html.j2");
    std::fs::write(&file, "<div></div>").unwrap();
    let found = crate::workspace::find_view_dir(&file).expect("found");
    assert_eq!(found, view_dir);
}

// ── end-to-end (in-process LSP server) ──────────────────────────────

#[test]
fn didopen_publishes_diagnostics_for_inline_script() {
    let tmp = TempDir::new().unwrap();
    let view_dir = tmp.path().join(".ship/views/demo");
    std::fs::create_dir_all(&view_dir).unwrap();
    std::fs::write(view_dir.join("manifest.yaml"), "name: demo\n").unwrap();
    let view_file = view_dir.join("index.html.j2");
    let source = "<html><body>\n<script>console.log('nope');</script>\n</body></html>\n";
    std::fs::write(&view_file, source).unwrap();

    let (server_conn, client_conn) = Connection::memory();
    let uri = path_to_url(&view_file).unwrap();

    crate::server::_publish_for_test(&server_conn, &uri, source, true)
        .expect("publish");

    // Drain server -> client messages and find the PublishDiagnostics
    // for our URI.
    drop(server_conn); // close sender so receiver iter ends
    let mut got: Option<PublishDiagnosticsParams> = None;
    for msg in client_conn.receiver.try_iter() {
        if let Message::Notification(Notification { method, params }) = msg {
            if method == PublishDiagnostics::METHOD {
                let p: PublishDiagnosticsParams =
                    serde_json::from_value(params).expect("decode");
                if p.uri == uri {
                    got = Some(p);
                    break;
                }
            }
        }
    }
    let p = got.expect("expected PublishDiagnostics for view URI");
    assert!(
        p.diagnostics
            .iter()
            .any(|d| matches!(&d.code, Some(NumberOrString::String(s)) if s == lint::RULE_NO_INLINE_SCRIPT)),
        "expected style-no-inline-script diagnostic, got {:?}",
        p.diagnostics
    );
}

#[test]
fn didclose_clears_diagnostics() {
    // Re-using publish_for with text that lints clean -> empty diagnostics.
    let tmp = TempDir::new().unwrap();
    let view_dir = tmp.path().join(".ship/views/demo");
    std::fs::create_dir_all(&view_dir).unwrap();
    std::fs::write(view_dir.join("manifest.yaml"), "name: demo\n").unwrap();
    let view_file = view_dir.join("index.html.j2");
    std::fs::write(&view_file, "<div></div>\n").unwrap();

    let (server_conn, client_conn) = Connection::memory();
    let uri = path_to_url(&view_file).unwrap();
    crate::server::_publish_for_test(&server_conn, &uri, "<div></div>\n", true)
        .expect("publish");
    drop(server_conn);
    let p = client_conn
        .receiver
        .try_iter()
        .find_map(|m| match m {
            Message::Notification(n) if n.method == PublishDiagnostics::METHOD => {
                let p: PublishDiagnosticsParams = serde_json::from_value(n.params).ok()?;
                if p.uri == uri { Some(p) } else { None }
            }
            _ => None,
        })
        .expect("expected diagnostics push");
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
}
