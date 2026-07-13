//! Static publish-time validation for the stack one-way valve.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, bail};
use serde_json::Value;

/// Validate that `root` contains only stack-tier, publishable content.
///
/// This walk is intentionally strict: publication is a one-way valve, so a
/// construct whose scope cannot be proven stack-local is refused with its path.
pub fn validate_publishable_stack(root: &Path) -> anyhow::Result<()> {
    if !root.join("stack.config.ts").is_file() {
        bail!(
            "no stack.config.ts found in {} — not a stack root",
            root.display()
        );
    }

    let rules_dir = root.join("rules");
    if rules_dir.exists() {
        bail!(
            "publish refused {}: rules/ is not publishable; rules must be inline on an agent",
            rules_dir.display()
        );
    }
    let hooks_dir = root.join("hooks");
    if hooks_dir.exists() {
        bail!(
            "publish refused {}: hooks must be agent-scoped under agents/<id>/hooks/",
            hooks_dir.display()
        );
    }
    let triggers_dir = root.join("triggers");
    if triggers_dir.exists() {
        bail!(
            "publish refused {}: triggers are not publishable stack vocabulary",
            triggers_dir.display()
        );
    }
    let mut files = collect_files(root)?;
    files.sort();
    for rel in files {
        validate_path(&rel)?;
        validate_content(root, &rel)?;
    }
    Ok(())
}

fn collect_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".git")
    {
        let entry = entry.context("walking stack publish tree")?;
        if entry.depth() == 0 {
            continue;
        }
        if entry.file_type().is_symlink() {
            bail!("publish refused symlink {}", entry.path().display());
        }
        if entry.file_type().is_file() {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .context("stripping publish root")?
                    .to_path_buf(),
            );
        }
    }
    Ok(files)
}

fn validate_path(rel: &Path) -> anyhow::Result<()> {
    let components: Vec<&str> = rel
        .components()
        .filter_map(|part| match part {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect();
    let display = display_path(rel);

    if components.contains(&"rules") {
        bail!(
            "publish refused {display}: rules/ is not publishable; rules must be inline on an agent"
        );
    }
    if components.first() == Some(&"triggers") {
        bail!("publish refused {display}: triggers are not publishable stack vocabulary");
    }
    if components.first() == Some(&"hooks") || is_nested_unscoped_hook(&components) {
        bail!("publish refused {display}: hooks must be agent-scoped under agents/<id>/hooks/");
    }
    if matches!(components.as_slice(), ["space.config.ts"] | ["ship.state"])
        || matches!(
            components.as_slice(),
            [".ship", "hooks.jsonc"] | [".ship", "permissions.jsonc"]
        )
    {
        bail!(
            "publish refused {display}: space-scoped content cannot cross the stack publish valve"
        );
    }
    Ok(())
}

fn is_nested_unscoped_hook(components: &[&str]) -> bool {
    let Some(position) = components
        .iter()
        .position(|component| *component == "hooks")
    else {
        return false;
    };
    position > 0
        && !(position == 2
            && components.first() == Some(&"agents")
            && components.get(1).is_some_and(|agent| !agent.is_empty()))
}

fn validate_content(root: &Path, rel: &Path) -> anyhow::Result<()> {
    let extension = rel
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if matches!(extension, "ts" | "js" | "mts" | "mjs") {
        let source = fs::read_to_string(root.join(rel))
            .with_context(|| format!("reading publish source {}", rel.display()))?;
        if source.contains("defineTrigger(") {
            bail!(
                "publish refused {}: defineTrigger is not publishable stack vocabulary",
                display_path(rel)
            );
        }
    }
    if matches!(extension, "json" | "jsonc") {
        validate_json_events(root, rel)?;
    }
    Ok(())
}

fn validate_json_events(root: &Path, rel: &Path) -> anyhow::Result<()> {
    let source = fs::read_to_string(root.join(rel))
        .with_context(|| format!("reading publish config {}", rel.display()))?;
    let cleaned = stack_model::jsonc::strip_jsonc_comments(&source);
    let value: Value = serde_json::from_str(&cleaned)
        .with_context(|| format!("parsing publish config {}", rel.display()))?;
    let mut events = Vec::new();
    collect_event_names(&value, None, &mut events);
    for event in events {
        if ship_types::event_registry::lookup(event).is_none() {
            bail!(
                "publish refused {}: custom event '{}' is not platform vocabulary",
                display_path(rel),
                event
            );
        }
    }
    Ok(())
}

fn collect_event_names<'a>(value: &'a Value, key: Option<&str>, events: &mut Vec<&'a str>) {
    match value {
        Value::Object(map) => {
            for (name, child) in map {
                if matches!(name.as_str(), "event" | "event_type") {
                    match child {
                        Value::String(event) => events.push(event),
                        Value::Object(nested) => {
                            if let Some(Value::String(event)) = nested.get("event") {
                                events.push(event);
                            }
                        }
                        _ => {}
                    }
                }
                collect_event_names(child, Some(name), events);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_event_names(child, key, events);
            }
        }
        Value::String(event) if matches!(key, Some("event") | Some("event_type")) => {
            events.push(event);
        }
        _ => {}
    }
}

fn display_path(path: &Path) -> String {
    path.components()
        .filter_map(|part| match part {
            Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
