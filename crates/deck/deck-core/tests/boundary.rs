//! Boundary invariant I2 — Deck's HOST crates name no Ship app concepts.
//!
//! deck-core / deck-axum / deck-host serve SUI views + assets for ANY host.
//! They are TOLD their roots (project dir, `SHIP_FIRST_PARTY_VIEWS`) and must
//! never hardcode a Ship app's layout (`apps/shell`). Hardcoding it couples the
//! framework to one consumer — the recurring "where does this live" smell.
//!
//! Scope: the host crates only. The deck BUNDLER (packages/deck/bundler) is
//! build-time tooling that legitimately enumerates what to build, so it's a
//! separate concern (externalizing its roots is tracked elsewhere).
//!
//! Test fixtures simulate a project tree (`.ship/views/...`) — that's expected
//! and not a hardcode of the host's own layout, so we gate only on `apps/shell`,
//! which has no legitimate place in deck host source at all.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: &str = "apps/shell";

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs")
            // Skip dedicated test files — they legitimately use a realistic
            // apps/shell root as a fixture (the host's role is to PASS such a
            // root). The gate is about production host code.
            && !p.file_name().is_some_and(|n| n.to_string_lossy().ends_with("_tests.rs"))
        {
            out.push(p);
        }
    }
}

#[test]
fn deck_host_crates_name_no_ship_app() {
    // crates/deck/deck-core/  → ../  → crates/deck/
    let deck_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let mut files = Vec::new();
    for host in ["deck-core", "deck-axum", "deck-host"] {
        collect_rs(&deck_dir.join(host).join("src"), &mut files);
    }
    assert!(!files.is_empty(), "found no host-crate source to scan ({deck_dir:?})");

    let mut violations = Vec::new();
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        for (n, line) in text.lines().enumerate() {
            // Doc/line comments may reference apps/shell as an EXAMPLE root
            // (the host passes it); the gate is about code hardcodes.
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains(FORBIDDEN) {
                violations.push(format!("{}:{}: {}", f.display(), n + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "I2 violation: deck host crates must not hardcode `{FORBIDDEN}` (deck is told its \
         roots, never assumes a Ship app layout):\n{}",
        violations.join("\n"),
    );
}
