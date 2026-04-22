//! File-based router tests. Real tempdirs + actual .html.j2 files.

use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn seed(root: &Path, files: &[&str]) {
    for f in files {
        let p = root.join(f);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, b"(template)").unwrap();
    }
}

#[test]
fn matches_static_index() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/index.html.j2"]);
    let m = match_path(t.path(), "agents").unwrap();
    assert_eq!(m.template, t.path().join("agents/index.html.j2"));
    assert!(m.params.is_empty());
    assert_eq!(m.view_name, "agents");
}

#[test]
fn matches_nested_static() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/create.html.j2"]);
    let m = match_path(t.path(), "agents/create").unwrap();
    assert_eq!(m.template, t.path().join("agents/create.html.j2"));
    assert!(m.params.is_empty());
}

#[test]
fn dynamic_segment_binds_param() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/[id]/index.html.j2"]);
    let m = match_path(t.path(), "agents/default").unwrap();
    assert_eq!(m.params.get("id").unwrap(), "default");
    assert_eq!(m.view_name, "agents/[id]");
}

#[test]
fn dynamic_then_static() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/[id]/wire.html.j2"]);
    let m = match_path(t.path(), "agents/default/wire").unwrap();
    assert_eq!(m.params.get("id").unwrap(), "default");
    assert_eq!(m.template, t.path().join("agents/[id]/wire.html.j2"));
    assert_eq!(m.view_name, "agents/[id]/wire");
}

#[test]
fn static_beats_dynamic() {
    let t = TempDir::new().unwrap();
    seed(
        t.path(),
        &[
            "agents/[id]/index.html.j2",
            "agents/create.html.j2",
        ],
    );
    // `/agents/create` should hit the static file, not the dynamic.
    let m = match_path(t.path(), "agents/create").unwrap();
    assert!(m.template.to_string_lossy().contains("create.html.j2"));
    assert!(m.params.is_empty());
}

#[test]
fn catch_all_matches_remainder() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["docs/[...path]/index.html.j2"]);
    let m = match_path(t.path(), "docs/a/b/c").unwrap();
    assert_eq!(m.params.get("path").unwrap(), "a/b/c");
}

#[test]
fn catch_all_leaf_file() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["docs/[...path].html.j2"]);
    let m = match_path(t.path(), "docs/a/b").unwrap();
    assert_eq!(m.params.get("path").unwrap(), "a/b");
    assert_eq!(m.template, t.path().join("docs/[...path].html.j2"));
}

#[test]
fn unmatched_returns_none() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/index.html.j2"]);
    assert!(match_path(t.path(), "nonexistent").is_none());
    assert!(match_path(t.path(), "agents/nope").is_none());
}

#[test]
fn layouts_accumulate_by_depth() {
    let t = TempDir::new().unwrap();
    seed(
        t.path(),
        &[
            "agents/_layout.html.j2",
            "agents/[id]/_layout.html.j2",
            "agents/[id]/wire.html.j2",
        ],
    );
    let m = match_path(t.path(), "agents/x/wire").unwrap();
    assert_eq!(m.layouts.len(), 2);
    // Outermost first.
    assert!(m.layouts[0].ends_with("agents/_layout.html.j2"));
    assert!(m.layouts[1].ends_with("agents/[id]/_layout.html.j2"));
}

#[test]
fn underscore_file_is_not_routed() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["agents/_private.html.j2", "agents/index.html.j2"]);
    assert!(match_path(t.path(), "agents/_private").is_none());
    // The index still matches /agents
    assert!(match_path(t.path(), "agents").is_some());
}

#[test]
fn dynamic_segment_has_lower_priority_than_static_deeper() {
    // /agents/create/preview should match static `create/preview` if
    // present, not dynamic `[id]/preview`.
    let t = TempDir::new().unwrap();
    seed(
        t.path(),
        &[
            "agents/[id]/preview.html.j2",
            "agents/create/preview.html.j2",
        ],
    );
    let m = match_path(t.path(), "agents/create/preview").unwrap();
    assert!(m.template.to_string_lossy().contains("/create/preview"));
    assert!(m.params.is_empty());
}

#[test]
fn dotted_view_name_tracks_dynamic_segments() {
    let t = TempDir::new().unwrap();
    seed(
        t.path(),
        &[
            "pipelines/runs/[slug]/index.html.j2",
        ],
    );
    let m = match_path(t.path(), "pipelines/runs/abc").unwrap();
    assert_eq!(m.view_name, "pipelines/runs/[slug]");
}

#[test]
fn empty_path_returns_none() {
    let t = TempDir::new().unwrap();
    seed(t.path(), &["index.html.j2"]);
    // Root-level index.html.j2 matches empty request path.
    let m = match_path(t.path(), "").unwrap();
    assert!(m.template.ends_with("index.html.j2"));
}
