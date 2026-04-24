//! Hook definition types — shape of a `.ship/hooks/*.jsonc` document.

use serde::{Deserialize, Serialize};

/// A single hook definition loaded from `.ship/hooks/<name>.jsonc`.
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
    /// Internal event type (e.g. `"job.failed"`). Mutually exclusive with `source`.
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
