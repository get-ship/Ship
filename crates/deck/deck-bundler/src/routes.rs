//! Typed dynamic-route codegen (#168) — deck owns route discovery, since deck
//! builds the app. Walks an app's `routes/` view tree for `[param]` route dirs
//! and emits `routes.gen.ts` (the `RouteShape[]` the shell registers via
//! `defineRoutes`, plus typed path builders). Ported from the retired
//! `scripts/gen-routes.ts`; run by `deck build` (and `deck routes`).
//!
//! Multi-root: a single invocation can walk several roots (the shell's first-
//! party `routes/`, the spaces scaffold's `routes/`) and emit a `routes.gen.ts`
//! at each requested output path. Every root is a `routes/` dir — a route IS a
//! directory with a `page.sui` and (optionally) `controller.ts`. A `[param]`
//! segment in the dir name is a dynamic. No manifests, no per-route metadata.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// One codegen spec: walk `dir` recursively for mountable views (`page.sui`),
/// emit the generated `routes.gen.ts` content to `out`. `label` is the
/// human-readable surface the file is for (e.g. `shell`, `spaces`) — it shows
/// up in the file header so the file is self-describing.
#[derive(Debug, Clone)]
pub struct RouteGenSpec {
    pub dir: PathBuf,
    pub out: PathBuf,
    pub label: String,
}

struct Route {
    pattern: String,
    params: Vec<String>,
    builder: String,
}

/// Recursively collect dirs holding a mountable view (`page.sui`).
fn view_dirs(dir: &Path, acc: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let child = entry.path();
            if child.join("page.sui").exists() {
                acc.push(child.clone());
            }
            view_dirs(&child, acc)?;
        }
    }
    Ok(())
}

/// `studio/agents/[id]/settings` → pattern `/studio/agents/:id/settings`,
/// params `[id]`, builder `studioAgentsIdSettings`. Static routes (no
/// `[param]`) return `None` — they need no `defineRoutes` entry.
///
/// The first segment of the relative path is preserved as the URL's
/// first segment (e.g. `studio`, `home`, anything under the routes root).
/// The codegen is fs-routing at the app level; it has no opinion about
/// which segments are valid route roots — every dir under the routes root
/// that contains a mountable view is a route.
fn to_route(routes_dir: &Path, dir: &Path) -> Option<Route> {
    let rel = dir.strip_prefix(routes_dir).ok()?;
    let segs_raw: Vec<String> =
        rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    let mut params = Vec::new();
    let pattern_segs: Vec<String> = segs_raw
        .iter()
        .map(|s| {
            if let Some(p) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                params.push(p.to_string());
                format!(":{p}")
            } else {
                s.clone()
            }
        })
        .collect();
    if params.is_empty() {
        return None;
    }
    let pattern = format!("/{}", pattern_segs.join("/"));
    // Builder name: camelCase of the segments, including the first
    // segment (the top-level route name) so builders namespace per
    // route. e.g. `studio/agents/[id]` → `studioAgentsId`.
    let joined = segs_raw
        .iter()
        .map(|s| s.trim_start_matches('[').trim_end_matches(']'))
        .collect::<Vec<_>>()
        .join("-");
    let builder = camel_case(&joined);
    Some(Route { pattern, params, builder })
}

/// `agents-id-settings` → `agentsIdSettings` (uppercase the char after each `-`).
fn camel_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for c in s.chars() {
        if c == '-' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Generate `routes.gen.ts` for every spec. Each spec walks its `dir`
/// (a `routes/` tree) and writes a `routes.gen.ts` to its `out` path.
/// Returns the total number of dynamic routes emitted across all specs.
///
/// No-op for a spec whose `dir` is absent. Per-spec write-if-changed
/// (`routes.gen.ts` lives under the watched `apps/...` root and the app
/// build regenerates it on every rebuild — an unconditional write would
/// self-trigger the bundler watcher and self-perpetuate a rebuild/reload
/// loop. See the inline regression test.)
pub fn generate_routes(specs: &[RouteGenSpec]) -> Result<usize> {
    let mut total = 0;
    for spec in specs {
        total += generate_routes_one(spec)?;
    }
    Ok(total)
}

fn generate_routes_one(spec: &RouteGenSpec) -> Result<usize> {
    let routes_dir = &spec.dir;
    if !routes_dir.is_dir() {
        return Ok(0);
    }
    let mut dirs = Vec::new();
    view_dirs(routes_dir, &mut dirs)
        .with_context(|| format!("walking routes tree for `{}`", spec.label))?;
    let mut routes: Vec<Route> = dirs.iter().filter_map(|d| to_route(routes_dir, d)).collect();
    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern));
    let pattern_lines = routes
        .iter()
        .map(|r| format!("  {{ pattern: '{}' }},", r.pattern))
        .collect::<Vec<_>>()
        .join("\n");
    let builder_lines = routes
        .iter()
        .map(|r| {
            let arg = format!(
                "p: {{ {} }}",
                r.params.iter().map(|k| format!("{k}: string")).collect::<Vec<_>>().join("; ")
            );
            // `/studio/agents/:id` → template `/studio/agents/${p.id}`.
            let mut path = r.pattern.clone();
            for k in &r.params {
                path = path.replace(&format!(":{k}"), &format!("${{p.{k}}}"));
            }
            format!("  /** {} */\n  {}: ({arg}): string => `{path}`,", r.pattern, r.builder)
        })
        .collect::<Vec<_>>()
        .join("\n");
    // Flush-left literal so the JSDoc ` *` alignment is byte-exact (a `\`-joined
    // string would strip the leading space). Leading `\` drops the first newline.
    let body = format!(
        "\
// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Typed dynamic-route table for the `{label}` view tree (every `[param]`
// dir under `routes/`). Regenerate with `deck routes` (also run inside
// `deck build`) after adding or renaming a `[param]` route dir.
// See ADR unified-deck-router.

import type {{ RouteShape }} from '@ship/sui-runtime'

/** Every dynamic route pattern the host should pre-register via
 *  `defineRoutes` so `route.params.<name>()` resolves on the matching
 *  pathname. Static routes (no `[param]`) need no entry — the runtime
 *  matches them by segment equality. */
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
{pattern_lines}
]

/** Typed path builders — `navigate(routes.studioAgentsId({{ id }}))`.
 *  The first segment namespaces the builder so future `[param]` dirs
 *  under home/account/etc. don't collide. The param keys are checked
 *  at compile time; you cannot typo or omit one. */
export const routes = {{
{builder_lines}
}}
",
        label = spec.label,
    );
    // Write-if-changed (#129-class loop fix): the app build regenerates this on
    // EVERY rebuild, and the file lives under the watched `apps/...` root.
    // An unconditional write retriggers the bundler watcher and self-perpetuates
    // a rebuild/reload loop (shipd CPU spin, page reloads ~6×/sec). Only touch
    // the file when the generated content actually moved.
    let target = &spec.out;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).context("create routes.gen.ts parent")?;
    }
    let changed = std::fs::read_to_string(target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(target, body).context("write routes.gen.ts")?;
    }
    Ok(routes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(root: &Path, label: &str) -> RouteGenSpec {
        RouteGenSpec {
            dir: root.join("routes"),
            out: root.join("routes.gen.ts"),
            label: label.to_string(),
        }
    }

    #[test]
    fn shell_route_discovery() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        for d in ["studio/agents/[id]/settings", "studio/components/[name]", "studio/skills"] {
            std::fs::create_dir_all(r.join(d)).unwrap();
            std::fs::write(r.join(d).join("page.sui"), "<div/>").unwrap();
        }
        let n = generate_routes(&[spec(root.path(), "shell")]).unwrap();
        assert_eq!(n, 2, "two dynamic routes (skills is static)");
        let ts = std::fs::read_to_string(r.parent().unwrap().join("routes.gen.ts")).unwrap();
        assert!(ts.contains("{ pattern: '/studio/agents/:id/settings' }"));
        assert!(ts.contains("{ pattern: '/studio/components/:name' }"));
        // Builders are namespaced by the top-level segment (studio), so
        // future `[param]` dirs under home/account get their own namespace
        // without collision.
        assert!(ts.contains("studioAgentsIdSettings: (p: { id: string }): string => `/studio/agents/${p.id}/settings`"));
        assert!(ts.contains("studioComponentsName: (p: { name: string }): string => `/studio/components/${p.name}`"));
        assert!(!ts.contains("skills"), "static route omitted");
        // The label appears in the header so the file is self-describing.
        assert!(ts.contains("for the `shell` view tree"));
    }

    #[test]
    fn multi_root_emits_one_file_per_spec() {
        // Two roots in the same tempdir, each with a different mountable view.
        // The codegen must walk both and write two distinct output files.
        let root = tempfile::tempdir().unwrap();
        let shell_routes = root.path().join("apps/shell/routes");
        let space_routes = root.path().join("apps/spaces/routes");
        std::fs::create_dir_all(shell_routes.join("studio/agents/[id]")).unwrap();
        std::fs::write(shell_routes.join("studio/agents/[id]/page.sui"), "<div/>").unwrap();
        std::fs::create_dir_all(space_routes.join("[appId]")).unwrap();
        std::fs::write(space_routes.join("[appId]/page.sui"), "<div/>").unwrap();
        let specs = vec![
            RouteGenSpec {
                dir: shell_routes,
                out: root.path().join("apps/shell/routes.gen.ts"),
                label: "shell".to_string(),
            },
            RouteGenSpec {
                dir: space_routes,
                out: root.path().join("apps/spaces/routes.gen.ts"),
                label: "spaces".to_string(),
            },
        ];
        let n = generate_routes(&specs).unwrap();
        assert_eq!(n, 2, "one dynamic route per root, total two");
        let shell_ts =
            std::fs::read_to_string(root.path().join("apps/shell/routes.gen.ts")).unwrap();
        let spaces_ts =
            std::fs::read_to_string(root.path().join("apps/spaces/routes.gen.ts")).unwrap();
        assert!(shell_ts.contains("/studio/agents/:id"));
        assert!(shell_ts.contains("for the `shell` view tree"));
        assert!(spaces_ts.contains("/:appId"));
        assert!(spaces_ts.contains("for the `spaces` view tree"));
    }

    #[test]
    fn missing_routes_dir_is_noop() {
        let root = tempfile::tempdir().unwrap();
        // The `routes/` dir doesn't exist; the spec is a no-op (returns 0)
        // and no file is written.
        let n = generate_routes(&[spec(root.path(), "shell")]).unwrap();
        assert_eq!(n, 0);
        assert!(!root.path().join("routes.gen.ts").exists());
    }

    // Regression: routes.gen.ts lives under the watched `apps/...` root and
    // the app build regenerates it every rebuild. An unconditional write
    // self-triggered the bundler watcher → endless rebuild/reload loop (shipd CPU
    // spin, ~6 reloads/sec). A second generate with an UNCHANGED tree must NOT
    // rewrite the file. Proof: make it read-only — a skipped write succeeds; an
    // attempted write fails with permission denied.
    #[cfg(unix)]
    #[test]
    fn unchanged_tree_does_not_rewrite_routes_gen() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes/studio");
        std::fs::create_dir_all(r.join("agents/[id]")).unwrap();
        std::fs::write(r.join("agents/[id]/page.sui"), "<div/>").unwrap();
        let s = spec(root.path(), "shell");
        generate_routes(&[s.clone()]).unwrap();
        let target = &s.out;
        let mut perms = std::fs::metadata(target).unwrap().permissions();
        perms.set_mode(0o444); // read-only: any write attempt now errors
        std::fs::set_permissions(target, perms).unwrap();
        // Unchanged tree → write-if-changed must SKIP the write, so this is Ok.
        // Before the fix this errored (EACCES) — the loop's signature.
        generate_routes(&[s])
            .expect("unchanged routes must not rewrite the file (would self-trigger the watcher)");
    }
}
