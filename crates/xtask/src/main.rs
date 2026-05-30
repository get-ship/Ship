//! Dev-time task runner.
//!
//! Usage:
//!   cargo xtask gen-types          # regenerate packages/deck/sdk/src/generated.ts
//!   cargo xtask gen-types --check  # verify generated file is up-to-date (CI)

mod build_assets;
mod cli;
mod coverage;
mod gen_audit_readme;
mod gen_contract;
mod gen_events;
mod gen_fixtures;
mod gen_indexes;
mod gen_indexes_substrate;
mod gen_ops;
mod gen_skills;
mod gen_sdk;
mod lint_imports;
mod lint_layers;
mod port_descriptions;
mod types;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    cli::dispatch(cmd, &args[2..]);
}
