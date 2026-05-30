/// Integration tests — compose sub-domain.
///
/// Exercises `materialize_space` and `composed_root_for` at the
/// cross-boundary level: depless fallback, dep namespace layout, cache
/// re-use, and the collection-schema collision gate.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use compiler::lockfile::{LockPackage, ShipLock};

use crate::registry::compose::{composed_root_for, materialize_space};

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full, content).unwrap();
}

fn lock_pkg(path: &str, hex: &str) -> LockPackage {
    LockPackage {
        path: path.into(),
        version: "1.0.0".into(),
        commit: "a".repeat(40),
        hash: format!("sha256:{hex}"),
        export_hashes: BTreeMap::new(),
    }
}

fn write_lock(ship_dir: &Path, pkgs: Vec<LockPackage>) {
    let lock = ShipLock { version: 1, packages: pkgs };
    lock.write_atomic(&ship_dir.join("ship.lock")).unwrap();
}

// ── depless space: returns ship_dir unchanged ─────────────────────────────────

#[test]
fn registry_compose_depless_space_returns_raw_ship_dir() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    write_file(&ship_dir, "pipelines/build.jsonc", r#"{"id":"build"}"#);
    // No ship.lock → depless.

    let composed = materialize_space(&ship_dir, Some(&cache_root))?;
    assert!(
        composed.join("pipelines/build.jsonc").is_file(),
        "local pipeline mirrored into composed root"
    );
    Ok(())
}

// ── dep adds pipeline under package namespace ─────────────────────────────────

#[test]
fn registry_compose_dep_pipeline_namespaced_under_package_path() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");
    let hex = "feedface";

    write_file(&ship_dir, "pipelines/local.jsonc", r#"{"id":"local"}"#);

    // Dep object in cache.
    let obj = cache_root.join("objects").join(hex);
    write_file(
        &obj,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/x/y", "version": "1.0.0" },
            "exports": { "pipelines": ["pipelines/dep.jsonc"] } }"#,
    );
    write_file(&obj, "pipelines/dep.jsonc", r#"{"id":"dep"}"#);
    write_lock(&ship_dir, vec![lock_pkg("github.com/x/y", hex)]);

    let composed = materialize_space(&ship_dir, Some(&cache_root))?;

    assert!(
        composed.join("pipelines/local.jsonc").is_file(),
        "local pipeline preserved"
    );
    assert!(
        composed.join("pipelines/github.com/x/y/dep.jsonc").is_file(),
        "dep pipeline namespaced under package path"
    );
    Ok(())
}

// ── cache re-use (determinism) ────────────────────────────────────────────────

#[test]
fn registry_compose_same_inputs_reuse_cached_dir() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    write_file(&ship_dir, "pipelines/build.jsonc", r#"{"id":"build"}"#);

    let first = materialize_space(&ship_dir, Some(&cache_root))?;
    // Plant a marker — a rebuild would not carry it over.
    fs::write(first.join("MARKER"), "sentinel")?;

    let second = materialize_space(&ship_dir, Some(&cache_root))?;
    assert_eq!(first, second, "identical inputs must yield same composed dir path");
    assert!(second.join("MARKER").is_file(), "cached dir reused — MARKER still present");
    Ok(())
}

// ── collection-schema collision ───────────────────────────────────────────────

#[test]
fn registry_compose_collection_schema_collision_returns_error() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let ship_dir = tmp.path().join(".ship");
    let cache_root = tmp.path().join("cache");

    let hex_a = "aaaa0000";
    let obj_a = cache_root.join("objects").join(hex_a);
    write_file(
        &obj_a,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/a/one", "version": "1.0.0" },
            "exports": { "collections": ["collections/notes.jsonc"] } }"#,
    );
    write_file(&obj_a, "collections/notes.jsonc", r#"{"name":"notes"}"#);

    let hex_b = "bbbb1111";
    let obj_b = cache_root.join("objects").join(hex_b);
    write_file(
        &obj_b,
        "ship.jsonc",
        r#"{ "module": { "name": "github.com/b/two", "version": "1.0.0" },
            "exports": { "collections": ["collections/notes.jsonc"] } }"#,
    );
    write_file(&obj_b, "collections/notes.jsonc", r#"{"name":"notes"}"#);

    fs::create_dir_all(&ship_dir)?;
    write_lock(
        &ship_dir,
        vec![lock_pkg("github.com/a/one", hex_a), lock_pkg("github.com/b/two", hex_b)],
    );

    let err = materialize_space(&ship_dir, Some(&cache_root)).expect_err("collision must error");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("collection-schema collision"),
        "expected collision error, got: {msg}"
    );
    Ok(())
}

// ── composed_root_for — depless fallback ──────────────────────────────────────

#[test]
fn registry_compose_composed_root_for_depless_returns_ship_dir() {
    let tmp = tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    fs::create_dir_all(&ship_dir).unwrap();
    // No ship.lock → depless fallback.
    let result = composed_root_for(&ship_dir);
    assert_eq!(result, ship_dir, "depless space must return ship_dir unchanged");
}

#[test]
fn registry_compose_composed_root_for_empty_lock_returns_ship_dir() {
    let tmp = tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    fs::create_dir_all(&ship_dir).unwrap();
    // Write a lock with no packages — still depless.
    let lock = ShipLock { version: 1, packages: vec![] };
    lock.write_atomic(&ship_dir.join("ship.lock")).unwrap();
    let result = composed_root_for(&ship_dir);
    assert_eq!(result, ship_dir, "empty lock must return ship_dir unchanged");
}
