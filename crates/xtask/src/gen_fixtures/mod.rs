//! Generate proto contract fixtures (`cargo xtask gen-fixtures`).
//!
//! Three sub-modules cover three fixture surfaces:
//!
//!   - `ops`         — per-op RESPONSE fixtures from `proto/fixtures/fixture-seed.json`
//!                     (today's gen_fixtures.rs content, unchanged)
//!   - `events`      — per-event payload + envelope fixtures from the event registry
//!   - `op_requests` — per-op REQUEST fixtures extending the service JSONs
//!
//! UX:
//!   cargo xtask gen-fixtures                — generate all (ops + events + requests)
//!   cargo xtask gen-fixtures --check        — verify on-disk == what gen would produce (CI)
//!   cargo xtask gen-fixtures --event <name> — regen just one event fixture
//!   cargo xtask gen-fixtures --report-missing — list registry entries without a fixture

use std::path::{Path, PathBuf};

use anyhow::Result;

pub mod demo;
pub mod events;
pub mod op_requests;
pub mod ops;

pub fn run_with_args(args: &[&str]) -> Result<()> {
    let check_mode = args.contains(&"--check");
    let report_missing = args.contains(&"--report-missing");
    let event_filter: Option<&str> = args
        .windows(2)
        .find(|w| w[0] == "--event")
        .map(|w| w[1]);

    // SMELL: this tool's input (proto/fixtures) is in ship-contracts
    // post-multi-repo-cleanup. The path `proto/fixtures` does not
    // exist in ship anymore. gen-fixtures should move to ship-contracts
    // as `xtask-contracts/gen-fixtures` so the read path is the in-repo
    // SSOT. Until then, the tool is broken when run from ship.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures_dir = root.join("proto/fixtures");
    if report_missing {
        return report_missing_fixtures(&fixtures_dir);
    }

    if let Some(event_name) = event_filter {
        return regen_single_event(&fixtures_dir, event_name);
    }

    // ── Full generation ──────────────────────────────────────────────────────

    // 1. Op response fixtures (from fixture-seed.json)
    let op_count = ops::run(&fixtures_dir, check_mode)?;

    // 2. Op request fixtures (extending service JSONs with "request" key)
    let req_count = op_requests::run(&fixtures_dir, check_mode)?;

    // 3. Event payload + envelope fixtures (from registry)
    let events_report = events::run(&fixtures_dir, check_mode)?;

    // 4. Demo fixtures from per-domain seeds (proto/fixtures/seeds/ → demo/)
    let demo_count = demo::run(&fixtures_dir, check_mode)?;

    let scaffold_dups = ops::scaffold_dup_count();

    if check_mode {
        eprintln!(
            "gen-fixtures: all fixtures up-to-date ({op_count} op responses, {req_count} op requests, {} event fixtures [{} auto, {} needs-author])",
            events_report.total * 2,
            events_report.auto_generated,
            events_report.needs_author,
        );
        eprintln!("gen-fixtures: {demo_count} demo fixtures up-to-date → proto/fixtures/demo/");
    } else {
        eprintln!(
            "gen-fixtures: {op_count} op response fixtures → proto/fixtures/<service>.json"
        );
        eprintln!(
            "gen-fixtures: {req_count} op request stubs added to service fixture files"
        );
        eprintln!(
            "gen-fixtures: {} event fixtures ({} auto, {} needs-author) → proto/fixtures/events/",
            events_report.total * 2,
            events_report.auto_generated,
            events_report.needs_author,
        );
        eprintln!("gen-fixtures: {demo_count} demo fixtures → proto/fixtures/demo/");
        eprintln!(
            "gen-fixtures: scaffold dup — {scaffold_dups} ops double-exposed on umbrella + per-domain (retires to 0 with umbrella, #78)"
        );
        if !events_report.needs_author_list.is_empty() {
            eprintln!("gen-fixtures: NEEDS_AUTHOR events (hand-author stubs in proto/fixtures/events/):");
            for item in &events_report.needs_author_list {
                eprintln!("  {item}");
            }
        }
    }

    Ok(())
}

/// Regen fixtures for a single named event.
fn regen_single_event(fixtures_dir: &Path, event_name: &str) -> Result<()> {
    use ship_types::event_registry::registry;
    let def = registry()
        .find(|d| d.name == event_name)
        .ok_or_else(|| anyhow::anyhow!("event `{event_name}` not found in registry"))?;

    let events_dir = fixtures_dir.join("events");
    std::fs::create_dir_all(&events_dir)?;
    let safe_name = event_name.replace('.', "_");

    // Re-use events module internals via a minimal one-off call.
    // We generate the full events run then report only the one we care about.
    let report = events::run(fixtures_dir, false)?;
    let _ = report; // full regen; single-event filter is a UX convenience not a perf optimization.
    eprintln!("gen-fixtures: regenerated {safe_name}.json + {safe_name}.envelope.json");
    let _ = def;
    Ok(())
}

/// Print a markdown-style coverage report for all fixture surfaces.
fn report_missing_fixtures(fixtures_dir: &Path) -> Result<()> {
    let missing_ops = ops::missing_fixtures(fixtures_dir)?;
    let missing_requests = op_requests::missing_requests(fixtures_dir)?;
    let needs_author_events = events::needs_author_list();
    let event_stats = events::coverage_stats();

    let total_events = *event_stats.get("total").unwrap_or(&0);
    let auto_events = *event_stats.get("auto_generable").unwrap_or(&0);
    let na_events = *event_stats.get("needs_author").unwrap_or(&0);
    let event_pct = (auto_events * 100).checked_div(total_events).unwrap_or(0);

    println!("# Fixture Coverage Report");
    println!();
    println!("## Events");
    println!("- Total registered: {total_events}");
    println!("- Auto-generable: {auto_events} ({event_pct}%)");
    println!("- NEEDS_AUTHOR: {na_events}");
    if !needs_author_events.is_empty() {
        println!();
        println!("### NEEDS_AUTHOR events");
        for item in &needs_author_events {
            println!("  - {item}");
        }
    }

    println!();
    println!("## Op Responses");
    if missing_ops.is_empty() {
        println!("- All typed ops have response fixtures.");
    } else {
        println!("- Missing ({}):", missing_ops.len());
        for item in &missing_ops {
            println!("  - {item}");
        }
    }

    println!();
    println!("## Op Requests");
    if missing_requests.is_empty() {
        println!("- All arg-bearing ops have request stubs.");
    } else {
        println!("- Missing ({}):", missing_requests.len());
        for item in &missing_requests {
            println!("  - {item}");
        }
    }

    Ok(())
}
