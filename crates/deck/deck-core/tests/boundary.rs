//! Boundary invariant I2 — Deck's HOST and BUNDLER crates name no Ship app
//! concepts.
//!
//! deck-core / deck-axum / deck-host serve SUI views + assets for ANY host.
//! deck-bundler builds those assets. deck-config holds the config schema.
//! None of them may hardcode a Ship app layout (`apps/shell`, `apps/shipd`,
//! `apps/site`) or a Ship-specific identifier (`sui-chrome`, `sui-marketing`,
//! `sui-arcade`, `sui-pdf`, `__SHIP_APP_ID__`) in PRODUCTION source lines.
//!
//! The four deck-own singleton bundle names (sui-runtime, sui-dom,
//! sui-sdk-client, sui-components) are legitimately hardcoded in deck-bundler
//! and ARE expected.
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
    for crate_name in ["deck-core", "deck-axum", "deck-host", "deck-bundler", "deck-config"] {
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

    // Framework singletons are always present.
    assert!(names.contains(&"sui-runtime"), "sui-runtime present");
    assert!(names.contains(&"sui-dom"), "sui-dom present");
    assert!(names.contains(&"sui-sdk-client"), "sui-sdk-client present");
    assert!(names.contains(&"sui-components"), "sui-components present");

    // Discovered view.
    assert!(names.contains(&"views/foo/view"), "views/foo/view discovered");

    // Zero consumer targets from defaults (no deck.jsonc declares any).
    assert!(
        layout.consumer_targets().is_empty(),
        "zero consumer targets with defaults"
    );
}
