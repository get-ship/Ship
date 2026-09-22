# Handoff: Ship "Views" applet layer → `ship-platform/crates/ship-ui`

**Source:** excised from `sui` deck (`job/views-decouple`, mission
m-deck-standalone) on 2026-07-18.
**Target repo:** the **ship** repo (Rust), `crates/core/ship-platform/crates/ship-ui`.
**NOT** this `sui` repo, **NOT** ship-apps (JS-only). Do not commit this into ship
from here — this doc is the cross-repo handoff so the port isn't lost.

## Why this was ripped out of deck

"Views" is a **Ship product feature**: agent/user-authored applets rendered in a
tree inside a deck app. They are stripped-down apps with no own deck config; the
host metadata is `auth` / `rail` / `layout` / `title` — all Ship-shell concepts.
This had leaked into **deck**, the generic framework. Deck must have ZERO typed
knowledge of `auth`/`rail`/`layout`/`title` or of the `manifest.jsonc` schema.

What deck KEEPS (generic capability, renamed `views`→`routes`):
- Directory-tree route discovery (a route = a dir with `page.sail`; `[param]`
  dirs are dynamics) → the host-agnostic `routes.gen.ts` (URL pattern → resource
  name only, no metadata).
- Multi-entry bundling + cross-entry shared-chunk dedup.
- The runtime resolver that serves `<bucket>/<name>/<leaf>` by URL.

What Ship must RE-HOME here (this handoff): the **applet manifest layer** — the
`manifest.jsonc` schema (`auth`/`rail`/`layout`/`title`), its discovery
semantics, and the split emission into a **router table** (`routes.gen.ts`,
pattern + resource name) plus a **host-metadata table** (`meta.gen.ts`,
title/layout/auth/rail).

## The seam to re-integrate through

Ship re-integrates via the proper seam later ("we'll get there when we get
there"). The intended shape:

- deck emits the generic, host-agnostic route table for a directory tree.
- ship-ui owns the **applet manifest** layer: walk `<bucket>/<name>/manifest.jsonc`,
  parse the Ship metadata, and emit the host-metadata table (`meta.gen.ts`) that
  the Ship shell reads to choose chrome (`layout`), gate access (`auth`), and
  build the launcher `rail`.
- If deck ever needs to carry per-route metadata, it does so **opaquely**
  (copies unknown fields through, interprets none) — never with a typed
  `auth`/`rail`/`layout` schema.

## Extracted code (verbatim, from `deck-bundler/src/routes.rs`)

The `views/` bucket discovery, the `ViewManifest` schema, the JSONC strip, and
the two-file (`routes.gen.ts` router / `meta.gen.ts` host) split emission. Port
these into ship-ui, adapting the `RouteShape` / `ViewRoute` / `ViewMeta` imports
to ship-ui's own module names.

### `ViewManifest` — the Ship applet manifest schema

```rust
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
```

### `strip_jsonc` — tolerate `//` line comments in the manifest

```rust
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
```

### `discover_views` — walk `views/<name>/manifest.jsonc`

```rust
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
        out.push(ViewManifest { name: dir_name, path, title, layout, auth, rail });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}
```

### `write_views_routes` — the host-agnostic router table (`views/routes.gen.ts`)

Note: the emitted `routes.gen.ts` half is host-agnostic (pattern + resource name
only). If ship-ui wants deck to emit this, it can instead feed the discovered
patterns into deck's generic route-table emitter. Kept here for completeness.

```rust
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
// `views/<name>/manifest.jsonc`). ...
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
```

### `write_views_meta` — the Ship host-metadata table (`views/meta.gen.ts`)

**This is the core Ship leak** — the `title`/`layout`/`auth`/`rail` emission.

```rust
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
// Host-only view metadata for the `{label}` view tree ...

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
```

### `generate_view_artifacts` + `write_view_artifacts` — the driver

```rust
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

/// Resolve the views dir relative to a `routes/` spec (sibling `views/`),
/// then delegate to `generate_view_artifacts`.
fn write_view_artifacts(spec: &RouteGenSpec) -> Result<()> {
    let candidates = [
        spec.dir.parent().map(|p| p.join("views")),
        Some(spec.dir.join("views")),
    ];
    let views_dir = candidates.into_iter().flatten().find(|p| p.is_dir());
    let Some(views_dir) = views_dir else { return Ok(()); };
    generate_view_artifacts(&views_dir, &spec.label)?;
    Ok(())
}
```

### `build_all` driver loop (from `deck-bundler/src/lib.rs`)

The per-bucket call that invoked the above during the asset bake:

```rust
// Step 0: regenerate the view route/meta tables for every discovered view
// bucket. Folded out of the dev-only Justfile `gen-routes` recipe so a fresh
// clone + `build-assets` produces them with no manual `deck routes --views` step.
for views_dir in layout.view_bucket_dirs() {
    let label = views_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "views".to_string());
    if let Err(e) = crate::routes::generate_view_artifacts(&views_dir, &label) {
        error!(bucket = %views_dir.display(), error = %e, "bundler: view route/meta gen failed");
        failed.push(format!("view-tables {} ({e})", views_dir.display()));
    }
}
```

### Scaffold (`deck-cli/src/scaffold.rs`) — the applet manifest generator

`deck scaffold view` wrote a `manifest.jsonc` with the Ship metadata. Port the
manifest-emitting shape (the applet scaffold) into ship-ui; deck's own scaffold
now writes a generic `routes/<name>/page.sail` with no manifest.

```rust
let manifest = format!(
    "// Route metadata for views/{name}/. See\n\
     // docs/ship-adrs/references/docs/view-directory-convention.md.\n\
     {{\n  \
     \"name\": \"{name}\",\n  \
     \"path\": \"{route}\",\n  \
     \"title\": \"{title}\",\n  \
     \"layout\": \"default\",\n  \
     \"auth\": \"public\"\n\
     }}\n"
);
fs::write(base.join("manifest.jsonc"), manifest)?;
// next-steps guidance that was Ship-specific:
//   "Edit manifest.jsonc — set rail/preload/events as needed"
//   "Reload the shell — views auto-mount via the daemon"
```

## Tests that encoded the Ship contract (port alongside)

From `deck-bundler/src/routes.rs` (removed): `generate_view_artifacts_emits_routes_and_meta`
(asserts the two-file split, `rail` passthrough into `meta.gen.ts`, the
`title`/`layout`/`auth` fields) and `generate_view_artifacts_noop_without_dir_or_manifests`
(absent/empty bucket → no artifacts). Both belong with the port.

## Port checklist for ship-ui

- [ ] Re-home `ViewManifest` + `strip_jsonc` + `discover_views` +
      `write_views_meta` (and the router-half `write_views_routes` if ship-ui
      wants deck to skip its generic emitter for applets).
- [ ] Re-home the `build_all` Step-0 per-bucket driver as a ship-ui build hook.
- [ ] Re-home the `deck scaffold view` manifest generator as a ship-ui applet
      scaffold.
- [ ] Wire ship-ui into deck through the opaque per-route-metadata seam (deck
      copies unknown manifest fields through, interprets none) OR a ship-ui-owned
      post-pass over deck's discovered route tree.
- [ ] Restore `docs/ship-adrs/references/docs/view-directory-convention.md`
      references (the manifest convention doc) under ship-ui's docs.
