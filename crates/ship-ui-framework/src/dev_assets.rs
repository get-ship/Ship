//! Dev-mode hot reload for SUI bundle assets.
//!
//! Production `include_str!`s every bundle. In `shipd --dev` we install
//! a runtime override that reads bundle outputs from disk, watches the
//! input sources listed in `apps/shipd/assets/.cache/<bundle>.inputs.json`
//! plus walked `.sui` / `.controller.ts` raw sources (esbuild's metafile
//! only records the post-compile `.compiled.js`), invalidates the
//! cached bytes on change, and reruns `node scripts/build-bundles.mjs`
//! on the next request.

use anyhow::{Result, anyhow};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::Semaphore;

#[derive(Debug, Deserialize)]
struct InputsManifest {
    inputs: Vec<String>,
}

#[derive(Clone, Copy)]
struct BundleSpec {
    name: &'static str,
    output_relpath: &'static str,
    inputs_relpath: &'static str,
    extra_source_roots: &'static [&'static str],
}

const SUI_BUNDLES: &[BundleSpec] = &[
    BundleSpec {
        name: "sui-runtime",
        output_relpath: "crates/ship-ui-framework/assets/sui-runtime.js",
        inputs_relpath: "apps/shipd/assets/.cache/sui-runtime.inputs.json",
        extra_source_roots: &[],
    },
    BundleSpec {
        name: "sui-dom",
        output_relpath: "crates/ship-ui-framework/assets/sui-dom.js",
        inputs_relpath: "apps/shipd/assets/.cache/sui-dom.inputs.json",
        extra_source_roots: &[],
    },
    BundleSpec {
        name: "sui-sdk-client",
        output_relpath: "crates/ship-ui-framework/assets/sui-sdk-client.js",
        inputs_relpath: "apps/shipd/assets/.cache/sui-sdk-client.inputs.json",
        extra_source_roots: &[],
    },
    BundleSpec {
        name: "sui-components",
        output_relpath: "crates/ship-ui-framework/assets/sui-components.js",
        inputs_relpath: "apps/shipd/assets/.cache/sui-components.inputs.json",
        extra_source_roots: &["packages/ui-components/src"],
    },
];

struct BundleState {
    output_path: PathBuf,
    body: Option<Vec<u8>>,
}

struct Inner {
    repo_root: PathBuf,
    bundles: Mutex<HashMap<String, BundleState>>,
    rebuild_lock: Semaphore,
}

static DEV_ASSETS: OnceLock<Arc<Inner>> = OnceLock::new();
static WATCHER: OnceLock<Mutex<RecommendedWatcher>> = OnceLock::new();

/// Install the hot-reload override. Idempotent.
pub fn install(repo_root: PathBuf) -> Result<()> {
    if DEV_ASSETS.get().is_some() { return Ok(()); }
    let bundles_state: HashMap<String, BundleState> = SUI_BUNDLES.iter().map(|s| (
        s.name.to_string(),
        BundleState { output_path: repo_root.join(s.output_relpath), body: None },
    )).collect();
    let input_map = build_input_map(&repo_root, SUI_BUNDLES);
    let inner = Arc::new(Inner {
        repo_root,
        bundles: Mutex::new(bundles_state),
        rebuild_lock: Semaphore::new(1),
    });
    if DEV_ASSETS.set(inner.clone()).is_err() { return Ok(()); }
    let watcher = build_watcher(inner, input_map)?;
    let _ = WATCHER.set(Mutex::new(watcher));
    Ok(())
}

/// Latest bytes for `bundle_name` if dev mode is active. `None` →
/// fall through to the embedded body.
pub async fn override_for(bundle_name: &str) -> Option<Vec<u8>> {
    let inner = DEV_ASSETS.get()?.clone();
    if let Some(b) = peek(&inner, bundle_name) { return Some(b); }
    let _permit = inner.rebuild_lock.acquire().await.ok()?;
    if let Some(b) = peek(&inner, bundle_name) { return Some(b); }
    if let Err(e) = run_rebuild(&inner.repo_root).await {
        tracing::warn!("dev_assets: rebuild failed: {e}");
        return None;
    }
    let mut bundles = inner.bundles.lock().ok()?;
    for (name, state) in bundles.iter_mut() {
        match std::fs::read(&state.output_path) {
            Ok(bytes) => state.body = Some(bytes),
            Err(e) => tracing::warn!(bundle = %name, "dev_assets: read after rebuild failed: {e}"),
        }
    }
    bundles.get(bundle_name).and_then(|s| s.body.clone())
}

fn peek(inner: &Inner, bundle_name: &str) -> Option<Vec<u8>> {
    let bundles = inner.bundles.lock().ok()?;
    let s = bundles.get(bundle_name)?;
    s.body.clone()
}

fn read_inputs(path: &Path) -> Result<Vec<String>> {
    let s = std::fs::read_to_string(path)?;
    let m: InputsManifest = serde_json::from_str(&s)?;
    Ok(m.inputs)
}

fn build_input_map(repo_root: &Path, specs: &[BundleSpec]) -> HashMap<PathBuf, HashSet<String>> {
    let mut map: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    for spec in specs {
        match read_inputs(&repo_root.join(spec.inputs_relpath)) {
            Ok(inputs) => for i in inputs {
                map.entry(repo_root.join(&i)).or_default().insert(spec.name.to_string());
            },
            Err(e) => tracing::info!(bundle = spec.name, "dev_assets: no inputs manifest yet ({e}); rebuild on first request"),
        }
        for root_rel in spec.extra_source_roots {
            for path in walk_extra_sources(&repo_root.join(root_rel)) {
                map.entry(path).or_default().insert(spec.name.to_string());
            }
        }
    }
    map
}

fn walk_extra_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, u8)> = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 16 { continue; }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let p = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                stack.push((p, depth + 1));
            } else if ft.is_file() {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.ends_with(".sui") || name.ends_with(".controller.ts") {
                    out.push(p);
                }
            }
        }
    }
    out
}

fn build_watcher(inner: Arc<Inner>, input_map: HashMap<PathBuf, HashSet<String>>) -> Result<RecommendedWatcher> {
    let map = Arc::new(input_map);
    let map_cb = map.clone();
    let inner_cb = inner;
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        let Ok(ev) = res else { return };
        if !matches!(ev.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) { return; }
        let mut affected: HashSet<String> = HashSet::new();
        for p in &ev.paths {
            if let Some(bs) = map_cb.get(p) { affected.extend(bs.iter().cloned()); }
        }
        if affected.is_empty() { return; }
        let Ok(mut bundles) = inner_cb.bundles.lock() else { return };
        for name in &affected {
            if let Some(s) = bundles.get_mut(name) {
                s.body = None;
                tracing::info!(bundle = %name, "dev_assets: marked stale");
            }
        }
    })?;
    let mut dirs: HashSet<PathBuf> = HashSet::new();
    for input in map.keys() {
        let Some(parent) = input.parent() else { continue };
        if dirs.insert(parent.to_path_buf()) && parent.exists() {
            if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                tracing::warn!("dev_assets: cannot watch {}: {e}", parent.display());
            }
        }
    }
    tracing::info!(dirs = dirs.len(), inputs = map.len(), "dev_assets: hot-reload watcher armed");
    Ok(watcher)
}

async fn run_rebuild(repo_root: &Path) -> Result<()> {
    let t0 = std::time::Instant::now();
    let out = tokio::process::Command::new("node")
        .arg("scripts/build-bundles.mjs")
        .current_dir(repo_root)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().await?;
    if !out.status.success() {
        return Err(anyhow!("build-bundles.mjs exit {:?}: {}", out.status.code(), String::from_utf8_lossy(&out.stderr)));
    }
    tracing::info!(elapsed_ms = t0.elapsed().as_millis() as u64, "dev_assets: rebuild ok");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_inputs_manifest() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("foo.inputs.json");
        fs::write(&p, r#"{"hash":"abc","inputs":["a/b.ts","c/d/e.ts"]}"#).unwrap();
        let inputs = read_inputs(&p).unwrap();
        assert_eq!(inputs, vec!["a/b.ts".to_string(), "c/d/e.ts".to_string()]);
    }

    #[test]
    fn walks_sui_sources_and_controllers() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("sui-button")).unwrap();
        fs::write(root.join("sui-button/sui-button.sui"), "").unwrap();
        fs::write(root.join("sui-button/sui-button.controller.ts"), "").unwrap();
        fs::write(root.join("sui-button/sui-button.css"), "").unwrap();
        fs::write(root.join("sui-button/sui-button.compiled.js"), "").unwrap();
        fs::create_dir_all(root.join("sui-popover")).unwrap();
        fs::write(root.join("sui-popover/sui-popover.sui"), "").unwrap();

        let mut found = walk_extra_sources(root);
        found.sort();
        let mut expected = vec![
            root.join("sui-button/sui-button.sui"),
            root.join("sui-button/sui-button.controller.ts"),
            root.join("sui-popover/sui-popover.sui"),
        ];
        expected.sort();
        assert_eq!(found, expected);
    }

    #[test]
    fn input_map_scopes_per_bundle() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let cache_dir = root.join("apps/shipd/assets/.cache");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(
            cache_dir.join("sui-components.inputs.json"),
            r#"{"hash":"x","inputs":["packages/ui-components/src/sui-button/sui-button.compiled.js"]}"#,
        )
        .unwrap();
        let src_dir = root.join("packages/ui-components/src/sui-button");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("sui-button.sui"), "").unwrap();

        let map = build_input_map(root, SUI_BUNDLES);
        let compiled = root.join(
            "packages/ui-components/src/sui-button/sui-button.compiled.js",
        );
        let raw = root.join("packages/ui-components/src/sui-button/sui-button.sui");
        assert!(map.get(&compiled).map_or(false, |b| b.contains("sui-components")));
        assert!(map.get(&raw).map_or(false, |b| b.contains("sui-components")));
        // Per-bundle scoping: no input maps to sui-runtime (no manifest planted).
        assert!(!map.values().any(|b| b.contains("sui-runtime")));
    }
}
