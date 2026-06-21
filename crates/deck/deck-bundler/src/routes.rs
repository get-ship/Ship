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

use anyhow::{Context, Result, bail};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Route {
    pub(crate) pattern: String,
    pub(crate) params: Vec<String>,
    pub(crate) builder: String,
}

/// A `_layout.sui` discovered at the top level of a `routes/` dir. The
/// runtime wraps the subtree's content in the layout component; the layout
/// template has a `<slot>` that the route content fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) path_prefix: String,
    pub(crate) tag: String,
}

/// Recursively collect dirs holding a mountable view (`page.sui`).
/// The root `dir` itself is also a candidate — `routes/page.sui`
/// is a mount at `/` (the index route). Subdirs are walked for
/// nested mounts.
fn view_dirs(dir: &Path, acc: &mut Vec<std::path::PathBuf>) -> Result<()> {
    if dir.join("page.sui").is_file() {
        acc.push(dir.to_path_buf());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            view_dirs(&entry.path(), acc)?;
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
    let pattern = if pattern_segs.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", pattern_segs.join("/"))
    };
    // Builder name: camelCase of the segments, including the first
    // segment (the top-level route name) so builders namespace per
    // route. e.g. `studio/agents/[id]` → `studioAgentsId`. The root
    // `routes/page.sui` (a mount at `/`) gets the reserved `index`
    // name — a single segment is too short to camelCase meaningfully.
    let builder = if segs_raw.is_empty() {
        "index".to_string()
    } else {
        let joined = segs_raw
            .iter()
            .map(|s| s.trim_start_matches('[').trim_end_matches(']'))
            .collect::<Vec<_>>()
            .join("-");
        camel_case(&joined)
    };
    Some(Route { pattern, params, builder })
}

/// Reconstruct a route's view-module import path from its URL pattern — the
/// inverse of `to_route`'s segment mapping (`:param` ↔ `[param]` dir):
/// `/studio/agents/:id` → `./routes/studio/agents/[id]/page.sui`; `/` →
/// `./routes/page.sui`. The emitter uses it to statically import each view.
fn module_import_path(pattern: &str) -> String {
    let segs: Vec<String> = pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.strip_prefix(':')
                .map(|p| format!("[{p}]"))
                .unwrap_or_else(|| s.to_string())
        })
        .collect();
    if segs.is_empty() {
        "./routes/page.sui".to_string()
    } else {
        format!("./routes/{}/page.sui", segs.join("/"))
    }
}

/// Inverse of the layout discovery: `/` → `./routes/_layout.sui`
/// (the root layout, sibling of `routes/page.sui`); `/studio` →
/// `./routes/studio/_layout.sui`. The codegen imports the layout's
/// `.sui` so the chrome wrapper it composes registers at first
/// paint (the per-view resolver chain — layout → its first
/// `<sui-*` / `<ship-*` tag → that wrapper's source `.sui` —
/// lands the `customElements.define` call when the module loads).
fn layout_module_path(path_prefix: &str) -> String {
    if path_prefix == "/" {
        return "./routes/_layout.sui".to_string();
    }
    let rest = path_prefix.strip_prefix('/').unwrap_or(path_prefix);
    format!("./routes/{rest}/_layout.sui")
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

/// Walk `routes_dir` for `_layout.sui` files. Two flavors:
///   - A root `routes/_layout.sui` (sibling of `routes/page.sui`) becomes
///     a layout entry with prefix `/` — the runtime's longest-prefix
///     match treats `/` as "every pathname" (see
///     `packages/deck/route/src/layouts.ts::matchPrefix`). This is how
///     the hub root layout covers the index route `/` and every other
///     route under it; without it the nav had to be composed per-page.
///   - A `routes/<name>/_layout.sui` becomes a layout entry with prefix
///     `/<name>`. Unchanged from the pre-root-layout codegen.
///
/// The tag is read from the first `<sui-…>` or `<ship-…>` element in
/// the file. Layouts are emitted sorted by `path_prefix` so the
/// generated `routes.gen.ts` is byte-stable across runs.
fn discover_layouts(routes_dir: &Path) -> Result<Vec<Layout>> {
    let mut layouts = Vec::new();
    // Root layout: `routes/_layout.sui` → prefix `/`. Discovered before
    // the subdir loop so the prefix `/` sorts first (shortest path_prefix,
    // and the loop below sorts by `path_prefix` for byte stability).
    let root_layout = routes_dir.join("_layout.sui");
    if root_layout.is_file() {
        let tag = extract_layout_tag(&root_layout)
            .with_context(|| format!("parsing `{}`", root_layout.display()))?;
        layouts.push(Layout { path_prefix: "/".to_string(), tag });
    }
    for entry in std::fs::read_dir(routes_dir).with_context(|| {
        format!("reading routes dir `{}`", routes_dir.display())
    })? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let child = entry.path();
        let layout_file = child.join("_layout.sui");
        if !layout_file.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let prefix = format!("/{name}");
        let tag = extract_layout_tag(&layout_file)
            .with_context(|| format!("parsing `{}`", layout_file.display()))?;
        layouts.push(Layout { path_prefix: prefix, tag });
    }
    layouts.sort_by(|a, b| a.path_prefix.cmp(&b.path_prefix));
    Ok(layouts)
}

/// Read up to the first 4 KiB of a `_layout.sui` and return the tag name
/// of the first `<sui-…>` or `<ship-…>` element. The SUI template grammar
/// is HTML-flavoured; the codegen does not need a full parse — it just needs
/// the wrapper component the runtime should import. If no qualifying tag is
/// found, error: a layout with no `<sui-*` / `<ship-*` element has no
/// component to mount.
fn extract_layout_tag(path: &Path) -> Result<String> {
    // Cap the read so a stray giant file can't pin the codegen. The tag is
    // always on the first line in practice; 4 KiB is generous.
    const HEAD_BYTES: usize = 4096;
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading `{}`", path.display()))?;
    let head_bytes = if bytes.len() > HEAD_BYTES { &bytes[..HEAD_BYTES] } else { &bytes[..] };
    let head = std::str::from_utf8(head_bytes)
        .with_context(|| format!("non-utf8 in `{}`", path.display()))?;
    // Match the opening of the first element whose tag starts with `sui-`
    // or `ship-`. Whitespace, a tag name (letters/digits/`-`/`_`), and
    // either a `>` or whitespace ends the tag name. This is intentionally
    // minimal — the layout is just a chrome wrapper, not a full template
    // parse.
    let mut idx = 0;
    let bytes_str = head.as_bytes();
    while idx < bytes_str.len() {
        if bytes_str[idx] != b'<' {
            idx += 1;
            continue;
        }
        let after_lt = &bytes_str[idx + 1..];
        // Skip closing tags (`</foo>`) and doctype/processing (`<!…`, `<?…`).
        if let Some(&b) = after_lt.first() {
            if b == b'/' || b == b'!' || b == b'?' {
                idx += 1;
                continue;
            }
        }
        // Read tag name bytes.
        let name_start = idx + 1;
        let mut name_end = name_start;
        while name_end < bytes_str.len() {
            let b = bytes_str[name_end];
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                name_end += 1;
            } else {
                break;
            }
        }
        if name_end == name_start {
            idx += 1;
            continue;
        }
        let tag = std::str::from_utf8(&bytes_str[name_start..name_end])
            .context("non-utf8 tag name")?;
        if tag.starts_with("sui-") || tag.starts_with("ship-") {
            return Ok(tag.to_string());
        }
        idx = name_end;
    }
    bail!(
        "`{}` has no `<sui-…>` or `<ship-…>` element — a layout must name a wrapper component",
        path.display()
    );
}
/// Generate `routes.gen.ts` for every spec. Each spec walks its `dir`
/// (a `routes/` tree) and writes a `routes.gen.ts` to its `out` path.
/// Returns the total number of dynamic routes emitted across all specs.
///
/// No-op for a spec whose `dir` is absent. Per-spec write-if-changed
/// (`routes.gen.ts` lives under the watched `apps/...` root and the app
/// build regenerates it on every rebuild — an unconditional write would
pub fn generate_routes(specs: &[RouteGenSpec]) -> Result<usize> {
    let mut total = 0;
    for spec in specs {
        total += generate_routes_one(spec)?;
        // View-route discovery is a sibling walker: emit
        // `views/routes.gen.ts` (router fields) and
        // `views/meta.gen.ts` (host fields) for the same spec.
        // No-op if no `<root>/views/` dir exists.
        write_view_artifacts(spec)?;
    }
    Ok(total)
}

/// One raw view manifest entry. Parsed from `views/<name>/manifest.jsonc`.
/// The JSON allows comments (`// …`) which serde_json doesn't natively
/// accept, so we strip line-`//` comments before parsing. The router
/// fields (pattern, view name) go to `views/routes.gen.ts`; the host
/// fields (title, layout, auth, rail) go to `views/meta.gen.ts`. The
/// split is the contract — the router is host-agnostic, the host reads
/// what it needs.
struct ViewManifest {
    name: String,
    path: String,
    title: String,
    layout: String,
    auth: String,
    rail: Option<serde_json::Value>,
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
            // Skip until newline.
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
fn discover_views(views_dir: &Path) -> Result<Vec<ViewManifest>> {
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
        out.push(ViewManifest {
            name: dir_name,
            path,
            title,
            layout,
            auth,
            rail,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Write `views/routes.gen.ts` — the router's contract. `ViewRoute[]`
/// (pattern + view) and a derived `DYNAMIC_ROUTE_PATTERNS` for the
/// host to pass to `defineRoutes`. The router is host-agnostic.
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
// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Router-only view table for the `{label}` view tree (every
// `views/<name>/manifest.jsonc`). The router cares about the URL
// pattern and the view resource name; host fields (title, layout,
// auth, rail) live in `views/meta.gen.ts`. The two files are
// separate contracts: the router is host-agnostic.
// See ADR unified-deck-router.

import type {{ RouteShape }} from '@ship/deck/route'

/** Per-view route entries. `pattern` is the URL; `view` is the daemon
 *  resource name (the view is served at `/_deck/views/<view>/…`).
 *  The host registers the patterns with the runtime's `defineRoutes`
 *  IF the view is downloaded/active at boot. */
export const VIEW_ROUTES: ViewRoute[] = [
{entries}
]

/** Convenience table for `defineRoutes` — the patterns the host
 *  should pre-register. Static patterns need no entry (the runtime
 *  matches them by segment equality) but listing them here is
 *  defensive and matches the codegen's behavior for `routes/`. */
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
    let changed = std::fs::read_to_string(&target)
        .map(|cur| cur != body)
        .unwrap_or(true);
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
// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Host-only view metadata for the `{label}` view tree (every
// `views/<name>/manifest.jsonc`). The host reads this to choose
// chrome (layout), gate access (auth), and add launcher rail
// entries. The router is host-agnostic and does NOT see this file
// (the router's contract lives in `views/routes.gen.ts`).
// See ADR unified-deck-router.

/** Per-view host metadata. The host builds its launcher rail from
 *  `name` (the navigation key) and the `rail` shape; chooses the
 *  chrome wrapper from `layout`; gates access on `auth`. `path` is
 *  NOT here — the router is the only consumer of `path` and it
 *  reads it from `views/routes.gen.ts`. */
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
    let changed = std::fs::read_to_string(&target)
        .map(|cur| cur != body)
        .unwrap_or(true);
    if changed {
        std::fs::write(&target, body).context("write views/meta.gen.ts")?;
    }
    Ok(())
}

/// The full set of data `routes.gen.ts` registers. Walking the `routes/`
/// tree produces one of these; the formatter turns it into a TypeScript
/// module. Tests assert on this data layer (stable across format
/// changes), not on the emitted text — the text is presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RouteTree {
    pub label: String,
    pub routes: Vec<Route>,
    pub layouts: Vec<Layout>,
}

fn build_route_tree(routes_dir: &Path, label: &str) -> Result<RouteTree> {
    if !routes_dir.is_dir() {
        return Ok(RouteTree { label: label.to_string(), routes: vec![], layouts: vec![] });
    }
    let mut dirs = Vec::new();
    view_dirs(routes_dir, &mut dirs)
        .with_context(|| format!("walking routes tree for `{label}`"))?;
    let mut routes: Vec<Route> = dirs.iter().filter_map(|d| to_route(routes_dir, d)).collect();
    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern));
    let layouts = discover_layouts(routes_dir)
        .with_context(|| format!("walking layouts for `{label}`"))?;
    Ok(RouteTree { label: label.to_string(), routes, layouts })
}

fn render_routes_gen(tree: &RouteTree) -> String {
    let pattern_lines = tree
        .routes
        .iter()
        .map(|r| format!("  {{ pattern: '{}' }},", r.pattern))
        .collect::<Vec<_>>()
        .join("\n");
    let builder_lines = tree
        .routes
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
    let layout_lines = tree
        .layouts
        .iter()
        .map(|l| format!("  {{ pathPrefix: '{}', tag: '{}' }},", l.path_prefix, l.tag))
        .collect::<Vec<_>>()
        .join("\n");
    // Each route's view module is statically imported (route0, route1, …) so
    // the bundler pulls it into the graph, and keyed by pattern in the module
    // map handed to `mountRouter`. The map reference is a real use, so the
    // tree-shaker keeps the modules — no globalThis stash needed.
    let route_imports = tree
        .routes
        .iter()
        .enumerate()
        .map(|(i, r)| format!("import * as route{i} from '{}';", module_import_path(&r.pattern)))
        .collect::<Vec<_>>()
        .join("\n");
    // Each layout's `.sui` is statically imported alongside the route
    // views so the chrome wrapper it composes (its first `<sui-*` /
    // `<ship-*` element) is registered at first paint. The runtime
    // builds the chain via `document.createElement(layout.tag)`; if the
    // tag isn't registered, the createElement call returns a bare
    // HTMLElement that never upgrades, and the route content's named
    // slot (the `route` slot) has nothing to project into. The
    // shell's `apps/shell/src/components/index.ts` is the
    // host-side fix for the same gap; the framework does the same
    // here so every per-app host gets the registration without
    // hand-seeding.
    //
    // Import paths mirror the on-disk layout files relative to the
    // routes root. A root `routes/_layout.sui` imports as
    // `./routes/_layout.sui`; a `routes/<name>/_layout.sui` imports
    // as `./routes/<name>/_layout.sui`. The module side-effect
    // (the per-view resolver compiles the layout, the layout's
    // first tag is emitted as `@sui-tag/<tag>`, that resolves to the
    // chrome wrapper's source `.sui`, that compiles as a Component
    // and registers the custom element) is the registration
    // mechanism — no tree-shake removal.
    // The design tokens (the OKLCH palette, semantic surfaces, tone ramps,
    // role tokens, motion/density channels) ride with every per-app build by
    // convention: the entry graph imports `@ship/tokens/tokens.css`, and
    // Rolldown's existing CSS-asset path (`module_types: { ".css" -> Asset }`,
    // `deck-bundler/src/app.rs`) bundles it as a content-hashed
    // `assets/tokens-<hash>.css` and lifts a `<link rel="stylesheet">` into
    // the generated `index.html` BEFORE the entry script. Zero per-app
    // config, zero hand-authored import — the generated entry pulls them in.
    //
    // Why `@ship/tokens/tokens.css` and not a runtime-injected `<link>` to a
    // daemon-served path: the runtime injection in this file used to fetch
    // `/api/assets/view-tokens.css` (a deck-core static asset). That path
    // doesn't exist in a static-only deploy (no daemon), so the bundled app
    // would ship with NO tokens and every `var(--token)` would resolve to
    // nothing — the unstyled-hub bug. Bundling ships tokens in the same
    // graph as the SPA itself, so the gate (built `index.html` has a token
    // `<link>`, emitted CSS has `:root` tokens) holds for daemon AND
    // static-only deploys.
    let token_import = format!("import '{}';", "@ship/tokens/tokens.css");
    let layout_imports = tree
        .layouts
        .iter()
        .map(|l| format!("import '{}';", layout_module_path(&l.path_prefix)))
        .collect::<Vec<_>>()
        .join("\n");
    let module_map = tree
        .routes
        .iter()
        .enumerate()
        .map(|(i, r)| format!("    '{}': route{i},", r.pattern))
        .collect::<Vec<_>>()
        .join("\n");
    // No routes → no entry behavior (nothing to import or mount).
    let router_block = if tree.routes.is_empty() {
        String::new()
    } else {
        format!(
"
{token_import}
// Hand the generated tables + statically-imported view modules to the
// framework router. `mountRouter` (@ship/deck/route) owns history, route
// match, the layout chain, and SPA navigation — one router for every
// per-app deck host. See ADR `unified-deck-router`.
mountRouter({{
  root: '#{root_id}',
  routes: DYNAMIC_ROUTE_PATTERNS,
  layouts: LAYOUTS,
  modules: {{
{module_map}
  }},
  clients: {{}},
}});
",
            root_id = crate::templates::ROOT_ID,
            token_import = token_import,
        )
    };
    format!(
"// @generated by `deck routes` (deck-bundler) — DO NOT EDIT.
// Typed route table + entry for the `{label}` view tree (every `[param]`
// dir under `routes/`). This file IS the entry — the dev authors no
// main.ts. It declares the route table, statically imports each route's
// view module, and hands them to the framework router `mountRouter`
// (@ship/deck/route), which owns route match + layout chains + SPA
// navigation. See ADR `unified-deck-router`.
import {{ mountRouter, type RouteShape, type LayoutShape }} from '@ship/deck/route'
{route_imports}
{layout_imports}
export const DYNAMIC_ROUTE_PATTERNS: RouteShape[] = [
{pattern_lines}
]
export const LAYOUTS: LayoutShape[] = [
{layout_lines}
]
export const routes = {{
{builder_lines}
}}
{router_block}",
        label = tree.label,
        layout_imports = layout_imports,
     )
}

fn generate_routes_one(spec: &RouteGenSpec) -> Result<usize> {

    let tree = build_route_tree(&spec.dir, &spec.label)?;
    // No-op when the source dir is absent: nothing to register, no
    // file to write. Specs with a missing `routes/` must be safe to
    // call repeatedly without creating empty artifacts.
    if !spec.dir.is_dir() {
        return Ok(0);
    }
    let body = render_routes_gen(&tree);
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
    Ok(tree.routes.len())
}
// files and emit TWO files: `views/routes.gen.ts` (router fields only)
// and `views/meta.gen.ts` (host fields only). The split is the
// contract — the router is host-agnostic; the host reads the meta.

/// Generate the view route + meta tables for a single views bucket dir
/// (`views/`, `.ship/views/`). Walks `<views_dir>/<name>/manifest.jsonc`
/// and writes `<views_dir>/routes.gen.ts` (router fields) +
/// `<views_dir>/meta.gen.ts` (host fields). No-op when the dir is absent
/// or holds no manifests. Returns the number of views discovered.
///
/// This is the framework-owned home for the view tables: the bundler's
/// `build_all` (the asset bake / `deck build-assets`) calls it for every
/// discovered view bucket, so a fresh clone + `build-assets` produces them
/// with no manual `deck routes --views` step. The bundler's watch path
/// ignores `.gen.ts`, so these writes never self-trigger a rebuild loop.
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

/// Per-spec view-artifact emit used by `generate_routes` (the `deck routes`
/// CLI path). Resolves the views dir relative to a `routes/` spec, then
/// delegates to `generate_view_artifacts`.
fn write_view_artifacts(spec: &RouteGenSpec) -> Result<()> {
    // Sibling of `routes/`: `<root>/../views/` (or `<root>/views/` if
    // `routes/` is a peer dir, e.g. `apps/shell/routes/…` →
    // `apps/shell/views/…`). Try the parent first; fall back to the
    // dir itself if it has a `views/` child.
    let candidates = [
        spec.dir.parent().map(|p| p.join("views")),
        Some(spec.dir.join("views")),
    ];
    let views_dir = candidates
        .into_iter()
        .flatten()
        .find(|p| p.is_dir());
    let Some(views_dir) = views_dir else {
        return Ok(());
    };
    generate_view_artifacts(&views_dir, &spec.label)?;
    Ok(())
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

    fn shell_route_discovery() {
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        for d in ["a/b/[id]/c", "a/d/[name]", "a/e"] {
            std::fs::create_dir_all(r.join(d)).unwrap();
            std::fs::write(r.join(d).join("page.sui"), "<div/>").unwrap();
        }
        let tree = build_route_tree(&r, "app").unwrap();
        // Static routes are NOT filtered out: a static mount (`/a/e`, no
        // `[param]` segment) must still land in the table, not be dropped.
        let patterns: Vec<&str> = tree.routes.iter().map(|r| r.pattern.as_str()).collect();
        assert_eq!(patterns, vec!["/a/b/:id/c", "/a/d/:name", "/a/e"]);
        // The static route has no params.
        let stat = tree.routes.iter().find(|r| r.pattern == "/a/e").unwrap();
        assert!(stat.params.is_empty());
        // Builders are namespaced by the top-level segment.
        let builders: Vec<&str> = tree.routes.iter().map(|r| r.builder.as_str()).collect();
        assert_eq!(builders, vec!["aBIdC", "aDName", "aE"]);
    }

    #[test]
    fn multi_root_emits_one_file_per_spec() {
        // Two roots in the same tempdir, each with a different mountable
        // view. The codegen must walk both and write one file per spec
        // with the right data. Asserts the DISPATCH behavior; the
        // per-spec data correctness is covered by `shell_route_discovery`.
        let root = tempfile::tempdir().unwrap();
        let one_routes = root.path().join("apps/one/routes");
        let two_routes = root.path().join("apps/two/routes");
        std::fs::create_dir_all(one_routes.join("x/[id]")).unwrap();
        std::fs::write(one_routes.join("x/[id]/page.sui"), "<div/>").unwrap();
        std::fs::create_dir_all(two_routes.join("[id]")).unwrap();
        std::fs::write(two_routes.join("[id]/page.sui"), "<div/>").unwrap();
        let specs = vec![
            RouteGenSpec {
                dir: one_routes,
                out: root.path().join("apps/one/routes.gen.ts"),
                label: "one".to_string(),
            },
            RouteGenSpec {
                dir: two_routes,
                out: root.path().join("apps/two/routes.gen.ts"),
                label: "two".to_string(),
            },
        ];
        let n = generate_routes(&specs).unwrap();
        assert_eq!(n, 2, "one dynamic route per root, total two");
        // Both files were written. The contents are the data table —
        // not the format, which is render_routes_gen's contract.
        let one = build_route_tree(&specs[0].dir, "one").unwrap();
        let two = build_route_tree(&specs[1].dir, "two").unwrap();
        assert_eq!(one.routes[0].pattern, "/x/:id");
        assert_eq!(two.routes[0].pattern, "/:id");
        assert_eq!(one.label, "one");
        assert_eq!(two.label, "two");
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

    #[test]
    fn layout_discovery_emits_layouts_block() {
        // A `routes/foo/_layout.sui` becomes a LAYOUTS entry with
        // `pathPrefix: '/foo'` and the tag read from the file.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("foo/bar/[id]")).unwrap();
        std::fs::write(r.join("foo/bar/[id]/page.sui"), "<div/>").unwrap();
        // Realistic layout: comment + a `<sui-*` wrapper + a `<slot>`.
        std::fs::write(
            r.join("foo/_layout.sui"),
            "// chrome\n<sui-foo-chrome>\n  <slot />\n</sui-foo-chrome>\n",
        )
        .unwrap();
        let tree = build_route_tree(&r, "app").unwrap();
        assert_eq!(tree.layouts.len(), 1);
        assert_eq!(tree.layouts[0].path_prefix, "/foo");
        assert_eq!(tree.layouts[0].tag, "sui-foo-chrome");
    }

    #[test]
    fn layout_discovery_ship_prefixed_tag() {
        // `<ship-*` is also accepted (chrome wrappers under @ship/* packs).
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("baz")).unwrap();
        std::fs::write(r.join("baz/page.sui"), "<div/>").unwrap();
        std::fs::write(r.join("baz/_layout.sui"), "<ship-baz-chrome/>\n").unwrap();
        let tree = build_route_tree(&r, "app").unwrap();
        assert_eq!(tree.layouts[0].tag, "ship-baz-chrome");
        assert_eq!(tree.layouts[0].path_prefix, "/baz");
    }

    #[test]
    fn layout_discovery_root_layout_has_slash_prefix() {
        // A `routes/_layout.sui` (sibling of `routes/page.sui`) becomes a
        // LAYOUTS entry with `pathPrefix: '/'` — the runtime's longest-
        // prefix match treats `/` as "every pathname" (see
        // `packages/deck/route/src/layouts.ts::matchPrefix`). The root
        // layout covers the index route AND every nested route (the
        // longest-prefix match keeps nested layouts outermost-in-chain).
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        // Index route + a nested subdir with its OWN layout (the
        // subdir layout must coexist with the root layout).
        std::fs::create_dir_all(r.join("studio/components/[name]")).unwrap();
        std::fs::write(r.join("page.sui"), "<div/>").unwrap();
        std::fs::write(
            r.join("studio/components/[name]/page.sui"),
            "<div/>",
        )
        .unwrap();
        std::fs::write(r.join("studio/_layout.sui"), "<sui-studio-chrome/>\n").unwrap();
        std::fs::write(
            r.join("_layout.sui"),
            "<ship-hub-chrome>\n  <slot name=\"route\"/>\n</ship-hub-chrome>\n",
        )
        .unwrap();
        let tree = build_route_tree(&r, "hub").unwrap();
        // Both layouts discovered, sorted by path_prefix (`/` before
        // `/studio`).
        assert_eq!(tree.layouts.len(), 2);
        assert_eq!(tree.layouts[0].path_prefix, "/");
        assert_eq!(tree.layouts[0].tag, "ship-hub-chrome");
        assert_eq!(tree.layouts[1].path_prefix, "/studio");
        assert_eq!(tree.layouts[1].tag, "sui-studio-chrome");
    }

    #[test]
    fn layout_discovery_root_layout_only_when_no_subdir_layouts() {
        // A sole `routes/_layout.sui` → a single LAYOUTS entry with
        // prefix `/`. The /studio case above is a richer version of
        // this; this one isolates the root-only path.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(&r).unwrap();
        std::fs::write(r.join("page.sui"), "<div/>").unwrap();
        std::fs::write(r.join("_layout.sui"), "<ship-app-chrome/>\n").unwrap();
        let tree = build_route_tree(&r, "hub").unwrap();
        assert_eq!(tree.layouts.len(), 1);
        assert_eq!(tree.layouts[0].path_prefix, "/");
        assert_eq!(tree.layouts[0].tag, "ship-app-chrome");
    }

    #[test]
    fn layout_discovery_skips_div_only_layout() {
        // A `_layout.sui` with no `<sui-*` / `<ship-*` element errors out
        // — a layout must name a wrapper component.
        let root = tempfile::tempdir().unwrap();
        let r = root.path().join("routes");
        std::fs::create_dir_all(r.join("home")).unwrap();
        std::fs::write(r.join("home/page.sui"), "<div/>").unwrap();
        std::fs::write(r.join("home/_layout.sui"), "<div>no wrapper here</div>\n").unwrap();
        let err = build_route_tree(&r, "shell").unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no `<sui-…>` or `<ship-…>` element"),
            "expected layout-tag error, got: {msg}"
        );
    }

    #[test]
    fn module_import_path_inverts_to_route_segment_mapping() {
        // Pure logic, generic inputs — no app routes. `:param` ↔ `[param]` dir.
        assert_eq!(module_import_path("/"), "./routes/page.sui");
        assert_eq!(module_import_path("/x"), "./routes/x/page.sui");
        assert_eq!(module_import_path("/x/:id"), "./routes/x/[id]/page.sui");
        assert_eq!(module_import_path("/a/:b/c/:d"), "./routes/a/[b]/c/[d]/page.sui");
    }

    #[test]
    fn layout_module_path_inverts_to_layout_file() {
        // Pure logic, mirrors `module_import_path` for layouts. The
        // codegen statically imports each layout's `.sui` so the
        // chrome wrapper it composes registers at first paint; the
        // generated path is the on-disk layout relative to the
        // entry (the entry lives next to `routes/`).
        //   `/`        → `./routes/_layout.sui` (the root layout, sibling
        //                  of `routes/page.sui`)
        //   `/studio`  → `./routes/studio/_layout.sui` (the subdir layout)
        assert_eq!(layout_module_path("/"), "./routes/_layout.sui");
        assert_eq!(layout_module_path("/studio"), "./routes/studio/_layout.sui");
        assert_eq!(layout_module_path("/a/b"), "./routes/a/b/_layout.sui");
    }

    #[test]
    fn render_wires_every_route_into_the_router() {
        // Contract, not a text snapshot, and built from a GENERIC tree so a
        // framework test never names an app's routes. The entry must wire the
        // framework router and place EVERY route in the module map keyed by its
        // own pattern; the retired v0 hand-rolled mount must not reappear.
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![
                Route { pattern: "/".to_string(), params: vec![], builder: "index".to_string() },
                Route {
                    pattern: "/a/:id".to_string(),
                    params: vec!["id".to_string()],
                    builder: "aId".to_string(),
                },
            ],
            layouts: vec![],
        };
        let ts = render_routes_gen(&tree);
        assert!(ts.contains("mountRouter("), "entry wires the framework router");
        for r in &tree.routes {
            assert!(
                ts.contains(&format!("'{}':", r.pattern)),
                "every route is in the module map; `{}` is missing",
                r.pattern,
            );
        }
        assert!(!ts.contains("getElementById"), "the v0 hand-rolled index mount is gone");
    }

    #[test]
    fn render_emits_layout_imports_for_chrome_registration() {
        // Each layout's `.sui` is statically imported alongside the
        // route views so the chrome wrapper it composes (its first
        // `<sui-*` / `<ship-*` element) registers at first paint.
        // The runtime creates the chrome via
        // `document.createElement(layout.tag)`; an unregistered tag
        // is a no-op and the named-slot projection is empty. The
        // shell's `apps/shell/src/components/index.ts` is the
        // host-side fix for the same gap; the framework does it
        // here so every per-app host gets the registration for
        // free.
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![
                Route { pattern: "/".to_string(), params: vec![], builder: "index".to_string() },
            ],
            layouts: vec![
                Layout { path_prefix: "/".to_string(), tag: "ship-app-chrome".to_string() },
                Layout { path_prefix: "/studio".to_string(), tag: "sui-studio-chrome".to_string() },
            ],
        };
        let ts = render_routes_gen(&tree);
        assert!(
            ts.contains("import './routes/_layout.sui';"),
            "root layout must be statically imported: {ts}",
        );
        assert!(
            ts.contains("import './routes/studio/_layout.sui';"),
            "subdir layout must be statically imported: {ts}",
        );
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

    /// The generated `routes.gen.ts` entry imports `@ship/tokens/tokens.css`
    /// so the design tokens ride with every per-app build via Rolldown's
    /// CSS-asset path. The token CSS package is the canonical home
    /// (`packages/sui/tokens/package.json` declares `./tokens.css` as a
    /// subpath export); the import here is the convention. A rename on
    /// either side breaks this test, surfacing the drift.
    #[test]
    fn ssot_tokens_import_pinned() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let tokens_pkg = manifest_dir
            .parent() // crates/deck
            .and_then(|p| p.parent()) // crates
            .and_then(|p| p.parent()) // repo root
            .expect("CARGO_MANIFEST_DIR has three ancestors")
            .join("packages/sui/tokens/package.json");
        let pkg = std::fs::read_to_string(&tokens_pkg)
            .unwrap_or_else(|e| panic!("read {}: {e}", tokens_pkg.display()));
        assert!(
            pkg.contains("\"./tokens.css\": \"./src/tokens.css\""),
            "tokens package.json must expose the token CSS subpath `\
             ./tokens.css → ./src/tokens.css` (got: {pkg})",
        );

        // The codegen must render the import verbatim — no runtime link
        // injection, no daemon-served path. A change in the entry contract
        // (e.g. dropping the token import) breaks the per-app build gate
        // and surfaces here.
        let tree = RouteTree {
            label: "x".to_string(),
            routes: vec![Route {
                pattern: "/".to_string(),
                params: vec![],
                builder: "index".to_string(),
            }],
            layouts: vec![],
        };
        let ts = render_routes_gen(&tree);
        assert!(
            ts.contains("import '@ship/tokens/tokens.css';"),
            "rendered routes.gen.ts must import @ship/tokens/tokens.css (got: {ts})",
        );
        // And the old daemon-served link must be gone — it's a static-only
        // deploy hazard and the SSOT literal this used to pin (`<DECK_API_ASSETS>
        // /view-tokens.css`) is now exclusively deck-core's concern.
        assert!(
            !ts.contains("view-tokens.css"),
            "rendered routes.gen.ts must NOT reference the daemon-served view-tokens.css path (got: {ts})",
        );
    }

    /// `generate_view_artifacts` is the framework-owned view-table codegen
    /// `build_all` calls per bucket. Given a `views/` bucket with manifests, it
    /// must emit BOTH `routes.gen.ts` (router fields: pattern + view) and
    /// `meta.gen.ts` (host fields: title/layout/auth/rail) — the host-agnostic
    /// split is the contract. JSONC comments in the manifest are tolerated.
    #[test]
    fn generate_view_artifacts_emits_routes_and_meta() {
        let root = tempfile::tempdir().unwrap();
        let views = root.path().join("views");
        std::fs::create_dir_all(views.join("git")).unwrap();
        std::fs::write(
            views.join("git/manifest.jsonc"),
            "// a comment\n{ \"name\": \"git\", \"path\": \"/git\", \"title\": \"Git\", \
             \"layout\": \"default\", \"auth\": \"user\", \
             \"rail\": { \"label\": \"Git\", \"icon\": \"git-branch\" } }\n",
        )
        .unwrap();
        std::fs::create_dir_all(views.join("eval")).unwrap();
        std::fs::write(
            views.join("eval/manifest.jsonc"),
            "{ \"name\": \"eval\", \"path\": \"/eval\" }\n",
        )
        .unwrap();

        let n = generate_view_artifacts(&views, "views").unwrap();
        assert_eq!(n, 2, "two manifests discovered");

        // routes.gen.ts: router contract (pattern + view name). The host table
        // is a SEPARATE file — the router export must not appear here.
        let routes = std::fs::read_to_string(views.join("routes.gen.ts")).unwrap();
        assert!(routes.contains("VIEW_ROUTES"), "router table present");
        assert!(routes.contains("pattern: '/git', view: 'git'"), "git route entry");
        assert!(routes.contains("pattern: '/eval', view: 'eval'"), "eval route entry");
        assert!(!routes.contains("VIEW_META"), "host table stays out of the router file");

        // meta.gen.ts: host contract (title/layout/auth/rail). The router table
        // export must not appear here — the split is the contract.
        let meta = std::fs::read_to_string(views.join("meta.gen.ts")).unwrap();
        assert!(meta.contains("VIEW_META"), "host table present");
        assert!(meta.contains("name: 'git'") && meta.contains("title: 'Git'"), "git meta");
        assert!(meta.contains("\"icon\":\"git-branch\""), "rail rides through to meta");
        assert!(!meta.contains("VIEW_ROUTES"), "router table stays out of the meta file");
    }

    /// No-op safety: an absent bucket dir and a present-but-empty bucket both
    /// return 0 and write nothing. `build_all` calls this for EVERY configured
    /// bucket (`views`, `.ship/views`); a host without a `.ship/views` must not
    /// get an empty artifact written there.
    #[test]
    fn generate_view_artifacts_noop_without_dir_or_manifests() {
        let root = tempfile::tempdir().unwrap();
        // Absent dir → 0, nothing written.
        let absent = root.path().join("ship-views");
        assert_eq!(generate_view_artifacts(&absent, "x").unwrap(), 0);
        assert!(!absent.exists());
        // Present but no manifests → 0, no artifacts.
        let empty = root.path().join("views");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(generate_view_artifacts(&empty, "x").unwrap(), 0);
        assert!(!empty.join("routes.gen.ts").exists());
        assert!(!empty.join("meta.gen.ts").exists());
    }
}
