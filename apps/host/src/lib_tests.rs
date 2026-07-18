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

// ---- error_url ----------------------------------------------------------
//
// `navigate` takes an absolute `tauri::Url` — a relative string like
// "index.html#error" can never parse (RelativeUrlWithoutBase), which is why
// the helper derives the error URL from the page URL the webview reports.
// The bundled frontendDist origin differs per platform, so both known
// origins must round-trip: `tauri://localhost` (macOS/Linux) and
// `http://tauri.localhost` (Windows).

#[test]
fn error_url_is_absolute_for_the_linux_macos_origin() {
    let page: tauri::Url = "tauri://localhost/index.html"
        .parse()
        .expect("platform origin must parse as an absolute URL");
    let got = error_url(&page);
    assert_eq!(got.as_str(), "tauri://localhost/index.html#error");
    assert!(!got.cannot_be_a_base(), "must stay a navigable absolute URL");
}

#[test]
fn error_url_is_absolute_for_the_windows_origin() {
    let page: tauri::Url = "http://tauri.localhost/"
        .parse()
        .expect("platform origin must parse as an absolute URL");
    let got = error_url(&page);
    assert_eq!(got.as_str(), "http://tauri.localhost/#error");
    assert_eq!(got.fragment(), Some("error"));
}

#[test]
fn error_url_preserves_origin_and_path_and_replaces_any_fragment() {
    // The helper must not disturb scheme/host/path (the daemon-served page
    // and the bundled page have different origins — whatever the webview is
    // on is what we stay on), and a stale fragment must be replaced, not
    // appended to.
    let page: tauri::Url = "tauri://localhost/index.html#stale".parse().unwrap();
    let got = error_url(&page);
    assert_eq!(got.scheme(), "tauri");
    assert_eq!(got.host_str(), Some("localhost"));
    assert_eq!(got.path(), "/index.html");
    assert_eq!(got.fragment(), Some("error"));
}

// ---- watch_tick ---------------------------------------------------------
//
// The passive watcher must never stop probing: the generous fast window
// (2s cadence, ~40s) gives a slow handshake time to land, the `#error`
// surface fires exactly once when it closes, and probing continues forever
// at the slow cadence so a daemon appearing at any later time is picked up.

#[test]
fn watch_tick_polls_fast_without_error_through_the_generous_window() {
    for tick in [0u32, 1, 10, 19] {
        let step = watch_tick(tick);
        assert_eq!(step.delay, Duration::from_secs(2), "tick {tick}");
        assert!(!step.surface_error, "tick {tick} must not surface the error");
    }
}

#[test]
fn watch_tick_generous_window_totals_forty_seconds() {
    // The cumulative fast-window wait before `#error` surfaces is the
    // user-facing "how long until the shell admits failure" contract.
    let total: Duration = (0..20).map(|t| watch_tick(t).delay).sum();
    assert_eq!(total, Duration::from_secs(40));
}

#[test]
fn watch_tick_surfaces_error_exactly_once_at_window_close() {
    assert!(watch_tick(20).surface_error);
    for tick in [21u32, 22, 100, u32::MAX] {
        assert!(
            !watch_tick(tick).surface_error,
            "tick {tick} must not re-surface the error"
        );
    }
}

#[test]
fn watch_tick_keeps_probing_at_slow_cadence_after_the_window() {
    // u32::MAX is where the loop's saturating counter parks — the policy
    // must still return a finite delay there (watch forever, never panic).
    for tick in [20u32, 21, 1_000, u32::MAX] {
        assert_eq!(watch_tick(tick).delay, Duration::from_secs(5), "tick {tick}");
    }
}
