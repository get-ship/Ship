use super::*;

#[test]
fn session_name_sanitizes_dots_and_colons() {
    let name = session_name("ab.cd:ef12345");
    assert_eq!(name, "ship-ab_cd_ef12345");
    assert!(!name.contains('.'));
    assert!(!name.contains(':'));
}

#[test]
fn session_name_uses_full_slug_no_truncation() {
    // Regression: two slugs sharing an 8-char prefix used to collide under
    // the old `short_session_name`. They must now map to distinct sessions.
    let a = session_name("tdd-01kp9abd");
    let b = session_name("tdd-01kp9atk");
    assert_ne!(a, b, "full-slug session names must not collide");
    assert_eq!(a, "ship-tdd-01kp9abd");
    assert_eq!(b, "ship-tdd-01kp9atk");
}

#[test]
fn step_log_rel_is_zero_padded() {
    assert_eq!(
        step_log_rel(0, "test-writer"),
        ".artifacts/pipeline/01-test-writer.log"
    );
    assert_eq!(
        step_log_rel(9, "impl"),
        ".artifacts/pipeline/10-impl.log"
    );
}

#[test]
fn tmux_window_new_fails_when_tmux_absent_or_windows() {
    let result = TmuxWindowExecutor::new();
    if cfg!(windows) {
        assert!(result.is_err(), "TmuxWindowExecutor must fail on Windows");
    } else if !tmux_available() {
        assert!(result.is_err(), "must fail when tmux is absent");
    } else {
        assert!(result.is_ok(), "must succeed when tmux is present");
    }
}

#[tokio::test]
async fn tmux_window_spawn_captures_window_id_and_survives_name_collision() {
    // Real-tmux integration test. Flaky under parallel test execution
    // because tmux's session init races with sibling tests. Gate with
    // SHIP_TMUX_TESTS=1 to run locally; the default test run skips it.
    if ship_substrate::env::read_raw(ship_substrate::env::Var::TmuxTests).ok().as_deref() != Some("1") {
        eprintln!("skipping: set SHIP_TMUX_TESTS=1 to run");
        return;
    }
    if !tmux_available() {
        eprintln!("skipping: tmux not available");
        return;
    }
    use ship_platform::services::dispatch_ports::{LaunchProfile, StepInfo};
    let tmp = tempfile::tempdir().unwrap();
    let slug = format!("tmuxwintest-{}", &ship_substrate::gen_ulid()[..8].to_lowercase());
    let session = session_name(&slug);

    let make_ctx = || SessionContext {
        job_id: "j1".into(),
        slug: slug.clone(),
        agent: "noop".into(),
        branch: "job/test".into(),
        spec_path: "spec.md".into(),
        spec_content: None,
        work_dir: tmp.path().to_path_buf(),
        env: Default::default(),
        exit_tx: None,
        launch: LaunchProfile {
            harness: Some("claude".into()),
            ..Default::default()
        },
        step: Some(StepInfo { index: 0, id: "tests".into() }),
        invocation: ship_platform::services::dispatch_ports::InvocationMode::Pty,
        keepalive: false,
    };

    let exec = TmuxWindowExecutor::new().unwrap();

    // First spawn: creates session + first "tests" window.
    let h1 = exec.spawn(&make_ctx()).await.expect("spawn 1");
    let s1 = h1.inner.downcast_ref::<TmuxWindowState>().unwrap();
    assert!(s1.window.starts_with('@'), "window must be a tmux @id, got {:?}", s1.window);
    assert!(window_id_exists(&session, &s1.window));

    // Second spawn in the same session with a window of the same name. The
    // captured window ids must differ and both must be independently alive.
    let h2 = exec.spawn(&make_ctx()).await.expect("spawn 2");
    let s2 = h2.inner.downcast_ref::<TmuxWindowState>().unwrap();
    assert_ne!(s1.window, s2.window, "duplicate-named windows must have distinct ids");
    assert!(window_id_exists(&session, &s1.window));
    assert!(window_id_exists(&session, &s2.window));

    // Cleanup.
    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .status();
    drop(h1);
    drop(h2);
}

#[tokio::test]
async fn resume_handle_refuses_ambiguous_name() {
    if !tmux_available() {
        eprintln!("skipping: tmux not available");
        return;
    }
    use ship_platform::services::dispatch_ports::{LaunchProfile, StepInfo};
    let tmp = tempfile::tempdir().unwrap();
    let slug = format!("tmuxwintest-{}", &ship_substrate::gen_ulid()[..8].to_lowercase());
    let session = session_name(&slug);

    let make_ctx = || SessionContext {
        job_id: "j1".into(),
        slug: slug.clone(),
        agent: "noop".into(),
        branch: "job/test".into(),
        spec_path: "spec.md".into(),
        spec_content: None,
        work_dir: tmp.path().to_path_buf(),
        env: Default::default(),
        exit_tx: None,
        launch: LaunchProfile {
            harness: Some("claude".into()),
            ..Default::default()
        },
        step: Some(StepInfo { index: 0, id: "tests".into() }),
        invocation: ship_platform::services::dispatch_ports::InvocationMode::Pty,
        keepalive: false,
    };

    let exec = TmuxWindowExecutor::new().unwrap();
    let h1 = exec.spawn(&make_ctx()).await.expect("spawn 1");
    let h2 = exec.spawn(&make_ctx()).await.expect("spawn 2");
    let w1 = h1
        .inner
        .downcast_ref::<TmuxWindowState>()
        .expect("h1 state")
        .window
        .clone();
    let w2 = h2
        .inner
        .downcast_ref::<TmuxWindowState>()
        .expect("h2 state")
        .window
        .clone();

    // Defeat tmux's two renaming mechanisms:
    //   - `automatic-rename` (default on): rename based on the running
    //     process — replaces our `-n tests` with e.g. `zsh` once the
    //     shell starts.
    //   - `allow-rename` (default on): honor shell-emitted OSC 0
    //     escape codes that set the window title.
    //
    // Under workspace-test parallelism the shells in these windows
    // occasionally race-rename one of the two windows before we get
    // to the assertion, making find_unique_window_by_name see exactly
    // one "tests" window and return Some(...). Then the test fails
    // ("ambiguous name must not resume"). Pinning both options off
    // per-window AND re-renaming right before the query gives the
    // assertion deterministic state.
    let pin = |w: &str| {
        for opt in ["automatic-rename", "allow-rename"] {
            let _ = std::process::Command::new("tmux")
                .args([
                    "set-window-option",
                    "-t",
                    &format!("{session}:{w}"),
                    opt,
                    "off",
                ])
                .status();
        }
        let _ = std::process::Command::new("tmux")
            .args(["rename-window", "-t", &format!("{session}:{w}"), "tests"])
            .status();
    };
    pin(&w1);
    pin(&w2);
    // Re-pin immediately before the assertion to close any TOCTOU
    // between option-set and the list-windows query.
    pin(&w1);
    pin(&w2);

    // Two windows named "tests" — rehydration must refuse rather than guess.
    let resumed = exec.resume_handle(&slug, None, Some("tests")).await;
    assert!(resumed.is_none(), "ambiguous name must not resume");

    let _ = std::process::Command::new("tmux")
        .args(["kill-session", "-t", &session])
        .status();
    drop(h1);
    drop(h2);
}
