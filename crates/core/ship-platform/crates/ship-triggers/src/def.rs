//! Hook definition types — the trigger engine's internal shape. Post
//! format-migration Wave 4 phase A, populated from the descriptor's
//! `Vec<stack_model::TriggerDef>` (the `defineTrigger` declare-form in
//! `ship.config.ts`) via [`HookDef::try_from_def`] — not from
//! `.ship/triggers/*.jsonc` files (census-format-migration.md §4 row 3).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use ship_types::event_registry;

/// A single hook definition loaded from `.ship/triggers/<name>.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub trigger: HookTrigger,
    #[serde(default)]
    pub actions: Vec<HookAction>,
}

/// What event (or external source) a hook listens for.
///
/// Exactly one of `event` or `source` must be set. Validated at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTrigger {
    /// Internal event type (e.g. `"pipeline.run.started"`). Mutually exclusive with `source`.
    #[serde(default)]
    pub event: Option<String>,
    /// External trigger source. Mutually exclusive with `event`.
    #[serde(default)]
    pub source: Option<TriggerSourceDef>,
    /// Optional dot-path filter on the event payload.
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

impl HookTrigger {
    /// Returns true if this is a source-based (external) trigger.
    pub fn is_source(&self) -> bool {
        self.source.is_some()
    }

    /// The event type this trigger matches in the HookEngine dispatch loop.
    ///
    /// For event triggers: the `event` field value directly.
    /// For source triggers: `"external.<source_kind>.<hook_name>"`.
    pub fn dispatch_event_type(&self, hook_name: &str) -> Option<String> {
        if let Some(e) = &self.event {
            return Some(e.clone());
        }
        if let Some(s) = &self.source {
            return Some(format!("external.{}.{}", s.kind_str(), hook_name));
        }
        None
    }
}

// ── Source kinds ─────────────────────────────────────────────────────────────

/// An external trigger source definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerSourceDef {
    Webhook(WebhookSourceDef),
    File(FileSourceDef),
    Cron(CronSourceDef),
}

impl TriggerSourceDef {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Webhook(_) => "webhook",
            Self::File(_) => "file",
            Self::Cron(_) => "cron",
        }
    }
}

/// Webhook source: fires when an HTTP POST arrives at the given path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSourceDef {
    /// Route suffix under `/api/hooks/webhook/`. Must start with `/webhooks/`.
    pub path: String,
    /// Secret matched against the `X-Ship-Webhook-Secret` header.
    /// Use `env:VAR_NAME` to read from an environment variable.
    #[serde(default)]
    pub secret: Option<String>,
}

/// File source: fires when the watched path matches a filesystem event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSourceDef {
    /// Path or glob to watch. Relative paths resolve from the project root.
    pub watch: String,
    /// Event kinds to react to. Defaults to all three if empty.
    #[serde(default)]
    pub events: Vec<FileEventKind>,
    /// Coalescing debounce in milliseconds. Defaults to 200.
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

impl FileSourceDef {
    /// True if the given event kind should trigger this hook.
    pub fn matches_kind(&self, kind: &FileEventKind) -> bool {
        self.events.is_empty() || self.events.contains(kind)
    }
}

fn default_debounce_ms() -> u64 {
    200
}

/// Filesystem event kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
}

/// Cron source: fires on a schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSourceDef {
    /// Standard 5-field cron expression (min hour dom month dow).
    /// Optionally prefix with a seconds field (6 fields total).
    pub expression: String,
    /// IANA timezone name. Defaults to `"UTC"`.
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

// ── Actions ──────────────────────────────────────────────────────────────────

/// A single action step in a hook's action list.
///
/// The action variant is flattened, so a JSONC action reads naturally:
///
/// ```jsonc
/// { "type": "collection.insert", "collection": "notes", "data": { ... } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    #[serde(flatten)]
    pub action_type: HookActionType,
    #[serde(default)]
    pub when: Option<String>,
}

/// All supported action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookActionType {
    #[serde(rename = "collection.insert")]
    CollectionInsert {
        collection: String,
        #[serde(default)]
        data: serde_json::Value,
        #[serde(default)]
        template_file: Option<String>,
    },
    #[serde(rename = "collection.update")]
    CollectionUpdate {
        collection: String,
        filter: serde_json::Value,
        data: serde_json::Value,
    },
    #[serde(rename = "event.emit")]
    EventEmit {
        event_type: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
    #[serde(rename = "pipeline.trigger")]
    PipelineTrigger {
        pipeline: String,
        #[serde(default)]
        args: Option<serde_json::Value>,
    },
}

impl HookActionType {
    /// Short action kind used in logs and test assertions.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CollectionInsert { .. } => "collection.insert",
            Self::CollectionUpdate { .. } => "collection.update",
            Self::EventEmit { .. } => "event.emit",
            Self::PipelineTrigger { .. } => "pipeline.trigger",
        }
    }
}

// ── Descriptor conversion (format-migration Wave 4 phase A) ─────────────────
//
// `stack_model::TriggerDef` is the `defineTrigger` declare-form's Rust shape
// (census-format-migration.md Wave 3); `try_from_def` maps it onto this
// module's `HookDef`, applying the same validation the retired JSONC loader
// ran at load time (unknown-event rejection, source shape checks, cron
// expression parsing, per-action field checks, template_file existence).

impl HookDef {
    /// Convert one descriptor trigger into the engine's internal shape.
    /// `ship_dir` resolves a `collection.insert` action's `template_file`
    /// (relative paths are checked for existence, same as the old loader).
    pub fn try_from_def(def: &stack_model::TriggerDef, ship_dir: &std::path::Path) -> Result<Self, String> {
        if def.id.trim().is_empty() {
            return Err("trigger id is required".to_string());
        }
        let trigger = convert_trigger_match(def)?;
        let mut actions = Vec::with_capacity(def.actions.len());
        for (idx, a) in def.actions.iter().enumerate() {
            let action = convert_action(a).map_err(|e| format!("action[{idx}]: {e}"))?;
            validate_converted_action(&action, idx, ship_dir)?;
            actions.push(action);
        }
        Ok(HookDef { name: def.id.clone(), description: None, trigger, actions })
    }
}

/// `stack_model::TriggerEventMatch.event` is a required string in the TS-facing
/// shape — there is no way to author a source-only trigger (cron/file/webhook,
/// no `event`) from `ship.config.ts` today. That's a real gap in Wave 1/3's
/// shape, not something this cutover can fix; when a `source` is present it
/// wins and `event` is dropped, so a descriptor author who sets both gets
/// source-triggered behavior rather than a hard validation error.
fn convert_trigger_match(def: &stack_model::TriggerDef) -> Result<HookTrigger, String> {
    if let Some(source_def) = &def.source {
        return Ok(HookTrigger {
            event: None,
            source: Some(convert_source(source_def)?),
            filter: None,
        });
    }
    if def.trigger.event.trim().is_empty() {
        return Err("trigger.event must not be empty".to_string());
    }
    if event_registry::lookup(&def.trigger.event).is_none() {
        return Err(format!("unknown event type '{}'", def.trigger.event));
    }
    Ok(HookTrigger {
        event: Some(def.trigger.event.clone()),
        source: None,
        // No TS shape for the dot-path payload filter yet — every
        // descriptor-sourced trigger fires unfiltered.
        filter: None,
    })
}

fn convert_source(s: &stack_model::TriggerSourceDef) -> Result<TriggerSourceDef, String> {
    match s {
        stack_model::TriggerSourceDef::Webhook { path, secret } => {
            if !path.starts_with("/webhooks/") {
                return Err(format!(
                    "webhook source path must start with '/webhooks/', got '{path}'"
                ));
            }
            Ok(TriggerSourceDef::Webhook(WebhookSourceDef {
                path: path.clone(),
                secret: secret.clone(),
            }))
        }
        stack_model::TriggerSourceDef::File { glob } => {
            if glob.trim().is_empty() {
                return Err("file source: 'glob' is required".to_string());
            }
            Ok(TriggerSourceDef::File(FileSourceDef {
                watch: glob.clone(),
                events: vec![],
                debounce_ms: default_debounce_ms(),
            }))
        }
        stack_model::TriggerSourceDef::Cron { schedule } => {
            if schedule.trim().is_empty() {
                return Err("cron source: 'schedule' is required".to_string());
            }
            normalize_and_parse_cron(schedule)
                .map_err(|e| format!("cron source: invalid expression '{schedule}': {e}"))?;
            Ok(TriggerSourceDef::Cron(CronSourceDef {
                expression: schedule.clone(),
                timezone: default_timezone(),
            }))
        }
    }
}

fn convert_action(a: &stack_model::TriggerAction) -> Result<HookAction, String> {
    let mut payload = a.payload.clone();
    let when = payload
        .remove("when")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let mut value = serde_json::to_value(&payload).map_err(|e| e.to_string())?;
    if let Value::Object(map) = &mut value {
        map.insert("type".to_string(), Value::String(a.action_type.clone()));
    }
    let action_type: HookActionType = serde_json::from_value(value)
        .map_err(|e| format!("action type '{}': {e}", a.action_type))?;
    Ok(HookAction { action_type, when })
}

fn validate_converted_action(action: &HookAction, idx: usize, ship_dir: &std::path::Path) -> Result<(), String> {
    match &action.action_type {
        HookActionType::CollectionInsert { collection, template_file, .. } => {
            if collection.trim().is_empty() {
                return Err(format!("action[{idx}] collection.insert: collection is required"));
            }
            if let Some(tf) = template_file {
                let path = ship_dir.join(tf);
                if !path.is_file() {
                    return Err(format!(
                        "action[{idx}] collection.insert: template_file not found: {tf}"
                    ));
                }
            }
            Ok(())
        }
        HookActionType::CollectionUpdate { collection, filter, .. } => {
            if collection.trim().is_empty() {
                return Err(format!("action[{idx}] collection.update: collection is required"));
            }
            if !filter.is_object() {
                return Err(format!("action[{idx}] collection.update: filter must be an object"));
            }
            Ok(())
        }
        HookActionType::EventEmit { event_type, .. } => {
            if event_type.trim().is_empty() {
                return Err(format!("action[{idx}] event.emit: event_type is required"));
            }
            Ok(())
        }
        HookActionType::PipelineTrigger { pipeline, .. } => {
            if pipeline.trim().is_empty() {
                return Err(format!("action[{idx}] pipeline.trigger: pipeline is required"));
            }
            Ok(())
        }
    }
}

/// Normalize a 5-field cron expression to 6 fields (prepend seconds=0),
/// then parse it.
pub(crate) fn normalize_and_parse_cron(expr: &str) -> Result<cron::Schedule, String> {
    use std::str::FromStr;
    let normalized = if expr.split_whitespace().count() == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    };
    cron::Schedule::from_str(&normalized).map_err(|e| e.to_string())
}
