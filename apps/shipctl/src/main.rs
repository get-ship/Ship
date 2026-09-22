mod cli;
mod daemon;
mod doctor;
mod fleet_membership;
mod http;
mod logging;
mod settings_client;
mod uds_transport;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigCommands};

pub fn build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn shipctl_build_info() -> build_info::BuildInfo {
    build_info::build_info!()
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    logging::init();
    let cli = Cli::parse();
    dispatch(cli.command)
}

fn dispatch(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Config { action }) => dispatch_config(action),
        Some(Commands::Daemon { action }) => {
            use cli::DaemonCommands;
            use daemon::{DaemonCommand, run_daemon};
            let cmd = match action {
                DaemonCommands::Start { host, unsafe_skip_auth } => DaemonCommand::Start { host, unsafe_skip_auth },
                DaemonCommands::Stop => DaemonCommand::Stop,
                DaemonCommands::Restart { host, unsafe_skip_auth } => DaemonCommand::Restart { host, unsafe_skip_auth },
                DaemonCommands::Status => DaemonCommand::Status,
                DaemonCommands::Dir => DaemonCommand::Dir,
                DaemonCommands::Cleanup => DaemonCommand::Cleanup,
                DaemonCommands::Rebuild { dry_run } => DaemonCommand::Rebuild { dry_run },
                DaemonCommands::Run { host, port, unsafe_skip_auth } => DaemonCommand::Run { host, port, unsafe_skip_auth },
                DaemonCommands::Supervise { host, unsafe_skip_auth } => DaemonCommand::Supervise { host, unsafe_skip_auth },
                DaemonCommands::InstallService { shipd_path, no_start, no_autostart } => {
                    DaemonCommand::InstallService { shipd_path, no_start, no_autostart }
                }
                DaemonCommands::UninstallService => DaemonCommand::UninstallService,
            };
            run_daemon(cmd)
        }
        Some(Commands::Doctor) => doctor::run(),
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().ok();
            println!();
            Ok(())
        }
    }
}

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
    }
    Ok(())
}
