use std::collections::HashMap;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

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

fn bun_on_path() -> bool {
    std::process::Command::new("bun")
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|o| o.status.success())
}

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

    if !bun_on_path() {
        return Err("paired-controller views require bun — install it or put it on PATH".to_string());
    }

    let tmp = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;

    let view_js_path = tmp.path().join(format!("{view_name}-view.js"));
    std::fs::write(&view_js_path, view_js).map_err(|e| format!("write view.js: {e}"))?;

    let ts_filename = canon_ts
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("controller.ts")
        .to_string();
    let ts_link = tmp.path().join(&ts_filename);
    symlink(&canon_ts, &ts_link).map_err(|e| format!("symlink {ts_filename}: {e}"))?;

    let outfile = tmp.path().join("bundle.js");
    let out = std::process::Command::new("bun")
        .arg("build")
        .arg(view_js_path.to_str().unwrap_or_default())
        .arg("--outfile")
        .arg(outfile.to_str().unwrap_or_default())
        .arg("--target=browser")
        .arg("--external=@ship/sui-runtime")
        .arg("--external=@ship/sui-dom")
        .arg("--external=@ship/sdk-client")
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("bun build exec: {e}"))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("bun build failed:\n{stderr}"));
    }

    let bundled =
        std::fs::read_to_string(&outfile).map_err(|e| format!("read bun build output: {e}"))?;

    let mut guard = bundle_cache().lock().expect("bundle cache poisoned");
    guard.insert(
        cache_key,
        BundleEntry { view_mtime, ts_mtime, bundled: bundled.clone() },
    );

    Ok(bundled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn with_tempdir<F: FnOnce(&Path)>(f: F) {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        f(tmp.path());
    }

    #[test]
    fn paired_ts_controller_is_bundled() {
        if !bun_on_path() {
            eprintln!("skip: bun not found");
            return;
        }
        with_tempdir(|root| {
            let view_dir = root.join(".ship/views/test-widget");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            let ts_path = view_dir.join("index.controller.ts");

            fs::write(&sui_path, r#"<state name="x" value="0" /><p>{x()}</p>"#).unwrap();
            fs::write(
                &ts_path,
                r#"export default function CONTROLLER_SENTINEL() { return 42; }"#,
            )
            .unwrap();

            let fake_view_js = r#"import setup from "./index.controller.ts"; export { setup };"#;

            let result = bundle_view("test-widget", fake_view_js, &sui_path, &ts_path);
            let bundled = result.expect("bundle succeeded");

            assert!(
                !bundled.contains("import setup from \"./index.controller.ts\""),
                "bare import not inlined"
            );
            assert!(
                bundled.contains("CONTROLLER_SENTINEL"),
                "controller export present in bundle"
            );
        });
    }

    #[test]
    fn missing_ts_returns_error() {
        with_tempdir(|root| {
            let view_dir = root.join(".ship/views/plain-view");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            fs::write(&sui_path, r#"<state name="x" value="0" /><p>{x()}</p>"#).unwrap();

            let nonexistent_ts = view_dir.join("index.controller.ts");
            let result = bundle_view("plain-view", "export {}", &sui_path, &nonexistent_ts);
            assert!(result.is_err(), "missing ts_path should error");
        });
    }

    #[test]
    fn cache_hit_returns_same_output() {
        if !bun_on_path() {
            eprintln!("skip: bun not found");
            return;
        }
        with_tempdir(|root| {
            let view_dir = root.join(".ship/views/cache-test");
            fs::create_dir_all(&view_dir).unwrap();
            let sui_path = view_dir.join("index.sui");
            let ts_path = view_dir.join("index.controller.ts");
            fs::write(&sui_path, r#"<p>hi</p>"#).unwrap();
            fs::write(&ts_path, r#"export default function CACHE_SENTINEL() {}"#).unwrap();

            let fake_view_js = r#"import setup from "./index.controller.ts"; export { setup };"#;

            let first = bundle_view("cache-test", fake_view_js, &sui_path, &ts_path)
                .expect("first bundle ok");
            let second = bundle_view("cache-test", fake_view_js, &sui_path, &ts_path)
                .expect("second bundle ok");
            assert_eq!(first, second, "cache hit returns identical output");
        });
    }
}
