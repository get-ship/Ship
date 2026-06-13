//! Unit tests for the host's arg-parsing, target-routing, and connect-url
//! composition. The Tauri `run()` entry is not testable in this harness (it
//! spins up a real webview); the Tauri-command surface is exercised by the
//! per-module `tests` blocks in `daemon` and `launcher`. The two functions in
//! `lib.rs` that are testable as pure data — argument parsing and URL
//! composition — live here.
//!
//! Included as `#[path]` from `lib.rs` so the test names render against the
//! crate's host (not `lib_tests::tests::*`). Each `#[test]` exercises one
//! decision in the URL / arg pipeline. All tests are pure (no env mutation)
//! — the env-bound functions take their inputs as parameters, so cargo's
//! default parallel test runner can execute them in any order without
//! stepping on global state.

use super::*;

fn args<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// ---- parse_app_id_from --------------------------------------------------

#[test]
fn parse_app_id_from_extracts_eq_form() {
    let got = parse_app_id_from(&args(["--app=daily-digest", "extra"]));
    assert_eq!(got.as_deref(), Some("daily-digest"));
}

#[test]
fn parse_app_id_from_extracts_space_form() {
    let got = parse_app_id_from(&args(["--app", "daily-digest", "extra"]));
    assert_eq!(got.as_deref(), Some("daily-digest"));
}

#[test]
fn parse_app_id_from_returns_none_when_absent() {
    assert_eq!(parse_app_id_from(&args(["--unrelated", "value"])), None);
    assert_eq!(parse_app_id_from(&args([])), None);
}

#[test]
fn parse_app_id_from_eq_form_preserves_dashes_in_id() {
    // Hub / space ids use kebab-case (e.g. team-acme-frontend); the parser
    // must not split on '-'. The author chose strip_prefix("--app="), which
    // is the only path that can have a '-' in the id, so this test guards
    // the explicit shape — the space-separated form is delimited by the
    // next arg and is trivially fine.
    let got = parse_app_id_from(&args(["--app=team-acme-frontend"]));
    assert_eq!(got.as_deref(), Some("team-acme-frontend"));
}

// ---- target_path_with ---------------------------------------------------

#[test]
fn target_path_routes_named_app_to_its_space_route() {
    // The route /space/<id> is the daemon-side contract; the webview
    // navigates there and the daemon's router takes over. The exact shape
    // is a load-bearing piece of the URL contract. The override is ignored
    // when an app id is present (the per-app window is a fixed route).
    assert_eq!(
        target_path_with(&Some("daily-digest".into()), Some("/ignored")),
        "/space/daily-digest"
    );
}

#[test]
fn target_path_hub_defaults_to_root_when_no_env_override() {
    // SHIP_SHELL_TARGET unset -> '/'. This is the production case.
    assert_eq!(target_path_with(&None, None), "/");
    assert_eq!(target_path_with(&None, Some("")), "/");
}

#[test]
fn target_path_hub_honors_ship_shell_target_when_absolute() {
    // Dev override: SHIP_SHELL_TARGET=/sui-canvas points the hub straight
    // at an internal view while the hub's default route is being wired.
    assert_eq!(target_path_with(&None, Some("/sui-canvas")), "/sui-canvas");
}

#[test]
fn target_path_hub_rejects_non_leading_slash_override() {
    // An override like "http://attacker" would be a redirect-out bug. The
    // function falls back to '/' when the value doesn't start with a slash.
    // This is the security test for the dev-override path.
    assert_eq!(target_path_with(&None, Some("http://attacker")), "/");
    assert_eq!(target_path_with(&None, Some("relative/path")), "/");
}
