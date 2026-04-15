// Included by crates/core/runtime/src/agents/export/mod.rs.
// Skills export + managed-skill cleanup helpers.

fn export_skills_to_claude(
    project_dir: &Path,
    project_root: &Path,
    skill_filter: Option<&[String]>,
) -> Result<()> {
    export_skills_to_dir(
        project_dir,
        &project_root.join(".claude").join("skills"),
        skill_filter,
    )
}

fn resolve_skills_for_export(
    project_dir: &Path,
    skill_filter: Option<&[String]>,
) -> Result<Vec<crate::skill::Skill>> {
    let mut skills = list_effective_skills(project_dir)?;
    if let Some(allowed) = skill_filter {
        let allowed = allowed
            .iter()
            .map(|id| id.trim())
            .filter(|id| !id.is_empty())
            .collect::<HashSet<_>>();
        skills.retain(|skill| allowed.contains(skill.id.as_str()));
    }
    Ok(skills)
}

#[derive(Serialize)]
struct ExportSkillMetadata<'a> {
    #[serde(rename = "display_name")]
    display_name: &'a str,
    source: &'a str,
}

#[derive(Serialize)]
struct ExportSkillFrontmatter<'a> {
    name: &'a str,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<ExportSkillMetadata<'a>>,
}

fn skill_source_label(source: &crate::skill::SkillSource) -> &'static str {
    match source {
        crate::skill::SkillSource::Builtin => "builtin",
        crate::skill::SkillSource::AiGenerated => "ai-generated",
        crate::skill::SkillSource::Community => "community",
        crate::skill::SkillSource::Imported => "imported",
        crate::skill::SkillSource::Custom => "custom",
    }
}

fn render_managed_skill_markdown(skill: &crate::skill::Skill) -> Result<String> {
    let description = skill
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Project skill '{}'", skill.id));
    let display_name = if skill.name.trim().is_empty() {
        skill.id.as_str()
    } else {
        skill.name.as_str()
    };
    let frontmatter = ExportSkillFrontmatter {
        name: &skill.id,
        description,
        metadata: Some(ExportSkillMetadata {
            display_name,
            source: skill_source_label(&skill.source),
        }),
    };
    let frontmatter_yaml = serde_yaml::to_string(&frontmatter)?;
    Ok(format!(
        "---\n{}---\n\n<!-- managed by ship — skill: {} -->\n\n{}\n",
        frontmatter_yaml,
        skill.id,
        skill.content.trim()
    ))
}

fn is_ship_managed_skill_markdown(content: &str) -> bool {
    if content.starts_with("<!-- managed by ship") {
        return true;
    }
    if !content.starts_with("---\n") {
        return false;
    }
    let rest = &content[4..];
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    let body = rest[end + 4..].trim_start_matches('\n');
    body.starts_with("<!-- managed by ship")
}

/// Write skills using the agentskills.io layout: `<skills_dir>/<skill-id>/SKILL.md`
fn export_skills_to_dir(
    project_dir: &Path,
    skills_dir: &Path,
    skill_filter: Option<&[String]>,
) -> Result<()> {
    let project_root = project_dir.parent().unwrap_or(project_dir);
    let legacy_agents_skills_dir = project_root.join("agents").join("skills");
    if skills_dir == legacy_agents_skills_dir {
        return Err(anyhow!(
            "Refusing to export skills to legacy path '{}'; use '.agents/skills' instead",
            legacy_agents_skills_dir.display()
        ));
    }

    let skills = resolve_skills_for_export(project_dir, skill_filter)?;
    let retain_ids: HashSet<String> = skills.iter().map(|skill| skill.id.clone()).collect();
    prune_stale_managed_skill_dirs(skills_dir, &retain_ids);
    if skills.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(skills_dir)?;
    for skill in &skills {
        let skill_dir = skills_dir.join(&skill.id);
        fs::create_dir_all(&skill_dir)?;
        let path = skill_dir.join("SKILL.md");
        let content = render_managed_skill_markdown(skill)?;
        crate::fs_util::write_atomic(&path, content)?;
    }
    Ok(())
}

fn prune_stale_managed_skill_dirs(skills_dir: &Path, retain_ids: &HashSet<String>) {
    if !skills_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }

            let skill_id = entry.file_name().to_string_lossy().to_string();
            if retain_ids.contains(&skill_id) {
                continue;
            }

            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(content) = fs::read_to_string(&skill_md)
                && is_ship_managed_skill_markdown(&content)
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}

/// Remove skill subdirectories that were written by Ship (identified by the
/// `<!-- managed by ship` header in their SKILL.md).
fn remove_ship_managed_skill_dirs(skills_dir: &Path) {
    if !skills_dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(c) = fs::read_to_string(&skill_md)
                && is_ship_managed_skill_markdown(&c)
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}
