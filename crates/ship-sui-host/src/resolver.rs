use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use sui_compiler::{
    parse,
    render::{self, RenderOutput},
};

#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ResolvedView {
    pub source: String,
    pub params: HashMap<String, String>,
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ResolveError {
    pub message: String,
}

pub fn resolve(
    project_root: &Path,
    name: &str,
    sub_path: Option<&str>,
) -> Result<ResolvedView, ResolveError> {
    let segments: Vec<&str> = sub_path
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut attempts: Vec<String> = Vec::new();

    let new_root = project_root.join("views").join(name);
    let mut params: HashMap<String, String> = HashMap::new();
    let mut tried: Vec<PathBuf> = Vec::new();
    let leaf = walk_disk_with_leaf(&new_root, &segments, "page.sui", &mut params, &mut tried);
    let new_views_root = project_root.join("views");
    for p in &tried {
        let rel = p
            .strip_prefix(&new_views_root)
            .map(|r| format!("views/{}", r.display()))
            .unwrap_or_else(|_| p.display().to_string());
        attempts.push(rel);
    }

    if leaf.is_none() {
        params.clear();
        tried.clear();
        let views_dir = project_root.join(".ship").join("views");
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
        let mut idx = dir.to_path_buf();
        for seg in segments {
            idx.push(seg);
        }
        idx.push(leaf_name);
        tried.push(idx);
        return None;
    }

    let (head, tail) = segments.split_first().unwrap();

    let static_dir = dir.join(head);
    if static_dir.is_dir()
        && let Some(found) = walk_disk_with_leaf(&static_dir, tail, leaf_name, params, tried)
    {
        return Some(found);
    }

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

struct CacheEntry {
    mtime: SystemTime,
    rendered: RenderOutput,
}

fn cache() -> &'static Mutex<HashMap<(PathBuf, String), CacheEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<(PathBuf, String), CacheEntry>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn render_resolved(
    resolved: &ResolvedView,
    view_name: &str,
) -> Result<RenderOutput, String> {
    let path = resolved.source_path.as_ref().ok_or_else(|| {
        "render_resolved called without a source_path".to_string()
    })?;
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
    Ok(rendered)
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

    fn with_tempdir<F: FnOnce(&Path)>(f: F) {
        let tmp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(tmp.path().join(".ship")).unwrap();
        f(tmp.path());
    }

    #[test]
    fn on_disk_index_resolves() {
        with_tempdir(|root| {
            let view_dir = root.join(".ship/views/widget");
            fs::create_dir_all(&view_dir).unwrap();
            fs::write(
                view_dir.join("index.sui"),
                r#"<state name="x" value="1" /><p>{x()}</p>"#,
            )
            .unwrap();

            let r = resolve(root, "widget", None).expect("on-disk widget resolves");
            assert!(r.source.contains("name=\"x\""));
            assert!(r.source_path.is_some());
            assert!(r.params.is_empty());
        });
    }

    #[test]
    fn on_disk_param_segment_captures() {
        with_tempdir(|root| {
            let leaf = root.join(".ship/views/things/[id]");
            fs::create_dir_all(&leaf).unwrap();
            fs::write(
                leaf.join("index.sui"),
                r#"<state name="x" value="1" /><p>{x()}</p>"#,
            )
            .unwrap();

            let r = resolve(root, "things", Some("42")).expect("param dir resolves");
            assert_eq!(r.params.get("id").map(String::as_str), Some("42"));
        });
    }

    #[test]
    fn missing_view_returns_diagnostic() {
        with_tempdir(|root| {
            let err = resolve(root, "ghost", None).unwrap_err();
            assert!(err.message.contains("ghost"));
            assert!(err.message.contains(".ship/views"));
        });
    }

    fn bump_mtime(path: &Path, secs_forward: u64) {
        let now = std::fs::metadata(path).and_then(|m| m.modified()).unwrap();
        let bumped = now + std::time::Duration::from_secs(secs_forward);
        filetime::set_file_mtime(path, filetime::FileTime::from_system_time(bumped))
            .expect("bump mtime");
    }

    #[test]
    fn mtime_cache_reuses_render_until_source_touched() {
        with_tempdir(|root| {
            let view_dir = root.join(".ship/views/cachey");
            fs::create_dir_all(&view_dir).unwrap();
            let path = view_dir.join("index.sui");
            fs::write(&path, r#"<p>first</p>"#).unwrap();

            let r = resolve(root, "cachey", None).unwrap();
            let first = render_resolved(&r, "cachey").unwrap();

            let second = render_resolved(&r, "cachey").unwrap();
            assert_eq!(first.html, second.html);

            fs::write(&path, r#"<p>second</p>"#).unwrap();
            bump_mtime(&path, 5);
            let r2 = resolve(root, "cachey", None).unwrap();
            let third = render_resolved(&r2, "cachey").unwrap();
            assert_ne!(third.html, first.html);
        });
    }

    #[test]
    fn on_disk_page_sui_resolves() {
        with_tempdir(|root| {
            let view_dir = root.join("views/widget");
            fs::create_dir_all(&view_dir).unwrap();
            fs::write(
                view_dir.join("page.sui"),
                r#"<state name="x" value="1" /><p>{x()}</p>"#,
            )
            .unwrap();

            let r = resolve(root, "widget", None).expect("on-disk widget resolves");
            assert!(r.source.contains("name=\"x\""));
            assert!(r.source_path.is_some());
            assert!(r.params.is_empty());
        });
    }
}
