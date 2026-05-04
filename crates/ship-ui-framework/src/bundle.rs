//! On-demand esbuild bundling for project views with a paired `index.ts`
//! controller.
//!
//! When `compile_view_js` detects a sibling `index.ts` alongside the
//! resolved `index.sui`, it calls `bundle_view` here. The compiled view.js
//! (which carries `import setup from "./index.ts"`) is written to a tempdir,
//! `index.ts` is symlinked next to it so that esbuild can follow the symlink
//! and resolve `index.ts`'s own relative imports from the real view directory.
//! The bundled string is mtime-cached; re-bundling only occurs when either
//! `index.sui` (view mtime) or `index.ts` advances.

use std::collections::HashMap;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

// ── Cache ──────────────────────────────────────────────────────────────

struct BundleEntry {
    view_mtime: SystemTime,
    ts_mtime: SystemTime,
    bundled: String,
}

fn bundle_cache() -> &'static Mutex<HashMap<(PathBuf, PathBuf), BundleEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<(PathBuf, PathBuf), BundleEntry>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ── esbuild discovery ──────────────────────────────────────────────────

/// Locate the esbuild binary: workspace `node_modules/.bin/esbuild` first,
/// then `which esbuild` on PATH.
pub fn esbuild_path() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    if let Some(ship_dir) = runtime::project::resolve_project_ship_dir_no_redirect(&cwd)
        && let Some(project_root) = ship_dir.parent() {
            let candidate = project_root.join("node_modules/.bin/esbuild");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    // Fall back to PATH.
    let out = std::process::Command::new("which").arg("esbuild").output().ok()?;
    if out.status.success() {
        let path = std::str::from_utf8(&out.stdout).ok()?.trim().to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}

// ── Bundler ────────────────────────────────────────────────────────────

/// Bundle a compiled view.js that carries `import setup from "./index.ts"`
/// using esbuild so the controller is inlined.
///
/// `view_source_path` is the path to `index.sui` (used as the cache key and
/// for mtime tracking). `ts_path` is the paired `index.ts`.
pub fn bundle_view(
    view_name: &str,
    view_js: &str,
    view_source_path: &Path,
    ts_path: &Path,
) -> Result<String, String> {
    let canon_sui = view_source_path
        .canonicalize()
        .map_err(|e| format!("canonicalize view path: {e}"))?;
    let canon_ts = ts_path
        .canonicalize()
        .map_err(|e| format!("canonicalize ts path: {e}"))?;

    let view_mtime = std::fs::metadata(&canon_sui)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let ts_mtime = std::fs::metadata(&canon_ts)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let cache_key = (canon_sui.clone(), canon_ts.clone());
    {
        let guard = bundle_cache().lock().expect("bundle cache poisoned");
        if let Some(entry) = guard.get(&cache_key)
            && entry.view_mtime == view_mtime && entry.ts_mtime == ts_mtime {
                return Ok(entry.bundled.clone());
            }
    }

    let esbuild = esbuild_path().ok_or_else(|| {
        "paired-controller views require esbuild — install it via pnpm or put it on PATH"
            .to_string()
    })?;

    let tmp = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;

    let view_js_path = tmp.path().join(format!("{view_name}-view.js"));
    std::fs::write(&view_js_path, view_js).map_err(|e| format!("write view.js: {e}"))?;

    // Symlink index.ts so esbuild resolves it and follows relative imports
    // from the real view directory (e.g. ./partials/catalog.ts).
    let ts_link = tmp.path().join("index.ts");
    symlink(&canon_ts, &ts_link).map_err(|e| format!("symlink index.ts: {e}"))?;

    let out = std::process::Command::new(&esbuild)
        .arg(view_js_path.to_str().unwrap_or_default())
        .arg("--bundle")
        .arg("--format=esm")
        .arg("--target=es2020")
        .arg("--loader:.ts=ts")
        .arg("--loader:.js=js")
        .arg("--external:@ship/ui-runtime")
        .arg("--external:@ship/ui-dom")
        .arg("--external:@ship/ui-sdk-client")
        .arg("--resolve-extensions=.ts,.js,.json")
        .output()
        .map_err(|e| format!("esbuild exec: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("esbuild bundling failed:\n{stderr}"));
    }

    let bundled =
        String::from_utf8(out.stdout).map_err(|e| format!("esbuild output encoding: {e}"))?;

    let mut guard = bundle_cache().lock().expect("bundle cache poisoned");
    guard.insert(
        cache_key,
        BundleEntry { view_mtime, ts_mtime, bundled: bundled.clone() },
    );

    Ok(bundled)
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn with_project_dir<F: FnOnce(&Path)>(f: F) {
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        const VAR: &str = "SHIP_DEV_PROJECT_DIR";
        let prev = std::env::var_os(VAR);
        fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        unsafe { std::env::set_var(VAR, tmp.path()) };
        f(tmp.path());
        match prev {
            Some(v) => unsafe { std::env::set_var(VAR, v) },
            None => unsafe { std::env::remove_var(VAR) },
        }
    }

    /// Verify the happy path: a view with a paired .ts controller produces
    /// bundled output that contains the controller's exports (no bare
    /// `import setup from "./index.ts"` line — esbuild has inlined it).
    #[test]
    fn paired_ts_controller_is_bundled() {
        if esbuild_path().is_none() {
            eprintln!("skip: esbuild not found");
            return;
        }
        with_project_dir(|root| {
            let view_dir = root.join(".ship/views/test-widget");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            let ts_path = view_dir.join("index.ts");

            fs::write(&sui_path, r#"<state name="x" value="0" /><p>{x()}</p>"#).unwrap();
            fs::write(
                &ts_path,
                r#"export default function CONTROLLER_SENTINEL() { return 42; }"#,
            )
            .unwrap();

            // Simulate the compiled view.js that carries the import line.
            let fake_view_js = r#"import setup from "./index.ts"; export { setup };"#;

            let result = bundle_view("test-widget", fake_view_js, &sui_path, &ts_path);
            let bundled = result.expect("bundle succeeded");

            assert!(
                !bundled.contains("import setup from \"./index.ts\""),
                "bare import not inlined"
            );
            assert!(
                bundled.contains("CONTROLLER_SENTINEL"),
                "controller export present in bundle"
            );
        });
    }

    /// A view WITHOUT a paired .ts goes through the normal compile path and
    /// is never passed to bundle_view at all — this test verifies that the
    /// bundler does not break the no-ts path by checking that calling
    /// bundle_view with a non-existent ts_path returns an error.
    #[test]
    fn missing_ts_returns_error() {
        with_project_dir(|root| {
            let view_dir = root.join(".ship/views/plain-view");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            fs::write(&sui_path, r#"<state name="x" value="0" /><p>{x()}</p>"#).unwrap();

            let nonexistent_ts = view_dir.join("index.ts"); // does not exist
            let result = bundle_view("plain-view", "export {}", &sui_path, &nonexistent_ts);
            assert!(result.is_err(), "missing ts_path should error");
        });
    }

    /// Cache hit: two calls with the same mtimes return the same string without
    /// re-invoking esbuild (we verify the string is identical; we don't need a
    /// spy since the cache-coherence property is what matters here).
    #[test]
    fn cache_hit_returns_same_output() {
        if esbuild_path().is_none() {
            eprintln!("skip: esbuild not found");
            return;
        }
        with_project_dir(|root| {
            let view_dir = root.join(".ship/views/cache-test");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            let ts_path = view_dir.join("index.ts");
            fs::write(&sui_path, r#"<p>hi</p>"#).unwrap();
            fs::write(&ts_path, r#"export default function CACHE_SENTINEL() {}"#).unwrap();

            let fake_view_js = r#"import setup from "./index.ts"; export { setup };"#;

            let first = bundle_view("cache-test", fake_view_js, &sui_path, &ts_path)
                .expect("first bundle ok");
            let second = bundle_view("cache-test", fake_view_js, &sui_path, &ts_path)
                .expect("second bundle ok");
            assert_eq!(first, second, "cache hit returns identical output");
        });
    }
}
