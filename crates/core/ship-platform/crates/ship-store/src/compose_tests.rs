use std::path::Path;

use super::*;
use compiler::lockfile::{LockPackage, ShipLock};
use std::collections::BTreeMap;
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
}

fn lock_with(packages: Vec<LockPackage>) -> ShipLock {
    ShipLock { version: 1, packages }
}

fn pkg(path: &str, hex: &str) -> LockPackage {
    LockPackage {
        path: path.into(),
        version: "1.0.0".into(),
        commit: "a".repeat(40),
        hash: format!("sha256:{hex}"),
        export_hashes: BTreeMap::new(),
    }
}

// ── tests ─────────────────────────────────────────────────────────────

#[test]
fn local_only_mirrors_traveling_dirs() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    // `views/` is intentionally NOT in COMPOSED_TYPES — the resolver owns
    // view composition (`compiler::compose::compose_space` /
    // `ComposedLibrary::route_tree`), not this file materializer. So
    // this test exercises one of the OTHER traveling dirs (pipelines) and
    // confirms a local `views/` is NOT mirrored into the composed overlay.
    write_file(&ship_dir, "pipelines/build.jsonc", r#"{ "id": "build" }"#);
    write_file(&ship_dir, "views/dashboard/page.sui", "<view/>");
    // No lockfile → depless space.

    let composed = materialize_space(&ship_dir, Some(&cache_root)).unwrap();

    assert!(composed.is_dir(), "composed root exists");
    assert!(
        composed.join("pipelines/build.jsonc").is_file(),
        "local pipeline mirrored verbatim"
    );
    // Local `views/` is NOT mirrored — `views` was removed from
    // COMPOSED_TYPES in J2 (resolver-owned, not file-materialized).
    assert!(
        !composed.join("views/dashboard/page.sui").exists(),
        "views/ is composed by the resolver, not mirrored into the file overlay"
    );
    // No dep namespace dirs created for absent types.
    assert!(!composed.join("triggers").exists());
    assert!(!composed.join("collections").exists());
}

#[test]
fn one_dep_pipeline_namespaced_locals_untouched() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    // Local pipeline.
    write_file(&ship_dir, "pipelines/local.jsonc", r#"{ "id": "local" }"#);

    // A dep exporting a pipeline, stored in the object cache.
    let hex = "deadbeef";
    let obj = cache_root.join("objects").join(hex);
    write_file(
        &obj,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/x/y", "version": "1.0.0" },
            "exports": { "pipelines": ["pipelines/dep-pipe.jsonc"] } }"#,
    );
    write_file(&obj, "pipelines/dep-pipe.jsonc", r#"{ "id": "dep-pipe" }"#);

    let lock = lock_with(vec![pkg("github.com/x/y", hex)]);
    lock.write_atomic(&tmp.path().join("ship.lock")).unwrap(); // lockfile at the stack root

    let composed = materialize_space(&ship_dir, Some(&cache_root)).unwrap();

    // Local pipeline untouched.
    assert!(
        composed.join("pipelines/local.jsonc").is_file(),
        "local pipeline preserved"
    );
    // Dep pipeline under its package namespace.
    assert!(
        composed
            .join("pipelines/github.com/x/y/dep-pipe.jsonc")
            .is_file(),
        "dep pipeline composed under package namespace"
    );
}

#[test]
fn one_dep_component_namespaced_resolvable_source() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    // A component pack dep exporting one component dir (`<tag>/<tag>.sui`).
    let hex = "c0ffee01";
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

    std::fs::create_dir_all(&ship_dir).unwrap();
    let lock = lock_with(vec![pkg("github.com/acme/agents", hex)]);
    lock.write_atomic(&tmp.path().join("ship.lock")).unwrap(); // lockfile at the stack root

    let composed = materialize_space(&ship_dir, Some(&cache_root)).unwrap();

    // The dep's component source lands under its package namespace, ready for
    // the bundler's component discovery to scan + resolve `ship-agent-card`.
    let src = composed
        .join("components/github.com/acme/agents/ship-agent-card/ship-agent-card.sui");
    assert!(
        src.is_file(),
        "dep component composed under package namespace at {}",
        src.display()
    );
}

#[test]
fn determinism_same_inputs_reuse_cached_dir() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    write_file(&ship_dir, "pipelines/build.jsonc", r#"{ "id": "build" }"#);

    let first = materialize_space(&ship_dir, Some(&cache_root)).unwrap();
    // Plant a marker; a rebuild into a fresh temp dir would NOT carry it over.
    let marker = first.join("MARKER");
    std::fs::write(&marker, "x").unwrap();

    let second = materialize_space(&ship_dir, Some(&cache_root)).unwrap();

    assert_eq!(first, second, "identical inputs → identical composed hash dir");
    assert!(
        second.join("MARKER").is_file(),
        "cached dir reused as-is (no rebuild)"
    );
}

#[test]
fn collection_schema_collision_errors() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    // Two deps each exporting a same-name collection schema.
    let hex_a = "aaaa1111";
    let obj_a = cache_root.join("objects").join(hex_a);
    write_file(
        &obj_a,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/a/one", "version": "1.0.0" },
            "exports": { "collections": ["collections/notes.jsonc"] } }"#,
    );
    write_file(&obj_a, "collections/notes.jsonc", r#"{ "name": "notes" }"#);

    let hex_b = "bbbb2222";
    let obj_b = cache_root.join("objects").join(hex_b);
    write_file(
        &obj_b,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/b/two", "version": "1.0.0" },
            "exports": { "collections": ["collections/notes.jsonc"] } }"#,
    );
    write_file(&obj_b, "collections/notes.jsonc", r#"{ "name": "notes" }"#);

    std::fs::create_dir_all(&ship_dir).unwrap();
    let lock = lock_with(vec![
        pkg("github.com/a/one", hex_a),
        pkg("github.com/b/two", hex_b),
    ]);
    lock.write_atomic(&tmp.path().join("ship.lock")).unwrap(); // lockfile at the stack root

    let err = materialize_space(&ship_dir, Some(&cache_root)).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("collection-schema collision"),
        "expected collision error, got: {msg}"
    );
}
