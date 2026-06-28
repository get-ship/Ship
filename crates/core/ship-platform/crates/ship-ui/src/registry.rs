//! `ComponentResolver` — the read-side convention for component sources.
//!
//! One trait, one job: given a `tag` (a `.sui` stem) or a `path` (a relative
//! path inside a pack), return the source. The wire (`crate::wire::UiWire`)
//! asks the resolver; the wire does NOT know where components live. shipd
//! plugs in the concrete resolver at mount time from its host config.
//!
//! The convention is that the resolver is a CHAIN: workspace → per-space
//! `.ship/views/` → a future shop registry → a future CDN. Order matters
//! (first match wins). The wire treats the chain as opaque — it never
//! reads `composed_root_for`, never constructs an npm path, never
//! hardcodes `packages/components/`. The host decides the order.
//!
//! Multiple roots: `roots: Vec<Root>` is the configuration. Each root
//! declares how to find a `.sui` for a given tag or path. The default
//! `Root::Workspace { dir }` walks the root `package.json` workspaces
//! declaration (NO hardcoded `packages/` path) and enumerates every
//! package's `.sui` source files. `Root::ViewsDir { dir }` walks
//! `<dir>/**/page.sui` for per-space views.
//!
//! Future roots: `Root::External(...)` is a host-supplied closure
//! (ship-store, CDN, etc.). The wire is agnostic to the transport.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use ship_ops_rpc::UiPack;

// ── Public types ────────────────────────────────────────────────────────────

/// One physical location a component can be resolved against.
///
/// The wire's `Packs` and `Read` RPCs go through the resolver chain in
/// declaration order; first root that knows the tag wins. The chain is
/// host-configured — the wire is generic.
#[derive(Clone)]
pub enum Root {
    /// A workspace-discoverable source root. The resolver walks every
    /// package under `<dir>/` whose `package.json` declares the
    /// workspace; the algorithm is the same one deck-bundler uses
    /// (root `package.json` `workspaces.packages` glob, every package
    /// contributes its `.sui` source). NO hardcoded `packages/` path.
    Workspace { dir: PathBuf },
    /// Per-space views (`<space>/.ship/views/**/page.sui`). The host
    /// already knows this directory marker — see the
    /// `detection_accepts_dot_ship_views_marker` function in
    /// sui's deck-core (post-carve; the deck-core crate moved to the
    /// external sui repo).
    ViewsDir { dir: PathBuf },
    /// Future ship-store / npm-equivalent resolver. Carries a
    /// closure the host has implemented; the wire is agnostic to
    /// the transport. Returns `None` if the tag isn't found.
    External(Arc<dyn ExternalResolver>),
}

/// A pluggable external resolver. Async because registries are
/// network. Shipd ships the concrete impls; the wire consumes the
/// trait only.
pub trait ExternalResolver: Send + Sync {
    fn resolve(
        &self,
        tag: &str,
    ) -> Pin<Box<dyn Future<Output = Option<ComponentLocation>> + Send + '_>>;
}

/// The answer to a `Read` / `Packs` lookup.
#[derive(Debug, Clone)]
pub struct ComponentLocation {
    /// Absolute path on disk when the resolver found it locally.
    /// `None` for external (shop / CDN) resolvers.
    pub abs_path: Option<PathBuf>,
    /// The `.sui` source text. Always present.
    pub source: String,
    /// Seconds since UNIX_EPOCH of last modification, for `Read::mtime`.
    pub mtime: String,
    /// A short tag the wire echoes back in `Read::resolved_at`. Format:
    /// `workspace://<package>`, `views://<rel>`, `shop://<package>`,
    /// `cdn://<url>`. The wire doesn't parse this — it's for
    /// diagnostics.
    pub origin: String,
}

/// The component registry contract. The wire holds one; shipd's mount
/// composes the chain from its host config.
pub trait ComponentResolver: Send + Sync {
    /// Enumerate the catalog for `Packs`. Every root contributes.
    /// Order is the host's; the wire filters in place.
    fn packs(
        &self,
        query: PacksQuery,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<UiPack>, String>> + Send + '_>>;

    /// Resolve one source for `Read`. First root that knows the path
    /// wins. The wire never composes paths from `tag`; the resolver
    /// decides whether `tag` maps to a `path` for each root.
    fn read(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = Result<ComponentLocation, String>> + Send + '_>>;
}

/// Query parameters for `Packs` — same shape the wire already accepts.
/// Owned (not borrowed) so the future returned from `packs` doesn't
/// capture a lifetime from the caller.
#[derive(Debug, Default, Clone)]
pub struct PacksQuery {
    pub category:     String,
    pub tag_contains: String,
}
// ── Default chain implementation ─────────────────────────────────────────────

/// The default resolver chain: every root tries; first match wins.
///
/// Constructed by `shipd` at mount time from the host's `ship.jsonc`
/// (or whatever config surface the host uses). The wire never sees
/// the construction — it just gets the `Arc<dyn ComponentResolver>`.
pub struct ChainResolver {
    roots: Vec<Root>,
}

impl ChainResolver {
    pub fn new(roots: Vec<Root>) -> Self {
        Self { roots }
    }
}

impl ComponentResolver for ChainResolver {
    fn packs(
        &self,
        query: PacksQuery,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<UiPack>, String>> + Send + '_>> {
        let roots = self.roots.clone();
        Box::pin(async move {
            let mut merged: Vec<UiPack> = Vec::new();
            for root in &roots {
                let from_root: Vec<UiPack> = match root {
                    Root::Workspace { dir } => catalog_from_workspace(dir.as_path())?,
                    Root::ViewsDir { dir } => catalog_from_views_dir(dir.as_path())?,
                    Root::External(_) => Vec::new(),
                };
                merged.extend(from_root);
            }
            Ok(apply_query_filter(merged, &query))
        })
    }

    fn read(
        &self,
        path: String,
    ) -> Pin<Box<dyn Future<Output = Result<ComponentLocation, String>> + Send + '_>> {
        let roots = self.roots.clone();
        Box::pin(async move {
            for root in &roots {
                let result: Option<ComponentLocation> = match root {
                    Root::Workspace { dir } => read_from_workspace(dir.as_path(), &path).await,
                    Root::ViewsDir { dir } => read_from_views_dir(dir.as_path(), &path).await,
                    Root::External(r) => r.resolve(&path).await,
                };
                if let Some(loc) = result {
                    return Ok(loc);
                }
            }
            Err(format!("component not found: {path}"))
        })
    }
}

// ── Workspace discovery (inlined; no deck-bundler dep) ────────────────────────

/// One package discovered in a workspace.
#[derive(Debug, Clone)]
struct WorkspacePackage {
    name: String,
    root: PathBuf,
}

/// Walk `<dir>/package.json` `workspaces.packages` globs and return
/// every package with a parseable manifest carrying a `name`. Mirrors
/// the algorithm in the deck-bundler's catalog/workspace.rs (the
/// the deck-bundler crate moved to the external sui repo post-carve) —
/// duplicated here so the resolver chain doesn't have to depend on
/// the bundler (the bundler depends on `ship-ui`, not the other way
/// around). The two implementations must stay in sync.
fn discover_workspace_packages(workspace_dir: &Path) -> Result<Vec<WorkspacePackage>, String> {
    let manifest = read_json(&workspace_dir.join("package.json"))
        .ok_or_else(|| format!("no package.json at {}", workspace_dir.display()))?;
    let Some(workspaces) = manifest.get("workspaces").and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };
    let Some(packages) = workspaces.get("packages").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in packages {
        let Some(glob) = entry.as_str() else { continue };
        for dir in expand_glob(workspace_dir, glob) {
            if !seen.insert(dir.clone()) {
                continue;
            }
            let Some(pkg_manifest) = read_json(&dir.join("package.json")) else {
                continue;
            };
            let Some(name) = pkg_manifest.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            out.push(WorkspacePackage {
                name: name.to_string(),
                root: dir,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn expand_glob(workspace_dir: &Path, glob: &str) -> Vec<PathBuf> {
    // Support the common npm convention: `<dir>` or `<dir>/*`. Strip
    // any trailing `/*`. We don't support `?` / `**` etc. — the
    // workspace globs in this repo are simple.
    let pattern = glob.trim_end_matches("/*").trim_end_matches('/');
    let parent_rel = pattern;
    let parent_abs = workspace_dir.join(parent_rel);
    let Ok(entries) = std::fs::read_dir(&parent_abs) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect()
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

/// Recursively walk `<pkg>/src/` for `.sui` files whose stem contains
/// a hyphen and is not "page" or "index" — the HTML custom-element
/// tag-name rule. Same guard as the `scan_sui_components` function
/// in the deck-bundler's catalog/workspace.rs (post-carve).
fn source_tags_for_package(pkg_root: &Path) -> Vec<(String, PathBuf)> {
    let src = pkg_root.join("src");
    if !src.is_dir() {
        return Vec::new();
    }
    let mut out = Vec::new();
    scan_sui_components(&src, &mut out);
    out
}

fn scan_sui_components(dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    const SKIP: [&str; 5] = ["node_modules", "dist", "target", ".git", ".deck"];
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if SKIP.contains(&name.as_str()) {
            continue;
        }
        let p = e.path();
        if p.is_dir() {
            scan_sui_components(&p, out);
        } else if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            if p.extension().and_then(|x| x.to_str()) != Some("sui") {
                continue;
            }
            if stem.contains('-') && stem != "page" && stem != "index" {
                out.push((stem.to_string(), p));
            }
        }
    }
}

// ── Per-root catalog / read ──────────────────────────────────────────────────

fn catalog_from_workspace(workspace_dir: &Path) -> Result<Vec<UiPack>, String> {
    let packages = discover_workspace_packages(workspace_dir)?;
    let mut packs: std::collections::BTreeMap<String, UiPack> = std::collections::BTreeMap::new();
    for pkg in &packages {
        let entries = source_tags_for_package(&pkg.root);
        if entries.is_empty() {
            continue;
        }
        // Group entries by npm package name. The pack id is the
        // package name (e.g. `@sui/primitives`, `@ship/agents`) — no
        // fabrication. First-seen tag wins (insertion order); the
        // workspace discovery sorts by name so the order is stable.
        let pack = packs.entry(pkg.name.clone()).or_insert_with(|| UiPack {
            id: pkg.name.clone(),
            category: pack_category_from_name(&pkg.name).to_string(),
            title: String::new(),
            components: Vec::new(),
            __buffa_unknown_fields: Default::default(),
        });
        for (tag, abs) in entries {
            let rel = rel_unix(workspace_dir, &abs);
            pack.components.push(ship_ops_rpc::UiComponent {
                tag,
                file: rel,
                title: String::new(),
                props: Vec::new(),
                slots: Vec::new(),
                events: Vec::new(),
                __buffa_unknown_fields: Default::default(),
            });
        }
    }
    Ok(packs.into_values().collect())
}

fn catalog_from_views_dir(views_dir: &Path) -> Result<Vec<UiPack>, String> {
    let mut packs: std::collections::BTreeMap<String, UiPack> = std::collections::BTreeMap::new();
    if !views_dir.is_dir() {
        return Ok(Vec::new());
    }
    walk_views(views_dir, views_dir, &mut packs)?;
    Ok(packs.into_values().collect())
}

fn walk_views(
    root: &Path,
    dir: &Path,
    packs: &mut std::collections::BTreeMap<String, UiPack>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let ty = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        if ty.is_dir() {
            walk_views(root, &path, packs)?;
        } else if ty.is_file() && path.file_name().and_then(|s| s.to_str()) == Some("page.sui") {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let space = rel
                .trim_end_matches("/page.sui")
                .trim_end_matches("page.sui")
                .trim_end_matches('/')
                .to_string();
            let id = if space.is_empty() {
                "@view/default".to_string()
            } else {
                format!("@view/{space}")
            };
            let pack = packs.entry(id.clone()).or_insert_with(|| UiPack {
                id,
                category: "view".to_string(),
                title: space.clone(),
                components: Vec::new(),
                __buffa_unknown_fields: Default::default(),
            });
            pack.components.push(ship_ops_rpc::UiComponent {
                tag: "page".to_string(),
                file: rel,
                title: String::new(),
                props: Vec::new(),
                slots: Vec::new(),
                events: Vec::new(),
                __buffa_unknown_fields: Default::default(),
            });
        }
    }
    Ok(())
}

async fn read_from_workspace(workspace_dir: &Path, path: &str) -> Option<ComponentLocation> {
    let packages = discover_workspace_packages(workspace_dir).ok()?;
    for pkg in &packages {
        let entries = source_tags_for_package(&pkg.root);
        for (tag, abs) in entries {
            let rel = rel_unix(workspace_dir, &abs);
            if rel == path {
                let (source, mtime) = read_file_sync(&abs).await?;
                return Some(ComponentLocation {
                    abs_path: Some(abs),
                    source,
                    mtime,
                    origin: format!("workspace://{}/{}", pkg.name, tag),
                });
            }
        }
    }
    None
}

async fn read_from_views_dir(views_dir: &Path, path: &str) -> Option<ComponentLocation> {
    let abs = views_dir.join(path);
    if !abs.is_file() {
        return None;
    }
    let (source, mtime) = read_file_sync(&abs).await?;
    Some(ComponentLocation {
        abs_path: Some(abs),
        source,
        mtime,
        origin: format!("views://{path}"),
    })
}

async fn read_file_sync(abs: &Path) -> Option<(String, String)> {
    let abs = abs.to_path_buf();
    tokio::task::spawn_blocking(move || -> std::io::Result<(String, String)> {
        let source = std::fs::read_to_string(&abs)?;
        let mtime = std::fs::metadata(&abs)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| format!("{}", d.as_secs()))
            .unwrap_or_default();
        Ok((source, mtime))
    })
    .await
    .ok()
    .and_then(|r| r.ok())
}

fn pack_category_from_name(name: &str) -> &'static str {
    // `@sui/<pack>` → `sui`; `@ship/<pack>` → `ship`; anything else →
    // `user`. The convention matches the design bundle. Tags from
    // non-namespaced packages (e.g. a third-party monorepo) go to
    // `user` — the host can refine via a `category_map` later.
    if name.starts_with("@sui/") { "sui" }
    else if name.starts_with("@ship/") { "ship" }
    else { "user" }
}

fn rel_unix(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn apply_query_filter(packs: Vec<UiPack>, query: &PacksQuery) -> Vec<UiPack> {
    packs
        .into_iter()
        .filter(|p| query.category.is_empty() || p.category == query.category)
        .map(|mut p| {
            p.components
                .retain(|c| query.tag_contains.is_empty() || c.tag.contains(&query.tag_contains));
            p
        })
        .filter(|p| !p.components.is_empty())
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn workspace_root_discovers_packaged_component() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("package.json"),
            r#"{"workspaces":{"packages":["packages/*"]}}"#,
        );
        write(
            &root.join("packages/primitives/package.json"),
            r#"{"name":"@sui/primitives"}"#,
        );
        write(&root.join("packages/primitives/src/sui-button.sui"), "<sui-button></sui-button>\n");
        // Views (page.sui) must NOT leak into the catalog.
        write(&root.join("packages/primitives/src/page.sui"), "<!-- view, skip -->");

        let packs = catalog_from_workspace(root).unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "@sui/primitives");
        assert_eq!(packs[0].category, "sui");
        assert_eq!(packs[0].components.len(), 1);
        assert_eq!(packs[0].components[0].tag, "sui-button");
        assert_eq!(packs[0].components[0].file, "packages/primitives/src/sui-button.sui");
    }

    #[test]
    fn views_root_emits_one_pack_per_page() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(&root.join("git/page.sui"), "<!-- git view -->");
        write(&root.join("git/sub/page.sui"), "<!-- nested view -->");

        let packs = catalog_from_views_dir(root).unwrap();
        assert_eq!(packs.len(), 2);
        let ids: Vec<_> = packs.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"@view/git"));
        assert!(ids.contains(&"@view/git/sub"));
        for p in &packs {
            assert_eq!(p.category, "view");
            assert_eq!(p.components.len(), 1);
            assert_eq!(p.components[0].tag, "page");
        }
    }

    #[test]
    fn chain_resolver_picks_first_root_that_owns() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("package.json"),
            r#"{"workspaces":{"packages":["packages/*"]}}"#,
        );
        write(
            &root.join("packages/p/package.json"),
            r#"{"name":"@sui/p"}"#,
        );
        write(&root.join("packages/p/src/sui-x.sui"), "<!-- x -->");
        write(&root.join("views/page.sui"), "<!-- view -->");

        let resolver = ChainResolver::new(vec![
            Root::Workspace { dir: root.to_path_buf() },
            Root::ViewsDir { dir: root.join("views") },
        ]);
        // Read a workspace component.
        let loc = tokio::runtime::Runtime::new().unwrap().block_on(resolver.read(
            "packages/p/src/sui-x.sui".to_string(),
        ))
        .unwrap();
        assert!(loc.abs_path.is_some());
        assert!(loc.origin.starts_with("workspace://"));
        // Read a view component.
        let loc = tokio::runtime::Runtime::new().unwrap().block_on(resolver.read("page.sui".to_string())).unwrap();
        assert!(loc.origin.starts_with("views://"));
        // Missing → error.
        assert!(tokio::runtime::Runtime::new().unwrap().block_on(resolver.read("nope.sui".to_string())).is_err());
    }

    #[test]
    fn category_filter_drops_other_packs() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            &root.join("package.json"),
            r#"{"workspaces":{"packages":["p/*"]}}"#,
        );
        write(&root.join("p/a/package.json"), r#"{"name":"@sui/a"}"#);
        write(&root.join("p/b/package.json"), r#"{"name":"@ship/b"}"#);
        write(&root.join("p/a/src/sui-a.sui"), "<!-- a -->");
        write(&root.join("p/b/src/ship-b.sui"), "<!-- b -->");

        let resolver = ChainResolver::new(vec![Root::Workspace { dir: root.to_path_buf() }]);
        let packs = tokio::runtime::Runtime::new().unwrap().block_on(resolver.packs(PacksQuery {
            category: "sui".to_string(),
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "@sui/a");
    }
}
