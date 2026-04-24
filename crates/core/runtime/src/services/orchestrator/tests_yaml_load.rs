//! End-to-end smoke tests for the pipeline YAML loader.
//!
//! Every other test in `pipeline_loader_tests` / `tests_step_io` constructs
//! `Pipeline` values directly, so the YAML entry point
//! (`load_pipeline_file → normalize_pipeline → parse_pipeline_content`)
//! has zero regression coverage. These tests write a real pipeline file
//! into a tempdir under `.ship/pipelines/`, call the loader against that
//! tempdir, and assert the parsed shape — a tripwire for loader regressions,
//! not comprehensive coverage.
//!
//! The loader walks up from `current_dir()` looking for `.ship/`. Changing
//! CWD is process-global, so these tests are serialized via a static mutex.

use std::sync::Mutex;

use crate::services::orchestrator::pipeline_loader::load_pipeline_file;

/// CWD is a process-global resource. Hold this while chdir'd or parallel
/// tests will stomp on each other's cwd.
static CWD_LOCK: Mutex<()> = Mutex::new(());

struct CwdGuard {
    prev: std::path::PathBuf,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl CwdGuard {
    fn enter(dir: &std::path::Path) -> Self {
        let lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir).expect("chdir into tempdir");
        Self { prev, _lock: lock }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev);
    }
}

const TWO_STEP_YAML: &str = r#"---
name: smoke-two-step
vars:
  model: sonnet
agents:
  coder:
    provider: claude
    model: "{{ model }}"
steps:
  - id: write-tests
    agent: coder
    goal: write failing tests
    produces:
      - name: failing_tests
        scope: run
        required: true
  - id: implement
    agent: coder
    after: [write-tests]
    consumes: [failing_tests]
    produces:
      - name: patch
        scope: run
        required: true
"#;

#[test]
fn two_step_yaml_loads_with_expected_name_and_steps() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let pipelines_dir = tmp.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();
    std::fs::write(pipelines_dir.join("smoke-two-step.yaml"), TWO_STEP_YAML).unwrap();

    let _guard = CwdGuard::enter(tmp.path());
    let (pipeline, meta) = load_pipeline_file("smoke-two-step")
        .expect("loader must not error")
        .expect("loader must find the file we just wrote");

    assert_eq!(meta.name.as_deref(), Some("smoke-two-step"));
    assert_eq!(pipeline.steps.len(), 2, "two declared steps");
    assert_eq!(pipeline.steps[0].id, "write-tests");
    assert_eq!(pipeline.steps[1].id, "implement");
    assert_eq!(pipeline.steps[1].consumes, vec!["failing_tests".to_string()]);
    assert_eq!(pipeline.steps[1].produces.len(), 1);
    assert_eq!(pipeline.steps[1].produces[0].name, "patch");
}

#[test]
fn malformed_yaml_surfaces_error_with_file_path() {
    // Tab characters are illegal in YAML indentation — triggers a serde error.
    // The surfaced error must reference the file path so the user sees which
    // pipeline file is broken, not a generic "pipeline not found".
    let tmp = tempfile::tempdir().expect("tmpdir");
    let pipelines_dir = tmp.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();
    let bad = "---\nname: broken\nsteps:\n\t- id: x\n";
    let bad_path = pipelines_dir.join("broken.yaml");
    std::fs::write(&bad_path, bad).unwrap();

    let _guard = CwdGuard::enter(tmp.path());
    let err = load_pipeline_file("broken")
        .expect_err("malformed YAML must surface as Err, not None");
    assert!(
        err.contains("broken.yaml"),
        "error must name the source file: {err}"
    );
    let lower = err.to_lowercase();
    assert!(
        lower.contains("parse") || lower.contains("yaml") || lower.contains("tab")
            || lower.contains("normalization") || lower.contains("deserialize"),
        "error must carry the underlying parse context: {err}"
    );
}

#[test]
fn missing_pipeline_returns_ok_none() {
    // "File not found" is distinct from "file present but malformed" —
    // callers that fall through to other resolution strategies rely on
    // this distinction (Ok(None) continues, Err stops).
    let tmp = tempfile::tempdir().expect("tmpdir");
    let pipelines_dir = tmp.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();

    let _guard = CwdGuard::enter(tmp.path());
    let result = load_pipeline_file("does-not-exist")
        .expect("missing file is not an error");
    assert!(result.is_none(), "missing file must map to Ok(None)");
}

/// Pipeline using the canonical (post-contract-freeze) gate shape:
/// list-of-rules + list-of-globs. This is the only shape the loader
/// accepts after contract-freeze Item 1.
const CANONICAL_GATE_YAML: &str = r#"---
name: canonical-gate
steps:
  - id: implement
    agent: coder
  - id: review
    type: review
    agent: reviewer
    after: [implement]
    gate:
      require_changed:
        - match: ["**/*.rs"]
          not_match: ["**/tests/**"]
          min: 1
      forbid_changed:
        - match: [".artifacts/**"]
          min: 0
"#;

#[test]
fn canonical_gate_shape_loads_cleanly() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let pipelines_dir = tmp.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();
    std::fs::write(pipelines_dir.join("canonical-gate.yaml"), CANONICAL_GATE_YAML).unwrap();
    let _guard = CwdGuard::enter(tmp.path());
    let (pipeline, _meta) = load_pipeline_file("canonical-gate")
        .expect("loader must not error")
        .expect("loader must find file");
    let review = pipeline.steps.iter().find(|s| s.id == "review").unwrap();
    let gate = review.gate.as_ref().expect("gate present");
    assert_eq!(gate.require_changed.len(), 1, "exactly one require rule");
    assert_eq!(gate.require_changed[0].match_patterns,
        vec!["**/*.rs".to_string()]);
    assert_eq!(gate.require_changed[0].not_match,
        vec!["**/tests/**".to_string()]);
    assert_eq!(gate.forbid_changed.len(), 1);
}

#[test]
fn legacy_single_object_gate_shape_is_rejected_at_load() {
    let yaml = r#"---
name: legacy-gate
steps:
  - id: review
    type: review
    agent: reviewer
    gate:
      require_changed:
        match: "**/*.rs"
        min: 1
"#;
    let tmp = tempfile::tempdir().expect("tmpdir");
    let pipelines_dir = tmp.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pipelines_dir).unwrap();
    std::fs::write(pipelines_dir.join("legacy-gate.yaml"), yaml).unwrap();
    let _guard = CwdGuard::enter(tmp.path());
    let err = load_pipeline_file("legacy-gate")
        .expect_err("legacy single-object gate shape must be rejected");
    assert!(
        err.contains("legacy-gate.yaml"),
        "error must name the source file: {err}"
    );
    assert!(
        err.contains("array") || err.contains("contract freeze"),
        "error must describe canonical shape: {err}"
    );
}
