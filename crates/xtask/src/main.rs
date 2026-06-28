//! Dev-time task runner.
//!
//! Post-multi-repo-cleanup, this crate hosts only the in-ship
//! auditing + linting tools. The cross-repo codegen tools
//! (gen-sdk, gen-collection-types, gen-events, gen-routes-doc,
//! gen-skills, gen-fixtures, gen-icons) were removed; they are
//! reconstructed in their consumer repos (ship-contracts, sui) when
//! the input source crates are published. See cli.rs for the full
//! help text.

mod check_topology;
mod cli;
mod coverage;
mod gen_audit_readme;
mod gen_contract;
mod gen_indexes;
mod gen_indexes_substrate;
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
