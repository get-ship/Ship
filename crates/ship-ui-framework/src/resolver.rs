//! Single source of truth for "where does SUI view `<name>` come
//! from?".
//!
//! Resolution order (each tried in turn until one matches):
//! 1. Bake list (`counter`, `agents`, `git`) — compile-time `include_str!`.
//!    Only matches when the request has no sub-path.
//! 2. `<project_root>/views/<name>/page.sui` — the new view-directory
//!    convention (page.sui + controller.ts + style.css + manifest.jsonc).
//!    Sub-paths walk into `[param]` directories under the view root.
//! 3. `<project_root>/.ship/views/<name>/index.sui` — the legacy
//!    extension-point convention. Kept as a fallback during the
//!    transition window; once no project ships views in `.ship/views/`
//!    this branch can be retired.
//!
//! Compiled output is mtime-cached: the resolver re-parses + re-renders
//! a `.sui` source only when its mtime advances. Cache lookup is keyed
//! by the file's canonical path, plus `view_name` so two callers with
//! different `RenderOpts.view_name` for the same source don't collide.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use sui_compiler::{
    parse,
    render::{self, RenderOutput},
};

use crate::render_helpers::view_source as bake_source;

/// Resolved view: source + captured params + (when on disk) the canonical
/// source path the resolver landed on. Bake-list hits return
/// `source_path = None` and an empty params map.
#[derive(Debug)]
pub struct ResolvedView {
    pub source: String,
    pub params: HashMap<String, String>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ResolveError {
    /// Human-readable diagnostic listing the lookups attempted.
    pub message: String,
}

/// Resolve `<name>` (top-level segment) plus optional sub-path.
///
/// `sub_path` is `None` for `/_sui/views/{name}` and `Some("a/b/c")`
/// for `/_sui/views/{name}/a/b/c`. The empty string and `Some("")`
/// are equivalent.
pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    let segments: Vec<&str> = sub_path
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut attempts: Vec<String> = Vec::new();

    // 1. Bake list — only without a sub-path.
    if segments.is_empty()
        && let Some(src) = bake_source(name)
    {
        return Ok(ResolvedView {
            source: src.to_string(),
            params: HashMap::new(),
            source_path: None,
        });
    }
    if segments.is_empty() {
        attempts.push(format!("bake/{name}"));
    }

    // 2 + 3. On-disk lookup under `<project>/.ship/views/<name>/`.
    //
    // Views in `.ship/views/` are per-worktree by design (each linked
    // worktree may carry its own SUI views in its own `.ship/views/`).
    // Use the no-redirect resolver so the daemon serving Ship-feat-sail
    // doesn't resolve views from Ship's main checkout. The dev-time
    // SHIP_DEV_PROJECT_DIR env override is honored first so the resolver's
    // own tests can pin a tempdir.
    let project = if let Ok(dir) = std::env::var("SHIP_DEV_PROJECT_DIR") {
        std::path::PathBuf::from(dir)
    } else {
        let cwd = match std::env::current_dir() {
            Ok(c) => c,
            Err(_) => {
                return Err(ResolveError {
                    message: format!(
                        "view not found: {} — no cwd; tried {}",
                        join_request(name, sub_path),
                        attempts.join(", ")
                    ),
                });
            }
        };
        match runtime::project::resolve_project_ship_dir_no_redirect(&cwd) {
            Some(s) => s
                .parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or(s),
            None => {
                return Err(ResolveError {
                    message: format!(
                        "view not found: {} — no project root; tried {}",
                        join_request(name, sub_path),
                        attempts.join(", ")
                    ),
                });
            }
        }
    };

    // 2. New convention: project_root/views/<name>/page.sui (with optional
    //    [param] sub-segments). Tried first so projects that have migrated
    //    away from .ship/views/ resolve cleanly.
    let new_root = project.join("views").join(name);
    let mut params: HashMap<String, String> = HashMap::new();
    let mut tried: Vec<PathBuf> = Vec::new();
    let leaf = walk_disk_with_leaf(&new_root, &segments, "page.sui", &mut params, &mut tried);
    let new_views_root = project.join("views");
    for p in &tried {
        let rel = p
            .strip_prefix(&new_views_root)
            .map(|r| format!("views/{}", r.display()))
            .unwrap_or_else(|_| p.display().to_string());
        attempts.push(rel);
    }

    // 3. Legacy fallback: project_root/.ship/views/<name>/index.sui.
    if leaf.is_none() {
        params.clear();
        tried.clear();
        let views_dir = project.join(".ship").join("views");
        let root = views_dir.join(name);
        if let Some(found) =
            walk_disk_with_leaf(&root, &segments, "index.sui", &mut params, &mut tried)
        {
            for p in &tried {
                let rel = p
                    .strip_prefix(&views_dir)
                    .map(|r| format!(".ship/views/{}", r.display()))
                    .unwrap_or_else(|_| p.display().to_string());
                attempts.push(rel);
            }
            return read_resolved(found, params);
        }
        for p in &tried {
            let rel = p
                .strip_prefix(&views_dir)
                .map(|r| format!(".ship/views/{}", r.display()))
                .unwrap_or_else(|_| p.display().to_string());
            attempts.push(rel);
        }
    }

    if let Some(leaf) = leaf {
        return read_resolved(leaf, params);
    }

    Err(ResolveError {
        message: format!(
            "view not found: {} — looked at: {}",
            join_request(name, sub_path),
            attempts.join(", "),
        ),
    })
}

fn read_resolved(
    leaf: PathBuf,
    params: HashMap<String, String>,
) -> Result<ResolvedView, ResolveError> {
    let source = match std::fs::read_to_string(&leaf) {
        Ok(s) => s,
        Err(e) => {
            return Err(ResolveError {
                message: format!("failed to read {}: {e}", leaf.display()),
            });
        }
    };
    let canonical = leaf.canonicalize().unwrap_or(leaf);
    Ok(ResolvedView {
        source,
        params,
        source_path: Some(canonical),
    })
}

/// Walk segments downward through `dir`, preferring static directories
/// over `[param]` captures. Returns the path to `<leaf_name>` on success.
/// Every leaf path inspected is pushed to `tried` so the caller can
/// render a diagnostic on miss.
fn walk_disk_with_leaf(
    dir: &Path,
    segments: &[&str],
    leaf_name: &str,
    params: &mut HashMap<String, String>,
    tried: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    if segments.is_empty() {
        let idx = dir.join(leaf_name);
        tried.push(idx.clone());
        return idx.is_file().then_some(idx);
    }

    if !dir.is_dir() {
        // Still record the would-be leaf so the diagnostic is informative
        // when the parent never existed.
        let mut idx = dir.to_path_buf();
        for seg in segments {
            idx.push(seg);
        }
        idx.push(leaf_name);
        tried.push(idx);
        return None;
    }

    let (head, tail) = segments.split_first().unwrap();

    // Static dir first.
    let static_dir = dir.join(head);
    if static_dir.is_dir()
        && let Some(found) = walk_disk_with_leaf(&static_dir, tail, leaf_name, params, tried)
    {
        return Some(found);
    }

    // Then any `[param]` dir at this level.
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if !name_str.starts_with('[') || !name_str.ends_with(']') {
                continue;
            }
            let param = name_str.trim_start_matches('[').trim_end_matches(']');
            if param.is_empty() {
                continue;
            }
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            params.insert(param.to_string(), (*head).to_string());
            if let Some(found) = walk_disk_with_leaf(&path, tail, leaf_name, params, tried) {
                return Some(found);
            }
            params.remove(param);
        }
    }

    None
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}

// ── Mtime-cached compilation ─────────────────────────────────────────

/// Cached parse + default render for a `.sui` file. Bake-list views skip
/// the cache: their source is `&'static str` and recompiles only happen
/// when the daemon binary itself changes.
struct CacheEntry {
    mtime: SystemTime,
    rendered: RenderOutput,
}

fn cache() -> &'static Mutex<HashMap<(PathBuf, String), CacheEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<(PathBuf, String), CacheEntry>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Render the resolved view with the given `view_name` (used by the
/// runtime's binding registry + island id). On-disk views are cached
/// and re-rendered only when their source mtime advances.
pub fn render_resolved(
    resolved: &ResolvedView,
    view_name: &str,
) -> Result<RenderOutput, String> {
    if let Some(path) = &resolved.source_path {
        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(|e| format!("stat {}: {e}", path.display()))?;
        let key = (path.clone(), view_name.to_string());
        {
            let guard = cache().lock().expect("sui_resolver cache poisoned");
            if let Some(entry) = guard.get(&key)
                && entry.mtime == mtime
            {
                return Ok(entry.rendered.clone());
            }
        }
        let rendered = compile(&resolved.source, view_name)?;
        let mut guard = cache().lock().expect("sui_resolver cache poisoned");
        guard.insert(
            key,
            CacheEntry {
                mtime,
                rendered: rendered.clone(),
            },
        );
        return Ok(rendered);
    }
    compile(&resolved.source, view_name)
}

fn compile(source: &str, view_name: &str) -> Result<RenderOutput, String> {
    let program = parse::parse(source).map_err(|e| e.render(source))?;
    let opts = render::RenderOpts {
        view_name: view_name.to_string(),
    };
    render::render_default(&program, &opts).map_err(|e| e.render(source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn bake_list_resolves_without_disk() {
        let r = resolve("counter", None).expect("counter is baked");
        assert!(r.source.contains("<state") || r.source.contains("<button"));
        assert!(r.source_path.is_none());
        assert!(r.params.is_empty());
    }

    #[test]
    fn bake_list_skipped_when_subpath_present() {
        // Counter has no on-disk variant, so this should 404.
        let err = resolve("counter", Some("nope")).unwrap_err();
        assert!(err.message.contains("not found"));
    }

    /// Build a temporary `.ship/views/<name>/...` tree and point
    /// `runtime::project::get_project_dir` at it via env override.
    /// Tests share a single process-wide mutex with the view_payload
    /// suite because both mutate the same env var.
    fn with_project_dir<F: FnOnce(&Path)>(f: F) {
        let _g = crate::test_lock::lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        const VAR: &str = "SHIP_DEV_PROJECT_DIR";
        let prev = std::env::var_os(VAR);
        fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        unsafe { std::env::set_var(VAR, tmp.path()); }
        f(tmp.path());
        match prev {
            Some(v) => unsafe { std::env::set_var(VAR, v) },
            None => unsafe { std::env::remove_var(VAR) },
        }
    }

    #[test]
    fn on_disk_index_resolves() {
        with_project_dir(|root| {
            let view_dir = root.join(".ship/views/widget");
            fs::create_dir_all(&view_dir).unwrap();
            fs::write(
                view_dir.join("index.sui"),
                r#"<state name="x" value="1" /><p>{x()}</p>"#,
            )
            .unwrap();

            let r = resolve("widget", None).expect("on-disk widget resolves");
            assert!(r.source.contains("name=\"x\""));
            assert!(r.source_path.is_some());
            assert!(r.params.is_empty());
        });
    }

    #[test]
    fn on_disk_param_segment_captures() {
        with_project_dir(|root| {
            let leaf = root.join(".ship/views/things/[id]");
            fs::create_dir_all(&leaf).unwrap();
            fs::write(
                leaf.join("index.sui"),
                r#"<state name="x" value="1" /><p>{x()}</p>"#,
            )
            .unwrap();

            let r = resolve("things", Some("42")).expect("param dir resolves");
            assert_eq!(r.params.get("id").map(String::as_str), Some("42"));
        });
    }

    #[test]
    fn missing_view_returns_diagnostic() {
        with_project_dir(|_| {
            let err = resolve("ghost", None).unwrap_err();
            assert!(err.message.contains("ghost"));
            assert!(err.message.contains(".ship/views"));
        });
    }

    /// Bump a file's mtime by `secs_forward` seconds via filetime —
    /// removes the dependency on host filesystem mtime resolution.
    /// Some filesystems quantize mtime at second granularity, so a
    /// sub-second rewrite leaves the cache hit and the test would
    /// see no change.
    fn bump_mtime(path: &Path, secs_forward: u64) {
        let now = std::fs::metadata(path).and_then(|m| m.modified()).unwrap();
        let bumped = now + std::time::Duration::from_secs(secs_forward);
        filetime::set_file_mtime(path, filetime::FileTime::from_system_time(bumped))
            .expect("bump mtime");
    }

    /// NOTE: this test mutates the `SHIP_DEV_PROJECT_DIR` env var. Run
    /// the suite with `--test-threads=1` to keep it from racing the
    /// `view_payload` suite (both touch the same env). The shared
    /// `crate::test_lock` helps but cargo's parallel scheduling can
    /// still surface flakes between mutex acquisition and tempdir
    /// creation. A future cleanup is to hide all env-touching tests
    /// behind a single integration test runner so cargo never tries
    /// to parallelize them.
    #[test]
    fn mtime_cache_reuses_render_until_source_touched() {
        with_project_dir(|root| {
            let view_dir = root.join(".ship/views/cachey");
            fs::create_dir_all(&view_dir).unwrap();
            let path = view_dir.join("index.sui");
            fs::write(&path, r#"<p>first</p>"#).unwrap();

            let r = resolve("cachey", None).unwrap();
            let first = render_resolved(&r, "cachey").unwrap();

            let second = render_resolved(&r, "cachey").unwrap();
            assert_eq!(first.html, second.html);

            // Rewrite + bump the mtime explicitly so the cache invalidation
            // doesn't depend on filesystem mtime resolution.
            fs::write(&path, r#"<p>second</p>"#).unwrap();
            bump_mtime(&path, 5);
            let r2 = resolve("cachey", None).unwrap();
            let third = render_resolved(&r2, "cachey").unwrap();
            assert_ne!(third.html, first.html);
        });
    }
}
