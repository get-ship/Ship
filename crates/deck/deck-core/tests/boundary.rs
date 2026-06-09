//! Boundary invariant I2 — Deck's HOST and BUNDLER crates name no Ship app
//! concepts.
//!
//! deck-core / deck-axum / deck-host serve SUI views + assets for ANY host.
//! deck-bundler builds those assets. deck-config holds the config schema.
//! None of them may hardcode a Ship app layout (`apps/shell`, `apps/shipd`,
//! `apps/site`) or a Ship-specific identifier (`sui-chrome`, `sui-marketing`,
//! `sui-arcade`, `sui-pdf`, `__SHIP_APP_ID__`) in PRODUCTION source lines.
//!
//! The three deck-own singleton bundle names (sui-runtime, sui-dom,
//! sui-sdk-client) are legitimately hardcoded in deck-bundler and ARE expected.
//! (`sui-components` is no longer a singleton — the umbrella was retired
//! (#221/#269); `sui-components/<tag>` is now the per-tag resolver namespace.)
//!
//! Exclusions:
//!   - Lines starting with `//` (doc/line comments — may cite paths as examples)
//!   - Lines inside `#[cfg(test)]` + `mod tests` blocks (fixtures legitimately
//!     use these paths as concrete examples of what a host passes in)
//!   - Dedicated `*_tests.rs` files

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &[&str] = &[
    "apps/shell",
    "apps/shipd",
    "apps/site",
    "sui-chrome",
    "sui-marketing",
    "sui-arcade",
    "sui-pdf",
    "__SHIP_APP_ID__",
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs")
            // Skip dedicated test files — they legitimately use realistic path
            // fixtures. The gate is about production source.
            && !p.file_name().is_some_and(|n| {
                let s = n.to_string_lossy();
                s.ends_with("_tests.rs") || s == "boundary.rs"
            })
        {
            out.push(p);
        }
    }
}

/// Scan a file's non-comment production lines for forbidden literals.
/// Skips lines inside `#[cfg(test)]` attribute + `mod tests { … }` blocks
/// so test fixtures that pass realistic Ship paths don't trigger.
fn scan_violations(f: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(f) else { return vec![] };
    let mut violations = Vec::new();
    let mut brace_depth: i32 = 0;
    // Track whether we've seen a `#[cfg(test)]` or `mod tests` opener at the
    // current outer brace depth. Simple heuristic: once we enter a `mod tests`
    // block we suppress until we leave that brace depth. This doesn't handle
    // all edge cases but is sufficient for the Rust source shapes in deck crates.
    let mut test_block_depth: Option<i32> = None;

    for (n, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();

        // Count braces to track depth (crude but reliable for well-formatted Rust).
        for ch in line.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    // If we just closed the test block, clear the marker.
                    if let Some(d) = test_block_depth {
                        if brace_depth < d {
                            test_block_depth = None;
                        }
                    }
                }
                _ => {}
            }
        }

        // Mark entry into a test block.
        if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
            if test_block_depth.is_none() {
                test_block_depth = Some(brace_depth);
            }
        }

        // Suppress lines inside test blocks.
        if test_block_depth.is_some() {
            continue;
        }

        // Suppress comment lines — they may cite paths as prose examples.
        if trimmed.starts_with("//") {
            continue;
        }

        for forbidden in FORBIDDEN {
            if line.contains(forbidden) {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, trimmed));
                break;
            }
        }
    }
    violations
}

#[test]
fn deck_crates_name_no_ship_app() {
    // crates/deck/deck-core/  → ../  → crates/deck/
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let mut files = Vec::new();
    for crate_name in [
        "deck", "deck-cli", "deck-core", "deck-axum", "deck-host", "deck-bundler", "deck-config",
    ] {
        let src = deck_dir.join(crate_name).join("src");
        collect_rs(&src, &mut files);
        // Also include build.rs at the crate root if present.
        let build_rs = deck_dir.join(crate_name).join("build.rs");
        if build_rs.is_file() {
            files.push(build_rs);
        }
    }
    assert!(!files.is_empty(), "found no deck crate source to scan ({deck_dir:?})");

    let mut violations = Vec::new();
    for f in &files {
        violations.extend(scan_violations(f));
    }
    assert!(
        violations.is_empty(),
        "I2 violation: deck crates must not hardcode Ship app concepts \
         in production source (deck is told its roots, never assumes a \
         Ship app layout):\n\n{}\n\nForbidden set: {:?}",
        violations.join("\n"),
        FORBIDDEN,
    );
}

/// Positive zero-config test: `DeckConfig::defaults()` + a temp `views/foo/page.sui`
/// yields exactly one views/foo target and zero consumer targets.
#[test]
fn zero_config_discovers_views_only() {
    use deck_bundler::targets::{BundleLayout, BundleTarget};

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Write a single view.
    let page_sui = root.join("views/foo/page.sui");
    std::fs::create_dir_all(page_sui.parent().unwrap()).unwrap();
    std::fs::write(&page_sui, "<div>foo</div>\n").unwrap();

    // Zero-config host: DeckConfig::defaults() (no deck.jsonc).
    let cfg = deck_config::DeckConfig::defaults();
    let layout = BundleLayout::with_config(root, None, cfg);

    let targets: Vec<BundleTarget> = layout.all_targets();
    let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

    // Framework singletons are always present (the 3 deck-own bundles).
    // sui-components is NOT one — the umbrella was retired (#221/#269); the
    // `sui-components/<tag>` prefix is now the per-tag resolver namespace, not a
    // singleton target. Asserting its presence would resurrect a deleted thing.
    assert!(names.contains(&"sui-runtime"), "sui-runtime present");
    assert!(names.contains(&"sui-dom"), "sui-dom present");
    assert!(names.contains(&"sui-sdk-client"), "sui-sdk-client present");

    // Discovered view.
    assert!(names.contains(&"views/foo/view"), "views/foo/view discovered");

    // Zero consumer targets from defaults (no deck.jsonc declares any).
    assert!(
        layout.consumer_targets().is_empty(),
        "zero consumer targets with defaults"
    );
}

/// Boundary invariant I2 (extension) — deck-core and deck-config declare
/// no `ship-substrate` dependency. The project-root resolver in
/// `deck_core::resolver` and the JSONC parser in `deck-config` must
/// operate on std alone (override + nearest-ancestor marker detection +
/// inlined stripper). Any re-introduction of `ship-substrate` into
/// either crate's `Cargo.toml` re-threads the up-leak this slice cut.
#[test]
fn deck_core_and_deck_config_have_no_ship_substrate_dep() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    for crate_name in ["deck-core", "deck-config"] {
        let toml_path = deck_dir.join(crate_name).join("Cargo.toml");
        let contents = fs::read_to_string(&toml_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
        // Search production `[dependencies]` + `[dev-dependencies]` tables
        // only — `[workspace.dependencies]` and any in-comment mentions
        // do not count as a dep declaration.
        assert!(
            !contents.contains("ship-substrate"),
            "{} declares a `ship-substrate` dependency — cut the up-leak \
             by inlining (deck-config) or replacing with std (deck-core)",
            toml_path.display(),
        );
    }
}

/// Belt-and-suspenders: also assert the production source itself has no
/// references to the deleted resolver helpers, the legacy project-root
/// env vars, or the `deck_substrate::*` paths. The Cargo.toml check above
/// catches the dep, this catches a residual dead-code reference that the
/// compiler might not warn about. Scans `deck-core` only (deck-config
/// is a tiny types crate with no FS coupling to worry about).
#[test]
fn deck_core_source_has_no_deleted_resolver_helpers_or_project_dir_env() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let src = deck_dir.join("deck-core").join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);
    let forbidden = [
        "DECK_PROJECT_DIR",
        "SHIP_DEV_PROJECT_DIR",
        "resolve_project_ship_dir",
        "daemon_dir_for",
        "workspace_from_daemon_file",
    ];
    let mut violations = Vec::new();
    for f in &files {
        let body = fs::read_to_string(f).unwrap();
        for token in &forbidden {
            if body.contains(token) {
                violations.push(format!("{}: contains `{}`", f.display(), token));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "deck-core production source must not name the legacy project-root \
         envs or the deleted resolver helpers:\n{}",
        violations.join("\n"),
    );
}

/// Boundary invariant I2 (extension) — deck crates name no Ship app env vars.
///
/// The previous slice renamed `SHIP_FIRST_PARTY_VIEWS` → `DECK_FIRST_PARTY_VIEWS`
/// to make the deck→ship dependency direction explicit. Deck crates now have
/// zero Ship-prefixed env vars; the boundary rule (deck names no Ship concept)
/// forbids re-introducing any `std::env::var("SHIP_…")` read inside deck
/// production source. Hosts that need Ship-app envs are wired them in via
/// the env module, not directly in deck.
///
/// Scans every deck crate (`deck-core`, `deck-axum`, `deck-host`,
/// `deck-bundler`, `deck-config`) for the `env::var("SHIP_…")` pattern.
/// Test files (`*_tests.rs` + the boundary.rs itself) are skipped — the
/// pattern is the literal phrase `env::var("SHIP` so a doc comment
/// mentioning the gate's intent is fine, but a real read in production
/// source is a violation.
#[test]
fn deck_crates_have_no_ship_prefixed_env_var_reads() {
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let mut files = Vec::new();
    for crate_name in [
        "deck", "deck-cli", "deck-core", "deck-axum", "deck-host", "deck-bundler", "deck-config",
    ] {
        let src = deck_dir.join(crate_name).join("src");
        collect_rs(&src, &mut files);
        let build_rs = deck_dir.join(crate_name).join("build.rs");
        if build_rs.is_file() {
            files.push(build_rs);
        }
    }
    assert!(!files.is_empty(), "found no deck crate source to scan ({deck_dir:?})");

    let mut violations = Vec::new();
    // We need a real shell boundary check: any line whose payload contains
    // the executable form `env::var("SHIP` outside a comment fails. We
    // reuse the same suppression machinery as the I2 scan: lines inside
    // `mod tests` blocks, doc comments, and `*_tests.rs` files are
    // excluded.
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        let mut brace_depth: i32 = 0;
        let mut test_block_depth: Option<i32> = None;
        for (n, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            for ch in line.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => {
                        brace_depth -= 1;
                        if let Some(d) = test_block_depth {
                            if brace_depth < d {
                                test_block_depth = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
                if test_block_depth.is_none() {
                    test_block_depth = Some(brace_depth);
                }
            }
            if test_block_depth.is_some() {
                continue;
            }
            if trimmed.starts_with("//") {
                continue;
            }
            // The exact pattern: `env::var("SHIP…` — any SHIP-prefixed env
            // name. This is a real env read, not a placeholder.
            if line.contains("env::var(\"SHIP") {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, trimmed));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "I2 (SHIP_ env) violation: deck crates must not read Ship-prefixed \
         env vars directly (deck names no Ship concept — wire the host's \
         env accessor instead):\n\n{}",
        violations.join("\n"),
    );
}
