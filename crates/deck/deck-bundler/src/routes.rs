//! Typed dynamic-route codegen (#168) — deck owns route discovery, since deck
//! builds the app. Walks an app's `src/` view tree for `[param]` route
//! dirs and emits `routes.gen.ts` (the `RouteShape[]` the shell registers via
//! `defineRoutes`, plus typed path builders). Ported from the retired
//! `scripts/gen-routes.ts`; run by `deck build` (and `deck routes`).

use std::path::Path;

use anyhow::{Context, Result};

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
/// first segment (e.g. `studio`, `home`, anything under the shell's
/// first-party root). The codegen is fs-routing at the shell level;
/// it has no opinion about which segments are valid route roots —
/// every dir under `src/` that contains a mountable view is a route.
fn to_route(src_dir: &Path, dir: &Path) -> Option<Route> {
    let rel = dir.strip_prefix(src_dir).ok()?;
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

/// Generate `<shell_root>/src/routes.gen.ts` from the shell's full view tree.
/// No-op (returns 0) when the app has no `src/` dir. Returns the count of
/// dynamic routes emitted.
///
/// Walks every dir under `src/` recursively for a mountable view
/// (`page.sui`) and emits a `RouteShape[]` for any dir with at least
/// one `[param]` segment, plus typed path builders namespaced by
/// the top-level route name. Studio's nested `[id]` sub-tree is one
/// such branch; future `[param]` sub-routes (under home, account,
/// etc.) get the same treatment for free.
pub fn generate_routes(shell_root: &Path) -> Result<usize> {
    let src_dir = shell_root.join("src");
    if !src_dir.is_dir() {
        return Ok(0);
    }
    let mut dirs = Vec::new();
    view_dirs(&src_dir, &mut dirs).context("walking shell view tree")?;
    let mut routes: Vec<Route> = dirs.iter().filter_map(|d| to_route(&src_dir, d)).collect();
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
// Typed dynamic-route table for the shell's view tree (every `[param]`
// dir under `src/`). Regenerate with `deck routes` (also run inside
// `deck build`) after adding or renaming a `[param]` route dir.
// See ADR unified-deck-router.

import type {{ RouteShape }} from '@ship/sui-runtime'

/** Every dynamic route pattern the shell should pre-register via
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
"
    );
    // Write-if-changed (#129-class loop fix): the app build regenerates this on
    // EVERY rebuild, and the file lives under the watched `apps/shell/src` root.
    // An unconditional write retriggers the bundler watcher and self-perpetuates
    // a rebuild/reload loop (shipd CPU spin, page reloads ~6×/sec). Only touch
    // the file when the generated content actually moved.
    let target = src_dir.join("routes.gen.ts");
    let changed = std::fs::read_to_string(&target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write routes.gen.ts")?;
    }
    Ok(routes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_route_discovery() {
        let root = tempfile::tempdir().unwrap();
        let s = root.path().join("src");
        for d in ["studio/agents/[id]/settings", "studio/components/[name]", "studio/skills"] {
            std::fs::create_dir_all(s.join(d)).unwrap();
            std::fs::write(s.join(d).join("page.sui"), "<div/>").unwrap();
        }
        let n = generate_routes(root.path()).unwrap();
        assert_eq!(n, 2, "two dynamic routes (skills is static)");
        let ts = std::fs::read_to_string(s.join("routes.gen.ts")).unwrap();
        assert!(ts.contains("{ pattern: '/studio/agents/:id/settings' }"));
        assert!(ts.contains("{ pattern: '/studio/components/:name' }"));
        // Builders are namespaced by the top-level segment (studio), so
        // future `[param]` dirs under home/account get their own namespace
        // without collision.
        assert!(ts.contains("studioAgentsIdSettings: (p: { id: string }): string => `/studio/agents/${p.id}/settings`"));
        assert!(ts.contains("studioComponentsName: (p: { name: string }): string => `/studio/components/${p.name}`"));
        assert!(!ts.contains("skills"), "static route omitted");
    }
    #[test]
    fn no_src_dir_is_noop() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(generate_routes(root.path()).unwrap(), 0);
    }
    // Regression: routes.gen.ts lives under the watched `apps/shell/src` root and
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
        let s = root.path().join("src/studio");
        std::fs::create_dir_all(s.join("agents/[id]")).unwrap();
        std::fs::write(s.join("agents/[id]/page.sui"), "<div/>").unwrap();
        generate_routes(root.path()).unwrap();
        let target = s.parent().unwrap().join("routes.gen.ts");
        let mut perms = std::fs::metadata(&target).unwrap().permissions();
        perms.set_mode(0o444); // read-only: any write attempt now errors
        std::fs::set_permissions(&target, perms).unwrap();
        // Unchanged tree → write-if-changed must SKIP the write, so this is Ok.
        // Before the fix this errored (EACCES) — the loop's signature.
        generate_routes(root.path())
            .expect("unchanged routes must not rewrite the file (would self-trigger the watcher)");
    }
}
