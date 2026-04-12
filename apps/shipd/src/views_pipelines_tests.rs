use super::*;
use chrono::Utc;
use runtime::events::job::{Pipeline, PipelineStep, StepType};
use runtime::projections::job::JobRecord;
use std::collections::HashMap;
use tempfile::TempDir;

fn mk_job(slug: &str, status: JobStatus) -> JobRecord {
    JobRecord {
        job_id: format!("j-{slug}"),
        slug: slug.into(),
        agent: "tester".into(),
        branch: "main".into(),
        spec_path: String::new(),
        depends_on: None,
        status,
        worktree: None,
        blocker: None,
        error: None,
        model: None,
        provider: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        provider_meta: None,
        pipeline: None,
        steps: Some(Pipeline {
            steps: vec![
                PipelineStep {
                    id: "impl".into(),
                    step_type: StepType::Agent,
                    agent: Some("tester".into()),
                    goal: None,
                    after: vec![],
                    model: None,
                    provider: None,
                    command: None,
                    max_retries: None,
                },
                PipelineStep {
                    id: "gate".into(),
                    step_type: StepType::Gate,
                    agent: Some("reviewer".into()),
                    goal: None,
                    after: vec!["impl".into()],
                    model: None,
                    provider: None,
                    command: None,
                    max_retries: None,
                },
            ],
        }),
        current_phase: None,
        completed_steps: vec![],
        current_step: Some("impl".into()),
        vars: None,
        step_retries: HashMap::new(),
    }
}

#[test]
fn parse_pipeline_meta_reads_description_and_steps() {
    let yaml = "name: foo\ndescription: hello\nagents: [a, b, c]\n";
    let (desc, count) = parse_pipeline_meta(yaml);
    assert_eq!(desc, "hello");
    assert_eq!(count, 3);
}

#[test]
fn parse_pipeline_meta_counts_steps_array() {
    let yaml = "name: f\nsteps:\n  - id: a\n  - id: b\n";
    let (_desc, count) = parse_pipeline_meta(yaml);
    assert_eq!(count, 2);
}

#[test]
fn parse_pipeline_meta_handles_missing() {
    let (desc, count) = parse_pipeline_meta("name: only\n");
    assert_eq!(desc, "");
    assert_eq!(count, 0);
}

#[test]
fn load_pipeline_definitions_reads_yaml_files() {
    let dir = TempDir::new().unwrap();
    let pdir = dir.path().join(".ship").join("pipelines");
    std::fs::create_dir_all(&pdir).unwrap();
    std::fs::write(
        pdir.join("tdd.yaml"),
        "name: tdd\ndescription: test first\nagents: [a, b]\n",
    )
    .unwrap();
    std::fs::write(pdir.join("README.md"), "ignore me").unwrap();

    unsafe { std::env::set_var("SHIP_DIR", dir.path().join(".ship").to_str().unwrap()) };
    let defs = load_pipeline_definitions();
    unsafe { std::env::remove_var("SHIP_DIR") };

    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].0, "tdd");
    assert_eq!(defs[0].1, "test first");
    assert_eq!(defs[0].2, 2);
}

#[test]
fn render_definitions_tab_shows_empty_state() {
    let html = render_definitions_tab(&[]);
    assert!(html.contains("No pipelines defined"));
}

#[test]
fn render_definitions_tab_lists_pipelines() {
    let defs = vec![("tdd".into(), "test-driven".into(), 3usize)];
    let html = render_definitions_tab(&defs);
    assert!(html.contains(r#"name="tdd""#));
    assert!(html.contains(r#"description="test-driven""#));
    assert!(html.contains(r#"steps="3""#));
}

#[test]
fn render_active_tab_filters_non_active() {
    let mut jobs = HashMap::new();
    jobs.insert("a".into(), mk_job("active-one", JobStatus::Dispatched));
    jobs.insert("b".into(), mk_job("done-one", JobStatus::Completed));
    let html = render_active_tab(&jobs);
    assert!(html.contains("active-one"));
    assert!(!html.contains("done-one"));
}

#[test]
fn render_active_tab_empty_state() {
    let jobs = HashMap::new();
    let html = render_active_tab(&jobs);
    assert!(html.contains("No active runs"));
}

#[test]
fn render_history_tab_shows_completed_and_failed() {
    let mut jobs = HashMap::new();
    jobs.insert("a".into(), mk_job("active", JobStatus::Dispatched));
    jobs.insert("b".into(), mk_job("done", JobStatus::Completed));
    jobs.insert("c".into(), mk_job("failed", JobStatus::Failed));
    let html = render_history_tab(&jobs);
    assert!(!html.contains(">active<"));
    assert!(html.contains("done"));
    assert!(html.contains("failed"));
}

#[test]
fn render_history_tab_empty() {
    let jobs = HashMap::new();
    let html = render_history_tab(&jobs);
    assert!(html.contains("No run history yet"));
}

#[test]
fn render_pipelines_view_contains_all_tabs() {
    let jobs = HashMap::new();
    let defs = vec![];
    let html = render_pipelines_view(&jobs, &defs);
    assert!(html.contains(r#"id="pipelines-content""#));
    assert!(html.contains("Definitions"));
    assert!(html.contains("Active"));
    assert!(html.contains("History"));
    assert!(html.contains("$tab"));
}

#[test]
fn html_escape_blocks_injection() {
    let s = html_escape("<script>&\"</script>");
    assert!(!s.contains("<script>"));
    assert!(s.contains("&lt;"));
    assert!(s.contains("&amp;"));
    assert!(s.contains("&quot;"));
}

#[test]
fn format_duration_variants() {
    assert_eq!(format_duration(45), "45s");
    assert_eq!(format_duration(90), "1m 30s");
    assert_eq!(format_duration(3700), "1h 1m");
}

/// Deterministic job for fixture capture — no Utc::now so output is stable.
fn mk_fixture_job(slug: &str, status: JobStatus, current: Option<&str>, completed: &[&str]) -> JobRecord {
    // Derive a stable ordering for fixture tests: hash the slug into a minute
    // offset so HashMap iteration + sort_by(updated_at) yields a deterministic
    // order regardless of HashMap hash seed randomization.
    use chrono::TimeZone;
    let offset_mins = slug.bytes().map(|b| b as i64).sum::<i64>() % 60;
    let t = Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5).unwrap()
        + chrono::Duration::minutes(offset_mins);
    JobRecord {
        job_id: format!("j-{slug}"),
        slug: slug.into(),
        agent: "coder".into(),
        branch: "main".into(),
        spec_path: String::new(),
        depends_on: None,
        status,
        worktree: None,
        blocker: None,
        error: None,
        model: None,
        provider: None,
        created_at: t,
        updated_at: t,
        provider_meta: None,
        pipeline: None,
        steps: Some(Pipeline {
            steps: vec![
                PipelineStep { id: "impl".into(), step_type: StepType::Agent, agent: Some("coder".into()), goal: None, after: vec![], model: None, provider: None, command: None, max_retries: None },
                PipelineStep { id: "gate".into(), step_type: StepType::Gate, agent: Some("reviewer".into()), goal: None, after: vec!["impl".into()], model: None, provider: None, command: None, max_retries: None },
            ],
        }),
        current_phase: None,
        completed_steps: completed.iter().map(|s| (*s).into()).collect(),
        current_step: current.map(|s| s.into()),
        vars: None,
        step_retries: HashMap::new(),
    }
}

/// Produce a representative set of rendered HTML fixtures. Tests compare
/// against committed snapshots under `src/view_fixtures/`. To regenerate
/// after an intentional change, set SHIP_UPDATE_FIXTURES=1 when running.
#[test]
fn fixtures_match_snapshots() {
    use std::fs;
    use std::path::PathBuf;

    let defs = vec![
        ("tdd".to_string(), "Test-driven dev loop".to_string(), 3usize),
        ("ship-it".to_string(), String::new(), 0usize),
    ];

    let mut jobs: HashMap<String, JobRecord> = HashMap::new();
    jobs.insert("a".into(), mk_fixture_job("fix-bug", JobStatus::Dispatched, Some("impl"), &[]));
    jobs.insert("b".into(), mk_fixture_job("feat-x", JobStatus::Completed, None, &["impl", "gate"]));
    jobs.insert("c".into(), mk_fixture_job("regress", JobStatus::Failed, Some("gate"), &["impl"]));
    jobs.insert("d".into(), mk_fixture_job("gate-wait", JobStatus::GatePending, Some("gate"), &["impl"]));

    let fixtures: Vec<(&str, String)> = vec![
        ("definitions_tab.html", render_definitions_tab(&defs)),
        ("definitions_tab_empty.html", render_definitions_tab(&[])),
        ("active_tab.html", render_active_tab(&jobs)),
        ("active_tab_empty.html", render_active_tab(&HashMap::new())),
        ("history_tab.html", render_history_tab(&jobs)),
        ("history_tab_empty.html", render_history_tab(&HashMap::new())),
        ("full_view.html", render_pipelines_view(&jobs, &defs)),
    ];

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/view_fixtures");
    let update = std::env::var("SHIP_UPDATE_FIXTURES").is_ok();
    if update {
        fs::create_dir_all(&dir).unwrap();
    }

    for (name, actual) in fixtures {
        let path = dir.join(name);
        if update {
            fs::write(&path, &actual).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("missing fixture {}; regenerate with SHIP_UPDATE_FIXTURES=1", path.display()));
        assert_eq!(
            expected.trim_end(), actual.trim_end(),
            "fixture {} drifted. Regenerate with SHIP_UPDATE_FIXTURES=1 after reviewing.",
            name,
        );
    }
}
