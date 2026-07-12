use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The kernel event a trigger fires on. Mirrors `@ship/stack`'s
/// `TriggerEventMatch`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerEventMatch {
    pub event: String,
}

/// One action a trigger runs when its match fires. `action_type` is an open
/// tag (`"event.emit"` today; more may land) — the payload varies per type,
/// so it stays a flattened bag alongside the tag. Mirrors `@ship/stack`'s
/// `TriggerAction`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(flatten)]
    pub payload: BTreeMap<String, serde_json::Value>,
}

/// Where a trigger's match is raised from — an inbound webhook, a file
/// glob, or a cron schedule. Mirrors `@ship/stack`'s `TriggerSourceDef`
/// union (tagged on `kind`).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerSourceDef {
    Webhook {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        secret: Option<String>,
    },
    File {
        glob: String,
    },
    Cron {
        schedule: String,
    },
}

/// A trigger definition — the authoring shape replacing
/// `triggers/*.jsonc`. Mirrors `@ship/stack`'s `TriggerDef`
/// (census-format-migration.md Wave 3).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TriggerDef {
    pub id: String,
    pub trigger: TriggerEventMatch,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<TriggerSourceDef>,
    #[serde(default)]
    pub actions: Vec<TriggerAction>,
}
