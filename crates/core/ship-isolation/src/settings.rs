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

/// Wrap a launch (`bin`, `args`) for a given strategy. The wrapper transforms
/// what the supervisor would have spawned directly into what it should spawn
/// *through* the chosen isolation strategy.
///
/// Returns `None` when the strategy is `Noop` (passthrough) or when the
/// strategy requires lifecycle management beyond simple argv rewriting
/// (Docker, Youki, Lima, Fly — those need `prepare`/`shutdown`). Callers
/// treat `None` as "supervisor should keep its existing launch".
///
/// For `SrtWrapper` / legacy `HarnessSandbox`: returns the bun + srt
/// invocation. Requires `bun_bin` resolved by the capability probe (or
/// settings) and a settings-file path the caller writes the srt JSON
/// to.
///
/// For `HarnessNative`: appends the harness's native sandbox flag (if
/// any) to args. The flag is resolved per (harness, OS) via
/// `harness::tier1_mechanism_for` — supervisor passes the right flag
/// via `WrapContext.harness_native_flag`. When `None`, no flag is
/// needed (e.g. codex on macOS uses Seatbelt internally).
pub fn wrap_launch(
    strategy: crate::tier::IsolationStrategy,
    bin: std::path::PathBuf,
    args: Vec<String>,
    ctx: &WrapContext<'_>,
) -> Option<WrappedLaunch> {
    use crate::tier::IsolationStrategy::*;
    match strategy {
        Noop => None,
        HarnessNative => {
            // No bin/args rewrite — harness binary stays put. Append
            // the native flag (if any) to args. When the flag is None
            // (e.g. codex on macOS), there's nothing to do; the
            // harness sandboxes itself by default.
            match ctx.harness_native_flag {
                Some(flag) => {
                    let mut new_args = args;
                    new_args.push(flag.to_string());
                    Some(WrappedLaunch { bin, args: new_args })
                }
                None => Some(WrappedLaunch { bin, args }),
            }
        }
        SrtWrapper | HarnessSandbox => {
            let bun = ctx.bun_bin?;
            let srt = ctx.srt_entry?;
            let settings = ctx.srt_settings_path?;
            let mut new_args = vec![
                "run".to_string(),
                srt.to_string_lossy().into_owned(),
                "--settings".to_string(),
                settings.to_string_lossy().into_owned(),
                bin.to_string_lossy().into_owned(),
            ];
            new_args.extend(args);
            Some(WrappedLaunch {
                bin: bun.to_path_buf(),
                args: new_args,
            })
        }
        DockerEngine | YoukiRootless | Wsl2 | LimaVm | FlyMachines => None,
    }
}

/// Pre-resolved paths + flags the caller hands to `wrap_launch`. Each
/// field is optional because not every strategy needs every value; the
/// wrapper returns `None` when a required path is missing.
#[derive(Debug, Default, Clone, Copy)]
pub struct WrapContext<'a> {
    pub bun_bin: Option<&'a std::path::Path>,
    pub srt_entry: Option<&'a std::path::Path>,
    pub srt_settings_path: Option<&'a std::path::Path>,
    /// CLI flag for `HarnessNative`. `None` means "no flag" (codex on
    /// macOS) — still a valid wrap. The wrapper distinguishes
    /// `Some(None)` (no flag needed) from `Some(_)` (flag to add) by
    /// reading the WrapContext's literal `harness_native_flag` field.
    pub harness_native_flag: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct WrappedLaunch {
    pub bin: std::path::PathBuf,
    pub args: Vec<String>,
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
    fn wrap_noop_returns_none() {
        use crate::tier::IsolationStrategy;
        let out = wrap_launch(
            IsolationStrategy::Noop,
            std::path::PathBuf::from("/bin/echo"),
            vec!["hi".into()],
            &WrapContext::default(),
        );
        assert!(out.is_none());
    }

    #[test]
    fn wrap_harness_sandbox_builds_bun_srt_invocation() {
        use crate::tier::IsolationStrategy;
        let bun = std::path::PathBuf::from("/usr/local/bin/bun");
        let srt = std::path::PathBuf::from("/opt/srt/bin.js");
        let settings = std::path::PathBuf::from("/tmp/srt-1234.json");
        let out = wrap_launch(
            IsolationStrategy::HarnessSandbox,
            std::path::PathBuf::from("/usr/local/bin/claude"),
            vec!["-p".into(), "hello".into()],
            &WrapContext {
                bun_bin: Some(&bun),
                srt_entry: Some(&srt),
                srt_settings_path: Some(&settings),
                harness_native_flag: None,
            },
        )
        .expect("should wrap");
        assert_eq!(out.bin, bun);
        assert_eq!(out.args[0], "run");
        assert_eq!(out.args[1], "/opt/srt/bin.js");
        assert_eq!(out.args[2], "--settings");
        assert_eq!(out.args[3], "/tmp/srt-1234.json");
        assert_eq!(out.args[4], "/usr/local/bin/claude");
        assert_eq!(out.args[5], "-p");
        assert_eq!(out.args[6], "hello");
    }

    #[test]
    fn wrap_harness_sandbox_needs_all_paths() {
        use crate::tier::IsolationStrategy;
        let bun = std::path::PathBuf::from("/usr/local/bin/bun");
        let out = wrap_launch(
            IsolationStrategy::HarnessSandbox,
            std::path::PathBuf::from("/usr/local/bin/claude"),
            vec![],
            &WrapContext {
                bun_bin: Some(&bun),
                srt_entry: None,
                srt_settings_path: None,
                harness_native_flag: None,
            },
        );
        assert!(out.is_none(), "missing srt_entry should yield None");
    }

    #[test]
    fn wrap_harness_native_with_flag_appends_to_args() {
        use crate::tier::IsolationStrategy;
        let out = wrap_launch(
            IsolationStrategy::HarnessNative,
            std::path::PathBuf::from("/usr/local/bin/claude"),
            vec!["-p".into(), "hello".into()],
            &WrapContext {
                harness_native_flag: Some("--sandbox"),
                ..Default::default()
            },
        )
        .expect("should wrap");
        // Binary unchanged; flag appended to args.
        assert_eq!(out.bin, std::path::PathBuf::from("/usr/local/bin/claude"));
        assert_eq!(out.args, vec!["-p", "hello", "--sandbox"]);
    }

    #[test]
    fn wrap_harness_native_without_flag_is_passthrough() {
        // codex on macOS sandboxes itself with Seatbelt — no flag needed.
        use crate::tier::IsolationStrategy;
        let out = wrap_launch(
            IsolationStrategy::HarnessNative,
            std::path::PathBuf::from("/usr/local/bin/codex"),
            vec!["exec".into(), "hi".into()],
            &WrapContext::default(),
        )
        .expect("should wrap");
        assert_eq!(out.bin, std::path::PathBuf::from("/usr/local/bin/codex"));
        assert_eq!(out.args, vec!["exec", "hi"]);
    }

    #[test]
    fn wrap_srt_wrapper_matches_legacy_harness_sandbox() {
        use crate::tier::IsolationStrategy;
        let bun = std::path::PathBuf::from("/usr/local/bin/bun");
        let srt = std::path::PathBuf::from("/opt/srt/bin.js");
        let settings = std::path::PathBuf::from("/tmp/srt-x.json");
        let ctx = WrapContext {
            bun_bin: Some(&bun),
            srt_entry: Some(&srt),
            srt_settings_path: Some(&settings),
            harness_native_flag: None,
        };
        let a = wrap_launch(
            IsolationStrategy::SrtWrapper,
            std::path::PathBuf::from("/usr/local/bin/opencode"),
            vec!["go".into()],
            &ctx,
        )
        .expect("srt wrap");
        let b = wrap_launch(
            IsolationStrategy::HarnessSandbox,
            std::path::PathBuf::from("/usr/local/bin/opencode"),
            vec!["go".into()],
            &ctx,
        )
        .expect("legacy wrap");
        assert_eq!(a.bin, b.bin);
        assert_eq!(a.args, b.args);
    }

    #[test]
    fn wrap_container_strategies_return_none() {
        use crate::tier::IsolationStrategy;
        for s in [
            IsolationStrategy::DockerEngine,
            IsolationStrategy::YoukiRootless,
            IsolationStrategy::Wsl2,
            IsolationStrategy::LimaVm,
            IsolationStrategy::FlyMachines,
        ] {
            let out = wrap_launch(
                s,
                std::path::PathBuf::from("/bin/echo"),
                vec![],
                &WrapContext::default(),
            );
            assert!(out.is_none(), "{s:?} should not be wrappable via argv-only");
        }
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
