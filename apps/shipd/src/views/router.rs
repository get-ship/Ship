//! File-based router for Ship views.
//!
//! Walks `.ship/views/` on demand and matches a request path against
//! the directory tree using Next.js-style conventions:
//!
//! - `index.html.j2` — directory default
//! - `[id].html.j2` / `[id]/...` — single dynamic segment
//! - `[...rest].html.j2` / `[...rest]/...` — catch-all
//! - `[[id]].html.j2` — optional single segment
//! - `_layout.html.j2` — wraps child templates at any level
//! - Files / dirs starting with `_` are not routed
//!
//! Precedence at each level: **static > dynamic > catch-all**.
//!
//! O(segments × directory entries) per request. Re-scan on every
//! request — hot reload parity with existing views.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of a successful route match.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// Absolute path to the template file on disk.
    pub template: PathBuf,
    /// `_layout.html.j2` files along the ancestor chain, outermost
    /// → innermost. Render pipeline wraps the child with each layout.
    pub layouts: Vec<PathBuf>,
    /// Path params, keyed by `[name]`. Catch-all segments land as the
    /// remainder joined by `/`.
    pub params: HashMap<String, String>,
    /// Dotted view name derived from the filesystem path — e.g.
    /// `agents/[id]/wire`. Used for SSE topics + logging.
    pub view_name: String,
}

/// Classification of a filesystem entry's routing segment.
#[derive(Debug, Clone)]
enum Segment {
    Static(String),
    Dynamic(String),
    OptionalDyn(String),
    CatchAll(String),
}

impl Segment {
    fn parse(raw: &str) -> Self {
        if let Some(inner) = raw.strip_prefix("[...").and_then(|s| s.strip_suffix(']')) {
            return Segment::CatchAll(inner.to_string());
        }
        if let Some(inner) = raw.strip_prefix("[[").and_then(|s| s.strip_suffix("]]")) {
            return Segment::OptionalDyn(inner.to_string());
        }
        if let Some(inner) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            return Segment::Dynamic(inner.to_string());
        }
        Segment::Static(raw.to_string())
    }
}

/// Strip the `.html.j2` suffix; `None` for non-template files.
fn template_stem(name: &str) -> Option<&str> {
    name.strip_suffix(".html.j2")
}

/// Files / dirs starting with `_` are not routable.
fn is_routable(name: &str) -> bool {
    !name.starts_with('_')
}

/// Resolve `request_path` against `views_root`.
///
/// `request_path` is path *relative to the view prefix* — e.g.
/// `"agents/default/wire"`. No leading slash.
pub fn match_path(views_root: &Path, request_path: &str) -> Option<RouteMatch> {
    let segments: Vec<&str> = request_path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut acc = MatchAcc::default();
    find_match(views_root, &segments, &mut acc)
}

#[derive(Default)]
struct MatchAcc {
    layouts: Vec<PathBuf>,
    params: HashMap<String, String>,
    view_name_segs: Vec<String>,
}

struct Candidate {
    segment: Segment,
    path: PathBuf,
    is_dir: bool,
}

fn find_match(dir: &Path, segments: &[&str], acc: &mut MatchAcc) -> Option<RouteMatch> {
    if !dir.is_dir() {
        return None;
    }

    let layout = dir.join("_layout.html.j2");
    let pushed = if layout.is_file() {
        acc.layouts.push(layout);
        true
    } else {
        false
    };

    let result = resolve_at(dir, segments, acc);
    if result.is_none() && pushed {
        acc.layouts.pop();
    }
    result
}

fn resolve_at(dir: &Path, segments: &[&str], acc: &mut MatchAcc) -> Option<RouteMatch> {
    if segments.is_empty() {
        let idx = dir.join("index.html.j2");
        if idx.is_file() {
            return Some(build_match(idx, acc));
        }
        return None;
    }
    let (head, tail) = segments.split_first().unwrap();
    let entries = std::fs::read_dir(dir).ok()?;

    let mut candidates: Vec<Candidate> = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !is_routable(&name) {
            continue;
        }
        let path = e.path();
        let is_dir = path.is_dir();
        let seg_name = if is_dir {
            name
        } else {
            match template_stem(&name) {
                Some(stem) => stem.to_string(),
                None => continue,
            }
        };
        candidates.push(Candidate {
            segment: Segment::parse(&seg_name),
            path,
            is_dir,
        });
    }

    // Precedence: static > dynamic > catch-all > optional.
    // Within kind: dirs before files so `[id]/wire.html.j2` beats
    // `[id].html.j2` when more segments follow.
    candidates.sort_by_key(|c| {
        let kind_ord = match &c.segment {
            Segment::Static(_) => 0,
            Segment::Dynamic(_) => 1,
            Segment::CatchAll(_) => 2,
            Segment::OptionalDyn(_) => 3,
        };
        let dir_ord = if c.is_dir { 0 } else { 1 };
        (kind_ord, dir_ord)
    });

    for cand in &candidates {
        if let Some(m) = try_candidate(cand, head, tail, acc) {
            return Some(m);
        }
    }
    None
}

fn try_candidate(
    cand: &Candidate,
    head: &str,
    tail: &[&str],
    acc: &mut MatchAcc,
) -> Option<RouteMatch> {
    match &cand.segment {
        Segment::Static(name) => {
            if name != head {
                return None;
            }
            descend(cand, tail, acc, head.to_string())
        }
        Segment::Dynamic(param) => {
            acc.params.insert(param.clone(), head.to_string());
            let res = descend(cand, tail, acc, format!("[{param}]"));
            if res.is_none() {
                acc.params.remove(param);
            }
            res
        }
        Segment::CatchAll(param) => {
            let mut remainder: Vec<String> = vec![head.to_string()];
            remainder.extend(tail.iter().map(|s| s.to_string()));
            acc.params.insert(param.clone(), remainder.join("/"));
            let name = format!("[...{param}]");
            let res = if cand.is_dir {
                descend(cand, &[], acc, name)
            } else {
                acc.view_name_segs.push(name);
                Some(build_match(cand.path.clone(), acc))
            };
            if res.is_none() {
                acc.params.remove(param);
            }
            res
        }
        Segment::OptionalDyn(param) => {
            // Try absent case first — match without consuming `head`.
            acc.params.insert(param.clone(), String::new());
            let name = format!("[[{param}]]");
            let all_segments: Vec<&str> = std::iter::once(head).chain(tail.iter().copied()).collect();
            if let Some(m) = descend(cand, &all_segments, acc, name.clone()) {
                return Some(m);
            }
            acc.params.insert(param.clone(), head.to_string());
            let res = descend(cand, tail, acc, name);
            if res.is_none() {
                acc.params.remove(param);
            }
            res
        }
    }
}

fn descend(
    cand: &Candidate,
    remaining: &[&str],
    acc: &mut MatchAcc,
    view_seg: String,
) -> Option<RouteMatch> {
    acc.view_name_segs.push(view_seg);
    let res = if cand.is_dir {
        find_match(&cand.path, remaining, acc)
    } else if remaining.is_empty() {
        Some(build_match(cand.path.clone(), acc))
    } else {
        None
    };
    if res.is_none() {
        acc.view_name_segs.pop();
    }
    res
}

fn build_match(template: PathBuf, acc: &MatchAcc) -> RouteMatch {
    RouteMatch {
        template,
        layouts: acc.layouts.clone(),
        params: acc.params.clone(),
        view_name: acc.view_name_segs.join("/"),
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
