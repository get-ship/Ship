//! Composition-root loaders for ship-isolation decision inputs.
//!
//! ship-isolation is pure: settings + capability cache live here (daemon
//! settings KV) and are passed in as plain structs.

use anyhow::Result;
use ship_isolation::decision::{
    CAPS_CACHE_KEY, DecisionInputs, decision_inputs_from_strings, probe_capabilities,
    serialize_capabilities,
};
use ship_isolation::strategies_config::{SETTINGS_KEY, StrategiesSettings};
use ship_substrate::db::daemon_settings;

fn string_setting(key: &str) -> Result<Option<String>> {
    Ok(match daemon_settings::get(key)? {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Bool(b)) => Some(b.to_string()),
        Some(v) => Some(v.to_string()),
        None => None,
    })
}

/// Load decision inputs from daemon_settings. When `rescan` is true, force a
/// live capability probe and refresh the cache; otherwise use the cached
/// snapshot (or probe+write on first use).
pub fn load_decision_inputs(rescan: bool) -> Result<DecisionInputs> {
    let caps_cache = if rescan {
        let probed = probe_capabilities();
        if let Ok(serialized) = serialize_capabilities(&probed) {
            let _ = daemon_settings::set(CAPS_CACHE_KEY, &serde_json::Value::String(serialized));
        }
        // Pass None so decision_inputs_from_strings re-probes... actually we
        // already probed; inject via decision_inputs_from_strings by serializing.
        Some(serialize_capabilities(&probed)?)
    } else {
        match daemon_settings::get(CAPS_CACHE_KEY)? {
            Some(serde_json::Value::String(s)) => Some(s),
            _ => {
                // First use: probe and persist.
                let probed = probe_capabilities();
                if let Ok(serialized) = serialize_capabilities(&probed) {
                    let _ = daemon_settings::set(
                        CAPS_CACHE_KEY,
                        &serde_json::Value::String(serialized.clone()),
                    );
                    Some(serialized)
                } else {
                    None
                }
            }
        }
    };

    let tier = string_setting("isolation.default_tier")?;
    let strategy = string_setting("isolation.default_strategy")?;
    let allow_tier_0 = string_setting("isolation.allow_tier_0")?;
    let network = string_setting("isolation.network_default")?;
    let allowlist = string_setting("isolation.allowlist_default")?;

    Ok(decision_inputs_from_strings(
        tier.as_deref(),
        strategy.as_deref(),
        allow_tier_0.as_deref(),
        network.as_deref(),
        allowlist.as_deref(),
        caps_cache.as_deref(),
    ))
}

/// Read the nested per-strategy settings blob. Defaults when unset.
pub fn load_strategies_settings() -> Result<StrategiesSettings> {
    let raw = daemon_settings::get(SETTINGS_KEY)?;
    match raw {
        Some(value) => Ok(serde_json::from_value(value).unwrap_or_default()),
        None => Ok(Default::default()),
    }
}

/// Master switch `isolation.enforce`. Defaults to false.
pub fn load_enforcement_enabled() -> bool {
    string_setting("isolation.enforce")
        .ok()
        .flatten()
        .as_deref()
        .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(false)
}
