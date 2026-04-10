mod add;
mod add_from;
mod add_from_write;
mod agent_config;
mod audit;
mod auth;
mod cli;
mod commands;
mod compile;
mod config;
mod convert;
mod daemon;
mod dep_skills;
#[cfg(feature = "unstable")]
mod events_cmd;
#[cfg(feature = "unstable")]
mod top;
mod help_topics;
mod hook;
mod init;
mod install;
mod loader;
mod logging;
mod mcp;
mod mcp_serve;
mod paths;
mod profile;
mod publish;
mod ship_run;
mod validate;
mod vars;
mod workspace_cmd;

use anyhow::Result;
use cli::{Cli, Commands, ConfigCommands, HookCommands, McpCommands, NetworkCommands, WorkspaceCommands};
#[cfg(feature = "unstable")]
use cli::EventsCommands;
use std::path::PathBuf;

/// Initialize the global KernelRouter for CLI commands that emit events.
/// Silently skips if the global dir is unavailable or the router fails to start.
fn try_init_kernel_router() {
    if let Ok(dir) = runtime::project::get_global_dir() {
        let _ = runtime::events::init_kernel_router(dir);
    }
}

pub fn build_version() -> &'static str {
    if cfg!(feature = "unstable") {
        concat!(env!("CARGO_PKG_VERSION"), "+unstable.", env!("GIT_COMMIT_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

fn main() -> Result<()> {
    use clap::Parser;
    let _log_guard = logging::init();
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        None => run_status(None),
        Some(cmd) => match cmd {
            Commands::Init {
                global,
                provider,
                force: _,
            } => init::run(global, provider),
            Commands::Config { action } => dispatch_config(action),
            Commands::Login => auth::run_login(),
            Commands::Logout => auth::run_logout(),
            Commands::Whoami => auth::run_whoami(),
            Commands::Use {
                agent_id,
                path,
                compile: _,
                with_skills,
            } => run_use(Some(&agent_id), path, with_skills),
            Commands::Status { path } => run_status(path),
            Commands::Agents { .. } => removed_command(
                "agents",
                "Edit .ship/agents/ directly. See: ship docs agents",
            ),
            Commands::Compile {
                provider,
                dry_run,
                path,
            } => run_compile_cmd(provider.as_deref(), dry_run, path),
            Commands::Skills { .. } => removed_command(
                "skills",
                "Edit .ship/skills/ directly or use ship.jsonc. See: ship docs skills",
            ),
            Commands::Vars { .. } => removed_command(
                "vars",
                "Manage skill variables in .ship/skills/ state files. See: ship docs skills",
            ),
            Commands::Workspace { action } => dispatch_workspace(action),
            Commands::Mcp { action } => dispatch_mcp(action),
            Commands::Convert { source } => convert::run_convert(&source),
            Commands::Docs { topic } => help_topics::run(topic.as_deref()),
            #[cfg(feature = "unstable")]
            Commands::Adrs => run_adrs(),
            Commands::Publish {
                export_path,
                dry_run,
                tag,
            } => {
                let root = std::env::current_dir()?;
                publish::run_publish(&root, export_path.as_deref(), dry_run, tag.as_deref())
            }
            Commands::Install { frozen, offline } => {
                let root = std::env::current_dir()?;
                install::run_install(&root, frozen, offline)
            }
            Commands::Add { package, from } => match (package, from) {
                (_, Some(url)) => add_from::run_add_from(&url),
                (Some(pkg), None) => {
                    let root = std::env::current_dir()?;
                    add::run_add(&root, &pkg)
                }
                (None, None) => anyhow::bail!("provide a package name or --from <url>"),
            },
            Commands::Audit { path, json } => audit::run_audit(path, json),
            Commands::Validate { agent, json, path } => {
                let root = path
                    .as_deref()
                    .map(std::fs::canonicalize)
                    .transpose()?
                    .unwrap_or_else(|| std::env::current_dir().unwrap());
                validate::run_validate(agent.as_deref(), json, &root)
            }
            #[cfg(feature = "unstable")]
            Commands::Events { action } => dispatch_events(action),
            #[cfg(feature = "unstable")]
            Commands::RebuildProjections => run_rebuild_projections(),
            #[cfg(feature = "unstable")]
            Commands::Top => run_top(),
            Commands::Run {
                pipeline,
                spec_text,
                spec,
                workspace,
                vars,
                dry_run,
            } => {
                let parsed_vars: Vec<(String, String)> = vars
                    .iter()
                    .filter_map(|v| {
                        let (k, val) = v.split_once('=')?;
                        Some((k.to_string(), val.to_string()))
                    })
                    .collect();
                ship_run::run(ship_run::RunOptions {
                    pipeline_name: pipeline,
                    spec_text,
                    spec_file: spec,
                    workspace,
                    vars: parsed_vars,
                    dry_run,
                })
            }
            Commands::Hook { action } => dispatch_hook(action),
            Commands::Network { action } => dispatch_network(action),
            Commands::Studio { port, open } => run_studio(port, open),
            Commands::Daemon { action } => {
                use cli::DaemonCommands;
                use daemon::{DaemonCommand, run_daemon};
                let cmd = match action {
                    DaemonCommands::Start { host } => DaemonCommand::Start { host },
                    DaemonCommands::Stop => DaemonCommand::Stop,
                    DaemonCommands::Status => DaemonCommand::Status,
                    DaemonCommands::Cleanup => DaemonCommand::Cleanup,
                };
                run_daemon(cmd)
            }
            Commands::Help => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                Ok(())
            }
        },
    }
}

fn run_studio(port: u16, open: bool) -> Result<()> {
    eprintln!("Ship Studio MCP listening on http://localhost:{port}/mcp");

    if open {
        // Read daemon token and pass via URL fragment (anonymous auth path)
        let token: Option<String> = read_daemon_token();
        let url: String = match token {
            Some(t) => format!("https://getship.dev/studio#token={t}"),
            None => "https://getship.dev/studio".to_string(),
        };
        eprintln!("Opening {}", url.split('#').next().unwrap_or(&url));
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn();
    } else {
        eprintln!("Open https://getship.dev/studio to connect");
    }
    mcp_serve::run_studio(port)
}

fn read_daemon_token() -> Option<String> {
    let global_dir: std::path::PathBuf = runtime::project::get_global_dir()
        .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".ship"));
    let path: std::path::PathBuf = global_dir.join("daemon.token");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

#[cfg(feature = "unstable")]
fn run_rebuild_projections() -> Result<()> {
    let report = runtime::db::rebuild::rebuild_projections_default()?;
    println!(
        "Rebuilt {} projection(s) from {} event(s) in {}ms",
        report.projections_rebuilt.len(),
        report.events_replayed,
        report.duration_ms,
    );
    for name in &report.projections_rebuilt {
        println!("  ✓ {name}");
    }
    Ok(())
}

#[cfg(feature = "unstable")]
fn run_top() -> Result<()> {
    let ship_dir = runtime::project::get_project_dir(None)?;
    top::run(ship_dir)
}

// ── Config ────────────────────────────────────────────────────────────────────

fn dispatch_config(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Get { key } => {
            let cfg = config::ShipConfig::load();
            match cfg.get(&key) {
                Some(v) => println!("{}", v),
                None => {
                    eprintln!("{} is not set", key);
                    std::process::exit(1);
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            let mut cfg = config::ShipConfig::load();
            cfg.set(&key, &value)?;
            cfg.save()?;
            println!("{} = {}", key, value);
        }
        ConfigCommands::List => {
            let cfg = config::ShipConfig::load();
            let entries = cfg.list();
            if entries.is_empty() {
                println!(
                    "No config set. File: {}",
                    config::ShipConfig::path().display()
                );
            } else {
                for (k, v) in &entries {
                    println!("{} = {}", k, v);
                }
            }
        }
        ConfigCommands::Path => {
            println!("{}", config::ShipConfig::path().display());
        }
    }
    Ok(())
}

// ── Use ───────────────────────────────────────────────────────────────────────

/// Activate an agent: load, compile, install plugins, write workspace state.
fn run_use(agent_id: Option<&str>, path: Option<PathBuf>, extra_skills: Vec<String>) -> Result<()> {
    try_init_kernel_router();
    let ship_dir = match path {
        Some(ref p) => {
            let sd = std::fs::canonicalize(p)?.join(".ship");
            anyhow::ensure!(
                sd.exists(),
                ".ship/ not found in {}. Run: ship init",
                p.display()
            );
            sd
        }
        None => paths::project_ship_dir_required()?,
    };
    let project_root = ship_dir.parent().unwrap().to_path_buf();

    // When --path is not given and project_root was resolved via worktree pointer,
    // the cwd (worktree) differs from project_root (main repo). Compiled output
    // (CLAUDE.md, .claude/settings.json) must go to the cwd, not the main repo.
    let output_root = if path.is_none() {
        let cwd = std::env::current_dir()?;
        if cwd != project_root { Some(cwd) } else { None }
    } else {
        None
    };

    profile::activate_agent(agent_id, &project_root, output_root.as_deref(), extra_skills)
}

// ── Status ────────────────────────────────────────────────────────────────────

fn run_status(path: Option<PathBuf>) -> Result<()> {
    let target = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let ship_dir = target.join(".ship");
    let state = profile::WorkspaceState::load(&ship_dir);
    match state.active_agent {
        Some(ref p) => {
            println!("active agent: {}", p);
            if let Some(ref at) = state.compiled_at {
                println!("compiled: {}", at);
            }
            if !state.plugins_installed.is_empty() {
                println!("plugins: {}", state.plugins_installed.join(", "));
            }
        }
        None => {
            println!("No active agent for {}", target.display());
            println!("Run: ship use <agent-id>");
        }
    }
    Ok(())
}

// ── Removed commands ─────────────────────────────────────────────────────────

fn removed_command(name: &str, guidance: &str) -> Result<()> {
    eprintln!("Removed: `ship {name}` commands have been removed.");
    eprintln!("{guidance}");
    std::process::exit(1);
}

// ── Compile ───────────────────────────────────────────────────────────────────

fn run_compile_cmd(provider: Option<&str>, dry_run: bool, path: Option<PathBuf>) -> Result<()> {
    let project_root = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if !project_root.join(".ship").exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }

    let state = profile::WorkspaceState::load(&project_root.join(".ship"));
    compile::run_compile(compile::CompileOptions {
        project_root: &project_root,
        output_root: None,
        provider,
        dry_run,
        active_agent: state.active_agent.as_deref(),
        extra_skills: vec![],
    })
}

// ── Workspace ────────────────────────────────────────────────────────────────

fn dispatch_workspace(action: WorkspaceCommands) -> Result<()> {
    match action {
        WorkspaceCommands::Trash { purge } => {
            if purge {
                workspace_cmd::run_trash_purge()
            } else {
                workspace_cmd::run_trash_list()
            }
        }
        WorkspaceCommands::Restore { filename, dest } => {
            workspace_cmd::run_restore(&filename, dest)
        }
    }
}

// ── Subcommand dispatchers ────────────────────────────────────────────────────

fn dispatch_mcp(action: McpCommands) -> Result<()> {
    match action {
        McpCommands::Serve { http, port } => mcp_serve::run(http, port),
        McpCommands::List => mcp::list(),
        McpCommands::Add { id, name, url, .. } => {
            let url =
                url.ok_or_else(|| anyhow::anyhow!("--url is required for HTTP/SSE servers"))?;
            mcp::add_http(&id, name, &url)
        }
        McpCommands::AddStdio {
            id,
            command,
            args,
            name,
            ..
        } => mcp::add_stdio(&id, name, &command, args),
        McpCommands::Remove { id } => mcp::remove(&id),
    }
}

// ── Network (deprecated) ─────────────────────────────────────────────────────

fn dispatch_network(action: NetworkCommands) -> Result<()> {
    let replacement = match action {
        NetworkCommands::Serve { .. } => "ship daemon start",
        NetworkCommands::Status => "ship daemon status",
        NetworkCommands::Stop => "ship daemon stop",
    };
    eprintln!("`ship network` has been removed. Use `{replacement}` instead.");
    std::process::exit(1);
}

// ── Hidden / legacy ──────────────────────────────────────────────────────────

#[cfg(feature = "unstable")]
fn run_adrs() -> Result<()> {
    let _ship_dir = paths::project_ship_dir_required()?;
    let adrs = runtime::db::adrs::list_adrs()?;
    if adrs.is_empty() {
        println!("No ADRs found.");
    } else {
        for entry in &adrs {
            println!("{}\t[{}]\t{}", entry.id, entry.status, entry.title);
        }
    }
    Ok(())
}

fn dispatch_hook(action: HookCommands) -> Result<()> {
    try_init_kernel_router();
    match action {
        HookCommands::SessionStart => hook::run_session_start(),
        HookCommands::BeforeTool => hook::run_before_tool(),
        HookCommands::AfterTool => hook::run_after_tool(),
        HookCommands::SessionEnd => hook::run_session_end(),
    }
}

#[cfg(feature = "unstable")]
fn dispatch_events(action: EventsCommands) -> Result<()> {
    let ship_dir = paths::project_ship_dir_required()?;
    match action {
        EventsCommands::List {
            since,
            actor,
            entity,
            action,
            limit,
            json,
        } => events_cmd::run_events(&ship_dir, since, actor, entity, action, limit, json),
    }
}
