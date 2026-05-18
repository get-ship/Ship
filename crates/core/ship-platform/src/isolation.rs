//! Platform-side isolation glue.
//!
//! Reads the user's `isolation.*` settings (from `daemon_settings`),
//! resolves the typed `IsolationProfile`, probes host capabilities, and
//! recommends a concrete `IsolationStrategy` for an agent run.
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
use ship_isolation::capabilities::{Capabilities, HostOs};
use ship_isolation::recommender::{RecommendInput, RecommenderError, ResolvedIsolation, recommend};
use ship_isolation::settings::{UserDefaults, parse_allow_tier_0, user_profile};
use ship_isolation::tier::IsolationProfile;

use crate::config::{get_setting, set_setting};

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
    if let Some(serde_json::Value::String(s)) = get_setting(CAPS_KEY)? {
        if let Ok(parsed) = serde_json::from_str::<Capabilities>(&s) {
            return Ok(parsed);
        }
    }
    let probed = Capabilities::probe();
    let serialized = serde_json::to_string(&probed)?;
    let _ = set_setting(CAPS_KEY, serde_json::Value::String(serialized));
    Ok(probed)
}

/// Force a re-probe. Returns the new snapshot.
pub fn rescan_capabilities() -> Result<Capabilities> {
    let probed = Capabilities::probe();
    let serialized = serde_json::to_string(&probed)?;
    let _ = set_setting(CAPS_KEY, serde_json::Value::String(serialized));
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

/// Decide which strategy to use for a specific agent, reading the
/// agent's optional `isolation:` block from `.ship/agents/<id>.jsonc`.
/// `run_override` is the per-launch CLI flag (`ship use --isolation=…`).
///
/// Returns the same `IsolationDecision` as `decide()`. Used by the
/// supervisor when it spawns a job so the launch can observe (and
/// later enforce) the resolution.
pub fn decide_for_agent(
    ship_dir: &std::path::Path,
    agent_id: &str,
    run_override: Option<&IsolationProfile>,
) -> Result<IsolationDecision, IsolationDecisionError> {
    let agent_override = match crate::config::get_config(Some(ship_dir.to_path_buf())) {
        Ok(cfg) => cfg
            .agents
            .iter()
            .find(|a| a.id == agent_id)
            .and_then(|a| {
                parse_agent_override(a.isolation.as_ref())
                    .map_err(|e| {
                        tracing::warn!(
                            agent = agent_id,
                            error = %e,
                            "agent has invalid isolation: block — ignoring"
                        );
                        e
                    })
                    .ok()
                    .flatten()
            }),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "config read failed in decide_for_agent — using defaults"
            );
            None
        }
    };
    decide(agent_override.as_ref(), run_override)
}

fn string_setting(key: &str) -> Result<Option<String>> {
    Ok(match get_setting(key)? {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Bool(b)) => Some(b.to_string()),
        Some(v) => Some(v.to_string()),
        None => None,
    })
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
            Some(ship_isolation::tier::IsolationTier::Container)
        );
        assert_eq!(
            parsed.strategy,
            Some(ship_isolation::tier::IsolationStrategy::DockerEngine)
        );
    }

    #[test]
    fn parse_agent_override_errors_on_malformed() {
        let raw = serde_json::json!({ "tier": "totally-wrong" });
        assert!(parse_agent_override(Some(&raw)).is_err());
    }
}
