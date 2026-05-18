//! Trigger loader — scan `.ship/triggers/*.jsonc` and return validated [`HookDef`]s.

use std::path::Path;

use ship_types::event_types::ALL as KNOWN_EVENT_TYPES;

use super::def::{HookAction, HookActionType, HookDef, TriggerSourceDef};

/// A single hook that failed to load, with a human-readable reason.
#[derive(Debug, Clone)]
pub struct LoadError {
    pub file: String,
    pub reason: String,
}

/// Result of scanning a triggers directory: parsed hooks plus any failures.
#[derive(Debug, Default)]
pub struct LoadReport {
    pub hooks: Vec<HookDef>,
    pub errors: Vec<LoadError>,
}

/// Load every `*.jsonc` / `*.json` trigger definition under
/// `<ship_dir>/triggers/`.
///
/// Invalid hooks are collected in `errors` — the loader never panics. Missing
/// directory is not an error; it returns an empty report.
pub fn load_triggers_dir(ship_dir: &Path) -> LoadReport {
    let triggers_dir = ship_dir.join("triggers");
    let mut report = LoadReport::default();
    let entries = match std::fs::read_dir(&triggers_dir) {
        Ok(it) => it,
        Err(_) => return report,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext_ok = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "jsonc" || e == "json")
            .unwrap_or(false);
        if !ext_ok {
            continue;
        }

        let file = path.display().to_string();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                report.errors.push(LoadError {
                    file,
                    reason: format!("read failed: {e}"),
                });
                continue;
            }
        };
        match parse_hook(&content) {
            Ok(hook) => match validate_hook(&hook, ship_dir) {
                Ok(()) => report.hooks.push(hook),
                Err(reason) => report.errors.push(LoadError { file, reason }),
            },
            Err(reason) => report.errors.push(LoadError { file, reason }),
        }
    }
    log_errors(&report);
    report
}

/// Parse a single JSONC hook document.
pub fn parse_hook(jsonc: &str) -> Result<HookDef, String> {
    serde_json::from_str::<HookDef>(&ship_substrate::fs_util::strip_jsonc_comments(jsonc))
        .map_err(|e| format!("parse failed: {e}"))
}

/// Validate event type, actions, and template files. Pure, takes the ship
/// directory so template existence checks can resolve relative paths.
pub fn validate_hook(hook: &HookDef, ship_dir: &Path) -> Result<(), String> {
    if hook.name.trim().is_empty() {
        return Err("hook name is required".to_string());
    }
    match (&hook.trigger.event, &hook.trigger.source) {
        (Some(_), Some(_)) => {
            return Err(
                "trigger must have either 'event' or 'source', not both".to_string(),
            );
        }
        (None, None) => {
            return Err("trigger must have either 'event' or 'source'".to_string());
        }
        (Some(event), None) => {
            if !is_known_event(event) {
                return Err(format!("unknown event type '{event}'"));
            }
        }
        (None, Some(source)) => {
            validate_source(source)?;
        }
    }
    for (idx, action) in hook.actions.iter().enumerate() {
        validate_action(action, idx, ship_dir)?;
    }
    Ok(())
}

fn validate_source(source: &TriggerSourceDef) -> Result<(), String> {
    match source {
        TriggerSourceDef::Webhook(w) => {
            if !w.path.starts_with("/webhooks/") {
                return Err(format!(
                    "webhook source path must start with '/webhooks/', got '{}'",
                    w.path
                ));
            }
        }
        TriggerSourceDef::File(f) => {
            if f.watch.trim().is_empty() {
                return Err("file source: 'watch' path is required".to_string());
            }
        }
        TriggerSourceDef::Cron(c) => {
            if c.expression.trim().is_empty() {
                return Err("cron source: 'expression' is required".to_string());
            }
            normalize_and_parse_cron(&c.expression)
                .map_err(|e| format!("cron source: invalid expression '{}': {e}", c.expression))?;
        }
    }
    Ok(())
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

fn validate_action(action: &HookAction, idx: usize, ship_dir: &Path) -> Result<(), String> {
    match &action.action_type {
        HookActionType::CollectionInsert {
            collection,
            template_file,
            ..
        } => {
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
        HookActionType::CollectionUpdate {
            collection, filter, ..
        } => {
            if collection.trim().is_empty() {
                return Err(format!("action[{idx}] collection.update: collection is required"));
            }
            if !filter.is_object() {
                return Err(format!(
                    "action[{idx}] collection.update: filter must be an object"
                ));
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

fn is_known_event(event: &str) -> bool {
    KNOWN_EVENT_TYPES.contains(&event)
}

fn log_errors(report: &LoadReport) {
    for err in &report.errors {
        tracing::warn!(file = %err.file, reason = %err.reason, "triggers: invalid trigger skipped");
    }
}
