//! Settings tier — the `daemon_settings` KV store over `runtime.db`.
//!
//! This is the unified RUNTIME-scope settings API (S4): the daemon launch-
//! defaults tier. The crate calls `ship_substrate::db::daemon_settings::{get,
//! set,list_all}`; it never opens sqlite. The KV store is the sanctioned
//! substrate port — the rule-10 prohibition is on raw `fs` in the service,
//! which a DB accessor is not. RUNTIME reads come straight from the
//! `daemon_settings` accessor (which self-provisions its table), so they are
//! safe at early-boot points before the kernel/projections are live.
//!
//! `setting_default` carries the well-known infra keys; the resolvers
//! (`resolve_str`/`resolve_bool`/`resolve_*`) are the precedence chain.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use ship_substrate::db::daemon_settings;
use ship_substrate::events::{EmitContext, EventEnvelope, KernelRouter};
use ship_types::event_types::CONFIG_CHANGED;
use ship_types::payload::ConfigChanged;
use tokio::sync::Mutex;

use crate::types::HarnessProjectOverrides;

/// Return the default value for a well-known setting key.
///
/// Defaults are reserved for non-identity infrastructure choices (terminal
/// program, daemon port). Identity-shaped values like "which AI harness" or
/// "which model" never default — those need explicit user input on first run,
/// since silently picking an AI the user hasn't installed or paid for is a
/// footgun.
pub fn setting_default(key: &str) -> Option<&'static str> {
    match key {
        "terminal.program" => Some("tmux"),
        "daemon.port" => Some("9315"),
        // Command bar send key: "enter" sends on Enter (newline via Shift+Enter);
        // "cmd+enter" inserts newline on Enter and sends on Cmd/Ctrl+Enter.
        "cmdk.send_key" => Some("enter"),
        // When true, `ship use --launch` re-enters an existing (agent, path)
        // session instead of spawning fresh. Off by default so the user opts
        // in to implicit resumes.
        "agent_session.auto_resume" => Some("false"),
        // u64 ms — debounce window for the declarative `.ship/` watcher.
        "watcher.debounce_ms" => Some("200"),
        // usize — default `limit` for `git log` reads when no caller value is supplied.
        "git.log_limit" => Some("50"),
        // u32 — maximum hook recursion depth before the engine drops further events.
        "hooks.max_depth" => Some("3"),
        // ─── Isolation (see crates/core/ship-platform/crates/ship-isolation/docs/strategy.md) ───
        "isolation.default_tier" => Some("auto"),
        "isolation.default_strategy" => Some("auto"),
        "isolation.allow_tier_0" => Some("true"),
        "isolation.enforce" => Some("false"),
        "isolation.lazy_fetch" => Some("true"),
        "isolation.network_default" => Some("allowlist"),
        "isolation.allowlist_default" => {
            Some("api.anthropic.com,console.anthropic.com,*.github.com")
        }
        _ => None,
    }
}

/// Get a single setting from the daemon_settings table, falling back to the
/// hardcoded default.
pub fn get_setting(key: &str) -> Result<Option<serde_json::Value>> {
    if let Some(val) = daemon_settings::get(key)? {
        return Ok(Some(val));
    }
    Ok(setting_default(key).map(|d| serde_json::Value::String(d.to_string())))
}

/// True when the key has NO stored row — i.e. `get_setting` would return the
/// hardcoded default (or `None`). Used by the wire to set `Setting.is_default`.
pub fn is_default(key: &str) -> Result<bool> {
    Ok(daemon_settings::get(key)?.is_none())
}

/// Set a single setting in daemon_settings.
pub fn set_setting(key: &str, value: serde_json::Value) -> Result<()> {
    daemon_settings::set(key, &value)
}

/// Set a daemon (RUNTIME-scope) setting and publish a `config.changed` kernel
/// event.
///
/// The declarative file-tier watcher emits `agent.changed` / `skill.changed` /
/// etc. when files under `.ship/` are touched, but `daemon_settings` lives in
/// the runtime db and is invisible to `notify`. This is the canonical write
/// seam for daemon-managed RUNTIME settings — daemon route handlers that mutate
/// a setting call this so SSE consumers of `config.*` see the change. The
/// caller supplies the shared `KernelRouter` handle; the emit rides the
/// daemon-internal (Runtime) binding.
pub async fn set_setting_and_emit(
    kernel: &Arc<Mutex<KernelRouter>>,
    key: &str,
    value: serde_json::Value,
) -> Result<()> {
    set_setting(key, value.clone())?;
    let payload = ConfigChanged {
        subject: key.to_string(),
        detail: Some(value.to_string()),
    };
    let envelope = EventEnvelope::new(CONFIG_CHANGED, key, &payload)?;
    // RUNTIME setting — its own namespace, no space. Daemon-internal mint.
    let ctx = EmitContext::daemon_internal(None);
    kernel.lock().await.emit(envelope, &ctx).await
}

/// Return all persisted settings merged with defaults.
pub fn get_all_settings() -> Result<HashMap<String, serde_json::Value>> {
    let mut map = HashMap::new();
    for (key, default) in DEFAULT_KEYS {
        map.insert(
            (*key).to_string(),
            serde_json::Value::String((*default).to_string()),
        );
    }
    for (key, value) in daemon_settings::list_all()? {
        map.insert(key, value);
    }
    Ok(map)
}

/// The well-known infra keys + their string defaults, surfaced by
/// `get_all_settings`. Identity-shaped keys (`harness.*`) are intentionally
/// absent — they never default.
const DEFAULT_KEYS: &[(&str, &str)] = &[
    ("terminal.program", "tmux"),
    ("daemon.port", "9315"),
    ("cmdk.send_key", "enter"),
    ("agent_session.auto_resume", "false"),
    ("watcher.debounce_ms", "200"),
    ("git.log_limit", "50"),
    ("hooks.max_depth", "3"),
    ("isolation.default_tier", "auto"),
    ("isolation.default_strategy", "auto"),
    ("isolation.allow_tier_0", "true"),
    ("isolation.enforce", "false"),
    ("isolation.lazy_fetch", "true"),
    ("isolation.network_default", "allowlist"),
    (
        "isolation.allowlist_default",
        "api.anthropic.com,console.anthropic.com,*.github.com",
    ),
];

/// Resolve a string-typed launch default by walking the precedence chain:
/// per-launch flag → project override → user-KV → compile-time default.
///
/// Only the two harness-launch keys consult the project override —
/// `agent_session.auto_resume` and other user-preference keys are intentionally
/// user-only and skip the project tier.
pub fn resolve_str(
    key: &str,
    per_launch: Option<&str>,
    project: &HarnessProjectOverrides,
) -> Option<String> {
    if let Some(v) = per_launch {
        return Some(v.to_string());
    }
    let project_override = match key {
        "harness.default" => project.default.clone(),
        "harness.default_model" => project.default_model.clone(),
        _ => None,
    };
    if let Some(v) = project_override {
        return Some(v);
    }
    match get_setting(key).ok().flatten() {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
}

/// Resolve a bool-typed launch default: per-launch flag → user-KV → `false`.
///
/// No keys currently have a project-level bool override — booleans on this
/// surface are user preferences (e.g. `agent_session.auto_resume`) that should
/// not be shared across the team via project config.
pub fn resolve_bool(key: &str, per_launch: Option<bool>) -> bool {
    if let Some(v) = per_launch {
        return v;
    }
    match get_setting(key).ok().flatten() {
        Some(serde_json::Value::Bool(b)) => b,
        Some(serde_json::Value::String(s)) => {
            matches!(s.to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "on")
        }
        _ => false,
    }
}

/// Coerce a JSON value to `u64`, accepting either a JSON number or a stringified
/// number (settings are persisted as JSON; `setting_default` returns strings).
fn coerce_u64(v: &serde_json::Value) -> Option<u64> {
    match v {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// Fetch a setting and coerce to `u64`, falling back to `fallback` when the key
/// is unset, the stored value is unparseable, or the KV store cannot be opened.
pub fn get_setting_u64_or(key: &str, fallback: u64) -> u64 {
    match get_setting(key) {
        Ok(Some(v)) => coerce_u64(&v).unwrap_or(fallback),
        _ => fallback,
    }
}

/// Watcher debounce, resolved from `watcher.debounce_ms`. Historical default: 200ms.
pub fn resolve_watcher_debounce() -> std::time::Duration {
    std::time::Duration::from_millis(get_setting_u64_or("watcher.debounce_ms", 200))
}

/// Git log limit, resolved from `git.log_limit`. Historical default: 50.
pub fn resolve_git_log_limit() -> usize {
    get_setting_u64_or("git.log_limit", 50) as usize
}

/// Maximum hook recursion depth, resolved from `hooks.max_depth`. Historical default: 3.
pub fn resolve_hooks_max_depth() -> u32 {
    get_setting_u64_or("hooks.max_depth", 3) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::env::Var;
    use ship_substrate::testutil::EnvGuard;
    use tempfile::tempdir;

    /// Each test gets its own `SHIP_HOME`, so `daemon_settings` (over runtime.db
    /// derived from the global dir) is isolated. `open_runtime_db` self-provisions
    /// the `daemon_settings` table on a missing file.
    fn isolated_home() -> (tempfile::TempDir, EnvGuard) {
        let tmp = tempdir().unwrap();
        let guard = EnvGuard::set(Var::Home, tmp.path());
        let _ = ship_substrate::db::open_runtime_db(
            &ship_substrate::project::get_global_dir().unwrap(),
        );
        (tmp, guard)
    }

    // ── setting_default ───────────────────────────────────────────────────────

    #[test]
    fn setting_default_auto_resume_is_false() {
        assert_eq!(setting_default("agent_session.auto_resume"), Some("false"));
    }

    #[test]
    fn setting_default_harness_default_is_none() {
        // Identity-shaped keys NEVER default.
        assert_eq!(setting_default("harness.default"), None);
    }

    #[test]
    fn setting_default_harness_default_model_is_none() {
        assert_eq!(setting_default("harness.default_model"), None);
    }

    // ── resolve_str precedence ────────────────────────────────────────────────

    #[test]
    fn resolve_str_returns_none_when_unset_everywhere() {
        let (_tmp, _g) = isolated_home();
        let project = HarnessProjectOverrides::default();
        assert_eq!(resolve_str("harness.default", None, &project), None);
        assert_eq!(resolve_str("harness.default_model", None, &project), None);
    }

    #[test]
    fn resolve_str_per_launch_beats_project_beats_kv() {
        let (_tmp, _g) = isolated_home();
        set_setting("harness.default", serde_json::json!("claude")).unwrap();
        let project = HarnessProjectOverrides {
            default: Some("codex".into()),
            default_model: None,
        };
        assert_eq!(
            resolve_str("harness.default", None, &HarnessProjectOverrides::default()),
            Some("claude".into()),
            "KV used when no per-launch and no project override"
        );
        assert_eq!(
            resolve_str("harness.default", None, &project),
            Some("codex".into()),
            "project override beats KV"
        );
        assert_eq!(
            resolve_str("harness.default", Some("codex"), &project),
            Some("codex".into()),
            "per-launch beats project"
        );
    }

    #[test]
    fn resolve_str_non_harness_key_skips_project_tier() {
        let (_tmp, _g) = isolated_home();
        // A non-harness key never consults the project tier even if overrides exist.
        let project = HarnessProjectOverrides {
            default: Some("codex".into()),
            default_model: Some("opus".into()),
        };
        set_setting("terminal.program", serde_json::json!("wt")).unwrap();
        assert_eq!(resolve_str("terminal.program", None, &project), Some("wt".into()));
    }

    // ── resolve_bool ──────────────────────────────────────────────────────────

    #[test]
    fn resolve_bool_auto_resume_defaults_false() {
        let (_tmp, _g) = isolated_home();
        assert!(!resolve_bool("agent_session.auto_resume", None));
    }

    #[test]
    fn resolve_bool_per_launch_beats_kv() {
        let (_tmp, _g) = isolated_home();
        set_setting("agent_session.auto_resume", serde_json::json!(true)).unwrap();
        assert!(resolve_bool("agent_session.auto_resume", None));
        assert!(!resolve_bool("agent_session.auto_resume", Some(false)));
    }

    // ── set/get roundtrip + merge ─────────────────────────────────────────────

    #[test]
    fn set_then_get_setting_roundtrip() {
        let (_tmp, _g) = isolated_home();
        set_setting("harness.default", serde_json::json!("codex")).unwrap();
        assert_eq!(get_setting("harness.default").unwrap(), Some(serde_json::json!("codex")));
    }

    #[test]
    fn get_all_settings_merges_defaults_with_stored_rows() {
        let (_tmp, _g) = isolated_home();
        let before = get_all_settings().unwrap();
        // Default present even with no stored row.
        assert_eq!(before.get("terminal.program"), Some(&serde_json::json!("tmux")));
        // A stored row overrides the default.
        set_setting("terminal.program", serde_json::json!("wt")).unwrap();
        let after = get_all_settings().unwrap();
        assert_eq!(after.get("terminal.program"), Some(&serde_json::json!("wt")));
    }

    #[test]
    fn is_default_reflects_stored_row() {
        let (_tmp, _g) = isolated_home();
        assert!(is_default("terminal.program").unwrap(), "no row → default");
        set_setting("terminal.program", serde_json::json!("wt")).unwrap();
        assert!(!is_default("terminal.program").unwrap(), "stored row → not default");
    }

    // ── numeric resolvers + coercion ──────────────────────────────────────────

    #[test]
    fn watcher_debounce_defaults_to_historical_200ms() {
        let (_tmp, _g) = isolated_home();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(200));
    }

    #[test]
    fn watcher_debounce_honors_override_string_or_numeric() {
        let (_tmp, _g) = isolated_home();
        set_setting("watcher.debounce_ms", serde_json::json!("500")).unwrap();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(500));
        set_setting("watcher.debounce_ms", serde_json::json!(750)).unwrap();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(750));
    }

    #[test]
    fn git_log_limit_default_and_override() {
        let (_tmp, _g) = isolated_home();
        assert_eq!(resolve_git_log_limit(), 50);
        set_setting("git.log_limit", serde_json::json!(200)).unwrap();
        assert_eq!(resolve_git_log_limit(), 200);
    }

    #[test]
    fn hooks_max_depth_default_and_override_coerces_string() {
        let (_tmp, _g) = isolated_home();
        assert_eq!(resolve_hooks_max_depth(), 3);
        set_setting("hooks.max_depth", serde_json::json!("7")).unwrap();
        assert_eq!(resolve_hooks_max_depth(), 7);
    }
}
