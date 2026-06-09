//! CLI dispatch and help text.

use std::path::{Path, PathBuf};

use specta::TypeCollection;
use specta_typescript::{BigIntExportBehavior, Typescript};

use crate::{
    build_assets, coverage, gen_audit_readme, gen_collection_types, gen_contract, gen_events,
    gen_fixtures, gen_icons, gen_indexes, gen_skills, gen_sdk, lint_imports, lint_layers,
    port_descriptions, types,
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
        "build-assets" => {
            if let Err(e) = build_assets::run(&repo_root()) {
                eprintln!("build-assets failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-types" => {
            let check = args.iter().any(|a| a == "--check");
            gen_types(check);
        }
        "gen-sdk" => {
            if let Err(e) = gen_sdk::run(&repo_root()) {
                eprintln!("gen-sdk failed: {e:#}");
                std::process::exit(1);
            }
        }
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
        "gen-icons" => {
            if let Err(e) = gen_icons::run(&repo_root()) {
                eprintln!("gen-icons failed: {e:#}");
                std::process::exit(1);
            }
        }
        "gen-skills" => {
            if let Err(e) = gen_skills::run() {
                eprintln!("gen-skills failed: {e:#}");
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
        "gen-collection-types" => {
            let check = args.iter().any(|a| a == "--check");
            if let Err(e) = gen_collection_types::run(&repo_root(), check) {
                eprintln!("gen-collection-types failed: {e:#}");
                std::process::exit(1);
            }
        }
        "port-descriptions" => {
            if let Err(e) = port_descriptions::run() {
                eprintln!("port-descriptions failed: {e:#}");
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
    eprintln!("Commands:");
    eprintln!("  build-assets          Bake framework bundles via in-process Rolldown (replaces bun bundler/index.ts --all)");
    eprintln!("  gen-types             Regenerate packages/deck/sdk/src/generated.ts");
    eprintln!("  gen-types --check     Verify generated file is up-to-date (CI)");
    eprintln!("  gen-sdk               Emit packages/deck/sdk/src/generated/sdk-*.json + TS bindings");
    eprintln!("  gen-fixtures          Emit proto contract fixtures from proto/fixtures/fixture-seed.json");
    eprintln!("  gen-icons             Bake lucide-icons.json from node_modules/lucide-static");
    eprintln!("  lint-layers           Verify substrate/platform/application boundaries (Cargo dep level)");
    eprintln!("  lint-imports          Verify layer-direction rules at the `use`-statement level");
    eprintln!("  gen-events            Generate tier-filtered proto + TS from the event registry");
    eprintln!("  gen-events --check    Verify generated files are up-to-date (CI)");
    eprintln!("  gen-contract          Generate docs/contract/<domain>.md consumer guides");
    eprintln!("  gen-contract --check  Verify generated contract docs are up-to-date (CI)");
    eprintln!("  gen-audit-readme      Regenerate .artifacts/specs/module-audits/README.md");
    eprintln!("  gen-indexes           Regenerate emit-sites.md + substrate-fns.md indexes");
    eprintln!("  gen-indexes --check   Verify indexes are up-to-date (CI)");
    eprintln!("  gen-collection-types  Emit packages/deck/sdk/src/generated/collections.ts (typed Database from .ship/collections)");
    eprintln!("  gen-collection-types --check  Verify generated collection types are up-to-date (CI)");
    eprintln!("  coverage              Run workspace tests; emit .artifacts/coverage/report.md");
    eprintln!("  coverage --stdout     Print coverage report to stdout");
    eprintln!("  coverage --report-only  Skip test run; read test JSON from stdin");
    eprintln!("  coverage --format json  JSON output instead of markdown");
    eprintln!("  coverage --filter <svc>  Scope to one service/namespace prefix");
}


fn gen_types(check: bool) {
    let ts = Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .header("// This file is auto-generated by `cargo xtask gen-types`. Do not edit.")
        .framework_header("");

    let root = repo_root();
    let ui_out = root.join("packages/deck/sdk/src/generated.ts");
    let ui_content = ts
        .export(&types::collect_types())
        .expect("specta-typescript export failed (ui)");

    let cloud_out = root.join("packages/cloud-types/src/generated.ts");
    let cloud_content = build_cloud_types(&ts, &root);

    let framework_out = root.join("packages/sui/runtime/src/generated.ts");
    let framework_content = ts
        .export(&types::collect_framework_types())
        .expect("specta-typescript export failed (framework)");

    let ship_data_out = root.join("packages/deck/sdk-client/src/generated.ts");
    let ship_data_content = ts
        .export(&types::collect_ship_data_types())
        .expect("specta-typescript export failed (ship-data)");

    if check {
        let mut stale = false;
        for (path, expected) in [
            (&ui_out, &ui_content),
            (&cloud_out, &cloud_content),
            (&framework_out, &framework_content),
            (&ship_data_out, &ship_data_content),
        ] {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing != *expected {
                eprintln!("Out of date: {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-types` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("Types are up-to-date.");
        return;
    }

    std::fs::write(&ui_out, &ui_content).expect("failed to write ui generated.ts");
    eprintln!("Wrote {}", ui_out.display());
    std::fs::write(&cloud_out, &cloud_content).expect("failed to write cloud generated.ts");
    eprintln!("Wrote {}", cloud_out.display());
    std::fs::write(&framework_out, &framework_content)
        .expect("failed to write framework generated.ts");
    eprintln!("Wrote {}", framework_out.display());
    std::fs::write(&ship_data_out, &ship_data_content)
        .expect("failed to write ship-data generated.ts");
    eprintln!("Wrote {}", ship_data_out.display());
}

/// Build the cloud-side generated.ts: EventEnvelope struct + all event-type
/// string constants read from `ship-types/src/event_types/mod.rs`. Cloud's TS
/// surface must mirror Rust's event taxonomy exactly.
fn build_cloud_types(ts: &Typescript, root: &Path) -> String {
    let mut c = TypeCollection::default();
    c.register::<ship_types::EventEnvelope>();
    let struct_ts = ts
        .export(&c)
        .expect("specta-typescript export failed (cloud)");

    let event_types_src = root.join("crates/core/ship-substrate/crates/ship-types/src/event_types/mod.rs");
    let src = std::fs::read_to_string(&event_types_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", event_types_src.display()));
    let mut consts = String::new();
    for line in src.lines() {
        let trimmed = line.trim();
        // pub const NAME: &str = "value";
        let Some(rest) = trimmed.strip_prefix("pub const ") else { continue };
        let Some((name, rest)) = rest.split_once(": &str =") else { continue };
        let rest = rest.trim().trim_end_matches(';').trim();
        if !rest.starts_with('"') || !rest.ends_with('"') {
            continue;
        }
        consts.push_str(&format!(
            "export const {name} = {value} as const;\n",
            name = name.trim(),
            value = rest,
        ));
    }

    format!("{struct_ts}\n// ── Event-type constants (mirrored from crates/core/ship-substrate/crates/ship-types/src/event_types/mod.rs) ──\n\n{consts}")
}
