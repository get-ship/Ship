//! Bundle target resolution for the Rust bundler (#163).
//!
//! `TARGETS` + `discoverViewBundles` + `assetsDirFor` for the bundler. A
//! `BundleTarget` names what to build, where its entry is, what stays external,
//! and where the output lands. The Rolldown build (`rolldown_bundle.rs`)
//! iterates these; nothing here touches the filesystem beyond discovery reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use deck_config::DeckConfig;

/// Deck's OWN runtime bundles — baked into the deck-core crate (like Next
/// shipping its own runtime). Everything else under `sui-`/`views/` is app/user
/// content and goes to `.deck/`. Chrome components are NOT here — the
/// per-tag tree-shaking resolver (the `sui-components/<name>` prefix in
/// `assets_dir_for` + the `@sui-tag/<tag>` virtual import in `sui_plugin.rs`)
/// is the canonical way chrome loads. There is no umbrella.
const DECK_OWN_BUNDLES: [&str; 3] = ["sui-runtime", "sui-dom", "sui-sdk-client"];

/// A target's entry: a real on-disk module, or a synthetic aggregate that
/// imports each `source` by absolute path (the build phase generates the entry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetEntry {
    Real(PathBuf),
    Virtual { sources: Vec<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleTarget {
    /// Logical name, e.g. `sui-runtime`, `sui-components/sui-rail`, `views/git/view`.
    pub name: String,
    pub entry: TargetEntry,
    /// Externals the author declared. The build phase additionally forces
    /// `@ship/sui-runtime` external for every target except `sui-runtime`.
    pub externals: Vec<String>,
    /// Output path relative to the target's assets dir (`assets_dir_for`).
    pub out_file: String,
    /// Lazy intent (from a consumer bundle's self-declaration). Only the
    /// optional dev-loop skip (`DECK_DEV_SKIP_LAZY=1`) reads it — it omits heavy
    /// lazy packs from a dev bake. Always `false` for framework + view targets.
    /// The default bake (env unset) builds lazy and eager identically, so this
    /// flag never changes production output.
    pub lazy: bool,
}

/// Repo + workspace layout. `repo_root` is the ship monorepo (framework +
/// built-in views); `source_roots` is `[repo_root]` in contributor mode or
/// `[repo_root, workspace_root]` when the daemon serves an external project.
pub struct BundleLayout {
    pub repo_root: PathBuf,
    pub source_roots: Vec<PathBuf>,
    /// Base dir output paths are computed under (the `crates/.../assets`, `.deck`,
    /// `apps/shipd/assets` subtrees hang off this). Defaults to `repo_root`;
    /// redirect for tests / out-of-tree builds so output never clobbers
    /// committed assets.
    pub out_root: PathBuf,
    /// View-discovery + watch config (`view_dirs`, `view_leaf`, `packs`,
    /// `watch`). Consumer bundle targets are NO LONGER here — they are
    /// discovered from per-package `deck.bundles` (see `consumer_targets`).
    /// Defaults to `DeckConfig::defaults()` for zero-config hosts.
    pub cfg: DeckConfig,
}

impl BundleLayout {
    /// Construct loading `deck.jsonc` from `repo_root` (the normal default).
    /// Falls back to [`DeckConfig::defaults`] when the file is absent, so
    /// zero-config hosts (deck-conformance, integration test tempdirs) work without
    /// a manifest.
    pub fn new(repo_root: PathBuf, workspace_root: Option<PathBuf>) -> Self {
        let cfg = DeckConfig::load(&repo_root);
        Self::with_config(repo_root, workspace_root, cfg)
    }

    /// Construct with an explicit [`DeckConfig`] (the normal shipd/bake path —
    /// caller already called `DeckConfig::load` once; avoids a double read).
    pub fn with_config(
        repo_root: PathBuf,
        workspace_root: Option<PathBuf>,
        cfg: DeckConfig,
    ) -> Self {
        let source_roots = match &workspace_root {
            Some(ws) if *ws != repo_root => vec![repo_root.clone(), ws.clone()],
            _ => vec![repo_root.clone()],
        };
        Self { out_root: repo_root.clone(), repo_root, source_roots, cfg }
    }

    /// Redirect output under `out_root` (sources still resolve from `repo_root`).
    pub fn with_out_root(mut self, out_root: PathBuf) -> Self {
        self.out_root = out_root;
        self
    }

    /// `<out_root>/.deck` — the deck-owned output dir (framework bundles,
    /// `sources/` tree for the studio viewer, framework surface materialization).
    pub fn deck_out(&self) -> PathBuf {
        self.out_root.join(".deck")
    }
    /// Host-owned assets root, derived from `PathBuf::from(".")`.
    fn host_assets(&self) -> PathBuf {
        self.out_root.join(&PathBuf::from("."))
    }

    /// Output dir for a target by name. Deck-own framework singletons →
    /// `.deck/framework/` (the build-script staging path that `build.rs` copies
    /// into `crates/deck/deck-core/assets/` for `include_str!`); `views/*` and
    /// `sui-components/*` → `.deck/`; everything else → `PathBuf::from(".")`.
    pub fn assets_dir_for(&self, name: &str) -> PathBuf {
        if DECK_OWN_BUNDLES.contains(&name) {
            return self.deck_out().join("framework");
        }
        if name.starts_with("views/") || name.starts_with("sui-components/") {
            return self.deck_out();
        }
        self.host_assets()
    }

    /// All targets: framework singletons + discovered consumer bundles + view bundles.
    pub fn all_targets(&self) -> Vec<BundleTarget> {
        let mut out = self.framework_singletons();
        out.extend(self.consumer_targets());
        out.extend(self.discover_view_bundles());
        out
    }

    /// Deck's OWN framework singletons — always built, hardcoded, never in
    /// deck.jsonc. These are the 4 runtime primitives deck ships with itself
    /// (like Next.js shipping its own runtime).
    fn framework_singletons(&self) -> Vec<BundleTarget> {
        let r = |rel: &str| self.repo_root.join(rel);
        vec![
            BundleTarget {
                name: "sui-runtime".into(),
                entry: TargetEntry::Real(r("packages/sui/runtime/src/index.ts")),
                externals: vec![],
                out_file: "sui-runtime.js".into(),
                lazy: false,
            },
            BundleTarget {
                name: "sui-dom".into(),
                entry: TargetEntry::Real(r("packages/sui/dom/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-dom.js".into(),
                lazy: false,
            },
            BundleTarget {
                name: "sui-sdk-client".into(),
                entry: TargetEntry::Real(r("packages/deck/sdk-client/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-sdk-client.js".into(),
                lazy: false,
            },
        ]
    }

    /// Consumer bundle targets, DISCOVERED from per-package self-declaration
    /// (`package.json` `deck.bundles[]`) — the dissolved central `deck.jsonc
    /// targets`. The package that owns the entry declares the bundle; the
    /// bundler walks the workspace to find them, the same way it discovers
    /// `views/*`. Each declaration's `entry` is package-relative, joined onto
    /// the declaring package's root. The logical name drives `assets_dir_for`
    /// routing (anything not `views/`/`sui-components/`/a framework singleton →
    /// `host_assets`).
    pub fn consumer_targets(&self) -> Vec<BundleTarget> {
        crate::catalog::discover_bundle_declarations(&self.repo_root)
            .into_iter()
            .map(|(pkg_root, d)| BundleTarget {
                name: d.name.clone(),
                entry: TargetEntry::Real(pkg_root.join(&d.entry)),
                externals: d.externals,
                out_file: format!("{}.js", d.name),
                lazy: d.lazy,
            })
            .collect()
    }

    /// One bundle per discovered view. The scan is fully config-driven
    /// (#273 — deck hardcodes no path):
    ///
    /// - For each `source_root`, walk each `cfg.view_dirs` bucket
    ///   (defaults to `["views", ".ship/views"]`) and look for a
    ///   `<bucket>/<name>/<view_leaf>` artifact. The leaf is
    ///   `cfg.view_leaf` (defaults to `page.sui`) — one convention for
    ///   all buckets, fixing the #254 `index.sui` vs `page.sui` divergence.
    /// - If `cfg.` is set, its `root` is treated as a
    ///   first-party view root too: `<root>/<name>/<view_leaf>`. The
    ///   host (e.g. ship) declares its own first-party app layout in
    ///   `deck.jsonc`; deck does not name the app.
    ///
    /// First-seen wins: earlier `view_dirs` entries (and earlier
    /// `source_roots`) shadow later matches.
    fn discover_view_bundles(&self) -> Vec<BundleTarget> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let leaf = self.cfg.view_leaf.as_deref().unwrap_or("page.sui");
        let mut candidates: Vec<PathBuf> = Vec::new();
        for root in &self.source_roots {
            for bucket in &self.cfg.view_dirs {
                candidates.push(root.join(bucket));
            }
        }
        for dir in candidates {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if seen.contains(&name) || !e.path().is_dir() {
                    continue;
                }
                let entry = e.path().join(leaf);
                if !entry.exists() {
                    continue;
                }
                seen.insert(name.clone());
                out.push(BundleTarget {
                    name: format!("views/{name}/view"),
                    entry: TargetEntry::Real(entry),
                    externals: vec![
                        "@ship/sui-runtime".into(),
                        "@ship/sui-runtime/template-hooks".into(),
                        "@ship/sui-dom".into(),
                        "@ship/sdk-client".into(),
                    ],
                    out_file: format!("views/{name}/view.js"),
                    lazy: false,
                });
            }
        }
        out
    }
}

impl BundleLayout {
    /// Targets a changed file affects (watch rebuild). Virtual aggregates match
    /// their explicit `sources` (or their dirs); real entries match the entry
    /// file or its package dir. Mirrors the worker's `targetsForFile`.
    pub fn targets_for_file(&self, abs: &Path) -> Vec<BundleTarget> {
        self.all_targets()
            .into_iter()
            .filter(|t| match &t.entry {
                TargetEntry::Virtual { sources } => sources
                    .iter()
                    .any(|s| abs == s || s.parent().is_some_and(|d| abs.starts_with(d))),
                TargetEntry::Real(entry) => {
                    abs == entry || entry.parent().is_some_and(|d| abs.starts_with(d))
                }
            })
            .collect()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn assets_dir_routes_by_name() {
        // Framework singletons land in `.deck/framework/` (build-script staging).
        let l = BundleLayout::new(PathBuf::from("/repo"), None);
        assert_eq!(l.assets_dir_for("sui-runtime"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sui-dom"), PathBuf::from("/repo/.deck/framework"));
        assert_eq!(l.assets_dir_for("sui-sdk-client"), PathBuf::from("/repo/.deck/framework"));
        // Per-tag chrome (`sui-components/<name>`) lands in `.deck/`.
        // (The umbrella `sui-components` target no longer exists — chrome
        // is per-tag tree-shaking only.)
        assert_eq!(l.assets_dir_for("sui-components/sui-rail"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("views/git/view"), PathBuf::from("/repo/.deck"));
        // Consumer targets (e.g. sui-marketing, sui-arcade) also land in `.deck/`.
        // assets root (repo-relative, not under `.deck/`). Per-app
        // bundlers copy their `public/` to `<project>/.deck/`.
        assert_eq!(l.assets_dir_for("ship-something"), PathBuf::from("/repo"));
    }

    #[test]
    fn discovers_view_bundles() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // A view.
        write(&root.join("views/git/page.sui"), "<div>git</div>\n");

        let l = BundleLayout::new(root, None);
        let targets = l.all_targets();
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"sui-runtime"), "static framework target present");
        assert!(names.contains(&"views/git/view"), "discovered view");
    }

    /// Consumer bundles are DISCOVERED from per-package `deck.bundles` — no
    /// central `targets` list. The declaration's package-relative `entry` is
    /// joined onto the declaring package's root, and `lazy` rides through onto
    /// the `BundleTarget`. This is the dissolution's core invariant.
    #[test]
    fn consumer_targets_discovered_from_package_self_declaration() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        write(&root.join("package.json"), r#"{ "workspaces": ["packages/*", "apps/*"] }"#);
        // An eager marketing pack (apps/site) + a lazy component pack.
        write(
            &root.join("apps/site/package.json"),
            r#"{ "name": "@ship/site", "deck": { "bundles": [
                { "name": "sui-marketing", "entry": "src/marketing/marketing-bundle.ts",
                  "externals": ["@ship/sui-runtime", "@ship/sui-dom"], "lazy": false } ] } }"#,
        );
        write(
            &root.join("packages/arcade/package.json"),
            r#"{ "name": "@ship/arcade", "deck": { "bundles": [
                { "name": "sui-arcade", "entry": "src/index.ts",
                  "externals": ["@ship/sui-runtime", "@ship/sui-dom"], "lazy": true } ] } }"#,
        );

        let l = BundleLayout::new(root.clone(), None);
        let consumer = l.consumer_targets();

        let marketing = consumer.iter().find(|t| t.name == "sui-marketing").expect("marketing discovered");
        assert_eq!(
            marketing.entry,
            TargetEntry::Real(root.join("apps/site/src/marketing/marketing-bundle.ts")),
            "entry is package-relative, joined onto the declaring package root"
        );
        assert_eq!(marketing.externals, vec!["@ship/sui-runtime", "@ship/sui-dom"]);
        assert_eq!(marketing.out_file, "sui-marketing.js");
        assert!(!marketing.lazy, "marketing is eager");

        let arcade = consumer.iter().find(|t| t.name == "sui-arcade").expect("arcade discovered");
        assert!(arcade.lazy, "arcade's lazy intent rides onto the BundleTarget");

        // No central registry: discovery is the only source of consumer targets.
        assert_eq!(consumer.len(), 2, "exactly the two self-declared bundles");
    }
}
