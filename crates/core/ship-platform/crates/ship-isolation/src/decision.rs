//! Platform-side isolation decision layer.
//!
//! Reads the user's `isolation.*` settings (from `daemon_settings`),
//! resolves the typed `IsolationProfile`, probes host capabilities, and
//! recommends a concrete `IsolationStrategy` for an agent run.
//!
//! INFRACTION (deferred): this reads/writes config directly to
//! `daemon_settings`, bypassing the `SettingsService` (the sole settings
//! path). isolation.* is domain-level config that should route through the
//! service. ship-isolation is experimental ship-code, so this is left as a
//! flagged exception (see `scripts/lint/check-settings-single-owner.sh`)
//! until the ship-code isolation work resumes.
//!
//! Does NOT spawn the backend — that's the supervisor's job. This
//! module is the *decision layer*; the supervisor consults it before
//! launching.
//!
//! The capability snapshot is cached in `daemon_settings` under
//! `isolation.capabilities_cached`. The cache invalidates on shipd
//! restart and on the `isolation rescan` CLI/MCP command.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ship_substrate::db::daemon_settings;
use crate::capabilities::{Capabilities, HostOs};
use crate::recommender::{RecommendInput, RecommenderError, ResolvedIsolation, recommend};
use crate::settings::{UserDefaults, parse_allow_tier_0, user_profile};
use crate::tier::IsolationProfile;

/// Cache key for the serialized `Capabilities` snapshot.
const CAPS_KEY: &str = "isolation.capabilities_cached";

/// What the orchestrator gets back when it asks "what isolation should
/// this agent run use?". UI-friendly: the resolved strategy + tier
/// plus the capability snapshot + reason for transparency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationDecision {
    pub resolved: ResolvedIsolation,
    pub capabilities: Capabilities,
    pub user_profile_used: IsolationProfile,
}

/// Read or compute the host capability snapshot. First call (or after
/// `rescan_capabilities`) runs `Capabilities::probe()` and caches it.
pub fn cached_capabilities() -> Result<Capabilities> {
    if let Some(serde_json::Value::String(s)) = daemon_settings::get(CAPS_KEY)? {
        if let Ok(parsed) = serde_json::from_str::<Capabilities>(&s) {
            return Ok(parsed);
        }
    }
    let probed = Capabilities::probe();
    let serialized = serde_json::to_string(&probed)?;
    let _ = daemon_settings::set(CAPS_KEY, &serde_json::Value::String(serialized));
    Ok(probed)
}

/// Force a re-probe. Returns the new snapshot.
pub fn rescan_capabilities() -> Result<Capabilities> {
    let probed = Capabilities::probe();
    let serialized = serde_json::to_string(&probed)?;
    let _ = daemon_settings::set(CAPS_KEY, &serde_json::Value::String(serialized));
    Ok(probed)
}

/// Build the user-default `IsolationProfile` by reading every relevant
/// `isolation.*` setting. Empty / "auto" values resolve via the
/// per-OS table from `ship-isolation::recommender::default_tier_for_os`.
pub fn user_isolation_profile(host_os: HostOs) -> Result<(IsolationProfile, bool)> {
    let tier = string_setting("isolation.default_tier")?;
    let strategy = string_setting("isolation.default_strategy")?;
    let allow_tier_0_raw = string_setting("isolation.allow_tier_0")?
        .unwrap_or_else(|| "true".into());
    let network = string_setting("isolation.network_default")?;
    let allowlist = string_setting("isolation.allowlist_default")?;
    let defaults = UserDefaults::from_strings(
        tier.as_deref(),
        strategy.as_deref(),
        parse_allow_tier_0(&allow_tier_0_raw),
        network.as_deref(),
        allowlist.as_deref(),
    );
    let profile = user_profile(&defaults, host_os);
    Ok((profile, defaults.allow_tier_0))
}

/// Recommend a strategy for one agent run.
///
/// Inputs:
/// - `agent_override` — `isolation:` block from the agent profile, if any.
/// - `run_override` — per-launch flag (`ship use --isolation=container`).
///
/// All other inputs (user defaults, capabilities, allow_tier_0) come
/// from the daemon's settings.
pub fn decide(
    agent_override: Option<&IsolationProfile>,
    run_override: Option<&IsolationProfile>,
) -> Result<IsolationDecision, IsolationDecisionError> {
    let caps = cached_capabilities().map_err(IsolationDecisionError::Capabilities)?;
    let (user_pref, allow_tier_0) =
        user_isolation_profile(caps.os).map_err(IsolationDecisionError::Settings)?;
    let mut input = RecommendInput::new(&caps).with_allow_tier_0(allow_tier_0);
    if let Some(p) = run_override {
        input = input.with_run_override(p);
    }
    if let Some(p) = agent_override {
        input = input.with_agent_override(p);
    }
    input = input.with_user_pref(&user_pref);
    let resolved = recommend(input).map_err(IsolationDecisionError::Recommender)?;
    Ok(IsolationDecision {
        resolved,
        capabilities: caps,
        user_profile_used: user_pref,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum IsolationDecisionError {
    #[error("capability probe failed: {0}")]
    Capabilities(#[source] anyhow::Error),
    #[error("settings read failed: {0}")]
    Settings(#[source] anyhow::Error),
    #[error(transparent)]
    Recommender(#[from] RecommenderError),
}

/// Is the `isolation.enforce` master switch on? When false, the
/// supervisor observes the decision and propagates it via env vars but
/// does NOT wrap the launch. Defaults to false.
pub fn enforcement_enabled() -> bool {
    string_setting("isolation.enforce")
        .ok()
        .flatten()
        .as_deref()
        .map(parse_bool)
        .unwrap_or(false)
}

fn parse_bool(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes")
}

/// Parse the optional `isolation` field on an `AgentProfile` into a
/// typed `IsolationProfile`. Returns `Ok(None)` for missing or empty;
/// `Err` only when the value is malformed (caller should surface this
/// in the UI as "agent has an invalid isolation config").
pub fn parse_agent_override(
    raw: Option<&serde_json::Value>,
) -> Result<Option<IsolationProfile>, serde_json::Error> {
    let Some(value) = raw else { return Ok(None) };
    if value.is_null() {
        return Ok(None);
    }
    let parsed: IsolationProfile = serde_json::from_value(value.clone())?;
    Ok(Some(parsed))
}

// NOTE: the former `decide_for_agent(ship_dir, agent_id, …)` wrapper read the
// project config (`ship_config::project::get_config`) to resolve an agent's
// `isolation:` override — an L1 concern that leaked into this L0 crate. The
// config read now lives at the L1 caller (ship-agents executor), which passes
// the resolved override into the pure `decide()` above. ship-isolation no
// longer depends on ship-config.

/// Read the nested per-strategy settings blob. Returns defaults when
/// the key isn't set; only returns an error on actual DB failure.
pub fn strategies_settings()
-> Result<crate::strategies_config::StrategiesSettings> {
    let raw = daemon_settings::get(crate::strategies_config::SETTINGS_KEY)?;
    match raw {
        Some(value) => Ok(serde_json::from_value(value).unwrap_or_default()),
        None => Ok(Default::default()),
    }
}

/// Write the nested per-strategy settings blob. Mirrors set_setting
/// semantics; emits no kernel event (callers that want one use
/// set_setting_and_emit directly).
pub fn set_strategies_settings(
    value: &crate::strategies_config::StrategiesSettings,
) -> Result<()> {
    let json = serde_json::to_value(value)?;
    daemon_settings::set(crate::strategies_config::SETTINGS_KEY, &json)
}

fn string_setting(key: &str) -> Result<Option<String>> {
    Ok(match daemon_settings::get(key)? {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Bool(b)) => Some(b.to_string()),
        Some(v) => Some(v.to_string()),
        None => None,
    })
}

/// Process-wide handle to the daemon's own HTTP origin
/// (`http://<host>:<port>`), installed once by shipd at boot. The
/// supervisor reads it to inject `SHIP_DAEMON_URL` into a sandboxed
/// actor's env so the in-container agent can reach the app plane. Mirrors
/// the `session_registry()` singleton: the spawn path needs the one
/// process-wide value without threading the runtime config through every
/// layer. `None` outside the daemon (CLI, tests).
static DAEMON_ORIGIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// Install the daemon's reachable origin. Called once by shipd at boot
/// with `runtime.daemon_origin()`. Subsequent calls are no-ops.
pub fn set_daemon_origin(origin: String) {
    let _ = DAEMON_ORIGIN.set(origin);
}

/// The daemon's HTTP origin, if shipd has installed one.
pub fn daemon_origin() -> Option<String> {
    DAEMON_ORIGIN.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests run against the real daemon DB if it's initialized;
    // otherwise `cached_capabilities` falls through to a fresh probe
    // without persistence. We don't assert side effects on the DB —
    // those are covered by integration tests when the daemon is up.

    #[test]
    fn user_profile_round_trips_with_defaults() {
        // Smoke: builds a profile without panicking on a fresh DB.
        let res = user_isolation_profile(HostOs::Linux);
        assert!(res.is_ok());
    }

    #[test]
    fn decide_returns_a_resolution() {
        // Smoke: without overrides, decide() returns SOMETHING. The
        // contents depend on host caps; we just check it doesn't panic.
        let _ = decide(None, None);
    }

    #[test]
    fn parse_agent_override_handles_missing_null_and_typed() {
        assert!(parse_agent_override(None).unwrap().is_none());
        assert!(
            parse_agent_override(Some(&serde_json::Value::Null))
                .unwrap()
                .is_none()
        );
        let raw = serde_json::json!({ "tier": "container", "strategy": "docker" });
        let parsed = parse_agent_override(Some(&raw)).unwrap().unwrap();
        assert_eq!(
            parsed.tier,
            Some(crate::tier::IsolationTier::Container)
        );
        assert_eq!(
            parsed.strategy,
            Some(crate::tier::IsolationStrategy::DockerEngine)
        );
    }

    #[test]
    fn parse_agent_override_errors_on_malformed() {
        let raw = serde_json::json!({ "tier": "totally-wrong" });
        assert!(parse_agent_override(Some(&raw)).is_err());
    }
}
