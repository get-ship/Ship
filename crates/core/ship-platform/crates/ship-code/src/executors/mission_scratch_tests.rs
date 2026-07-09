//! Tests for the `SHIP_MISSION_SCRATCH` executor seam — both
//! `merge_session_env_into_launch` (process) and `emit_tmux_env`
//! (tmux_window) prove that the helm-side mission scratch path reaches
//! the spawned harness via the standard env contract.
//!
//! These are unit tests against the helper functions — they DON'T spawn
//! an actual child process or a tmux server, so they run on every
//! `cargo test` without `SHIP_TMUX_TESTS`.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use parking_lot::Mutex;

use crate::dispatch_ports::{InvocationMode, LaunchCommand, LaunchProfile, SessionContext};

use super::process::merge_session_env_into_launch;
use super::tmux_window::emit_tmux_env;

/// Build a `SessionContext` whose `env` carries exactly one entry:
/// `SHIP_MISSION_SCRATCH=<value>`. The other fields are filled with
/// the same defaults the dispatch builder uses.
fn ctx_with_mission_scratch(value: &str) -> SessionContext {
    let mut env = HashMap::new();
    env.insert("SHIP_MISSION_SCRATCH".to_string(), value.to_string());
    SessionContext {
        pipeline_run_id: "mission-scratch".into(),
        slug: "mission-slug".into(),
        agent: "agent-under-test".into(),
        branch: "main".into(),
        spec_path: ".artifacts/job-spec.md".into(),
        spec_content: None,
        work_dir: PathBuf::from("/tmp/land-hardening-does-not-run"),
        env,
        launch: LaunchProfile::default(),
        exit_tx: None,
        step: None,
        invocation: InvocationMode::Print,
        keepalive: false,
        // Test fixture — no per-call fork source.
        repo_path: None,
        base_ref: None,
    }
}

fn empty_launch() -> LaunchCommand {
    LaunchCommand {
        bin: "claude".into(),
        args: vec![],
        env: HashMap::new(),
    }
}

// ── process side ──────────────────────────────────────────────────────────

#[test]
fn merge_session_env_injects_ship_mission_scratch_into_launch_env() {
    let ctx = ctx_with_mission_scratch("/abs/helm/data/missions/m-scratch");
    let mut launch = empty_launch();

    merge_session_env_into_launch(&ctx, &mut launch);

    assert_eq!(
        launch.env.get("SHIP_MISSION_SCRATCH").map(String::as_str),
        Some("/abs/helm/data/missions/m-scratch"),
        "SHIP_MISSION_SCRATCH must reach launch.env verbatim — the seam that \
         puts the helm mission scratch path on the harness's environment."
    );
}

#[test]
fn merge_session_env_preserves_adapter_supplied_entries() {
    // Adapter-written entries (e.g. PATH, HOME) must win over a caller-
    // supplied value with the same name — the helper is "fold in only if
    // missing", never "clobber".
    let ctx = ctx_with_mission_scratch("/abs/scratch");
    let mut launch = empty_launch();
    launch.env.insert("PATH".into(), "/usr/bin".into());

    merge_session_env_into_launch(&ctx, &mut launch);

    assert_eq!(launch.env.get("PATH").map(String::as_str), Some("/usr/bin"));
    assert_eq!(
        launch.env.get("SHIP_MISSION_SCRATCH").map(String::as_str),
        Some("/abs/scratch")
    );
}

#[test]
fn merge_session_env_is_a_no_op_on_empty_ctx_env() {
    // A ctx with no entries must NOT clobber the launch env.
    let ctx = SessionContext {
        env: HashMap::new(),
        ..ctx_with_mission_scratch("/irrelevant")
    };
    let mut launch = empty_launch();
    launch.env.insert("EXISTING".into(), "keep".into());

    merge_session_env_into_launch(&ctx, &mut launch);

    assert_eq!(launch.env.get("EXISTING").map(String::as_str), Some("keep"));
    assert!(launch.env.get("SHIP_MISSION_SCRATCH").is_none());
}

#[test]
fn merge_session_env_propagates_multiple_vars() {
    // Non-SHIP_MISSION_SCRATCH entries must round-trip too (the seam is
    // for any caller-supplied env var, not just the mission one).
    let mut ctx = ctx_with_mission_scratch("/abs/scratch");
    ctx.env.insert("CARGO_TERM_COLOR".into(), "always".into());
    let mut launch = empty_launch();

    merge_session_env_into_launch(&ctx, &mut launch);

    assert_eq!(
        launch.env.get("SHIP_MISSION_SCRATCH").map(String::as_str),
        Some("/abs/scratch")
    );
    assert_eq!(
        launch.env.get("CARGO_TERM_COLOR").map(String::as_str),
        Some("always")
    );
}

// ── tmux side ──────────────────────────────────────────────────────────────
//
// The tmux executor surfaces the same `ctx.env` map to the harness via
// `tmux set-environment`. `emit_tmux_env` is the testable form —
// production passes a closure that shells out to `tmux`; here we inject
// a recording closure so the test can assert the exact argv.

#[test]
fn emit_tmux_env_emits_set_environment_for_each_entry() {
    let recorded: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());
    let mut env = HashMap::new();
    env.insert(
        "SHIP_MISSION_SCRATCH".to_string(),
        "/abs/missions/m-scratch".to_string(),
    );
    env.insert("CARGO_TERM_COLOR".to_string(), "always".to_string());

    emit_tmux_env("ship-test", &env, |argv| {
        recorded.lock().push(argv.iter().map(|s| s.to_string()).collect());
        Ok(())
    })
    .expect("emit should succeed with a recording runner");

    let calls = recorded.lock().clone();
    assert_eq!(calls.len(), 2, "one tmux call per env entry");
    for call in &calls {
        assert_eq!(call[0], "set-environment");
        assert_eq!(call[1], "-t");
        assert_eq!(call[2], "ship-test");
        assert!(
            call[3] == "SHIP_MISSION_SCRATCH" || call[3] == "CARGO_TERM_COLOR",
            "env var name: {}",
            call[3]
        );
    }
    let mission_call = calls
        .iter()
        .find(|c| c[3] == "SHIP_MISSION_SCRATCH")
        .expect("SHIP_MISSION_SCRATCH call present");
    assert_eq!(mission_call[4], "/abs/missions/m-scratch");
}

#[test]
fn emit_tmux_env_short_circuits_on_runner_error() {
    // A failed tmux call must surface an error rather than panicking.
    // The helper takes any `IntoIterator` and stops on the first failure —
    // pass a `Vec<(K, V)>` so iteration order is insertion order
    // (HashMap would randomize). Order: A_RUNS (ok), BOOM (err),
    // Z_NEVER (must NOT be reached).
    let recorded: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let env: Vec<(String, String)> = vec![
        ("A_RUNS".to_string(), "first".to_string()),
        ("BOOM".to_string(), "fails".to_string()),
        ("Z_NEVER".to_string(), "skipped".to_string()),
    ];

    let err = emit_tmux_env("ship-test", env.clone(), |argv| {
        let name = argv[3];
        if name == "BOOM" {
            return Err("tmux exited 1".to_string());
        }
        recorded.lock().push(name.to_string());
        Ok(())
    })
    .expect_err("BOOM must fail");

    assert!(err.contains("BOOM"), "error names the failing var; got: {err}");
    assert!(
        err.contains("tmux exited 1"),
        "error carries the cause; got: {err}"
    );
    let emitted = recorded.lock().clone();
    assert_eq!(
        emitted,
        vec!["A_RUNS".to_string()],
    );
}

#[test]
fn emit_tmux_env_with_empty_env_is_a_noop() {
    let recorded: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());
    let env: HashMap<String, String> = HashMap::new();

    emit_tmux_env("ship-test", &env, |argv| {
        recorded.lock().push(argv.iter().map(|s| s.to_string()).collect());
        Ok(())
    })
    .expect("empty env must be a no-op success");

    assert!(recorded.lock().is_empty(), "no tmux call for an empty env");
}
