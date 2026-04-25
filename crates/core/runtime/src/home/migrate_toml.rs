//! Convert legacy `~/.ship/config.toml` into the canonical pair of JSONC
//! files: `library/settings.jsonc` (portable) and `machine/local.jsonc`
//! (machine-specific).
//!
//! Only used by the home-layout migrator (`home::migrate`) — the runtime
//! config loader no longer reads TOML at all.

use anyhow::{Context, Result};
use serde_json::{Map, Value};

/// Keys that belong in the portable `settings.jsonc`. Everything else is
/// machine-local and lands in `machine/local.jsonc`. The split mirrors the
/// ADR's `[dispatch]`/`[terminal]` portable vs. `[worktrees]` machine-local
/// partition.
const PORTABLE_TABLES: &[&str] = &["identity", "defaults", "terminal", "dispatch", "cloud"];
const MACHINE_TABLES: &[&str] = &["worktrees"];

/// Returns `(settings_jsonc, local_jsonc)`. Both are pretty-printed JSON with
/// a leading comment banner so manual edits are obvious.
pub fn split_user_config(toml_text: &str) -> Result<(String, String)> {
    let parsed: toml::Value = toml::from_str(toml_text).context("parse config.toml")?;
    let map: Map<String, Value> = match toml_to_json(parsed) {
        Value::Object(m) => m,
        _ => Map::new(),
    };

    let mut settings = Map::new();
    let mut local = Map::new();
    for (key, value) in map.into_iter() {
        if PORTABLE_TABLES.contains(&key.as_str()) {
            settings.insert(key, value);
        } else if MACHINE_TABLES.contains(&key.as_str()) {
            local.insert(key, value);
        } else {
            // Unknown sections default to portable — losing them silently
            // would surprise the user. Better to keep them in the human-edited
            // file where they're visible.
            settings.insert(key, value);
        }
    }

    Ok((render_jsonc(&settings, PORTABLE_BANNER)?, render_jsonc(&local, LOCAL_BANNER)?))
}

const PORTABLE_BANNER: &str =
    "// ~/.ship/library/settings.jsonc — portable user preferences.\n\
     // Migrated from config.toml. Edit freely; comments are preserved.\n";
const LOCAL_BANNER: &str =
    "// ~/.ship/machine/local.jsonc — machine-specific overrides.\n\
     // Migrated from config.toml. Not portable across hosts.\n";

fn render_jsonc(map: &Map<String, Value>, banner: &str) -> Result<String> {
    let body = serde_json::to_string_pretty(&Value::Object(map.clone()))?;
    Ok(format!("{banner}{body}\n"))
}

fn toml_to_json(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Number(i.into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Array(arr) => Value::Array(arr.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => {
            let mut m = Map::new();
            for (k, v) in t {
                m.insert(k, toml_to_json(v));
            }
            Value::Object(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_portable_and_machine_keys() {
        let toml = r#"
[identity]
name = "Alice"

[terminal]
program = "tmux"

[worktrees]
dir = "/tmp/wt"
"#;
        let (settings, local) = split_user_config(toml).unwrap();
        assert!(settings.contains("\"identity\""));
        assert!(settings.contains("\"terminal\""));
        assert!(!settings.contains("\"worktrees\""));
        assert!(local.contains("\"worktrees\""));
        assert!(local.contains("\"/tmp/wt\""));
    }

    #[test]
    fn unknown_section_lands_in_settings() {
        let toml = r#"
[experimental]
flag = true
"#;
        let (settings, local) = split_user_config(toml).unwrap();
        assert!(settings.contains("experimental"));
        assert!(!local.contains("experimental"));
    }

    #[test]
    fn empty_toml_yields_empty_objects() {
        let (settings, local) = split_user_config("").unwrap();
        assert!(settings.contains("{}"));
        assert!(local.contains("{}"));
    }

    #[test]
    fn round_trips_via_serde_json() {
        let toml = r#"
[dispatch]
confirm = true

[worktrees]
dir = "/x"
"#;
        let (settings, local) = split_user_config(toml).unwrap();
        // Strip leading comments before parsing.
        let s = settings
            .lines()
            .filter(|l| !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let l = local
            .lines()
            .filter(|l| !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let _: Value = serde_json::from_str(&s).unwrap();
        let _: Value = serde_json::from_str(&l).unwrap();
    }
}
