//! PROOF (throwaway): a space resolves a component from a DECLARED dependency
//! through the registry compose layer — not from the monorepo workspace.
//!
//! This is the seam where `ship-platform` (registry/compose) meets `deck` (the
//! bundler's component discovery). The daemon is the only place both link, so
//! the end-to-end proof lives here. Test-only.

#![cfg(test)]

use std::collections::BTreeMap;
use std::path::Path;

use compiler::lockfile::{LockPackage, ShipLock};
use deck::bundler::catalog::composed_component_index;
use ship_platform::registry::compose::materialize_space;

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(&full, content).unwrap();
}

/// End-to-end: a space declares a component-pack dep in its lockfile; the pack
/// (exporting `ship-agent-card`) sits in the content-addressed object cache;
/// compose materializes the space; the deck bundler's component discovery then
/// resolves `ship-agent-card` from the COMPOSED root. The resolved path lives
/// under the composed cache — never `packages/*` — proving the space gets the
/// component from its declared dependency, with the monorepo coupling broken.
#[test]
fn space_resolves_component_from_declared_dep_via_compose() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    // Seed the object cache with a component pack exporting one component.
    let hex = "5eed1abc";
    let obj = cache_root.join("objects").join(hex);
    write_file(
        &obj,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/acme/agents", "version": "1.0.0" },
            "exports": { "components": ["components/ship-agent-card"] } }"#,
    );
    write_file(
        &obj,
        "components/ship-agent-card/ship-agent-card.sui",
        "<article class=\"agent-card\"><slot></slot></article>\n",
    );

    // The space declares the dep in its lockfile (what `ship add` would write).
    std::fs::create_dir_all(&ship_dir).unwrap();
    let lock = ShipLock {
        version: 1,
        packages: vec![LockPackage {
            path: "github.com/acme/agents".into(),
            version: "1.0.0".into(),
            commit: "a".repeat(40),
            hash: format!("sha256:{hex}"),
            export_hashes: BTreeMap::new(),
        }],
    };
    lock.write_atomic(&ship_dir.join("ship.lock")).unwrap();

    // ── the seam: compose → bundler discovery ──────────────────────────────
    let composed = materialize_space(&ship_dir, Some(&cache_root)).unwrap();
    let index = composed_component_index(&composed);

    // The declared dep's component resolves to a real `.sui` under the composed
    // root — NOT from the monorepo `packages/*`.
    let src = index
        .get("ship-agent-card")
        .expect("declared-dep component must resolve from the composed root");
    assert!(src.is_file(), "resolved path is a real .sui file");
    assert!(
        src.starts_with(&cache_root),
        "resolved from the composed cache ({}), not the monorepo: {}",
        cache_root.display(),
        src.display()
    );
    assert!(src.ends_with("ship-agent-card/ship-agent-card.sui"));

    // A component NOT declared as a dep is unreachable in this space
    // (access-by-dependency).
    assert!(
        !index.contains_key("ship-not-installed"),
        "an undeclared component must be unreachable in this space"
    );
}
