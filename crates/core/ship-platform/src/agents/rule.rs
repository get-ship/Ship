use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use ship_substrate::events::EventEnvelope;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_types::{RULE_CHANGED, RULE_CREATED, RULE_DELETED, RULE_UPDATED};
use ship_types::payload::{RuleChanged, RuleCreated, RuleDeleted, RuleUpdated};
use specta::Type;
use std::fs;
use std::path::PathBuf;

/// Strip the optional `NNNN-` numeric prefix and `.md` suffix to get the
/// canonical rule id used as the event entity_id.
fn rule_id_from_file(file_name: &str) -> String {
    let trimmed = file_name.trim_end_matches(".md");
    if let Some((prefix, rest)) = trimmed.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    trimmed.to_string()
}

fn content_hash_hex(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let bytes = hasher.finalize();
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Best-effort event emit. Failures warn-only — file write is the truth.
fn emit_rule_event(event_type: &'static str, id: &str, payload: &impl serde::Serialize) {
    let envelope = match EventEnvelope::new(event_type, id, payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("rule event {event_type}: build envelope failed: {e}");
            return;
        }
    };
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("rule event {event_type}: open store failed: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
        tracing::warn!("rule event {event_type}: append failed: {e}");
    }
}

// ─── Data types ───────────────────────────────────────────────────────────────

/// A rule file from `rules/*.md`. Rules are always active for the agent.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Rule {
    pub file_name: String,
    pub path: String,
    pub content: String,
}

// ─── Paths ────────────────────────────────────────────────────────────────────

fn rules_dir(ship_dir: &std::path::Path) -> PathBuf {
    ship_substrate::project::rules_dir(ship_dir)
}

fn normalize_rule_file_name(file_name: &str) -> Result<String> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Rule file name cannot be empty"));
    }

    let path = std::path::Path::new(trimmed);
    let components: Vec<_> = path.components().collect();
    if components.len() != 1 || !matches!(components[0], std::path::Component::Normal(_)) {
        return Err(anyhow!(
            "Rule file name must be a single file name without directories"
        ));
    }

    if trimmed.contains('\\') {
        return Err(anyhow!("Rule file name cannot contain path separators"));
    }

    let normalized = if trimmed.to_ascii_lowercase().ends_with(".md") {
        trimmed.to_string()
    } else if !trimmed.contains('.') {
        format!("{}.md", trimmed)
    } else {
        return Err(anyhow!(
            "Rule file name must end in .md (or omit extension)"
        ));
    };

    if normalized.eq_ignore_ascii_case("README.md") {
        return Err(anyhow!(
            "README.md is reserved and cannot be managed as a rule"
        ));
    }

    Ok(normalized)
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn list_rules(ship_dir: PathBuf) -> Result<Vec<Rule>> {
    let dir = rules_dir(&ship_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if file_name == "README.md" {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_default();
            rules.push(Rule {
                file_name,
                path: path.to_string_lossy().to_string(),
                content,
            });
        }
    }
    rules.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(rules)
}

pub fn get_rule(ship_dir: PathBuf, file_name: &str) -> Result<Rule> {
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(&file_name);
    let content = fs::read_to_string(&path)?;
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content,
    })
}

pub fn create_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let file_name = normalize_rule_file_name(file_name)?;
    let dir = rules_dir(&ship_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(&file_name);
    ship_substrate::fs_util::write_atomic(&path, content)?;
    emit_rule_event(
        RULE_CREATED,
        &rule_id_from_file(&file_name),
        &RuleCreated {
            file_name: file_name.clone(),
            content_hash: content_hash_hex(content),
        },
    );
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn update_rule(ship_dir: PathBuf, file_name: &str, content: &str) -> Result<Rule> {
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(&file_name);
    ship_substrate::fs_util::write_atomic(&path, content)?;
    emit_rule_event(
        RULE_UPDATED,
        &rule_id_from_file(&file_name),
        &RuleUpdated {
            file_name: file_name.clone(),
            content_hash: content_hash_hex(content),
        },
    );
    emit_rule_event(
        RULE_CHANGED,
        &rule_id_from_file(&file_name),
        &RuleChanged {
            path: path.to_string_lossy().to_string(),
            content_hash: content_hash_hex(content),
        },
    );
    Ok(Rule {
        file_name,
        path: path.to_string_lossy().to_string(),
        content: content.to_string(),
    })
}

pub fn delete_rule(ship_dir: PathBuf, file_name: &str) -> Result<()> {
    let file_name = normalize_rule_file_name(file_name)?;
    let path = rules_dir(&ship_dir).join(&file_name);
    fs::remove_file(path)?;
    emit_rule_event(
        RULE_DELETED,
        &rule_id_from_file(&file_name),
        &RuleDeleted {
            file_name: file_name.clone(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project;
    use tempfile::tempdir;

    #[test]
    fn create_get_update_delete_rule_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let created = create_rule(ship_dir.clone(), "runtime-hardening", "# Initial\n")?;
        assert_eq!(created.file_name, "runtime-hardening.md");

        let fetched = get_rule(ship_dir.clone(), "runtime-hardening")?;
        assert_eq!(fetched.file_name, "runtime-hardening.md");
        assert_eq!(fetched.content, "# Initial\n");

        let updated = update_rule(ship_dir.clone(), "runtime-hardening.md", "# Updated\n")?;
        assert_eq!(updated.file_name, "runtime-hardening.md");
        assert_eq!(updated.content, "# Updated\n");

        delete_rule(ship_dir.clone(), "runtime-hardening")?;
        assert!(
            get_rule(ship_dir, "runtime-hardening").is_err(),
            "rule should not exist after delete"
        );
        Ok(())
    }

    #[test]
    fn rule_crud_rejects_invalid_file_names() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        for invalid in ["", "README.md", "../escape.md", "nested/rule.md", "bad.txt"] {
            assert!(
                create_rule(ship_dir.clone(), invalid, "# Rule\n").is_err(),
                "create_rule should reject '{}'",
                invalid
            );
            assert!(
                get_rule(ship_dir.clone(), invalid).is_err(),
                "get_rule should reject '{}'",
                invalid
            );
            assert!(
                update_rule(ship_dir.clone(), invalid, "# Rule\n").is_err(),
                "update_rule should reject '{}'",
                invalid
            );
            assert!(
                delete_rule(ship_dir.clone(), invalid).is_err(),
                "delete_rule should reject '{}'",
                invalid
            );
        }

        Ok(())
    }
}
