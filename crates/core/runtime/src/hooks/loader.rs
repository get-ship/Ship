//! Hook loader — scan `.ship/hooks/*.jsonc` and return validated [`HookDef`]s.

use std::path::Path;

use ship_types::event_types::ALL as KNOWN_EVENT_TYPES;

use super::def::{HookAction, HookActionType, HookDef};

/// A single hook that failed to load, with a human-readable reason.
#[derive(Debug, Clone)]
pub struct LoadError {
    pub file: String,
    pub reason: String,
}

/// Result of scanning a hooks directory: parsed hooks plus any failures.
#[derive(Debug, Default)]
pub struct LoadReport {
    pub hooks: Vec<HookDef>,
    pub errors: Vec<LoadError>,
}

/// Load every `*.jsonc` / `*.json` hook under `<ship_dir>/hooks/`.
///
/// Invalid hooks are collected in `errors` — the loader never panics. Missing
/// directory is not an error; it returns an empty report.
pub fn load_hooks_dir(ship_dir: &Path) -> LoadReport {
    let hooks_dir = ship_dir.join("hooks");
    let mut report = LoadReport::default();
    let entries = match std::fs::read_dir(&hooks_dir) {
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
    compiler::jsonc::from_jsonc_str::<HookDef>(jsonc).map_err(|e| format!("parse failed: {e}"))
}

/// Validate event type, actions, and template files. Pure, takes the ship
/// directory so template existence checks can resolve relative paths.
pub fn validate_hook(hook: &HookDef, ship_dir: &Path) -> Result<(), String> {
    if hook.name.trim().is_empty() {
        return Err("hook name is required".to_string());
    }
    if !is_known_event(&hook.trigger.event) {
        return Err(format!("unknown event type '{}'", hook.trigger.event));
    }
    for (idx, action) in hook.actions.iter().enumerate() {
        validate_action(action, idx, ship_dir)?;
    }
    Ok(())
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
    KNOWN_EVENT_TYPES.iter().any(|e| *e == event)
}

fn log_errors(report: &LoadReport) {
    for err in &report.errors {
        tracing::warn!(file = %err.file, reason = %err.reason, "hooks: invalid hook skipped");
    }
}
