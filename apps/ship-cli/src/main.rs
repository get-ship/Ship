mod add;
mod audit;
mod auth;
mod build;
mod cli;
mod commands;
mod compile;
mod config;
mod convert;
#[cfg(test)]
mod test_env;
mod daemon;
mod events_cmd;
#[cfg(feature = "unstable")]
mod top;
mod help_topics;
mod doctor;
mod hook;
mod http;
mod init;
mod install;
mod procs;
mod runtime_cmd;
mod tail;
mod loader;
mod logging;
mod uninstall;
mod mcp;
mod mcp_serve;
mod paths;
mod profile;
mod skill;
mod publish;
mod ship_run;
mod validate;
mod vars;
mod openapi;
mod trash_cmd;
mod use_command;

use anyhow::Result;
use cli::{Cli, Commands, ConfigCommands, EventsCommands, HookCommands, McpCommands, OpenapiCommands, WorkspaceCommands};
use std::path::PathBuf;

/// Initialize the global KernelRouter for CLI commands that emit events.
/// Silently skips if the global dir is unavailable or the router fails to start.
///
/// Also installs the platform projection factory so the compile_state /
/// projects / auth projections register on the default event bus before any
/// `SqliteEventStore` is constructed.
fn try_init_kernel_router() {
    ship_platform::install_defaults();
    if let Ok(dir) = ship_substrate::project::get_global_dir() {
        let _ = ship_substrate::events::init_kernel_router(dir);
    }
}

/// Static version string for clap. Uses the canonical `BUILD_GIT_SHA` env
/// var emitted by `build.rs`; clap requires `&'static str`.
pub fn build_version() -> &'static str {
    if cfg!(feature = "unstable") {
        concat!(env!("CARGO_PKG_VERSION"), "+unstable.", env!("BUILD_GIT_SHA"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

/// Compile-time build identity for this CLI binary.
pub fn ship_build_info() -> build_info::BuildInfo {
    build_info::build_info!()
}

fn main() {
    use clap::Parser;
    // Install platform projection factory + supervisor hooks before any
    // SqliteEventStore is constructed downstream. Idempotent.
    ship_platform::install_defaults();
    let _log_guard = logging::init();
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli.command) {
        eprintln!("Error: {:#}", e);
        eprintln!();
        eprintln!("  Logs:   ~/.ship/machine/logs/ship.log");
        eprintln!("  Events: ship events list --since 5m");
        std::process::exit(1);
    }
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        None => run_status(None),
        Some(cmd) => match cmd {
            Commands::Init {
                global,
                harness,
                force: _,
            } => init::run(global, harness),
            Commands::Config { action } => dispatch_config(action),
            Commands::Login => auth::run_login(),
            Commands::Logout => auth::run_logout(),
            Commands::Whoami => auth::run_whoami(),
            Commands::Auth { action } => {
                use cli::AuthCommands;
                match action {
                    AuthCommands::Pair { code } => auth::run_auth_pair(&code),
                    AuthCommands::Status => auth::run_auth_status(),
                    AuthCommands::Revoke { token, all } => auth::run_auth_revoke(token.as_deref(), all),
                }
            }
            Commands::Use {
                agent_id,
                path,
                compile: _,
                with_skills,
                launch,
                auto,
                resume,
                prompt,
                spec,
                harness,
                model,
            } => {
                let opts = use_command::LaunchOpts {
                    agent: agent_id.clone(),
                    path: path.clone(),
                    launch,
                    auto,
                    resume,
                    prompt,
                    spec,
                    harness,
                    model,
                };
                run_use(Some(&agent_id), path, with_skills)?;
                if use_command::should_launch(&opts) {
                    use_command::run_launch(opts)?;
                }
                Ok(())
            }
            Commands::Status { path } => run_status(path),
            Commands::Compile {
                harness,
                dry_run,
                path,
            } => run_compile_cmd(harness.as_deref(), dry_run, path),
            Commands::Workspace { action } => dispatch_workspace(action),
            Commands::Mcp { action } => dispatch_mcp(action),
            Commands::Convert { source } => convert::run_convert(&source),
            Commands::Docs { topic } => help_topics::run(topic.as_deref()),
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
                (_, Some(url)) => add::from::run_add_from(&url),
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
                headless,
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
                    headless,
                })
            }
            Commands::Hook { action } => dispatch_hook(action),
            Commands::Procs { slug, kind, limit, json } => procs::run(slug.as_deref(), kind.as_deref(), limit, json),
            Commands::Tail { slug, namespace } => tail::run(&slug, namespace.as_deref()),
            Commands::Daemon { action } => {
                use cli::DaemonCommands;
                use daemon::{DaemonCommand, run_daemon};
                let cmd = match action {
                    DaemonCommands::Start { host, dev } => DaemonCommand::Start { host, dev },
                    DaemonCommands::Stop => DaemonCommand::Stop,
                    DaemonCommands::Restart { host, dev } => DaemonCommand::Restart { host, dev },
                    DaemonCommands::Status => DaemonCommand::Status,
                    DaemonCommands::Dir => DaemonCommand::Dir,
                    DaemonCommands::Cleanup => DaemonCommand::Cleanup,
                    DaemonCommands::RepairMigrations { reset } => DaemonCommand::RepairMigrations { reset },
                    DaemonCommands::Run { host, port, dev } => DaemonCommand::Run { host, port, dev },
                    DaemonCommands::Supervise { host, dev } => DaemonCommand::Supervise { host, dev },
                    DaemonCommands::InstallService { shipd_path, no_start, no_autostart } => {
                        DaemonCommand::InstallService { shipd_path, no_start, no_autostart }
                    }
                    DaemonCommands::UninstallService => DaemonCommand::UninstallService,
                };
                run_daemon(cmd)
            }
            Commands::Doctor => doctor::run(),
            Commands::Build { dry_run } => build::run(dry_run),
            Commands::Runtime { action } => match action {
                cli::RuntimeCommands::Install { bun, sui_compiler } => {
                    runtime_cmd::run_install(bun, sui_compiler)
                }
                cli::RuntimeCommands::Update { bun, sui_compiler } => {
                    runtime_cmd::run_update(bun, sui_compiler)
                }
                cli::RuntimeCommands::Doctor => runtime_cmd::run_doctor(),
            },
            Commands::Uninstall { yes } => uninstall::run(yes),
            Commands::Help => {
                use clap::CommandFactory;
                Cli::command().print_help()?;
                Ok(())
            }
            Commands::Openapi { action } => dispatch_openapi(action),
        },
    }
}

fn dispatch_openapi(action: OpenapiCommands) -> Result<()> {
    match action {
        OpenapiCommands::Export { pretty } => openapi::run(pretty),
    }
}

#[cfg(feature = "unstable")]
fn run_rebuild_projections() -> Result<()> {
    let report = ship_substrate::db::rebuild::rebuild_projections_default()?;
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
    let ship_dir = ship_substrate::project::get_project_dir(None)?;
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
            let sd = ship_substrate::ShipHome::project_ship_dir(std::fs::canonicalize(p)?);
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

    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&target);
    let state = profile::ProjectState::load(&ship_dir);
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

// ── Compile ───────────────────────────────────────────────────────────────────

fn run_compile_cmd(harness: Option<&str>, dry_run: bool, path: Option<PathBuf>) -> Result<()> {
    let project_root = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if !ship_substrate::ShipHome::project_ship_dir(&project_root).exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }

    let state = profile::ProjectState::load(&ship_substrate::ShipHome::project_ship_dir(&project_root));
    compile::run_compile(compile::CompileOptions {
        project_root: &project_root,
        output_root: None,
        harness,
        dry_run,
        active_agent: state.active_agent.as_deref(),
        extra_skills: vec![],
        plugins_installed: state.plugins_installed.clone(),
    })
}

// ── Workspace ────────────────────────────────────────────────────────────────

fn dispatch_workspace(action: WorkspaceCommands) -> Result<()> {
    match action {
        WorkspaceCommands::Trash { purge } => {
            if purge {
                trash_cmd::run_trash_purge()
            } else {
                trash_cmd::run_trash_list()
            }
        }
        WorkspaceCommands::Restore { filename, dest } => {
            trash_cmd::run_restore(&filename, dest)
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

// ── Internal (not user-facing) ───────────────────────────────────────────────

fn dispatch_hook(action: HookCommands) -> Result<()> {
    try_init_kernel_router();
    match action {
        HookCommands::SessionStart => hook::run_session_start(),
        HookCommands::BeforeTool => hook::run_before_tool(),
        HookCommands::AfterTool => hook::run_after_tool(),
        HookCommands::SessionEnd => hook::run_session_end(),
    }
}

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
