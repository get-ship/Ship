// -- Config / Settings aggregates ---------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigChanged {
    pub subject: String,
    pub detail: Option<String>,
}

/// Fine-grained, per-file config change (Format-2 reactivity).
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFileChanged {
    pub domain: String,
    pub path: String,
    pub content_hash: String,
}

/// Per-space settings override (`space.setting.changed`).
#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceSettingChanged {
    pub key: String,
    pub value_json: String,
}

/// Unified scope-discriminated settings change (`settings.changed`).
///
/// The keystone settings event: one type across all three scopes (SPACE /
/// ACCOUNT / RUNTIME). The scope rides the ENVELOPE — `EventScope` is chosen at
/// emit and the kernel routes to the matching per-scope db — so `scope` here is
/// the human-readable echo for read-model projections and audit, not the routing
/// key. `entity_id` is the setting key; `value_json` is the canonical JSON value.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingChanged {
    pub key: String,
    pub value_json: String,
    /// The scope this setting was written in (`space` / `account` / `runtime`),
    /// matching `EventScope::as_str`.
    pub scope: String,
}

