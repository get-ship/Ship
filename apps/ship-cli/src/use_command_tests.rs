use super::{LaunchOpts, build_dispatch_body, should_launch};
use std::path::PathBuf;
use tempfile::TempDir;

fn opts(agent: &str) -> LaunchOpts {
    LaunchOpts {
        agent: agent.to_string(),
        path: None,
        launch: false,
        resume: false,
        prompt: None,
        spec: None,
        harness: None,
        model: None,
        detach: false,
        transport: None,
        keepalive: false,
        no_keepalive: false,
    }
}

// ── should_launch contract ─────────────────────────────────────────

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
fn detach_implies_launch() {
    let mut o = opts("rust-lane");
    o.detach = true;
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
fn transport_implies_launch() {
    let mut o = opts("rust-lane");
    o.transport = Some("print".into());
    assert!(should_launch(&o));
}

#[test]
fn keepalive_implies_launch() {
    let mut o = opts("rust-lane");
    o.keepalive = true;
    assert!(should_launch(&o));
}

#[test]
fn path_alone_does_not_imply_launch() {
    let mut o = opts("rust-lane");
    o.path = Some(PathBuf::from("/tmp/proj"));
    assert!(!should_launch(&o));
}

// ── build_dispatch_body / new orthogonal axes ──────────────────────

#[test]
fn bare_launch_defaults_to_foreground_chat() {
    // No flags: chat shape — open_tab=true, invocation=pty, keepalive=true.
    let mut o = opts("rust-lane");
    o.launch = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["agent"], "rust-lane");
    assert_eq!(body["open_tab"], true);
    assert_eq!(body["invocation"], "pty");
    assert_eq!(body["keepalive"], true);
    assert_eq!(body["resume"], false);
    assert_eq!(body["workspace"], "proj");
}

#[test]
fn detach_alone_is_background_batch() {
    // --detach without --keepalive: PTY one-shot. No tab, no keepalive,
    // subscription-billed. The canonical "fire and forget" combination.
    let mut o = opts("rust-lane");
    o.detach = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["open_tab"], false);
    assert_eq!(body["invocation"], "pty");
    assert_eq!(body["keepalive"], false);
}

#[test]
fn detach_with_keepalive_is_background_worker() {
    // --detach --keepalive (-di): background subscription worker. Push
    // turns later via the daemon API. Old --auto-and-keepalive mutex
    // is gone; this combination is canonical now.
    let mut o = opts("rust-lane");
    o.detach = true;
    o.keepalive = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["open_tab"], false);
    assert_eq!(body["keepalive"], true);
}

#[test]
fn detach_with_transport_print_is_api_billed() {
    // --detach --transport print: API-billed claude -p batch. The
    // daemon returns a billing_warning in the response.
    let mut o = opts("rust-lane");
    o.detach = true;
    o.transport = Some("print".into());
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["open_tab"], false);
    assert_eq!(body["invocation"], "print");
    assert_eq!(body["keepalive"], false);
}

#[test]
fn no_keepalive_overrides_default() {
    // Foreground default is keepalive=true. --no-keepalive forces false
    // for the rare "open a chat tab but exit on first stop" case.
    let mut o = opts("rust-lane");
    o.launch = true;
    o.no_keepalive = true;
    let body = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap();
    assert_eq!(body["open_tab"], true);
    assert_eq!(body["keepalive"], false);
}

#[test]
fn invalid_transport_value_errors() {
    let mut o = opts("rust-lane");
    o.launch = true;
    o.transport = Some("bogus".into());
    let err = build_dispatch_body(&o, &PathBuf::from("/tmp/proj")).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("--transport") && msg.contains("pty, print, stream"),
        "error must list valid transports, got: {msg}"
    );
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

// ── clap CLI surface ────────────────────────────────────────────────

#[test]
fn clap_rejects_prompt_and_spec_together() {
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "--prompt", "x", "--spec", "./y.md",
    ])
    .expect_err("clap must reject --prompt + --spec");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("conflicts"),
        "expected clap conflicts error, got: {msg}"
    );
}

#[test]
fn clap_accepts_detach_short_flag() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "-d",
    ])
    .expect("clap parses -d");
    match cli.command {
        Some(crate::cli::Commands::Use { detach, .. }) => assert!(detach),
        _ => panic!("expected Use command"),
    }
}

#[test]
fn clap_accepts_keepalive_short_flag() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "-i",
    ])
    .expect("clap parses -i (keepalive)");
    match cli.command {
        Some(crate::cli::Commands::Use { keepalive, .. }) => assert!(keepalive),
        _ => panic!("expected Use command"),
    }
}

#[test]
fn clap_accepts_combined_short_flags_di() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "-di",
    ])
    .expect("clap parses combined -di");
    match cli.command {
        Some(crate::cli::Commands::Use { detach, keepalive, .. }) => {
            assert!(detach);
            assert!(keepalive);
        }
        _ => panic!("expected Use command"),
    }
}

#[test]
fn clap_accepts_transport_flag() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "-d", "--transport", "print",
    ])
    .expect("clap parses --transport");
    match cli.command {
        Some(crate::cli::Commands::Use { detach, transport, .. }) => {
            assert!(detach);
            assert_eq!(transport.as_deref(), Some("print"));
        }
        _ => panic!("expected Use command"),
    }
}

#[test]
fn clap_keepalive_conflicts_with_no_keepalive() {
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "--keepalive", "--no-keepalive",
    ])
    .expect_err("clap must reject --keepalive + --no-keepalive");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("conflicts"),
        "expected conflict error, got: {msg}"
    );
}

#[test]
fn clap_rejects_old_auto_flag() {
    // --auto is gone (the old conflation). Reject explicitly so
    // operators see the change rather than silently dropping the flag.
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from([
        "ship", "use", "rust-lane", "--auto",
    ])
    .expect_err("--auto must be rejected (renamed to -d/--detach + invocation)");
    let msg = err.to_string();
    assert!(
        msg.contains("auto") || msg.contains("unexpected"),
        "expected auto-flag rejection, got: {msg}"
    );
}

#[test]
fn body_missing_spec_file_errors() {
    let mut o = opts("rust-lane");
    o.launch = true;
    o.spec = Some(PathBuf::from("/nonexistent/spec.md"));
    let err = build_dispatch_body(&o, &PathBuf::from("/tmp")).unwrap_err();
    assert!(err.to_string().contains("cannot read spec file"));
}
