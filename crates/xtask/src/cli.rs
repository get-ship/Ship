//! CLI dispatch and help text.

use std::path::{Path, PathBuf};

use crate::{
    coverage, gen_audit_readme, gen_contract, gen_events, gen_fixtures,
    gen_indexes, lint_anti_theater,
    lint_imports, lint_layers, lint_residue,
};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("cannot locate repo root")
        .to_path_buf()
}

pub fn dispatch(cmd: &str, args: &[String]) {
    match cmd {
        "lint-layers" => {
            if let Err(e) = lint_layers::run(&repo_root()) {
                eprintln!("lint-layers failed: {e}");
                std::process::exit(1);
            }
        }
        "lint-imports" => {
            match lint_imports::run(&repo_root()) {
                Err(e) => {
                    eprintln!("lint-imports failed: {e}");
                    std::process::exit(1);
                }
                Ok(report) => {
                    if report.violations.is_empty() {
                        println!("import layer boundaries clean");
                    } else {
                        let count = report.violations.len() / 2;
                        for line in &report.violations {
                            eprintln!("{line}");
                        }
                        eprintln!("lint-imports: {count} violation(s)");
                        std::process::exit(1);
                    }
                }
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
        "gen-fixtures" => {
            let trailing: Vec<&str> = args.iter().map(String::as_str).collect();
            if let Err(e) = gen_fixtures::run_with_args(&trailing) {
                eprintln!("gen-fixtures failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-events" => {
            let check = args.iter().any(|a| a == "--check");
            if let Err(e) = gen_events::run(check) {
                eprintln!("gen-events failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-contract" => {
            let check = args.iter().any(|a| a == "--check");
            if let Err(e) = gen_contract::run(&repo_root(), check) {
                eprintln!("gen-contract failed: {e:#}");
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
            let check = args.iter().any(|a| a == "--check");
            if let Err(e) = gen_indexes::run(&repo_root(), check) {
                eprintln!("gen-indexes failed: {e:#}");
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
        "help" | "--help" | "-h" => {
            print_help();
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("  gen-fixtures          Emit proto contract fixtures from <PROTO_ROOT>/fixtures/fixture-seed.json (SMELL: tool lives in ship; output is ship-contracts; see .artifacts/handoff-post-cleanup.md)");
    eprintln!("  lint-layers           Verify L0/L1/L2 boundaries from the resolved cargo-metadata graph (R0-R4)");
    eprintln!("  lint-imports          Verify layer-direction rules at the `use`-statement level");
    eprintln!("  lint-anti-theater     Ban source-grep-in-tests (R18): include_str!/read_to_string of a .rs + .contains/.find");
    eprintln!("  lint-residue          Catch orphan .rs files + byte-identical .rs copied across crates");
    eprintln!("  gen-events            Generate tier-filtered proto + TS from the event registry");
    eprintln!("  gen-events --check    Verify generated files are up-to-date (CI)");
    eprintln!("  gen-contract          Generate docs/contract/<domain>.md consumer guides");
    eprintln!("  gen-contract --check  Verify generated contract docs are up-to-date (CI)");
    eprintln!("  gen-audit-readme      Regenerate .artifacts/specs/module-audits/README.md");
    eprintln!("  gen-indexes           Regenerate emit-sites.md + substrate-fns.md indexes");
    eprintln!("  gen-indexes --check   Verify indexes are up-to-date (CI)");
    eprintln!("  coverage              Run workspace tests; emit .artifacts/coverage/report.md");
    eprintln!("  coverage --stdout     Print coverage report to stdout");
    eprintln!("  coverage --report-only  Skip test run; read test JSON from stdin");
    eprintln!("  coverage --format json  JSON output instead of markdown");
    eprintln!("  coverage --filter <svc>  Scope to one service/namespace prefix");
}

