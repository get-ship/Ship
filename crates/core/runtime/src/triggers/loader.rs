//! Trigger loader — scan `.ship/triggers/*.jsonc` and return validated [`TriggerDef`]s.

use std::path::Path;

use ship_types::event_types::ALL as KNOWN_EVENT_TYPES;

use super::def::{ActionKind, TriggerAction, TriggerDef};

/// A single trigger that failed to load, with a human-readable reason.
#[derive(Debug, Clone)]
pub struct LoadError {
    pub file: String,
    pub reason: String,
}

/// Result of scanning a triggers directory: parsed triggers plus any failures.
#[derive(Debug, Default)]
pub struct LoadReport {
    pub triggers: Vec<TriggerDef>,
    pub errors: Vec<LoadError>,
}

/// Load every `*.jsonc` / `*.json` trigger under `<ship_dir>/triggers/`.
///
/// Invalid triggers are collected in `errors` — the loader never panics.
/// Missing directory is not an error; it returns an empty report.
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
        match parse_trigger(&content) {
            Ok(trigger) => match validate_trigger(&trigger, ship_dir) {
                Ok(()) => report.triggers.push(trigger),
                Err(reason) => report.errors.push(LoadError { file, reason }),
            },
            Err(reason) => report.errors.push(LoadError { file, reason }),
        }
    }
    log_errors(&report);
    report
}

/// Parse a single JSONC trigger document.
pub fn parse_trigger(jsonc: &str) -> Result<TriggerDef, String> {
    compiler::jsonc::from_jsonc_str::<TriggerDef>(jsonc).map_err(|e| format!("parse failed: {e}"))
}

/// Validate event type, actions, and template files. Pure, takes the ship
/// directory so template existence checks can resolve relative paths.
pub fn validate_trigger(trigger: &TriggerDef, ship_dir: &Path) -> Result<(), String> {
    if trigger.name.trim().is_empty() {
        return Err("trigger name is required".to_string());
    }
    if !is_known_event(&trigger.trigger.event) {
        return Err(format!("unknown event type '{}'", trigger.trigger.event));
    }
    for (idx, action) in trigger.actions.iter().enumerate() {
        validate_action(action, idx, ship_dir)?;
    }
    Ok(())
}

fn validate_action(action: &TriggerAction, idx: usize, ship_dir: &Path) -> Result<(), String> {
    match &action.action_type {
        ActionKind::CollectionInsert {
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
        ActionKind::CollectionUpdate {
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
        ActionKind::EventEmit { event_type, .. } => {
            if event_type.trim().is_empty() {
                return Err(format!("action[{idx}] event.emit: event_type is required"));
            }
            Ok(())
        }
        ActionKind::PipelineTrigger { pipeline, .. } => {
            if pipeline.trim().is_empty() {
                return Err(format!("action[{idx}] pipeline.trigger: pipeline is required"));
            }
            Ok(())
        }
    }
}

fn is_known_event(event: &str) -> bool {
    KNOWN_EVENT_TYPES.iter().any(|e| *e == event)
}

fn log_errors(report: &LoadReport) {
    for err in &report.errors {
        tracing::warn!(file = %err.file, reason = %err.reason, "triggers: invalid definition skipped");
    }
}
