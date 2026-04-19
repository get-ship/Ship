//! `ship init` — scaffold .ship/ in a project or configure ~/.ship/ globally.

use anyhow::Result;

use crate::paths;

pub fn run(global: bool, provider: Option<String>) -> Result<()> {
    if global {
        run_global()
    } else {
        run_project(provider)
    }
}

// ── Normal init paths ────────────────────────────────────────────────────────

fn run_global() -> Result<()> {
    paths::ensure_global_dirs()?;
    let gdir = paths::global_dir();
    let cfg_path = gdir.join("config.toml");
    if !cfg_path.exists() {
        std::fs::write(
            &cfg_path,
            "# Ship global configuration\n\n[identity]\nname = \"\"\n",
        )?;
    }
    if !gdir.join("README.md").exists() {
        std::fs::write(
            gdir.join("README.md"),
            "# Ship\n\nSee https://getship.dev\n",
        )?;
    }
    println!("initialized global config at ~/.ship/");
    println!("  Edit ~/.ship/config.toml to set your identity");
    Ok(())
}

fn run_project(provider: Option<String>) -> Result<()> {
    let project_root = std::env::current_dir()?;

    // ── Auto-detect existing provider configs when no --provider flag ─────────
    let is_fresh = !paths::project_ship_jsonc().exists() && !paths::project_ship_toml().exists();
    let detected = if is_fresh && provider.is_none() {
        let d = compiler::detect_providers(&project_root);
        if d.any() { Some(d) } else { None }
    } else {
        None
    };

    paths::ensure_project_dirs()?;
    let ship_jsonc = paths::project_ship_jsonc();

    if let Some(ref detected) = detected {
        // Import detected provider configs into .ship/
        let providers = detected.as_list();
        println!("Detected existing configs: {}", providers.join(", "));

        let library = compiler::decompile_all(&project_root);

        // Write ship.jsonc with detected providers and provider_defaults
        if !ship_jsonc.exists() {
            let mut project = serde_json::json!({
                "providers": providers.iter()
                    .map(|p| serde_json::Value::String(p.to_string()))
                    .collect::<Vec<_>>()
            });
            if !library.provider_defaults.is_empty() {
                let defaults: serde_json::Map<String, serde_json::Value> = library
                    .provider_defaults
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                project["provider_defaults"] = serde_json::Value::Object(defaults);
            }
            let manifest = serde_json::json!({
                "$schema": "../schemas/ship.schema.json",
                "project": project
            });
            std::fs::write(&ship_jsonc, serde_json::to_string_pretty(&manifest)?)?;
        }

        // Write MCP servers
        if !library.mcp_servers.is_empty() {
            let mcp_jsonc = paths::mcp_path();
            if !mcp_jsonc.exists() {
                let mut servers = serde_json::Map::new();
                for s in &library.mcp_servers {
                    let mut entry = serde_json::json!({});
                    if !s.command.is_empty() {
                        entry["command"] = serde_json::json!(s.command);
                    }
                    if !s.args.is_empty() {
                        entry["args"] = serde_json::json!(s.args);
                    }
                    if !s.env.is_empty() {
                        entry["env"] = serde_json::json!(s.env);
                    }
                    if let Some(url) = &s.url {
                        entry["url"] = serde_json::json!(url);
                    }
                    servers.insert(s.id.clone(), entry);
                }
                let mcp_obj = serde_json::json!({
                    "$schema": "../schemas/mcp.schema.json",
                    "mcp": { "servers": servers }
                });
                std::fs::write(&mcp_jsonc, serde_json::to_string_pretty(&mcp_obj)?)?;
                println!("  imported {} MCP servers", library.mcp_servers.len());
            }
        }

        // Write rules
        if !library.rules.is_empty() {
            let rules_dir = paths::rules_dir();
            std::fs::create_dir_all(&rules_dir)?;
            for rule in &library.rules {
                let file_name = if rule.file_name.ends_with(".md") {
                    rule.file_name.clone()
                } else {
                    format!("{}.md", rule.file_name)
                };
                let rule_path = rules_dir.join(&file_name);
                if !rule_path.exists() {
                    std::fs::write(&rule_path, &rule.content)?;
                }
            }
            println!("  imported {} rules", library.rules.len());
        }
    } else if !ship_jsonc.exists() && !paths::project_ship_toml().exists() {
        // No detection — scaffold default
        let prov = provider.as_deref().unwrap_or("claude");
        std::fs::write(
            &ship_jsonc,
            format!(
                "{{\n  \"$schema\": \"../schemas/ship.schema.json\",\n  \
                 \"project\": {{\n    \"providers\": [\"{prov}\"],\n  }},\n}}"
            ),
        )?;
    }

    let pdir = paths::project_dir();

    // Gitignore for compiled artifacts goes in the PROJECT ROOT (not .ship/),
    // because compiled output (CLAUDE.md, .mcp.json, etc.) lives at the root.
    let root_gitignore = project_root.join(".gitignore");
    if !root_gitignore.exists() {
        std::fs::write(
            &root_gitignore,
            "# Ship compiled artifacts (generated by `ship use`)\n\
             CLAUDE.md\nGEMINI.md\nAGENTS.md\n.mcp.json\n\
             .claude/\n.codex/\n.gemini/\n.cursor/\n.agents/\n\
             # Agent working artifacts\n.artifacts/\n",
        )?;
    } else {
        // If a root .gitignore exists, check if it already covers compiled output.
        // If not, append Ship entries.
        let existing = std::fs::read_to_string(&root_gitignore).unwrap_or_default();
        if !existing.contains("CLAUDE.md") {
            let mut appended = existing;
            if !appended.ends_with('\n') {
                appended.push('\n');
            }
            appended.push_str(
                "\n# Ship compiled artifacts (generated by `ship use`)\n\
                 CLAUDE.md\nGEMINI.md\nAGENTS.md\n.mcp.json\n\
                 .claude/\n.codex/\n.gemini/\n.cursor/\n.agents/\n\
                 # Agent working artifacts\n.artifacts/\n",
            );
            std::fs::write(&root_gitignore, appended)?;
        }
    }

    // Scaffold a default agent if no agents exist yet.
    let agents_dir = pdir.join("agents");
    let has_agents = agents_dir.is_dir()
        && std::fs::read_dir(&agents_dir)
            .map(|entries| {
                entries
                    .flatten()
                    .any(|e| {
                        let p = e.path();
                        p.is_file() && crate::paths::is_config_ext(&p)
                    })
            })
            .unwrap_or(false);

    if !has_agents {
        std::fs::create_dir_all(&agents_dir)?;
        let agent_jsonc = agents_dir.join("dev.jsonc");
        std::fs::write(
            &agent_jsonc,
            r#"{
  "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json",
  "agent": {
    "id": "dev",
    "name": "Dev",
    "version": "0.1.0",
    "description": "General-purpose development agent"
  },
  "skills": {
    "refs": []
  },
  "mcp": {
    "servers": []
  },
  "permissions": {
    "preset": "ship-standard"
  }
}
"#,
        )?;
    }

    install_post_checkout_hook(&project_root);

    let effective_providers = if let Some(ref d) = detected {
        d.as_list().join(", ")
    } else {
        provider.as_deref().unwrap_or("claude").to_string()
    };

    println!("initialized .ship/ in current directory");
    println!("  providers: {}", effective_providers);
    if detected.is_some() {
        println!("  imported existing provider configs into .ship/");
    }

    // Auto-activate the dev agent if we just scaffolded it.
    if !has_agents {
        println!();
        if let Err(e) = crate::profile::activate_agent(
            Some("dev"),
            &project_root,
            None,
            vec![],
        ) {
            eprintln!("warning: could not auto-activate dev agent: {}", e);
            println!("\nNext steps:");
            println!("  ship use dev            activate the dev agent");
        }
    } else {
        println!("\nNext steps:");
        println!("  ship use <agent-id>     activate an agent");
        println!("  ship compile            re-compile current agent");
    }
    Ok(())
}

/// Install a git post-checkout hook that notifies shipd of branch changes.
/// Skips if the hook already exists to avoid clobbering user hooks.
fn install_post_checkout_hook(project_root: &std::path::Path) {
    let git_dir = project_root.join(".git");
    if !git_dir.is_dir() {
        return;
    }
    let hooks_dir = git_dir.join("hooks");
    let hook_path = hooks_dir.join("post-checkout");
    if hook_path.exists() {
        return;
    }
    if std::fs::create_dir_all(&hooks_dir).is_err() {
        return;
    }

    let hook_script = r#"#!/bin/sh
# Ship post-checkout hook — notifies shipd of branch changes.
# Installed by `ship init`. Safe to remove if not needed.

# Only run on branch checkouts (flag=1), not file checkouts (flag=0).
if [ "$3" != "1" ]; then
  exit 0
fi

NEW_BRANCH=$(git branch --show-current 2>/dev/null)
if [ -z "$NEW_BRANCH" ]; then
  exit 0
fi

PORT_FILE="${HOME}/.ship/network.port"
if [ ! -f "$PORT_FILE" ]; then
  exit 0
fi
PORT=$(cat "$PORT_FILE")

# Fire and forget — fail silently if shipd isn't running.
curl -sf -X POST "http://127.0.0.1:${PORT}/api/hooks/checkout" \
  -H "Content-Type: application/json" \
  -d "{\"branch\":\"${NEW_BRANCH}\",\"path\":\"$(pwd)\"}" \
  >/dev/null 2>&1 || true
"#;

    if std::fs::write(&hook_path, hook_script).is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755));
        }
        println!("  installed .git/hooks/post-checkout");
    }
}

