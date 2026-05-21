use anyhow::Result;
use ship_platform::agents::import::import_from_harness;
use ship_platform::agents::skill::{
    SkillInstallScope, create_skill, create_user_skill, delete_skill, delete_user_skill,
    get_effective_skill, list_effective_skills, list_skills, list_user_skills, update_skill,
    update_user_skill,
};
use ship_platform::config::{
    McpServerConfig, McpServerType, add_mcp_server, list_mcp_servers,
    remove_mcp_server,
};
use ship_substrate::log::log_action;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum SkillReadScope {
    Project,
    User,
    Effective,
}

#[derive(Debug, Clone, Copy)]
pub enum SkillWriteScope {
    Project,
    User,
}

#[derive(Debug, Clone)]
pub enum SkillAction {
    Install {
        source: String,
        id: String,
        scope: SkillWriteScope,
        force: bool,
    },
    Create {
        id: String,
        name: String,
        content: String,
        scope: SkillWriteScope,
    },
    List {
        scope: SkillReadScope,
    },
    Get {
        id: String,
        scope: SkillReadScope,
    },
    Update {
        id: String,
        name: Option<String>,
        content: Option<String>,
        scope: SkillWriteScope,
    },
    Delete {
        id: String,
        scope: SkillWriteScope,
    },
}

#[derive(Debug, Clone)]
pub enum EventAction {
    List { limit: usize },
}

#[derive(Debug, Clone)]
pub enum McpAction {
    List,
    Export {
        target: String,
    },
    Import {
        harness: String,
    },
    Add {
        id: String,
        name: String,
        url: String,
        disabled: bool,
    },
    AddStdio {
        id: String,
        name: String,
        command: String,
        args: Vec<String>,
    },
    Remove {
        id: String,
    },
}

pub fn parse_skill_read_scope(raw: &str) -> Result<SkillReadScope> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "project" => Ok(SkillReadScope::Project),
        "user" | "global" => Ok(SkillReadScope::User),
        "effective" => Ok(SkillReadScope::Effective),
        other => anyhow::bail!(
            "Invalid skill scope '{}'. Expected one of: project, user, effective",
            other
        ),
    }
}

pub fn parse_skill_write_scope(raw: &str) -> Result<SkillWriteScope> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "project" => Ok(SkillWriteScope::Project),
        "user" | "global" => Ok(SkillWriteScope::User),
        other => anyhow::bail!(
            "Invalid skill scope '{}'. Expected one of: project, user",
            other
        ),
    }
}

pub fn handle_skill_action(action: SkillAction, project_dir: Option<&Path>) -> Result<()> {
    match action {
        SkillAction::Install {
            source,
            id,
            scope,
            force,
        } => {
            eprintln!(
                "[ship] warning: installing untrusted skill content from '{}'. Review SKILL.md and any scripts before use.",
                source
            );

            let install_scope = match scope {
                SkillWriteScope::Project => SkillInstallScope::Project,
                SkillWriteScope::User => SkillInstallScope::User,
            };
            let installed = ship_platform::agents::skill::install_skill_from_source(
                project_dir,
                &source,
                &id,
                None,
                None,
                install_scope,
                force,
            )?;

            let installed_path = match install_scope {
                SkillInstallScope::Project => {
                    let dir = require_project_dir(project_dir)?;
                    ship_substrate::project::skills_dir(dir).join(&installed.id)
                }
                SkillInstallScope::User => ship_substrate::project::user_skills_dir().join(&installed.id),
            };
            println!(
                "Installed skill: {} ({}) -> {}",
                installed.id,
                installed.name,
                installed_path.display()
            );
        }
        SkillAction::Create {
            id,
            name,
            content,
            scope,
        } => {
            let skill = match scope {
                SkillWriteScope::Project => {
                    let dir = require_project_dir(project_dir)?;
                    let skill = create_skill(dir, &id, &name, &content)?;
                    log_action(dir, "skill create", &format!("Created skill: {}", id)).ok();
                    skill
                }
                SkillWriteScope::User => create_user_skill(&id, &name, &content)?,
            };
            println!("Skill created: {} ({})", skill.id, skill.name);
        }
        SkillAction::List { scope } => {
            let skills = match scope {
                SkillReadScope::Project => list_skills(require_project_dir(project_dir)?)?,
                SkillReadScope::User => list_user_skills()?,
                SkillReadScope::Effective => {
                    list_effective_skills(require_project_dir(project_dir)?)?
                }
            };
            if skills.is_empty() {
                println!("No skills found.");
            } else {
                for skill in skills {
                    println!("{} ({})", skill.id, skill.name);
                }
            }
        }
        SkillAction::Get { id, scope } => {
            let skill = match scope {
                SkillReadScope::Project => {
                    let dir = require_project_dir(project_dir)?;
                    let skill = ship_platform::agents::skill::get_skill(dir, &id)?;
                    log_action(dir, "skill get", &format!("Got skill: {}", id)).ok();
                    skill
                }
                SkillReadScope::User => ship_platform::agents::skill::get_user_skill(&id)?,
                SkillReadScope::Effective => {
                    get_effective_skill(require_project_dir(project_dir)?, &id)?
                }
            };
            println!("{}", skill.content);
        }
        SkillAction::Update {
            id,
            name,
            content,
            scope,
        } => {
            let updated = match scope {
                SkillWriteScope::Project => {
                    let dir = require_project_dir(project_dir)?;
                    let updated = update_skill(dir, &id, name.as_deref(), content.as_deref())?;
                    log_action(dir, "skill update", &format!("Updated skill: {}", id)).ok();
                    updated
                }
                SkillWriteScope::User => {
                    update_user_skill(&id, name.as_deref(), content.as_deref())?
                }
            };
            println!("Updated skill: {} ({})", updated.id, updated.name);
        }
        SkillAction::Delete { id, scope } => {
            match scope {
                SkillWriteScope::Project => {
                    let dir = require_project_dir(project_dir)?;
                    delete_skill(dir, &id)?;
                    log_action(dir, "skill delete", &format!("Deleted skill: {}", id)).ok();
                }
                SkillWriteScope::User => delete_user_skill(&id)?,
            }
            println!("Deleted skill: {}", id);
        }
    }
    Ok(())
}

pub fn handle_event_action(action: EventAction, project_dir: &Path) -> Result<()> {
    match action {
        EventAction::List { limit } => {
            let _ = project_dir;
            let events = ship_substrate::events::read_recent_events(limit)?;
            if events.is_empty() {
                println!("No events found.");
            } else {
                for event in events {
                    println!(
                        "{} {} [{}] {} {}",
                        event.id,
                        event.created_at.format("%Y-%m-%d %H:%M:%S"),
                        event.actor,
                        event.event_type,
                        event.entity_id,
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn handle_mcp_action(action: McpAction, project_dir: &Path) -> Result<()> {
    match action {
        McpAction::List => {
            let servers = list_mcp_servers(Some(project_dir.to_path_buf()))?;
            if servers.is_empty() {
                println!("No MCP servers configured. Add one with `ship mcp add`.");
            } else {
                for server in &servers {
                    let transport = match &server.server_type {
                        McpServerType::Stdio => format!("stdio:{}", server.command),
                        McpServerType::Sse => {
                            format!("sse:{}", server.url.as_deref().unwrap_or("?"))
                        }
                        McpServerType::Http => {
                            format!("http:{}", server.url.as_deref().unwrap_or("?"))
                        }
                    };
                    let status = if server.disabled { " [disabled]" } else { "" };
                    println!("{} — {} ({}){}", server.id, server.name, transport, status);
                }
            }
        }
        McpAction::Export { target } => {
            let project_root = project_dir.parent().unwrap_or(project_dir);
            let status = std::process::Command::new("ship")
                .arg("compile")
                .arg("--harness")
                .arg(&target)
                .arg("--path")
                .arg(project_root)
                .status();
            match status {
                Ok(s) if s.success() => {}
                Ok(s) => anyhow::bail!(
                    "ship compile failed for harness '{}' (exit {})",
                    target,
                    s.code().unwrap_or(-1)
                ),
                Err(e) => anyhow::bail!("ship binary not found: {}", e),
            }
            println!("Exported MCP server registry to {} config.", target);
        }
        McpAction::Import { harness } => {
            let added = import_from_harness(
                &harness,
                project_dir.to_path_buf(),
            )?;
            println!("Imported {} MCP server(s) from {}.", added, harness);
        }
        McpAction::Add {
            id,
            name,
            url,
            disabled,
        } => {
            add_mcp_server(
                Some(project_dir.to_path_buf()),
                McpServerConfig {
                    id: id.clone(),
                    name,
                    command: String::new(),
                    args: vec![],
                    env: Default::default(),
                    scope: "project".to_string(),
                    server_type: McpServerType::Http,
                    url: Some(url),
                    disabled,
                    timeout_secs: None,
                },
            )?;
            println!("Added MCP server '{}'.", id);
        }
        McpAction::AddStdio {
            id,
            name,
            command,
            args,
        } => {
            add_mcp_server(
                Some(project_dir.to_path_buf()),
                McpServerConfig {
                    id: id.clone(),
                    name,
                    command,
                    args,
                    env: Default::default(),
                    scope: "project".to_string(),
                    server_type: McpServerType::Stdio,
                    url: None,
                    disabled: false,
                    timeout_secs: None,
                },
            )?;
            println!("Added stdio MCP server '{}'.", id);
        }
        McpAction::Remove { id } => {
            remove_mcp_server(Some(project_dir.to_path_buf()), &id)?;
            println!("Removed MCP server '{}'.", id);
        }
    }
    Ok(())
}

fn require_project_dir(project_dir: Option<&Path>) -> Result<&Path> {
    project_dir.ok_or_else(|| anyhow::anyhow!("No Ship project found in current directory"))
}
