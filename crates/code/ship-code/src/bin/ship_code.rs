mod cli;
mod compile;
mod logging;
mod meta_cmd;
mod ops_cmd;
mod paths;
mod profile;
mod uds_transport;
mod use_command;
#[cfg(test)]
mod test_env;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use cli::{Cli, Commands};

fn try_init_kernel_router() {
    if let Ok(dir) = ship_substrate::project::get_global_dir() {
        let _ = ship_substrate::events::init_kernel_router(dir);
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        eprintln!();
        eprintln!("  Logs:   ~/.ship/machine/logs/ship.log");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    try_init_kernel_router();
    let _log_guard = logging::init();
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Meta) => meta_cmd::run(),
        Some(Commands::Ops { action }) => match action {
            cli::OpsCommands::List => ops_cmd::list(),
            cli::OpsCommands::Call { name, json } => ops_cmd::call(&name, json),
        },
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
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().ok();
            println!();
            Ok(())
        }
    }
}

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
    let space_root = ship_dir;
    let output_root: Option<PathBuf> = None;

    profile::activate_agent(agent_id, &space_root, output_root.as_deref(), extra_skills, scoped)
}

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
