//! `gen-sdk` — emit five machine-readable JSON files that describe the entire
//! Ship authoring surface. Downstream tasks (AST lint, view-meta endpoint, MCP
//! scaffolder, editor tooling) read these instead of re-parsing raw source.
//!
//! Output lives under `packages/sdk/src/manifests/` (committed; un-ignored by
//! `.gitignore` so drift surfaces in PRs). The previous worktree-only landing
//! wrote to `packages/sdk/src/generated/` which stayed `.gitignore`d — the
//! drift gate was a lie. The R4 resolution in `census-phase1-gen.md` Part 7
//! J3 picks `manifests/` for the published SDK shape.
//!
//! `gen-sdk --check` regenerates in-memory and compares against the committed
//! files; exit 0 on match, 1 on drift. Run `gen-sdk` (no flag) to refresh.
//!
//! Inputs: `.ship/` config (collections, triggers, views), shipd view Ctx
//! structs, hand-authored framework manifest — all ship-side, so the tool
//! stays ship-internal, not contracts codegen.

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

const MANIFESTS_DIR: &str = "packages/sdk/src/manifests";

/// Entry point. With `check = false`: writes the 5 JSON manifests + 1 TS
/// typed-FrameworkManifest file under `packages/sdk/src/manifests/`. With
/// `check = true`: regenerates the same content in memory and diffs against
/// the committed files; exits 1 on any drift (or missing file).
pub fn run(check: bool) -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("cannot locate repo root")
        .to_path_buf();
    let out_dir = repo_root.join(MANIFESTS_DIR);

    let project = ship_substrate::ShipHome::project_ship_dir(&repo_root);
    let outputs = build_outputs(&repo_root, &project)?;

    if check {
        let mut stale = false;
        for (rel, content) in &outputs {
            let path = out_dir.join(rel);
            let existing = fs::read_to_string(&path).unwrap_or_default();
            if existing != *content {
                eprintln!("gen-sdk: out of date: {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-sdk` and commit the result.");
            std::process::exit(1);
        }
        eprintln!(
            "gen-sdk: up-to-date ({} files in {})",
            outputs.len(),
            out_dir.display()
        );
        return Ok(());
    }

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    for (rel, content) in &outputs {
        let path = out_dir.join(rel);
        fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    }
    eprintln!(
        "gen-sdk: wrote {} files to {}",
        outputs.len(),
        out_dir.display()
    );
    Ok(())
}

/// Compute every manifest in memory. Each entry is the file's relative path
/// (under `MANIFESTS_DIR`) and the bytes that would be written.
fn build_outputs(repo_root: &Path, project_ship_dir: &Path) -> Result<Vec<(&'static str, String)>> {
    Ok(vec![
        (
            "sdk-view-contexts.json",
            render_doc("sdk-view-contexts", view_contexts::extract(repo_root)?),
        ),
        (
            "sdk-collections.json",
            render_doc(
                "sdk-collections",
                collect_jsonc_dir(&project_ship_dir.join("collections"), |v| {
                    v.get("id").and_then(|x| x.as_str()).map(str::to_owned)
                })?,
            ),
        ),
        (
            "sdk-triggers.json",
            render_doc(
                "sdk-triggers",
                collect_jsonc_dir(&project_ship_dir.join("triggers"), |v| {
                    v.get("name").and_then(|x| x.as_str()).map(str::to_owned)
                })?,
            ),
        ),
        (
            "sdk-views.json",
            render_doc(
                "sdk-views",
                collect_view_manifests(&project_ship_dir.join("views"))?,
            ),
        ),
        ("sdk-framework.json", render_framework(repo_root)?),
        ("framework.ts", framework::emit_ts(repo_root)?),
    ])
}

/// Wrap a manifest payload in the standard envelope
/// `{_generated, version, $schema, entries}` and return the rendered JSON.
fn render_doc(name: &str, entries: Value) -> String {
    let mut root = Map::new();
    root.insert("_generated".into(), Value::String(HEADER.into()));
    root.insert("version".into(), Value::String(VERSION.into()));
    root.insert(
        "$schema".into(),
        Value::String(format!("https://ship.design/schemas/{name}.schema.json")),
    );
    root.insert("entries".into(), entries);
    serde_json::to_string_pretty(&Value::Object(root))
        .expect("infallible serde_json::to_string_pretty")
        + "\n"
}

/// Render `sdk-framework.json`. The framework payload sits alongside
/// `version`/`_generated` rather than under `entries` — the whole manifest
/// is the entry.
fn render_framework(repo_root: &Path) -> Result<String> {
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
    Ok(serde_json::to_string_pretty(&Value::Object(root))? + "\n")
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
    Ok(Value::Object(out))
}
