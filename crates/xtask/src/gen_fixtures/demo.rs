//! Demo fixtures from per-domain seeds (`proto/fixtures/seeds/<domain>.jsonc`).
//!
//! A **demo fixture** is realistic, decoded-shape example data for component
//! showcases + design handoff — distinct from the conformance fixtures (which
//! carry every field with placeholder values for wire round-trip tests). Both
//! fall out of `seed + proto` and are tracked + CI-drift-gated; only the seed is
//! hand-authored. Convention: `.artifacts/services/_demo-data.md`.
//!
//! Seeds (input, JSONC w/ `//` line comments) → `proto/fixtures/seeds/<domain>.jsonc`.
//! Demo (output, generated)                   → `proto/fixtures/demo/<domain>.json`.
//!
//! The output is the seed content verbatim plus a deterministic `_meta` header
//! (no timestamp — kept stable so the `git diff --exit-code proto/fixtures/` CI
//! gate catches drift). No seeds yet ⇒ this is a graceful no-op (returns 0).

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

/// Strip full-line `//` comments from a JSONC source (the seed comment subset —
/// seeds use whole-line comments only, so we never risk eating a `//` inside a
/// string value such as a URL).
fn strip_jsonc(src: &str) -> String {
    src.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate demo fixtures from every seed under `proto/fixtures/seeds/`.
/// Returns the count written (0 if no seeds dir yet). In `check_mode` it
/// byte-compares against the on-disk output and bails on drift.
pub fn run(fixtures_dir: &PathBuf, check_mode: bool) -> Result<usize> {
    let seeds_dir = fixtures_dir.join("seeds");
    let demo_dir = fixtures_dir.join("demo");
    if !seeds_dir.exists() {
        return Ok(0);
    }
    if !check_mode {
        std::fs::create_dir_all(&demo_dir)?;
    }

    let mut seeds: Vec<PathBuf> = std::fs::read_dir(&seeds_dir)
        .with_context(|| format!("read {}", seeds_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("jsonc"))
        .collect();
    seeds.sort();

    let mut count = 0usize;
    for seed_path in seeds {
        let domain = seed_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("seed file has no stem")?
            .to_string();
        let raw = std::fs::read_to_string(&seed_path)
            .with_context(|| format!("read {}", seed_path.display()))?;
        let seed: Value = serde_json::from_str(&strip_jsonc(&raw))
            .with_context(|| format!("parse seed {} (after JSONC strip)", seed_path.display()))?;

        let mut obj = match seed {
            Value::Object(m) => m,
            other => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), other);
                m
            }
        };
        obj.insert(
            "_meta".to_string(),
            serde_json::json!({
                "generated_by": "cargo xtask gen-fixtures",
                "source_seed": format!("proto/fixtures/seeds/{domain}.jsonc"),
                "kind": "demo",
            }),
        );

        let content = format!("{}\n", serde_json::to_string_pretty(&Value::Object(obj))?);
        let out_path = demo_dir.join(format!("{domain}.json"));
        if check_mode {
            let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
            if existing != content {
                anyhow::bail!(
                    "demo fixture drift: {} is out of date (run `cargo xtask gen-fixtures`)",
                    out_path.display()
                );
            }
        } else {
            std::fs::write(&out_path, &content)
                .with_context(|| format!("write {}", out_path.display()))?;
        }
        count += 1;
    }

    Ok(count)
}
