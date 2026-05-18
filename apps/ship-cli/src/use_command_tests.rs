use super::{LaunchOpts, build_dispatch_body, should_launch};
use std::path::PathBuf;
use tempfile::TempDir;

fn opts(agent: &str) -> LaunchOpts {
    LaunchOpts {
        agent: agent.to_string(),
        path: None,
        launch: false,
        auto: false,
        resume: false,
        prompt: None,
        spec: None,
        harness: None,
        model: None,
    }
}

#[test]
fn no_launch_flags_means_no_dispatch() {
    assert!(!should_launch(&opts("rust-lane")));
}

#[test]
fn explicit_launch_triggers_dispatch() {
    let mut o = opts("rust-lane");
    o.launch = true;
    assert!(should_launch(&o));
}

#[test]
fn auto_implies_launch() {
    let mut o = opts("rust-lane");
    o.auto = true;
    assert!(should_launch(&o));
}

#[test]
fn resume_implies_launch() {
    let mut o = opts("rust-lane");
    o.resume = true;
    assert!(should_launch(&o));
}

#[test]
fn prompt_implies_launch() {
    let mut o = opts("rust-lane");
    o.prompt = Some("hi".into());
    assert!(should_launch(&o));
}

#[test]
fn spec_implies_launch() {
    let mut o = opts("rust-lane");
    o.spec = Some(PathBuf::from("./x.md"));
    assert!(should_launch(&o));
}

#[test]
fn path_alone_does_not_imply_launch() {
    let mut o = opts("rust-lane");
    o.path = Some(PathBuf::from("/tmp/proj"));
    assert!(!should_launch(&o));
}

#[test]
fn body_for_bare_launch_sets_interactive_defaults() {
    let mut o = opts("rust-lane");
    o.launch = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["agent"], "rust-lane");
    assert_eq!(body["auto"], false);
    assert_eq!(body["resume"], false);
    assert_eq!(body["workspace"], "proj");
    assert!(body.get("prompt").is_none());
    assert!(body.get("spec").is_none());
    assert!(body.get("harness").is_none());
    assert!(body.get("model").is_none());
}

#[test]
fn body_for_auto_with_prompt() {
    let mut o = opts("rust-lane");
    o.auto = true;
    o.prompt = Some("hi".into());
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/x")).unwrap();
    assert_eq!(body["auto"], true);
    assert_eq!(body["prompt"], "hi");
}

#[test]
fn body_for_spec_inlines_file_content() {
    let tmp = TempDir::new().unwrap();
    let spec_path = tmp.path().join("test.md");
    std::fs::write(&spec_path, "# my job\n\nDo the thing.\n").unwrap();

    let mut o = opts("rust-lane");
    o.launch = true;
    o.spec = Some(spec_path.clone());
    let body = build_dispatch_body(&o, tmp.path()).unwrap();
    assert_eq!(body["spec"], "# my job\n\nDo the thing.\n");
    assert!(body["source_spec_path"].as_str().unwrap().ends_with("test.md"));
}

#[test]
fn body_for_resume_sets_flag() {
    let mut o = opts("rust-lane");
    o.resume = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/x")).unwrap();
    assert_eq!(body["resume"], true);
}

#[test]
fn body_threads_harness_and_model() {
    let mut o = opts("rust-lane");
    o.launch = true;
    o.harness = Some("codex".into());
    o.model = Some("sonnet".into());
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/x")).unwrap();
    assert_eq!(body["harness"], "codex");
    assert_eq!(body["model"], "sonnet");
}

#[test]
fn body_rejects_prompt_and_spec_together() {
    let tmp = TempDir::new().unwrap();
    let spec_path = tmp.path().join("y.md");
    std::fs::write(&spec_path, "x").unwrap();

    let mut o = opts("rust-lane");
    o.launch = true;
    o.prompt = Some("x".into());
    o.spec = Some(spec_path);
    let err = build_dispatch_body(&o, tmp.path()).unwrap_err();
    assert!(err.to_string().contains("mutually exclusive"));
}

#[test]
fn clap_rejects_prompt_and_spec_together() {
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "--prompt", "x", "--spec", "./y.md",
    ])
    .err()
    .expect("clap must reject --prompt + --spec");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("conflicts"),
        "expected clap conflicts error, got: {msg}"
    );
}

#[test]
fn clap_accepts_launch_flags() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "--auto", "--prompt", "go",
        "--harness", "codex", "--model", "sonnet",
    ])
    .expect("clap parses launch flags");
    match cli.command {
        Some(crate::cli::Commands::Use {
            agent_id,
            auto,
            prompt,
            harness,
            model,
            ..
        }) => {
            assert_eq!(agent_id, "rust-lane");
            assert!(auto);
            assert_eq!(prompt.as_deref(), Some("go"));
            assert_eq!(harness.as_deref(), Some("codex"));
            assert_eq!(model.as_deref(), Some("sonnet"));
        }
        _ => panic!("expected Use command"),
    }
}

#[test]
fn body_missing_spec_file_errors() {
    let mut o = opts("rust-lane");
    o.launch = true;
    o.spec = Some(PathBuf::from("/nonexistent/spec.md"));
    let err = build_dispatch_body(&o, &PathBuf::from("/tmp")).unwrap_err();
    assert!(err.to_string().contains("cannot read spec file"));
}
