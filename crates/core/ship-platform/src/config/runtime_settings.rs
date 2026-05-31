use super::project::ProjectConfig;
use ship_substrate::db::daemon_settings;
use ship_substrate::db::{block_on, ensure_kv_db, open_kv_db};
use ship_substrate::events::{
    CallerKind, EmitContext, EventEnvelope, KernelRouter,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use sqlx::Row;
use tokio::sync::Mutex;
use ship_types::event_types::CONFIG_CHANGED;
use ship_types::payload::ConfigChanged;

/// Return the default value for a well-known setting key.
///
/// Defaults are reserved for non-identity infrastructure choices (terminal
/// program, daemon port). Identity-shaped values like "which AI harness"
/// or "which model" never default — those need explicit user input on first
/// run, since silently picking an AI the user hasn't installed or paid for
/// is a footgun.
pub fn setting_default(key: &str) -> Option<&'static str> {
    match key {
        "terminal.program" => Some("tmux"),
        "daemon.port" => Some("9315"),
        // Command bar send key: "enter" sends on Enter (newline via Shift+Enter);
        // "cmd+enter" inserts newline on Enter and sends on Cmd/Ctrl+Enter.
        "cmdk.send_key" => Some("enter"),
        // When true, `ship use --launch` re-enters an existing (agent, path)
        // session instead of spawning fresh. Off by default so the user
        // opts in to implicit resumes.
        "agent_session.auto_resume" => Some("false"),
        // u64 ms — debounce window for the declarative `.ship/` watcher.
        "watcher.debounce_ms" => Some("200"),
        // usize — default `limit` for `git log` reads when no caller value is supplied.
        "git.log_limit" => Some("50"),
        // u32 — maximum hook recursion depth before the engine drops further events.
        "hooks.max_depth" => Some("3"),
        // ─── Isolation (see crates/core/ship-isolation/docs/strategy.md) ───
        // Preferred tier when no agent / per-run override applies.
        // "auto" means: derive per-OS from the strategy spec §4 table.
        "isolation.default_tier" => Some("auto"),
        // Optional pin within tier. "auto" means: let the recommender pick.
        "isolation.default_strategy" => Some("auto"),
        // If "false", the recommender refuses to fall back to Tier 0 and
        // an agent run fails instead of silently running unsandboxed.
        "isolation.allow_tier_0" => Some("true"),
        // Master switch. When "false" (the default), the supervisor
        // computes the isolation decision, exposes it via the API + env
        // vars, but DOES NOT wrap the launch — the agent still runs on
        // the host. Flip to "true" once strategies are validated on
        // your machine. Per-strategy enforcement coverage is partial;
        // strategies not yet wired downgrade to Noop with a warning.
        "isolation.enforce" => Some("false"),
        // Lazy-fetch heavy assets (rootfs, VM images) on first use.
        "isolation.lazy_fetch" => Some("true"),
        // Default network policy when an agent doesn't specify one.
        "isolation.network_default" => Some("allowlist"),
        // Comma-separated default egress hostnames. Augments per-agent
        // allowlists.
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

/// Set a single setting in daemon_settings.
pub fn set_setting(key: &str, value: serde_json::Value) -> Result<()> {
    daemon_settings::set(key, &value)
}

/// Set a daemon setting and publish a `config.changed` kernel event.
///
/// The file-tier watcher (`config::watcher`) emits `agent.changed` /
/// `skill.changed` / etc. when files under `.ship/` are touched, but
/// `daemon_settings` lives in SQLite and is invisible to `notify`. This
/// is the canonical write seam for daemon-managed settings — daemon
/// route handlers that mutate a setting should call this so SSE
/// consumers of `config.*` see the change.
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
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: None,
        session_id: None,
    };
    kernel.lock().await.emit(envelope, &ctx).await
}

/// Return all persisted settings merged with defaults.
pub fn get_all_settings() -> Result<HashMap<String, serde_json::Value>> {
    let mut map = HashMap::new();
    for (key, default) in &[
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
    ] {
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

/// Resolve a string-typed launch default by walking the precedence chain:
/// per-launch flag → project override → user-KV → compile-time default.
///
/// Only the two harness-launch keys consult the project override —
/// `agent_session.auto_resume` and other user-preference keys are
/// intentionally user-only and skip the project tier.
pub fn resolve_str(
    key: &str,
    per_launch: Option<&str>,
    project: &ProjectConfig,
) -> Option<String> {
    if let Some(v) = per_launch {
        return Some(v.to_string());
    }
    let project_override = match key {
        "harness.default" => project.harness.as_ref().and_then(|h| h.default.clone()),
        "harness.default_model" => project
            .harness
            .as_ref()
            .and_then(|h| h.default_model.clone()),
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

/// Resolve a bool-typed launch default by walking the precedence chain:
/// per-launch flag → user-KV → compile-time default (which is `false`
/// when not registered for the key).
///
/// No keys currently have a project-level bool override — booleans on
/// this surface are user preferences (e.g. `agent_session.auto_resume`)
/// that should not be shared across the team via project config.
pub fn resolve_bool(key: &str, per_launch: Option<bool>, _project: &ProjectConfig) -> bool {
    if let Some(v) = per_launch {
        return v;
    }
    match get_setting(key).ok().flatten() {
        Some(serde_json::Value::Bool(b)) => b,
        Some(serde_json::Value::String(s)) => matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        ),
        _ => false,
    }
}

/// Coerce a JSON value to `u64`, accepting either a JSON number or a
/// stringified number (settings are persisted as JSON; `setting_default`
/// returns its values as strings).
fn coerce_u64(v: &serde_json::Value) -> Option<u64> {
    match v {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// Fetch a setting and coerce to `u64`, falling back to `fallback` when the
/// key is unset, the stored value is unparseable, or the KV store cannot be
/// opened (e.g. during early-startup or in tests that haven't initialized
/// the daemon DB).
pub fn get_setting_u64_or(key: &str, fallback: u64) -> u64 {
    match get_setting(key) {
        Ok(Some(v)) => coerce_u64(&v).unwrap_or(fallback),
        _ => fallback,
    }
}

/// Watcher debounce, resolved from `watcher.debounce_ms`.
/// Historical default: 200ms.
pub fn resolve_watcher_debounce() -> std::time::Duration {
    std::time::Duration::from_millis(get_setting_u64_or("watcher.debounce_ms", 200))
}

/// Git log limit, resolved from `git.log_limit`. Historical default: 50.
pub fn resolve_git_log_limit() -> usize {
    get_setting_u64_or("git.log_limit", 50) as usize
}

/// Maximum hook recursion depth, resolved from `hooks.max_depth`.
/// Historical default: 3.
pub fn resolve_hooks_max_depth() -> u32 {
    get_setting_u64_or("hooks.max_depth", 3) as u32
}

/// One-shot drain of the legacy KV `settings` namespace into the
/// daemon_settings table. Idempotent: re-running on an already-drained
/// install is a no-op (UPSERT). Safe to call on every daemon startup;
/// removes the legacy KV rows after copying.
pub fn drain_legacy_settings_kv() -> Result<usize> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let rows = block_on(async {
        sqlx::query("SELECT key, value_json FROM kv_state WHERE namespace = 'settings'")
            .fetch_all(&mut conn)
            .await
    })?;
    if rows.is_empty() {
        return Ok(0);
    }
    let mut migrated = 0usize;
    for row in &rows {
        let key: String = row.get(0);
        let json_str: String = row.get(1);
        let value: serde_json::Value =
            serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
        daemon_settings::set(&key, &value)?;
        migrated += 1;
    }
    block_on(async {
        sqlx::query("DELETE FROM kv_state WHERE namespace = 'settings'")
            .execute(&mut conn)
            .await
    })?;
    Ok(migrated)
}

/// One-shot drain of legacy `harnesses` rows from the per-project
/// `runtime::*` KV namespace. Harnesses now live only in `ship.jsonc`,
/// so the legacy KV rows can be deleted. Other keys in `runtime::*`
/// (e.g. the `agents` cache) are preserved. Idempotent.
pub fn drain_legacy_project_config_kv() -> Result<usize> {
    ensure_kv_db()?;
    let mut conn = open_kv_db()?;
    let result = block_on(async {
        sqlx::query(
            "DELETE FROM kv_state WHERE namespace LIKE 'runtime::%' AND key = 'harnesses'",
        )
        .execute(&mut conn)
        .await
    })?;
    Ok(result.rows_affected() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HarnessProjectOverrides;
    use ship_substrate::env::Var;
    use ship_substrate::testutil::EnvGuard;
    use tempfile::tempdir;

    fn isolated_home() -> (tempfile::TempDir, EnvGuard) {
        let tmp = tempdir().unwrap();
        let guard = EnvGuard::set(Var::Home, tmp.path());
        (tmp, guard)
    }

    #[test]
    fn setting_default_auto_resume_is_false() {
        assert_eq!(setting_default("agent_session.auto_resume"), Some("false"));
    }

    #[test]
    fn setting_default_harness_default_is_none() {
        assert_eq!(setting_default("harness.default"), None);
    }

    #[test]
    fn setting_default_harness_default_model_is_none() {
        assert_eq!(setting_default("harness.default_model"), None);
    }

    #[test]
    fn resolve_str_returns_none_when_unset_everywhere() {
        let (_tmp, _guard) = isolated_home();
        let project = ProjectConfig::default();
        assert_eq!(resolve_str("harness.default", None, &project), None);
        assert_eq!(resolve_str("harness.default_model", None, &project), None);
    }

    #[test]
    fn resolve_str_per_launch_beats_project_beats_kv() {
        let (_tmp, _guard) = isolated_home();
        set_setting("harness.default", serde_json::json!("claude")).unwrap();
        let project = ProjectConfig {
            harness: Some(HarnessProjectOverrides {
                default: Some("codex".into()),
                default_model: None,
            }),
            ..Default::default()
        };
        assert_eq!(
            resolve_str("harness.default", None, &ProjectConfig::default()),
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
    fn resolve_str_default_model_chain() {
        let (_tmp, _guard) = isolated_home();
        set_setting("harness.default_model", serde_json::json!("sonnet")).unwrap();
        let project = ProjectConfig {
            harness: Some(HarnessProjectOverrides {
                default: None,
                default_model: Some("opus".into()),
            }),
            ..Default::default()
        };
        assert_eq!(
            resolve_str("harness.default_model", None, &ProjectConfig::default()),
            Some("sonnet".into())
        );
        assert_eq!(
            resolve_str("harness.default_model", None, &project),
            Some("opus".into())
        );
        assert_eq!(
            resolve_str("harness.default_model", Some("haiku"), &project),
            Some("haiku".into())
        );
    }

    #[test]
    fn resolve_bool_auto_resume_defaults_false() {
        let (_tmp, _guard) = isolated_home();
        assert!(!resolve_bool(
            "agent_session.auto_resume",
            None,
            &ProjectConfig::default()
        ));
    }

    #[test]
    fn resolve_bool_per_launch_beats_kv() {
        let (_tmp, _guard) = isolated_home();
        set_setting("agent_session.auto_resume", serde_json::json!(true)).unwrap();
        assert!(resolve_bool(
            "agent_session.auto_resume",
            None,
            &ProjectConfig::default()
        ));
        assert!(!resolve_bool(
            "agent_session.auto_resume",
            Some(false),
            &ProjectConfig::default()
        ));
    }

    #[test]
    fn set_then_get_setting_roundtrip() {
        let (_tmp, _guard) = isolated_home();
        set_setting("harness.default", serde_json::json!("codex")).unwrap();
        let got = get_setting("harness.default").unwrap();
        assert_eq!(got, Some(serde_json::json!("codex")));
    }

    #[test]
    fn project_with_harness_overrides_deserializes() {
        let raw = r#"{"id":"p","harness":{"default":"codex","default_model":"opus"}}"#;
        let project: ProjectConfig = serde_json::from_str(raw).unwrap();
        let h = project.harness.expect("harness present");
        assert_eq!(h.default.as_deref(), Some("codex"));
        assert_eq!(h.default_model.as_deref(), Some("opus"));
    }

    #[test]
    fn resolve_picks_project_codex_when_no_flag() {
        let (_tmp, _guard) = isolated_home();
        let raw = r#"{"id":"p","harness":{"default":"codex"}}"#;
        let project: ProjectConfig = serde_json::from_str(raw).unwrap();
        assert_eq!(
            resolve_str("harness.default", None, &project),
            Some("codex".into())
        );
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;

    /// `ship_substrate::project::get_global_dir` provides per-thread DB
    /// isolation when running under a cargo test binary, so each test thread
    /// gets its own `daemon_settings` table. We still scrub the three keys
    /// at the start of each test because a thread may be reused across
    /// tests within this module.
    fn fresh_settings() {
        ship_substrate::db::ensure_db().unwrap();
        let _ = daemon_settings::delete("watcher.debounce_ms");
        let _ = daemon_settings::delete("git.log_limit");
        let _ = daemon_settings::delete("hooks.max_depth");
    }

    #[test]
    fn watcher_debounce_defaults_to_historical_200ms() {
        fresh_settings();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(200));
    }

    #[test]
    fn watcher_debounce_honors_override() {
        fresh_settings();
        set_setting("watcher.debounce_ms", serde_json::json!("500")).unwrap();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(500));
        // Also accepts numeric JSON.
        set_setting("watcher.debounce_ms", serde_json::json!(750)).unwrap();
        assert_eq!(resolve_watcher_debounce(), std::time::Duration::from_millis(750));
    }

    #[test]
    fn git_log_limit_defaults_to_historical_50() {
        fresh_settings();
        assert_eq!(resolve_git_log_limit(), 50);
    }

    #[test]
    fn git_log_limit_honors_override() {
        fresh_settings();
        set_setting("git.log_limit", serde_json::json!(200)).unwrap();
        assert_eq!(resolve_git_log_limit(), 200);
    }

    #[test]
    fn hooks_max_depth_defaults_to_historical_3() {
        fresh_settings();
        assert_eq!(resolve_hooks_max_depth(), 3);
    }

    #[test]
    fn hooks_max_depth_honors_override() {
        fresh_settings();
        set_setting("hooks.max_depth", serde_json::json!("7")).unwrap();
        assert_eq!(resolve_hooks_max_depth(), 7);
    }
}
