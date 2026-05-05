//! Lightweight project bootstrap and registry-backed init flow.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use ship_substrate::project::{SHIP_DIR_NAME, skills_dir, vision_doc_path};

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Lightweight project bootstrap for runtime unit tests.
/// The full production project scaffolding lives in `ship-module-project`.
pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    let ship_path = base_dir.join(SHIP_DIR_NAME);
    fs::create_dir_all(&ship_path)?;

    for rel in [
        "project/adrs",
        "project/specs",
        "project/features",
        "project/releases",
        "project/notes",
        "skills",
        "rules",
    ] {
        fs::create_dir_all(ship_path.join(rel))?;
    }

    write_if_missing(
        &ship_path.join("project/features/TEMPLATE.md"),
        "+++\nrelease_id = \"\"\n+++\n\n## Why\n\n## Delivery Todos\n",
    )?;
    write_if_missing(
        &ship_path.join("project/releases/TEMPLATE.md"),
        "+++\nversion = \"\"\n+++\n\n## Scope\n",
    )?;
    write_if_missing(
        &ship_path.join("project/notes/TEMPLATE.md"),
        "+++\ntitle = \"\"\n+++\n\n",
    )?;
    write_if_missing(
        &vision_doc_path(&ship_path),
        "# Vision\n\nDescribe what this project is trying to achieve.\n",
    )?;

    // Write ship.toml (with a stable project ID) BEFORE any DB access so that
    // project_db_key can read the ID and derive a stable state directory path.
    if !ship_path.join(crate::config::PRIMARY_CONFIG_FILE).exists() {
        let config = crate::config::ProjectConfig {
            id: crate::gen_nanoid(),
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_path.clone()))?;
    }

    write_if_missing(
        &skills_dir(&ship_path).join("task-policy").join("SKILL.md"),
        r#"---
name: task-policy
stable-id: task-policy
description: Ship workflow policy and execution guardrails for daily delivery.
tags: [ship, workflow, builtin]
authors: [ship]
metadata:
  display_name: Ship Workflow Policy
  source: builtin
---

# Ship Workflow Policy

Use Ship as the system of record for workflow state changes.

## Canonical Flow

Vision -> Release -> Feature -> Spec -> Close Feature -> Ship Release
"#,
    )?;

    let config = crate::config::get_config(Some(ship_path.clone()))?;
    crate::config::ensure_standard_dirs(&ship_path)?;
    crate::config::generate_gitignore(&ship_path, &config.git)?;

    Ok(ship_path)
}

/// Result of [`init_project_with_registry`].
#[derive(Debug)]
pub struct InitWithRegistryResult {
    pub ship_dir: PathBuf,
    /// Number of default dependencies seeded into ship.jsonc.
    pub deps_seeded: usize,
    /// Whether registry install succeeded. `false` means offline fallback was used.
    pub registry_installed: bool,
}

/// Full project init with registry-backed default dependencies.
///
/// 1. Scaffold `.ship/` directories and default files via [`init_project`].
/// 2. Seed `@ship/*` default dependencies into `ship.jsonc`.
/// 3. Attempt to install them via the registry.
/// 4. If install fails (offline/unreachable), the bare scaffold skills from
///    step 1 remain as fallback.
///
/// Idempotent: safe to call multiple times on the same directory.
pub fn init_project_with_registry(base_dir: PathBuf) -> Result<InitWithRegistryResult> {
    let ship_dir = init_project(base_dir)?;

    // Seed default dependencies — idempotent, skips if deps already present.
    let seed = crate::registry::init_deps::seed_default_dependencies(&ship_dir)?;
    let deps_seeded = seed.added;

    // Best-effort install. Failure is not an error — bare scaffold skills
    // remain as the offline fallback.
    let registry_installed =
        crate::registry::init_deps::try_install_init_deps(&ship_dir).unwrap_or_default();

    Ok(InitWithRegistryResult {
        ship_dir,
        deps_seeded,
        registry_installed,
    })
}
