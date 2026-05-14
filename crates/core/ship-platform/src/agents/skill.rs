//! Skill primitive — callable instruction document stored under
//! `.ship/skills/<id>/SKILL.md` (project) or `~/.ship/skills/<id>/SKILL.md`
//! (user). See submodules for storage, install, and types.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};

pub mod artifact_events;
mod install;
mod storage;
mod types;

pub use install::{SkillInstallScope, install_skill_from_source};
pub use types::{Skill, SkillSource};

use storage::{
    ensure_project_skills_storage, ensure_user_skills_storage, list_skills_from_dir,
    parse_skill_spec, skill_dir, user_skill_dir, write_skill_spec,
};

use ship_substrate::events::EventEnvelope;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_types::{SKILL_CREATED, SKILL_DELETED, SKILL_UPDATED};
use ship_types::payload::{SkillCreated, SkillDeleted, SkillUpdated};

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

/// Best-effort event emit. Failures are warned, never propagated —
/// the on-disk write is the source of truth, the event is audit/sync
/// scaffolding. Callers stay decoupled from event-store availability.
fn emit_skill_event(event_type: &'static str, id: &str, payload: &impl serde::Serialize) {
    let envelope = match EventEnvelope::new(event_type, id, payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("skill event {event_type}: build envelope failed: {e}");
            return;
        }
    };
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("skill event {event_type}: open store failed: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
        tracing::warn!("skill event {event_type}: append failed: {e}");
    }
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn list_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let dir = ensure_project_skills_storage(project_dir)?;
    list_skills_from_dir(&dir)
}

pub fn list_user_skills() -> Result<Vec<Skill>> {
    let dir = ensure_user_skills_storage()?;
    list_skills_from_dir(&dir)
}

/// Returns merged user + project skills keyed by id.
/// Project-scoped skills override user-scoped skills with the same id.
pub fn list_effective_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let mut by_id: HashMap<String, Skill> = HashMap::new();
    for skill in list_user_skills()? {
        by_id.insert(skill.id.clone(), skill);
    }
    for skill in list_skills(project_dir)? {
        by_id.insert(skill.id.clone(), skill);
    }
    let mut merged = by_id.into_values().collect::<Vec<_>>();
    merged.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(merged)
}

pub fn get_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill_spec(&dir)
}

pub fn get_user_skill(id: &str) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill_spec(&dir)
}

/// Resolve a skill by checking project scope first, then user scope.
pub fn get_effective_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let _ = ensure_user_skills_storage()?;
    let local_dir = skill_dir(project_dir, id);
    if local_dir.exists() {
        return parse_skill_spec(&local_dir);
    }

    let global_dir = user_skill_dir(id)?;
    if global_dir.exists() {
        return parse_skill_spec(&global_dir);
    }

    Err(anyhow!("Skill '{}' not found in project or user scope", id))
}

pub fn create_skill(project_dir: &Path, id: &str, name: &str, content: &str) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("Project skill '{}'", id)),
        version: None,
        authors: vec!["local".to_string()],
        stable_id: Some(id.to_string()),
        tags: vec!["custom".to_string()],
        audience: None,
        content: content.to_string(),
        source: SkillSource::Custom,
        artifacts: vec![],
    };
    write_skill_spec(&dir, &skill)?;
    emit_skill_event(
        SKILL_CREATED,
        id,
        &SkillCreated {
            scope: "project".to_string(),
            name: name.to_string(),
            content_hash: content_hash_hex(content),
        },
    );
    // Register in project config so checkout hook includes this skill automatically.
    let mut config = crate::config::get_config(Some(project_dir.to_path_buf()))?;
    if !config.agent.skills.contains(&id.to_string()) {
        config.agent.skills.push(id.to_string());
        crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
    }
    Ok(skill)
}

pub fn create_user_skill(id: &str, name: &str, content: &str) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("User skill '{}'", id)),
        version: None,
        authors: vec!["local".to_string()],
        stable_id: Some(id.to_string()),
        tags: vec!["custom".to_string()],
        audience: None,
        content: content.to_string(),
        source: SkillSource::Custom,
        artifacts: vec![],
    };
    write_skill_spec(&dir, &skill)?;
    emit_skill_event(
        SKILL_CREATED,
        id,
        &SkillCreated {
            scope: "user".to_string(),
            name: name.to_string(),
            content_hash: content_hash_hex(content),
        },
    );
    Ok(skill)
}

pub fn update_skill(
    project_dir: &Path,
    id: &str,
    name: Option<&str>,
    content: Option<&str>,
) -> Result<Skill> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    let mut skill = parse_skill_spec(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill_spec(&dir, &skill)?;
    emit_skill_event(
        SKILL_UPDATED,
        id,
        &SkillUpdated {
            scope: "project".to_string(),
            content_hash: content_hash_hex(&skill.content),
        },
    );
    Ok(skill)
}

pub fn update_user_skill(id: &str, name: Option<&str>, content: Option<&str>) -> Result<Skill> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    let mut skill = parse_skill_spec(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill_spec(&dir, &skill)?;
    emit_skill_event(
        SKILL_UPDATED,
        id,
        &SkillUpdated {
            scope: "user".to_string(),
            content_hash: content_hash_hex(&skill.content),
        },
    );
    Ok(skill)
}

pub fn delete_skill(project_dir: &Path, id: &str) -> Result<()> {
    let _ = ensure_project_skills_storage(project_dir)?;
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    emit_skill_event(
        SKILL_DELETED,
        id,
        &SkillDeleted {
            scope: "project".to_string(),
        },
    );
    Ok(())
}

pub fn delete_user_skill(id: &str) -> Result<()> {
    let _ = ensure_user_skills_storage()?;
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    emit_skill_event(
        SKILL_DELETED,
        id,
        &SkillDeleted {
            scope: "user".to_string(),
        },
    );
    Ok(())
}


#[cfg(test)]
mod tests;
