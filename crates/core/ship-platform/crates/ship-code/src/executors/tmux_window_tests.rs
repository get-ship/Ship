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
    assert_eq!(step_log_rel(9, "impl"), ".artifacts/pipeline/10-impl.log");
}

#[test]
fn tmux_window_new_fails_when_tmux_absent_or_windows() {
    let _spawn_guard = crate::executors::real_spawn_guard();
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
    if ship_substrate::env::read_raw(ship_substrate::env::Var::TmuxTests)
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!("skipping: set SHIP_TMUX_TESTS=1 to run");
        return;
    }
    if !tmux_available() {
        eprintln!("skipping: tmux not available");
        return;
    }
    use crate::dispatch_ports::{LaunchProfile, StepInfo};
    let tmp = tempfile::tempdir().unwrap();
    let slug = format!(
        "tmuxwintest-{}",
        &ship_substrate::gen_ulid()[..8].to_lowercase()
    );
    let session = session_name(&slug);

    let make_ctx = || SessionContext {
        pipeline_run_id: "j1".into(),
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
        step: Some(StepInfo {
            index: 0,
            id: "tests".into(),
        }),
        invocation: crate::dispatch_ports::InvocationMode::Pty,
        keepalive: false,
    };

    let exec = TmuxWindowExecutor::new().unwrap();

    // First spawn: creates session + first "tests" window.
    let h1 = exec.spawn(&make_ctx()).await.expect("spawn 1");
    let s1 = h1.inner.downcast_ref::<TmuxWindowState>().unwrap();
    assert!(
        s1.window.starts_with('@'),
        "window must be a tmux @id, got {:?}",
        s1.window
    );
    assert!(window_id_exists(&session, &s1.window));

    // Second spawn in the same session with a window of the same name. The
    // captured window ids must differ and both must be independently alive.
    let h2 = exec.spawn(&make_ctx()).await.expect("spawn 2");
    let s2 = h2.inner.downcast_ref::<TmuxWindowState>().unwrap();
    assert_ne!(
        s1.window, s2.window,
        "duplicate-named windows must have distinct ids"
    );
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
    // Real-tmux integration test: spawns actual tmux sessions, so it's flaky
    // under parallel test load (tmux's fork/init races sibling spawns). Opt in
    // with SHIP_TMUX_TESTS=1, same as the other real-tmux tests.
    if ship_substrate::env::read_raw(ship_substrate::env::Var::TmuxTests)
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!("skipping: set SHIP_TMUX_TESTS=1 to run");
        return;
    }
    let _spawn_guard = crate::executors::real_spawn_guard();
    if !tmux_available() {
        eprintln!("skipping: tmux not available");
        return;
    }
    use crate::dispatch_ports::{LaunchProfile, StepInfo};
    let tmp = tempfile::tempdir().unwrap();
    let slug = format!(
        "tmuxwintest-{}",
        &ship_substrate::gen_ulid()[..8].to_lowercase()
    );
    let session = session_name(&slug);

    let make_ctx = || SessionContext {
        pipeline_run_id: "j1".into(),
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
        step: Some(StepInfo {
            index: 0,
            id: "tests".into(),
        }),
        invocation: crate::dispatch_ports::InvocationMode::Pty,
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

// ── wait_for_ready / PaneReader tests ──────────────────────────────────────
//
// The harness-launcher polls a tmux pane for the harness's
// `ready_pattern()` before firing the autostart prompt — closes the
// autostart race in c-2026-07-05T12-36-00-000Z-b2c3d4e5. These tests
// exercise the polling behavior with a fake PaneReader (no tmux
// server, no real subprocesses). The behavior under test is the
// poll/timeout/retry loop — NOT tmux itself.

/// Fake pane reader: each call returns the next string from `buf`; once
/// exhausted, returns the last element forever (so a "ready" state
/// sticks without infinite growth).
struct FakePaneReader {
    buf: Vec<String>,
    idx: usize,
}

impl PaneReader for FakePaneReader {
    fn read(&mut self, _window_target: &str) -> String {
        let i = self.idx.min(self.buf.len().saturating_sub(1));
        self.idx += 1;
        self.buf[i].clone()
    }
}

#[test]
fn wait_for_ready_returns_immediately_when_first_capture_matches() {
    let mut reader = FakePaneReader {
        buf: vec!["claude › hello\nbypass permissions on\n".to_string()],
        idx: 0,
    };
    let started = std::time::Instant::now();
    let result = wait_for_ready(
        "ship-foo:@1",
        "bypass permissions on",
        std::time::Duration::from_secs(5),
        std::time::Duration::from_millis(50),
        &mut reader,
    );
    assert!(result.is_ok(), "expected ready, got {result:?}");
    // First-poll success must NOT have burned anywhere near the timeout.
    assert!(started.elapsed() < std::time::Duration::from_millis(500));
}

#[test]
fn wait_for_ready_retries_until_pattern_appears() {
    // Three "still loading" frames, then the ready frame on the 4th call.
    // Proves the poll retries rather than failing on the first probe
    // (c-2026-07-05T12-36-00-000Z-b2c3d4e5 headline requirement).
    let mut reader = FakePaneReader {
        buf: vec![
            "loading…".to_string(),
            "still loading…".to_string(),
            "almost there…".to_string(),
            "› ready\n".to_string(),
        ],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-foo:@1",
        "› ready",
        std::time::Duration::from_secs(2),
        std::time::Duration::from_millis(10),
        &mut reader,
    );
    assert!(result.is_ok(), "expected ready on retry, got {result:?}");
}

#[test]
fn wait_for_ready_times_out_with_clear_error() {
    // Pane content never matches the ready pattern. The error MUST
    // include the ready pattern (so the operator can see what we were
    // looking for) and a slice of the last-seen pane (so they can
    // diagnose the harness state).
    let mut reader = FakePaneReader {
        buf: vec!["\n[some other output]\n".to_string()],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-foo:@1",
        "omp's-pi-glyph",
        std::time::Duration::from_millis(100),
        std::time::Duration::from_millis(20),
        &mut reader,
    );
    let err = result.expect_err("must time out");
    assert!(
        err.contains("omp's-pi-glyph"),
        "err should name the pattern, got: {err}"
    );
    assert!(
        err.contains("never reached ready pattern"),
        "err should explain the failure, got: {err}"
    );
    // The duration in the error gives the operator a precise budget read.
    assert!(
        err.contains("100ms") || err.contains("0.1"),
        "err should include the timeout, got: {err}"
    );
}

#[test]
fn wait_for_ready_truncates_very_long_pane_text() {
    // A 5KB pane that never matches — the error must not balloon.
    let huge = "x".repeat(5_000);
    let mut reader = FakePaneReader {
        buf: vec![huge],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-foo:@1",
        "never-there",
        std::time::Duration::from_millis(50),
        std::time::Duration::from_millis(20),
        &mut reader,
    );
    let err = result.expect_err("must time out");
    // The whole error must be small enough to log without truncation in
    // a typical operator's terminal. The 200-byte tail is the cap; the
    // message framing is ~100 bytes, so the whole err stays well under
    // 1KB. A simple sanity check:
    assert!(
        err.len() < 1_000,
        "error must be small, got len={} err={:?}",
        err.len(),
        err
    );
}

#[test]
fn wait_for_ready_sticks_when_pattern_already_in_pane() {
    // The harness's earlier output already contains the ready marker
    // (e.g. claude prints "bypass permissions on" once, and later the
    // same pane is captured again). The poll must succeed on the very
    // first read.
    let mut reader = FakePaneReader {
        buf: vec!["...lots of startup text...\nbypass permissions on\n".to_string()],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-foo:@1",
        "bypass permissions on",
        std::time::Duration::from_millis(500),
        std::time::Duration::from_millis(50),
        &mut reader,
    );
    assert!(result.is_ok());
}

/// Per-harness ready pattern — omp's pi glyph. Closes the
/// "omp dispatch time out at 60s" race in c-2026-07-05T12-36-00-000Z-b2c3d4e5
/// (the old OmpAdapter returned `PiAdapter::ready_pattern()`, which is `› `,
/// not the `π ` glyph the omp TUI actually renders). The fake pane emits
/// a frame containing `π ` and the poll must accept it; the prior behavior
/// (matching on `› `) would have refused and timed out.
#[test]
fn wait_for_ready_matches_omp_pi_glyph_ready_pattern() {
    use ship_harnesses::harnesses::omp::OmpAdapter;
    use harness_adapter::HarnessAdapter;

    let mut reader = FakePaneReader {
        buf: vec![
            "loading…\n".to_string(),
            "still loading…\n".to_string(),
            format!("\u{03C0} \n"),
        ],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-omp-foo:@1",
        OmpAdapter.ready_pattern(),
        std::time::Duration::from_secs(2),
        std::time::Duration::from_millis(10),
        &mut reader,
    );
    assert!(
        result.is_ok(),
        "wait_for_ready must accept the omp-ready pattern \u{03C0} (c-2026-07-05T12-36-00)"
    );
}

/// Per-harness ready pattern — claude's bypass-permissions marker, still
/// matched correctly. The omp fix must not regress the existing harnesses:
/// pinning the per-harness discipline on both ends of the registry proves
/// claude's `bypass permissions on` marker still drives the poll.
#[test]
fn wait_for_ready_matches_claude_bypass_permissions_marker() {
    use ship_harnesses::harnesses::claude::ClaudeAdapter;
    use harness_adapter::HarnessAdapter;

    let mut reader = FakePaneReader {
        buf: vec![format!(
            "claude \u{203A} hello\n{}\n",
            ClaudeAdapter.ready_pattern()
        )],
        idx: 0,
    };
    let result = wait_for_ready(
        "ship-claude-foo:@1",
        ClaudeAdapter.ready_pattern(),
        std::time::Duration::from_secs(2),
        std::time::Duration::from_millis(10),
        &mut reader,
    );
    assert!(result.is_ok(), "claude path unchanged: bypass permissions on still drives the poll");
}

// ── Autostart content selection ─────────────────────────────────────────────
//
// The live-path bug in c-2026-07-05T12-36-00-000Z-b2c3d4e5: after the
// readiness poll, the interactive path sent the adapter's STATIC
// `autostart_input()` ("go" for pi/omp) instead of the dispatch's actual
// content. `select_autostart` is the fix's decision seam: pointer to the
// spec source file first (typing a reference into a TUI is robust; typing
// a multi-line body fragments on every newline), then a single-line
// prompt verbatim, then a hard error for undeliverable multi-line
// content, then the adapter's static kick as last resort.

#[test]
fn select_autostart_prefers_spec_pointer_when_source_file_exists() {
    let got = select_autostart(
        "/helm/data/missions/m-x/scratch/spec.md",
        true,
        Some("# Spec\nmulti\nline\nbody"),
        "go",
    )
    .expect("pointer is deliverable");
    let msg = got.expect("must send something");
    assert!(
        msg.contains("/helm/data/missions/m-x/scratch/spec.md"),
        "pointer must name the spec source path, got: {msg}"
    );
    assert!(
        !msg.contains('\n'),
        "the pointer must be a single line — newlines submit mid-prompt in a TUI"
    );
    assert_ne!(msg, "go", "the static kick must not shadow real content");
}

#[test]
fn select_autostart_sends_single_line_prompt_verbatim_without_pointer() {
    let got = select_autostart("", false, Some("fix the failing test\n"), "go")
        .expect("single-line prompt is deliverable");
    assert_eq!(
        got.as_deref(),
        Some("fix the failing test"),
        "a one-line prompt is typed directly (trailing newline trimmed)"
    );
}

#[test]
fn select_autostart_errors_on_multiline_prompt_with_no_pointer() {
    let err = select_autostart("", false, Some("line one\nline two"), "go")
        .expect_err("multi-line content cannot be typed into a TUI pane");
    assert!(
        err.contains("multi-line"),
        "error must explain the fragmentation hazard, got: {err}"
    );
    assert!(
        err.contains("source_spec_path") || err.contains("spec"),
        "error must tell the caller the fix (pass the spec source path), got: {err}"
    );
}

#[test]
fn select_autostart_missing_spec_file_falls_through_to_prompt_rules() {
    // Path present but the file is gone (typo, moved scratch): do NOT
    // point the agent at a dead file — fall through to the prompt rules.
    let got = select_autostart("/gone/spec.md", false, Some("one line prompt"), "go")
        .expect("single-line fallback");
    assert_eq!(got.as_deref(), Some("one line prompt"));

    let err = select_autostart("/gone/spec.md", false, Some("multi\nline"), "go")
        .expect_err("dead pointer + multi-line prompt is undeliverable");
    assert!(err.contains("/gone/spec.md"), "error should name the missing file: {err}");
}

#[test]
fn select_autostart_falls_back_to_adapter_static_kick() {
    let got = select_autostart("", false, None, "go").expect("fallback is fine");
    assert_eq!(
        got.as_deref(),
        Some("go"),
        "no content anywhere → the adapter's static kick (pre-fix behavior) survives"
    );
    let none = select_autostart("", false, None, "").expect("nothing to send");
    assert!(none.is_none(), "no content and no kick → send nothing");
}

// ── Step-log gating (.artifacts is retired for bare dispatches) ────────────
//
// The executor unconditionally created `.artifacts/pipeline/` even for
// bare interactive dispatches that never tee a log. Only headless
// pipeline steps get a step log; interactive sessions create nothing.

#[test]
fn prepare_step_logging_creates_dir_and_path_for_headless_steps() {
    let tmp = tempfile::tempdir().unwrap();
    let log_rel = prepare_step_logging(tmp.path(), false, 0, "smoke")
        .expect("headless logging setup succeeds");
    assert_eq!(log_rel.as_deref(), Some(".artifacts/pipeline/01-smoke.log"));
    assert!(
        tmp.path().join(".artifacts/pipeline").is_dir(),
        "headless steps need the log dir so tee doesn't race"
    );
}

#[test]
fn prepare_step_logging_creates_nothing_for_interactive_sessions() {
    let tmp = tempfile::tempdir().unwrap();
    let log_rel = prepare_step_logging(tmp.path(), true, 0, "smoke")
        .expect("interactive setup is a no-op");
    assert!(log_rel.is_none(), "interactive sessions have no step log");
    assert!(
        !tmp.path().join(".artifacts").exists(),
        "a bare interactive dispatch must not create the retired .artifacts dir"
    );
}
