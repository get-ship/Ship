mod meta_cmd;
mod ops_cmd;
mod build;
mod cli;
mod compile;
#[cfg(test)]
mod test_env;
mod daemon;

mod help_topics;
mod doctor;
mod http;
mod logging;

mod paths;
mod profile;
mod settings_client;
mod use_command;

use anyhow::Result;
use cli::{Cli, Commands, ConfigCommands};
use std::path::PathBuf;

// #193 leak hunt: route every allocation through dhat's instrumented allocator
// when built with `--features dhat-heap`. The profiler itself is started only in
// the daemon-run process (see daemon::daemon_run) so CLI invocations stay clean.
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static DHAT_ALLOC: dhat::Alloc = dhat::Alloc;

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
    // (Engine cut 2026-07-10: the pipeline-run projection is gone;
    // install_projections() is now a no-op and dropped from the boot path.)
    ship_platform::install_defaults();
    let _log_guard = logging::init();
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli.command) {
        eprintln!("Error: {:#}", e);
        eprintln!();
        eprintln!("  Logs:   ~/.ship/machine/logs/ship.log");
        std::process::exit(1);
    }
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Meta) => meta_cmd::run(),
        Some(Commands::Ops { action }) => match action {
            cli::OpsCommands::List => ops_cmd::list(),
            cli::OpsCommands::Call { name, json } => ops_cmd::call(&name, json),
        },
        Some(Commands::Config { action }) => dispatch_config(action),
        Some(Commands::Use {
            agent_id,
            path,
            compile: _,
            scoped,
            with_skills,
            launch,
            resume,
            prompt,
            spec,
            harness,
            model,
            detach,
            transport,
            no_keepalive,
            keepalive,
        }) => {
            let opts = use_command::LaunchOpts {
                agent: agent_id.clone(),
                path: path.clone(),
                launch,
                resume,
                prompt,
                spec,
                harness,
                model,
                detach,
                transport,
                keepalive,
                no_keepalive,
            };
            run_use(Some(&agent_id), path, with_skills, scoped)?;
            if use_command::should_launch(&opts) {
                use_command::run_launch(opts)?;
            }
            Ok(())
        }
        Some(Commands::Compile {
            harness,
            dry_run,
            path,
        }) => run_compile_cmd(harness.as_deref(), dry_run, path),
        Some(Commands::Docs { topic }) => help_topics::run(topic.as_deref()),
        Some(Commands::Daemon { action }) => {
            use cli::DaemonCommands;
            use daemon::{DaemonCommand, run_daemon};
            let cmd = match action {
                DaemonCommands::Start { host, dev } => DaemonCommand::Start { host, dev },
                DaemonCommands::Stop => DaemonCommand::Stop,
                DaemonCommands::Restart { host, dev } => DaemonCommand::Restart { host, dev },
                DaemonCommands::Status => DaemonCommand::Status,
                DaemonCommands::Dir => DaemonCommand::Dir,
                DaemonCommands::Cleanup => DaemonCommand::Cleanup,
                DaemonCommands::Rebuild { dry_run } => DaemonCommand::Rebuild { dry_run },
                DaemonCommands::Run { host, port, dev } => DaemonCommand::Run { host, port, dev },
                DaemonCommands::Supervise { host, dev } => DaemonCommand::Supervise { host, dev },
                DaemonCommands::InstallService { shipd_path, no_start, no_autostart } => {
                    DaemonCommand::InstallService { shipd_path, no_start, no_autostart }
                }
                DaemonCommands::UninstallService => DaemonCommand::UninstallService,
            };
            run_daemon(cmd)
        }
        Some(Commands::Doctor) => doctor::run(),
        Some(Commands::Build { dry_run }) => build::run(dry_run),
        None => {
            // No subcommand: print the short help so users discover the surface.
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().ok();
            println!();
            Ok(())
        }
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

fn dispatch_config(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Get { key } => match settings_client::get(&key)? {
            Some(v) => println!("{v}"),
            None => {
                eprintln!("{key} is not set");
                std::process::exit(1);
            }
        },
        ConfigCommands::Set { key, value } => {
            settings_client::set(&key, &value)?;
            println!("{key} = {value}");
        }
        ConfigCommands::List => {
            let entries = settings_client::list()?;
            if entries.is_empty() {
                println!("No config set.");
            } else {
                for (k, v) in &entries {
                    println!("{k} = {v}");
                }
            }
        }
        ConfigCommands::Path => {
            println!("settings are stored by shipd per scope; use `ship config list`");
        }
    }
    Ok(())
}

// ── Use ───────────────────────────────────────────────────────────────────────

/// Activate an agent: load, compile, install plugins, write workspace state.
fn run_use(agent_id: Option<&str>, path: Option<PathBuf>, extra_skills: Vec<String>, scoped: bool) -> Result<()> {
    try_init_kernel_router();
    let ship_dir = match path {
        Some(ref p) => {
            let sd = std::fs::canonicalize(p)?;
            anyhow::ensure!(
                sd.join(ship_substrate::project::STACK_CONFIG_FILENAME).exists(),
                "no stack.config.ts found in {} — not a stack root.",
                p.display()
            );
            sd
        }
        None => paths::space_ship_dir_required()?,
    };
    // Marker cutover: discovery returns the STACK ROOT itself.
    let space_root = ship_dir;
    let output_root: Option<PathBuf> = None;

    profile::activate_agent(agent_id, &space_root, output_root.as_deref(), extra_skills, scoped)
}

// ── Compile ──────────────────────────────────────────────────────────────────

fn run_compile_cmd(harness: Option<&str>, dry_run: bool, path: Option<PathBuf>) -> Result<()> {
    let space_root = path
        .as_deref()
        .map(std::fs::canonicalize)
        .transpose()?
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    if !space_root.join(ship_substrate::project::STACK_CONFIG_FILENAME).exists() {
        anyhow::bail!("no stack.config.ts found in {} — not a stack root.", space_root.display());
    }

    let state = profile::SpaceState::load(&space_root);
    compile::run_compile(compile::CompileOptions {
        space_root: &space_root,
        output_root: None,
        harness,
        dry_run,
        active_agent: state.active_agent.as_deref(),
        extra_skills: vec![],
        plugins_installed: state.plugins_installed.clone(),
        scoped_launch: false,
        default_effort: None,
    })
}
