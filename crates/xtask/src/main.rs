//! Dev-time task runner.
//!
//! Usage:
//!   cargo xtask gen-types          # no-op: TS outputs moved to ship-contracts
//!   cargo xtask gen-sdk            # regenerate sdk-*.json manifests

mod check_topology;
mod cli;
mod coverage;
mod gen_audit_readme;
mod gen_contract;
mod gen_events;
mod gen_fixtures;
mod gen_icons;
mod gen_indexes;
mod gen_indexes_substrate;
mod gen_sdk;
mod lint_anti_theater;

mod lint_imports;
mod lint_layers;
mod lint_residue;
mod metadata;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    cli::dispatch(cmd, &args[2..]);
}
