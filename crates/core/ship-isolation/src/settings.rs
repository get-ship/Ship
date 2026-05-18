//! Resolves user-facing isolation settings into typed values.
//!
//! Reads strings (from `daemon_settings` via the caller) plus env vars,
//! resolves the "auto" sentinel against the per-OS default table from
//! `docs/strategy.md` §4, and returns typed `IsolationTier` /
//! `IsolationStrategy` / `IsolationProfile` values the orchestrator can
//! feed straight into the recommender.
//!
//! `ship-platform` owns the persistence (`daemon_settings`); this module
//! is pure: callers hand in the string values they already read, plus an
//! optional env-var override layer. No I/O on the daemon DB happens here.

use crate::capabilities::HostOs;
use crate::recommender::default_tier_for_os;
use crate::tier::{IsolationProfile, IsolationStrategy, IsolationTier};

/// One frame of user-default settings as the orchestrator sees them.
/// Values are pre-resolved from `daemon_settings`; "auto" sentinels are
/// kept so this module can do the OS-aware lookup.
#[derive(Debug, Clone, Default)]
pub struct UserDefaults {
    pub default_tier_raw: Option<String>,
    pub default_strategy_raw: Option<String>,
    pub allow_tier_0: bool,
    pub network_default_raw: Option<String>,
    pub allowlist_default_raw: Option<String>,
}

impl UserDefaults {
    pub fn from_strings(
        tier: Option<&str>,
        strategy: Option<&str>,
        allow_tier_0: bool,
        network: Option<&str>,
        allowlist: Option<&str>,
    ) -> Self {
        Self {
            default_tier_raw: tier.map(str::to_string),
            default_strategy_raw: strategy.map(str::to_string),
            allow_tier_0,
            network_default_raw: network.map(str::to_string),
            allowlist_default_raw: allowlist.map(str::to_string),
        }
    }
}

/// Env var names the daemon reads (per `docs/strategy.md` §9). Settings
/// from `daemon_settings` take precedence; env vars are the fallback so
/// CI / container runners can pin a tier without writing to the KV.
pub const ENV_DEFAULT_TIER: &str = "SHIP_ISOLATION_TIER";
pub const ENV_DEFAULT_STRATEGY: &str = "SHIP_ISOLATION_STRATEGY";
pub const ENV_CACHE_DIR: &str = "SHIP_ISOLATION_CACHE_DIR";
pub const ENV_YOUKI_BIN: &str = "SHIP_YOUKI_BIN";
pub const ENV_YOUKI_ROOTFS: &str = "SHIP_AGENT_ROOTFS";
pub const ENV_LIMA_BIN: &str = "SHIP_LIMA_BIN";
pub const ENV_FLY_ORG: &str = "SHIP_FLY_ORG";
pub const ENV_FLY_APP: &str = "SHIP_FLY_APP";

/// Convert user-default strings into a profile suitable for the
/// recommender's `user_pref` slot.
///
/// Resolution order:
///   1. env var override (`SHIP_ISOLATION_TIER` / `_STRATEGY`)
///   2. provided `UserDefaults` string
///   3. `"auto"` sentinel → per-OS default tier from §4
pub fn user_profile(defaults: &UserDefaults, host_os: HostOs) -> IsolationProfile {
    let env_tier = std::env::var(ENV_DEFAULT_TIER).ok();
    let env_strategy = std::env::var(ENV_DEFAULT_STRATEGY).ok();

    let strategy = env_strategy
        .as_deref()
        .or(defaults.default_strategy_raw.as_deref())
        .and_then(parse_strategy);

    let tier = strategy.map(|s| s.tier()).or_else(|| {
        env_tier
            .as_deref()
            .or(defaults.default_tier_raw.as_deref())
            .and_then(|raw| parse_tier(raw, host_os))
    });

    IsolationProfile {
        tier,
        strategy,
        policy: None,
    }
}

/// Read the `isolation.allow_tier_0` setting as a bool. Accepts "true",
/// "false", "1", "0", case-insensitive.
pub fn parse_allow_tier_0(raw: &str) -> bool {
    matches!(raw.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes")
}

/// Strategy parsing. Returns None for "auto"/empty so the caller falls
/// through to tier-based defaulting.
fn parse_strategy(raw: &str) -> Option<IsolationStrategy> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
        return None;
    }
    IsolationStrategy::from_id(trimmed)
}

/// Tier parsing with "auto" sentinel resolved to the per-OS default.
fn parse_tier(raw: &str, host_os: HostOs) -> Option<IsolationTier> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.eq_ignore_ascii_case("auto") {
        return Some(default_tier_for_os(host_os));
    }
    IsolationTier::from_id(trimmed)
}

/// Split a comma-separated allowlist string into trimmed entries, dropping
/// empties. Used for `isolation.allowlist_default`.
pub fn parse_allowlist(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_clear(key: &str) {
        // SAFETY: tests run in the same process; we don't expect concurrent
        // readers of these env vars during the unit test pass.
        unsafe {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn auto_tier_resolves_per_os() {
        env_clear(ENV_DEFAULT_TIER);
        env_clear(ENV_DEFAULT_STRATEGY);
        let defaults =
            UserDefaults::from_strings(Some("auto"), Some("auto"), true, None, None);
        let p = user_profile(&defaults, HostOs::Linux);
        assert_eq!(p.tier, Some(IsolationTier::Container));
        assert_eq!(p.strategy, None);
    }

    #[test]
    fn explicit_tier_string_wins_over_auto() {
        env_clear(ENV_DEFAULT_TIER);
        env_clear(ENV_DEFAULT_STRATEGY);
        let defaults =
            UserDefaults::from_strings(Some("process"), Some("auto"), true, None, None);
        let p = user_profile(&defaults, HostOs::Linux);
        assert_eq!(p.tier, Some(IsolationTier::Process));
    }

    #[test]
    fn strategy_pin_implies_its_tier() {
        env_clear(ENV_DEFAULT_TIER);
        env_clear(ENV_DEFAULT_STRATEGY);
        let defaults =
            UserDefaults::from_strings(Some("auto"), Some("docker"), true, None, None);
        let p = user_profile(&defaults, HostOs::Linux);
        assert_eq!(p.strategy, Some(IsolationStrategy::DockerEngine));
        assert_eq!(p.tier, Some(IsolationTier::Container));
    }

    #[test]
    fn allow_tier_0_parsing() {
        assert!(parse_allow_tier_0("true"));
        assert!(parse_allow_tier_0("TRUE"));
        assert!(parse_allow_tier_0("1"));
        assert!(!parse_allow_tier_0("false"));
        assert!(!parse_allow_tier_0(""));
        assert!(!parse_allow_tier_0("0"));
    }

    #[test]
    fn allowlist_splitter_trims_and_drops_empties() {
        let raw = "  api.anthropic.com, ,*.github.com,";
        let v = parse_allowlist(raw);
        assert_eq!(v, vec!["api.anthropic.com", "*.github.com"]);
    }

    #[test]
    fn unknown_strategy_string_is_ignored() {
        env_clear(ENV_DEFAULT_TIER);
        env_clear(ENV_DEFAULT_STRATEGY);
        let defaults = UserDefaults::from_strings(
            Some("auto"),
            Some("totally-made-up"),
            true,
            None,
            None,
        );
        let p = user_profile(&defaults, HostOs::Linux);
        assert_eq!(p.strategy, None);
        // tier falls back to auto -> per-OS
        assert_eq!(p.tier, Some(IsolationTier::Container));
    }
}
