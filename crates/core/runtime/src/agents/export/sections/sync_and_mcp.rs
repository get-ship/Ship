// Included by crates/core/runtime/src/agents/export/mod.rs.
// Top-level orchestration: SyncPayload, per-provider export and teardown,
// active-agent sync, and provider context-file writes.

pub struct SyncPayload {
    pub servers: Vec<McpServerConfig>,
    pub instruction_skill_id: Option<String>,
    pub instructions: Option<String>,
    pub hooks: Vec<HookConfig>,
    pub permissions: Permissions,
    pub active_agent_id: Option<String>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Write a context file (CLAUDE.md, GEMINI.md, Codex instructions, etc.) for the given provider.
///
/// Called by the git module after building provider-agnostic Markdown content.
/// Each provider has a specific destination:
/// - Claude  → `CLAUDE.md` at project root
/// - Gemini  → `GEMINI.md` at project root
/// - Codex / Roo / Amp / Goose → `AGENTS.md` at project root
/// - Unknown provider / `PromptOutput::None` → no-op
pub fn write_context(project_root: &Path, provider_id: &str, content: &str) -> Result<()> {
    let desc = match get_provider(provider_id) {
        Some(d) => d,
        None => return Ok(()),
    };
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let path = project_root.join("CLAUDE.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::GeminiMd => {
            let path = project_root.join("GEMINI.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::AgentsMd => {
            let path = project_root.join("AGENTS.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::None => {}
    }
    Ok(())
}

/// Export the active mode (or global config) to the specified provider.
pub fn export_to(project_dir: PathBuf, target: &str) -> Result<()> {
    export_to_inner(project_dir, target, None, None, None, None)
}

/// Export using an explicit project root for generated provider files.
pub fn export_to_at_root(project_dir: PathBuf, target: &str, project_root: &Path) -> Result<()> {
    export_to_inner(project_dir, target, None, None, None, Some(project_root))
}

/// Like `export_to` but restricts project MCP servers to those whose IDs appear in
/// `server_filter`. Pass `None` to write all project servers (same as `export_to`).
pub fn export_to_filtered(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
) -> Result<()> {
    export_to_inner(project_dir, target, server_filter, None, None, None)
}

/// Like `export_to_filtered` but writes generated files under `project_root`.
pub fn export_to_filtered_at_root(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    project_root: &Path,
) -> Result<()> {
    export_to_inner(
        project_dir,
        target,
        server_filter,
        None,
        None,
        Some(project_root),
    )
}

/// Like `export_to_with_mode_override` but writes generated files under `project_root`.
pub fn export_to_with_mode_override_at_root(
    project_dir: PathBuf,
    target: &str,
    active_agent_override: Option<&str>,
    project_root: &Path,
) -> Result<()> {
    export_to_inner(
        project_dir,
        target,
        None,
        None,
        active_agent_override,
        Some(project_root),
    )
}

/// Like `export_to_filtered_with_mode_override` but writes generated files under `project_root`.
pub fn export_to_filtered_with_mode_override_at_root(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    active_agent_override: Option<&str>,
    project_root: &Path,
) -> Result<()> {
    export_to_inner(
        project_dir,
        target,
        server_filter,
        None,
        active_agent_override,
        Some(project_root),
    )
}

/// Like `export_to_filtered_with_mode_override_at_root` but also restricts exported
/// skills to `skill_filter` IDs.
pub fn export_to_filtered_with_mode_override_and_skills_at_root(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    skill_filter: Option<&[String]>,
    active_agent_override: Option<&str>,
    project_root: &Path,
) -> Result<()> {
    export_to_inner(
        project_dir,
        target,
        server_filter,
        skill_filter,
        active_agent_override,
        Some(project_root),
    )
}

/// Like `export_to` but applies a mode override when building payload.
pub fn export_to_with_mode_override(
    project_dir: PathBuf,
    target: &str,
    active_agent_override: Option<&str>,
) -> Result<()> {
    export_to_inner(project_dir, target, None, None, active_agent_override, None)
}

/// Like `export_to_filtered` but applies a mode override when building payload.
pub fn export_to_filtered_with_mode_override(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    active_agent_override: Option<&str>,
) -> Result<()> {
    export_to_inner(
        project_dir,
        target,
        server_filter,
        None,
        active_agent_override,
        None,
    )
}

fn export_to_inner(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
    skill_filter: Option<&[String]>,
    active_agent_override: Option<&str>,
    project_root_override: Option<&Path>,
) -> Result<()> {
    let desc = require_provider(target)?;
    let mut payload = build_payload_with_mode_override(&project_dir, active_agent_override)?;
    if let Some(ids) = server_filter {
        payload.servers.retain(|s| ids.contains(&s.id));
    }
    let project_root = project_root_override
        .or_else(|| project_dir.parent())
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);

    match desc.config_format {
        ConfigFormat::Json => export_json(desc, &project_dir, project_root, &payload, &mut state)?,
        ConfigFormat::Toml => export_toml(desc, &project_dir, project_root, &payload, &mut state)?,
    }

    // Skills output (provider-specific)
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => {
            export_skills_to_claude(&project_dir, project_root, skill_filter)?
        }
        SkillsOutput::AgentSkills => export_skills_to_dir(
            &project_dir,
            &project_root.join(".gemini").join("skills"),
            skill_filter,
        )?,
        SkillsOutput::CodexSkills => export_skills_to_dir(
            &project_dir,
            &project_root.join(".agents").join("skills"),
            skill_filter,
        )?,
        SkillsOutput::None => {}
    }

    // Provider-native hooks + permissions.
    match target {
        "claude" => {
            let provider_hooks = hooks_for_provider("claude", &payload.hooks);
            if !provider_hooks.is_empty() || has_claude_permission_overrides(&payload.permissions) {
                export_claude_settings(project_root, &provider_hooks, &payload.permissions)?;
            }
        }
        "gemini" => {
            let provider_hooks = hooks_for_provider("gemini", &payload.hooks);
            export_gemini_settings(project_root, &provider_hooks)?;
            export_gemini_workspace_policy(project_root, &payload.permissions)?;
        }
        _ => {}
    }

    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Remove all Ship-generated config for the given provider.
pub fn teardown(project_dir: PathBuf, target: &str) -> Result<()> {
    let desc = require_provider(target)?;
    let project_root = project_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);
    let tool_state = state
        .providers
        .entry(target.to_string())
        .or_default()
        .clone();

    match desc.config_format {
        ConfigFormat::Json => {
            let config_path = project_root.join(desc.project_config);
            teardown_json(
                &config_path,
                desc.mcp_key,
                &desc.managed_marker,
                &tool_state,
            )?;
        }
        ConfigFormat::Toml => {
            let config_path = project_root.join(desc.project_config);
            teardown_toml(&config_path, desc.mcp_key, &tool_state)?;
        }
    }

    // Remove prompt file if applicable
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let f = project_root.join("CLAUDE.md");
            if f.exists() {
                fs::remove_file(&f).with_context(|| format!("Failed to remove {}", f.display()))?;
            }
        }
        PromptOutput::GeminiMd => {
            let f = project_root.join("GEMINI.md");
            if f.exists() {
                fs::remove_file(&f).ok();
            }
        }
        PromptOutput::AgentsMd => {
            let f = project_root.join("AGENTS.md");
            if f.exists() {
                fs::remove_file(&f).ok();
            }
        }
        PromptOutput::None => {}
    }

    // Remove skill files written by Ship
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".claude").join("skills"));
        }
        SkillsOutput::AgentSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".gemini").join("skills"));
        }
        SkillsOutput::CodexSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".agents").join("skills"));
        }
        SkillsOutput::None => {}
    }

    // Clear managed state for this provider
    if let Some(ts) = state.providers.get_mut(target) {
        ts.managed_servers.clear();
        ts.last_mode = None;
    }
    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Sync all target agents configured for the active agent profile.
pub fn sync_active_agent(project_dir: &Path) -> Result<Vec<String>> {
    sync_active_agent_with_override(project_dir, None)
}

/// Sync all target agents configured for the active agent profile (or the override when provided).
pub fn sync_active_agent_with_override(
    project_dir: &Path,
    active_agent_override: Option<&str>,
) -> Result<Vec<String>> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let mode_targets = resolve_mode_for_export(&config, active_agent_override)
        .map(|mode| mode.target_agents.clone())
        .unwrap_or_default();
    let targets: Vec<String> = if !mode_targets.is_empty() {
        mode_targets
    } else if !config.providers.is_empty() {
        config.providers.clone()
    } else {
        vec!["claude".to_string()]
    };

    let mut seen = std::collections::HashSet::new();
    let mut synced = Vec::new();
    for target in targets {
        let normalized = target.trim().to_ascii_lowercase();
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        if get_provider(&normalized).is_none() {
            eprintln!("[ship] warning: skipping unknown target agent '{}'", target);
            continue;
        }
        export_to_with_mode_override(project_dir.to_path_buf(), &normalized, active_agent_override)?;
        synced.push(normalized);
    }
    Ok(synced)
}

/// Non-destructive import of MCP servers from a provider's existing config.
/// Returns count of newly-added servers.
pub fn import_from_claude(project_dir: PathBuf) -> Result<usize> {
    import_from_provider("claude", project_dir)
}
