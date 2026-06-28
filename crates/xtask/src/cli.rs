//! CLI dispatch and help text.
//!
//! Post-multi-repo-cleanup (chore/umbrella-resolution, phase 1+2):
//! the 6 cross-repo codegen tools (gen-sdk, gen-collection-types,
//! gen-events, gen-routes-doc, gen-skills, gen-fixtures, gen-icons)
//! were DELETED from this crate. Their output destinations are
//! elsewhere (ship-contracts SDK, sui/deck assets) but their input
//! sources are still ship-internal. The rule "tools live with their
//! output SSOT" requires the input sources to be published as crates
//! first; until that's done, the tools belong in their consumer
//! repos (where they'll be re-constructed). The source for the
//! removed tools is preserved in git history at commit 4f152335e^.
//!
//! What stays in ship: 4 lints (lint-layers, lint-imports,
//! lint-anti-theater, lint-residue) + 5 audit tools (gen-contract,
//! gen-indexes + gen-indexes-substrate, gen-audit-readme, coverage,
//! check_topology) + the metadata helper. These operate on ship
//! source and produce ship-internal output (.artifacts/).

use std::path::{Path, PathBuf};

use crate::{
    check_topology, coverage, gen_audit_readme, gen_contract, gen_indexes,
    lint_anti_theater, lint_imports, lint_layers, lint_residue,
};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("cannot locate repo root")
        .to_path_buf()
}

pub fn dispatch(cmd: &str, args: &[String]) {
    let check = args.iter().any(|a| a == "--check");
    match cmd {
        "lint-layers" => {
            if let Err(e) = lint_layers::run(&repo_root()) {
                eprintln!("lint-layers failed: {e}");
                std::process::exit(1);
            }
        }
        "lint-imports" => {
            if let Err(e) = lint_imports::run(&repo_root()) {
                eprintln!("lint-imports failed: {e}");
                std::process::exit(1);
            }
        }
        "lint-anti-theater" => {
            if let Err(e) = lint_anti_theater::run(&repo_root()) {
                eprintln!("lint-anti-theater failed: {e}");
                std::process::exit(1);
            }
        }
        "lint-residue" => {
            if let Err(e) = lint_residue::run(&repo_root()) {
                eprintln!("lint-residue failed: {e}");
                std::process::exit(1);
            }
        }
        "gen-contract" => {
            if let Err(e) = gen_contract::run(&repo_root(), check) {
                eprintln!("gen-contract failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-contract-check" => {
            if let Err(e) = gen_contract::run(&repo_root(), true) {
                eprintln!("gen-contract --check failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-audit-readme" => {
            if let Err(e) = gen_audit_readme::run(&repo_root()) {
                eprintln!("gen-audit-readme failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-indexes" => {
            if let Err(e) = gen_indexes::run(&repo_root(), check) {
                eprintln!("gen-indexes failed: {e:#}");
                std::process::exit(1);
            }
        }
        "check-topology" => {
            if let Err(e) = check_topology::run() {
                eprintln!("check-topology failed: {e:#}");
                std::process::exit(1);
            }
        }
        "coverage" => {
            let trailing: Vec<&str> = args.iter().map(String::as_str).collect();
            if let Err(e) = coverage::run(&trailing) {
                eprintln!("coverage failed: {e:#}");
                std::process::exit(1);
            }
        }
        "--help" | "-h" | "help" | "" => print_help(),
        // The cross-repo codegen commands (gen-sdk, gen-events, etc.)
        // are gone. We deliberately do NOT print a helpful "did you
        // mean" — the absence is intentional. Reconstructed in their
        // consumer repos when the input publishing work lands.
        _ => {
            eprintln!("unknown command: {cmd}");
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("  lint-layers           Verify L0/L1/L2 boundaries from the resolved cargo-metadata graph (R0-R4)");
    eprintln!("  lint-imports          Verify layer-direction rules at the `use`-statement level");
    eprintln!("  lint-anti-theater     Ban source-grep-in-tests (R18): include_str!/read_to_string of a .rs + .contains/.find");
    eprintln!("  lint-residue          Catch orphan .rs files + byte-identical .rs copied across crates");
    eprintln!("  check-topology        Verify the workspace member graph (no path-based deps to external repos)");
    eprintln!("  gen-contract          Generate docs/contract/<domain>.md consumer guides");
    eprintln!("  gen-contract --check  Verify generated contract docs are up-to-date (CI drift gate)");
    eprintln!("  gen-audit-readme      Regenerate .artifacts/specs/module-audits/README.md");
    eprintln!("  gen-indexes           Regenerate emit-sites.md + substrate-fns.md indexes");
    eprintln!("  gen-indexes --check   Verify indexes are up-to-date (CI drift gate)");
    eprintln!("  coverage              Run workspace tests; emit .artifacts/coverage/report.md");
    eprintln!("  coverage --stdout     Print coverage report to stdout");
    eprintln!();
    eprintln!("Cross-repo codegen (gen-sdk, gen-events, gen-routes-doc, gen-skills,");
    eprintln!("gen-fixtures, gen-icons, gen-collection-types) was removed. The tools");
    eprintln!("are reconstructed in their consumer repos (ship-contracts, sui) once");
    eprintln!("the input source crates are published. See commit 4f152335e^ for the");
    eprintln!("prior source. `git log --all --oneline -- crates/xtask/src/gen_<name>.rs`");
    eprintln!("shows the history of each removed tool.");
}
