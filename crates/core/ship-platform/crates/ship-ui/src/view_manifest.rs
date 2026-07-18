//! The Views applet-manifest layer — re-homed from deck (sui repo,
//! `job/views-decouple`, 2026-07-18).
//!
//! A View is a Ship product feature: an agent/user-authored applet rendered
//! in a tree inside a deck app — a stripped-down app with no deck config of
//! its own. Its host metadata (`title` / `layout` / `auth` / `rail`) lives in
//! `views/<name>/manifest.jsonc`. This layer walks the manifests and emits
//! two generated tables with a strict split:
//!   - `views/routes.gen.ts` — router-only (pattern + view name), host-agnostic;
//!   - `views/meta.gen.ts`   — host-only (title/layout/auth/rail), read by the
//!     shell at boot to choose chrome, gate access, and build the launcher rail.
//!
//! Deck keeps ZERO typed knowledge of this schema — it carries per-route
//! metadata opaquely if at all. Integration back into a deck build runs
//! through that opaque seam (or a ship-ui post-pass over deck's discovered
//! route tree); see `docs/views-manifest-port.md` for the handoff checklist.

use anyhow::{Context, Result};
use std::path::Path;

/// One raw view manifest entry. Parsed from `views/<name>/manifest.jsonc`.
/// The JSON allows comments (`// …`) which serde_json doesn't natively
/// accept, so we strip line-`//` comments before parsing. The router
/// fields (pattern, view name) go to `views/routes.gen.ts`; the host
/// fields (title, layout, auth, rail) go to `views/meta.gen.ts`. The
/// split is the contract — the router is host-agnostic, the host reads
/// what it needs.
pub struct ViewManifest {
    pub name: String,
    pub path: String,
    pub title: String,
    pub layout: String,
    pub auth: String,
    pub rail: Option<serde_json::Value>,
}

/// Strip `//` line comments from a JSONC source so serde_json can parse
/// it. Trailing comments on a line are stripped before any JSON value
/// closes the line; strings containing `//` are preserved.
fn strip_jsonc(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_string = false;
    let mut escape = false;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for nc in chars.by_ref() {
                if nc == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Walk `views/<name>/` for top-level dirs with a `manifest.jsonc`.
/// Reads each manifest and returns the parsed entries. Skips entries
/// missing `path` or whose `path` doesn't start with `/` (the router
/// requires absolute patterns).
pub fn discover_views(views_dir: &Path) -> Result<Vec<ViewManifest>> {
    let mut out = Vec::new();
    if !views_dir.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(views_dir)
        .with_context(|| format!("reading `{}`", views_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.jsonc");
        if !manifest_path.exists() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let raw = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("reading `{}`", manifest_path.display()))?;
        let json = strip_jsonc(&raw);
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .with_context(|| format!("parsing `{}`", manifest_path.display()))?;
        let path = parsed.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if path.is_empty() {
            eprintln!(
                "view-discovery: skipping `{}` — manifest has no `path` field",
                manifest_path.display()
            );
            continue;
        }
        if !path.starts_with('/') {
            eprintln!(
                "view-discovery: skipping `{}` — `path` is `{}`, must start with `/`",
                manifest_path.display(),
                path
            );
            continue;
        }
        let manifest_name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        // The dir name is canonical; warn if the manifest's `name`
        // disagrees (defensive — keeps the dir as the source of truth).
        if !manifest_name.is_empty() && manifest_name != dir_name {
            eprintln!(
                "view-discovery: `{}` manifest `name` is `{}` but dir is `{}` — using dir",
                manifest_path.display(),
                manifest_name,
                dir_name
            );
        }
        let title = parsed.get("title").and_then(|v| v.as_str()).unwrap_or(&dir_name).to_string();
        let layout = parsed.get("layout").and_then(|v| v.as_str()).unwrap_or("default").to_string();
        let auth = parsed.get("auth").and_then(|v| v.as_str()).unwrap_or("user").to_string();
        let rail = parsed.get("rail").cloned();
        out.push(ViewManifest { name: dir_name, path, title, layout, auth, rail });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Write `views/routes.gen.ts` — the router-only view table (pattern +
/// view name, host-agnostic). If the shell's deck build already emits a
/// generic route table for the tree, this half can instead feed those
/// patterns into deck's emitter; kept for the standalone path.
fn write_views_routes(views_dir: &Path, label: &str, manifests: &[ViewManifest]) -> Result<()> {
    let entries = manifests
        .iter()
        .map(|m| format!("  {{ pattern: '{}', view: '{}' }},", m.path, m.name))
        .collect::<Vec<_>>()
        .join("\n");
    let patterns = manifests
        .iter()
        .map(|m| format!("  {{ pattern: '{}' }},", m.path))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "\
// @generated by ship-ui view-manifest gen — DO NOT EDIT.
// Router-only view table for the `{label}` view tree (every
// `views/<name>/manifest.jsonc`). Host metadata lives in meta.gen.ts;
// the router never reads it.
import type {{ RouteShape }} from '@ship/deck/router'

export const VIEW_ROUTES: ViewRoute[] = [
{entries}
]

export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
{patterns}
]

export type ViewRoute = {{ pattern: string; view: string }}
",
        label = label,
    );
    let target = views_dir.join("routes.gen.ts");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("create views/routes.gen.ts parent")?;
    }
    let changed = std::fs::read_to_string(&target).map(|cur| cur != body).unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write views/routes.gen.ts")?;
    }
    Ok(())
}

/// Write `views/meta.gen.ts` — the host's contract. `ViewMeta[]` with
/// chrome/auth/rail. The host reads this at boot to choose chrome,
/// gate access, and add launcher rail entries. The router does NOT
/// see this file.
fn write_views_meta(views_dir: &Path, label: &str, manifests: &[ViewManifest]) -> Result<()> {
    let entries = manifests
        .iter()
        .map(|m| {
            let rail_str = m
                .rail
                .as_ref()
                .map(|r| serde_json::to_string(r).unwrap_or_else(|_| "undefined".to_string()))
                .unwrap_or_else(|| "undefined".to_string());
            format!(
                "  {{ name: '{}', title: '{}', layout: '{}', auth: '{}', rail: {rail_str} }},",
                m.name, m.title, m.layout, m.auth
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        "\
// @generated by ship-ui view-manifest gen — DO NOT EDIT.
// Host-only view metadata for the `{label}` view tree. The shell reads
// this at boot to choose chrome (layout), gate access (auth), and build
// the launcher rail. The router never reads this file.

export const VIEW_META: ViewMeta[] = [
{entries}
]

export interface ViewRail {{
  label: string
  icon: string
}}

export interface ViewMeta {{
  name: string
  title: string
  layout: string
  auth: string
  rail?: ViewRail
}}
",
        label = label,
    );
    let target = views_dir.join("meta.gen.ts");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("create views/meta.gen.ts parent")?;
    }
    let changed = std::fs::read_to_string(&target).map(|cur| cur != body).unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write views/meta.gen.ts")?;
    }
    Ok(())
}

/// Given a `views/` bucket with manifests, emit BOTH `routes.gen.ts`
/// (router fields) and `meta.gen.ts` (host fields). No-op when the dir
/// is absent or holds no manifests. Returns the number of views discovered.
pub fn generate_view_artifacts(views_dir: &Path, label: &str) -> Result<usize> {
    if !views_dir.is_dir() {
        return Ok(0);
    }
    let manifests = discover_views(views_dir)
        .with_context(|| format!("walking views for `{label}`"))?;
    if manifests.is_empty() {
        return Ok(0);
    }
    write_views_routes(views_dir, label, &manifests)
        .with_context(|| format!("writing views/routes.gen.ts for `{label}`"))?;
    write_views_meta(views_dir, label, &manifests)
        .with_context(|| format!("writing views/meta.gen.ts for `{label}`"))?;
    Ok(manifests.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, name: &str, body: &str) {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("manifest.jsonc"), body).unwrap();
    }

    #[test]
    fn generate_view_artifacts_emits_routes_and_meta() {
        let tmp = tempfile::tempdir().unwrap();
        let views = tmp.path().join("views");
        std::fs::create_dir_all(&views).unwrap();
        write_manifest(
            &views,
            "board",
            r#"// applet manifest
{
  "name": "board",
  "path": "/board",
  "title": "Board",
  "layout": "focus",
  "auth": "user",
  "rail": { "label": "Board", "icon": "grid" }
}"#,
        );
        write_manifest(
            &views,
            "clock",
            r#"{ "name": "clock", "path": "/clock" }"#,
        );

        let n = generate_view_artifacts(&views, "test").unwrap();
        assert_eq!(n, 2);

        let routes = std::fs::read_to_string(views.join("routes.gen.ts")).unwrap();
        assert!(routes.contains("{ pattern: '/board', view: 'board' },"));
        assert!(routes.contains("{ pattern: '/clock', view: 'clock' },"));
        assert!(!routes.contains("auth"), "router table must stay host-agnostic");

        let meta = std::fs::read_to_string(views.join("meta.gen.ts")).unwrap();
        assert!(meta.contains(
            "{ name: 'board', title: 'Board', layout: 'focus', auth: 'user', rail: {\"label\":\"Board\",\"icon\":\"grid\"} },"
        ));
        // Defaults: title falls back to the dir name, layout to `default`, auth to `user`.
        assert!(meta.contains("{ name: 'clock', title: 'clock', layout: 'default', auth: 'user', rail: undefined },"));
    }

    #[test]
    fn generate_view_artifacts_noop_without_dir_or_manifests() {
        let tmp = tempfile::tempdir().unwrap();
        // Absent dir → 0, no files.
        let absent = tmp.path().join("nope");
        assert_eq!(generate_view_artifacts(&absent, "x").unwrap(), 0);
        // Present but empty → 0, no files.
        let empty = tmp.path().join("views");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(generate_view_artifacts(&empty, "x").unwrap(), 0);
        assert!(!empty.join("routes.gen.ts").exists());
        assert!(!empty.join("meta.gen.ts").exists());
    }
}
