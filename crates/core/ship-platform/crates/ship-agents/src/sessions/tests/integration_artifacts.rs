/// Integration tests — sessions artifacts: on-disk file side effects of
/// start_session, record_session_progress, and end_session.
///
/// Verifies that `persist_session_artifact` (called internally) writes
/// `session.json` and `timeline.ndjson` under the session artifacts dir,
/// and that `append_session_note_artifact` writes `notes.ndjson`.
///
/// Artifact path: `<global_dir>/projects/<slug>/sessions/<session_id>/`
use anyhow::Result;
use tempfile::tempdir;

use crate::sessions::{
    end_session, record_session_progress, start_session, EndSessionRequest,
};
use crate::testutil::init_project;

fn artifact_dir(ship_dir: &std::path::Path, session_id: &str) -> Option<std::path::PathBuf> {
    let slug = ship_substrate::project::project_slug_from_ship_dir(ship_dir);
    let global_dir = ship_substrate::project::get_global_dir().ok()?;
    let root = global_dir.join("projects").join(slug).join("sessions");
    Some(root.join(ship_substrate::project::sanitize_file_name(session_id)))
}

// ── start_session artifacts ───────────────────────────────────────────────────

#[test]
fn sessions_artifacts_start_writes_session_json() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-start", None, None, None)?;
    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");

    assert!(
        dir.join("session.json").exists(),
        "session.json must exist after start_session; dir: {}",
        dir.display()
    );
    Ok(())
}

#[test]
fn sessions_artifacts_start_writes_timeline_ndjson() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-timeline", None, None, None)?;
    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");

    assert!(
        dir.join("timeline.ndjson").exists(),
        "timeline.ndjson must exist after start_session"
    );
    Ok(())
}

#[test]
fn sessions_artifacts_start_session_json_is_valid_json() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-json", None, None, None)?;
    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");

    let raw = std::fs::read_to_string(dir.join("session.json"))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).expect("session.json must be valid JSON");
    assert_eq!(
        parsed["id"].as_str(),
        Some(session.id.as_str()),
        "session.json id must match session.id"
    );
    assert_eq!(
        parsed["branch"].as_str(),
        Some("feature/artifact-json"),
        "session.json branch must match"
    );
    Ok(())
}

// ── record_session_progress artifacts ────────────────────────────────────────

#[test]
fn sessions_artifacts_progress_writes_notes_ndjson() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-notes", None, None, None)?;
    record_session_progress(
        &ship_dir,
        "feature/artifact-notes",
        "progress checkpoint",
        None,
    )?;

    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");

    assert!(
        dir.join("notes.ndjson").exists(),
        "notes.ndjson must exist after record_session_progress"
    );
    Ok(())
}

#[test]
fn sessions_artifacts_progress_appends_to_notes_ndjson() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-append", None, None, None)?;
    record_session_progress(&ship_dir, "feature/artifact-append", "first note", None)?;
    record_session_progress(&ship_dir, "feature/artifact-append", "second note", None)?;

    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");
    let content = std::fs::read_to_string(dir.join("notes.ndjson"))?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2, "notes.ndjson must have one line per note");
    assert!(lines[0].contains("first note"));
    assert!(lines[1].contains("second note"));
    Ok(())
}

// ── end_session artifacts ─────────────────────────────────────────────────────

#[test]
fn sessions_artifacts_end_updates_session_json_status() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-end", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/artifact-end",
        EndSessionRequest {
            summary: Some("done".to_string()),
            ..Default::default()
        },
    )?;

    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");
    let raw = std::fs::read_to_string(dir.join("session.json"))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(
        parsed["status"].as_str(),
        Some("ended"),
        "session.json status must be 'ended' after end_session"
    );
    Ok(())
}

#[test]
fn sessions_artifacts_end_appends_timeline_entry() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let session = start_session(&ship_dir, "feature/artifact-timeline-end", None, None, None)?;
    end_session(
        &ship_dir,
        "feature/artifact-timeline-end",
        EndSessionRequest::default(),
    )?;

    let dir = artifact_dir(&ship_dir, &session.id)
        .expect("artifact dir path must resolve");
    let content = std::fs::read_to_string(dir.join("timeline.ndjson"))?;
    // start_session writes "start", end_session writes "end"
    let lines: Vec<&str> = content.lines().collect();
    assert!(
        lines.len() >= 2,
        "timeline must have at least 2 entries (start + end); got {}",
        lines.len()
    );
    assert!(
        lines.iter().any(|l| l.contains("\"end\"")),
        "timeline must contain an 'end' phase entry"
    );
    Ok(())
}
