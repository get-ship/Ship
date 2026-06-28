//! `gen-sdk` — emit five machine-readable JSON files that describe the entire
//! Ship authoring surface. Downstream tasks (AST lint, view-meta endpoint, MCP
//! scaffolder, editor tooling) read these instead of re-parsing raw source.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

mod framework;
mod jsonc;
mod view_contexts;

const VERSION: &str = "0.1.0";
const HEADER: &str = "just gen-sdk — DO NOT EDIT";

/// Entry point. Writes the JSON artifacts + typed TS bindings to the
/// in-repo path `packages/sdk/src/generated/` (the path shipd's
/// include_str! expects; see the bootstrap assert in
/// `.github/workflows/ci.yml`).
///
/// SMELL: this gen-* tool lives in ship but its output (the SDK) lives
/// in ship-contracts post-multi-repo-cleanup. The default path
/// `packages/sdk/src/generated/` does not exist in ship anymore —
/// gen-sdk is one of the cross-repo codegen tools that the architecture
/// rule says should live with its output SSOT (i.e. ship-contracts).
/// It still runs in ship because the input (apps/shipd routes, the
/// schemas, the framework surface) is ship-side; the right answer is
/// to move it AND publish the input sources as ship crates. Until
/// then, the build is broken because the default output path doesn't
/// exist.
pub fn run(repo_root: &Path) -> Result<()> {
    let out_dir = repo_root.join("packages/sdk/src/generated");
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    write_doc(
        &out_dir.join("sdk-view-contexts.json"),
        "sdk-view-contexts",
        view_contexts::extract(repo_root)?,
    )?;
    write_doc(
        &out_dir.join("sdk-collections.json"),
        "sdk-collections",
        collect_jsonc_dir(
            &ship_substrate::ShipHome::project_ship_dir(repo_root).join("collections"),
            |v| v.get("id").and_then(|x| x.as_str()).map(str::to_owned),
        )?,
    )?;
    write_doc(
        &out_dir.join("sdk-triggers.json"),
        "sdk-triggers",
        collect_jsonc_dir(
            &ship_substrate::ShipHome::project_ship_dir(repo_root).join("triggers"),
            |v| v.get("name").and_then(|x| x.as_str()).map(str::to_owned),
        )?,
    )?;
    write_doc(
        &out_dir.join("sdk-views.json"),
        "sdk-views",
        collect_view_manifests(&ship_substrate::ShipHome::project_ship_dir(repo_root).join("views"))?,
    )?;
    // sdk-framework.json describes framework-level concepts (directives,
    // scope bindings, template syntax, reactivity rules). Its envelope
    // version tracks the manifest's own surface, not gen-sdk's VERSION —
    // those evolve independently.
    write_framework(&out_dir.join("sdk-framework.json"), repo_root)?;

    // Typed framework manifest — `FrameworkManifest` interface +
    // `FRAMEWORK` const. Pairs with the JSON for TS-side consumers.
    let framework_ts_path = out_dir.join("framework.ts");
    fs::write(&framework_ts_path, framework::emit_ts(repo_root)?)
        .with_context(|| format!("write {}", framework_ts_path.display()))?;

    eprintln!(
        "gen-sdk: wrote 5 JSON + 1 TS files to {}",
        out_dir.display()
    );
    Ok(())
}

fn write_framework(path: &PathBuf, repo_root: &Path) -> Result<()> {
    let mut root = Map::new();
    root.insert("_generated".into(), Value::String(HEADER.into()));
    root.insert(
        "version".into(),
        Value::String(framework::manifest_version().into()),
    );
    root.insert(
        "$schema".into(),
        Value::String("https://ship.design/schemas/sdk-framework.schema.json".into()),
    );
    // The manifest payload sits alongside `version`/`_generated` rather
    // than under an `entries` key — the entire manifest is the entry.
    if let Value::Object(map) = framework::json_entries(repo_root)? {
        for (k, v) in map {
            // Skip the inner `version` to avoid shadowing the envelope
            // version; the same value is exposed on the envelope.
            if k == "version" {
                continue;
            }
            root.insert(k, v);
        }
    }
    let text = serde_json::to_string_pretty(&Value::Object(root))? + "\n";
    fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_doc(path: &PathBuf, name: &str, entries: Value) -> Result<()> {
    let mut root = Map::new();
    root.insert("_generated".into(), Value::String(HEADER.into()));
    root.insert("version".into(), Value::String(VERSION.into()));
    root.insert(
        "$schema".into(),
        Value::String(format!("https://ship.design/schemas/{name}.schema.json")),
    );
    root.insert("entries".into(), entries);
    let text = serde_json::to_string_pretty(&Value::Object(root))? + "\n";
    fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Walk a directory of `.jsonc` files, strip comments, return a map keyed by
/// whatever `key_of` extracts from each parsed value (falls back to file stem).
fn collect_jsonc_dir<F>(dir: &Path, key_of: F) -> Result<Value>
where
    F: Fn(&Value) -> Option<String>,
{
    let mut out = Map::new();
    if !dir.exists() {
        return Ok(Value::Object(out));
    }
    let mut files: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("jsonc"))
        .collect();
    files.sort();
    for path in files {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let value = jsonc::parse(&raw)
            .with_context(|| format!("parse {}", path.display()))?;
        let key = key_of(&value)
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_owned())
            })
            .unwrap_or_default();
        out.insert(key, value);
    }
    Ok(Value::Object(out))
}

/// Walk `.ship/views/<name>/manifest.jsonc`, keyed by view name.
fn collect_view_manifests(views_dir: &Path) -> Result<Value> {
    let mut out = Map::new();
    if !views_dir.exists() {
        return Ok(Value::Object(out));
    }
    let mut entries: Vec<_> = fs::read_dir(views_dir)
        .with_context(|| format!("read_dir {}", views_dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    for view_dir in entries {
        let manifest = view_dir.join("manifest.jsonc");
        if !manifest.exists() {
            continue;
        }
        let name = view_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let raw = fs::read_to_string(&manifest)
            .with_context(|| format!("read {}", manifest.display()))?;
        let mut value = jsonc::parse(&raw)
            .with_context(|| format!("parse {}", manifest.display()))?;
        // Fold in companion manifest.yaml data if view author used yaml (some
        // views use yaml per the skill docs); pragmatic passthrough only.
        if let Value::Object(obj) = &mut value {
            obj.entry("name".to_string())
                .or_insert_with(|| Value::String(name.clone()));
        }
        out.insert(name, value);
    }
    // Also surface yaml-only manifests (data loaders / events / packs).
    let mut yaml_entries: Vec<_> = fs::read_dir(views_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    yaml_entries.sort();
    for view_dir in yaml_entries {
        let name = view_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        if out.contains_key(&name) {
            continue;
        }
        let yml = view_dir.join("manifest.yaml");
        if yml.exists() {
            out.insert(
                name,
                json!({
                    "source": "manifest.yaml",
                    "note": "yaml manifest not parsed; raw passthrough reserved for downstream tooling"
                }),
            );
        }
    }
}
