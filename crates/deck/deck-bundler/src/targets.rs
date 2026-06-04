//! Bundle target resolution for the Rust bundler (#163).
//!
//! `TARGETS` + `discoverViewBundles` + `assetsDirFor` for the bundler. A
//! `BundleTarget` names what to build, where its entry is, what stays external,
//! and where the output lands. The Rolldown build (`rolldown_bundle.rs`)
//! iterates these; nothing here touches the filesystem beyond discovery reads.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Deck's OWN runtime bundles — baked into the deck-core crate (like Next
/// shipping its own runtime). Everything else under `sui-`/`views/` is app/user
/// content and goes to `.deck/`.
const DECK_OWN_BUNDLES: [&str; 4] = ["sui-runtime", "sui-dom", "sui-sdk-client", "sui-components"];

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
}

impl BundleLayout {
    pub fn new(repo_root: PathBuf, workspace_root: Option<PathBuf>) -> Self {
        let source_roots = match &workspace_root {
            Some(ws) if *ws != repo_root => vec![repo_root.clone(), ws.clone()],
            _ => vec![repo_root.clone()],
        };
        Self { out_root: repo_root.clone(), repo_root, source_roots }
    }

    /// Redirect output under `out_root` (sources still resolve from `repo_root`).
    pub fn with_out_root(mut self, out_root: PathBuf) -> Self {
        self.out_root = out_root;
        self
    }

    fn deck_own_assets(&self) -> PathBuf {
        self.out_root.join("crates/deck/deck-core/assets")
    }
    fn deck_out(&self) -> PathBuf {
        self.out_root.join(".deck")
    }
    fn lit_assets(&self) -> PathBuf {
        self.out_root.join("apps/shipd/assets")
    }

    /// Output dir for a target by name. Deck-own bundles → the crate's assets;
    /// `views/*` and `sui-components/sui-chrome` → `.deck/`; everything else →
    /// `apps/shipd/assets`.
    pub fn assets_dir_for(&self, name: &str) -> PathBuf {
        if DECK_OWN_BUNDLES.contains(&name) {
            return self.deck_own_assets();
        }
        if name.starts_with("views/") || name.starts_with("sui-components/") {
            return self.deck_out();
        }
        self.lit_assets()
    }

    /// All targets: the static framework + aggregates, plus view bundles.
    pub fn all_targets(&self) -> Vec<BundleTarget> {
        let mut out = self.static_targets();
        out.extend(self.discover_view_bundles());
        out
    }

    fn static_targets(&self) -> Vec<BundleTarget> {
        let r = |rel: &str| self.repo_root.join(rel);
        vec![
            BundleTarget {
                name: "sui-runtime".into(),
                entry: TargetEntry::Real(r("packages/sui/runtime/src/index.ts")),
                externals: vec![],
                out_file: "sui-runtime.js".into(),
            },
            BundleTarget {
                name: "sui-dom".into(),
                entry: TargetEntry::Real(r("packages/sui/dom/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-dom.js".into(),
            },
            BundleTarget {
                name: "sui-sdk-client".into(),
                entry: TargetEntry::Real(r("packages/deck/sdk-client/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-sdk-client.js".into(),
            },
            BundleTarget {
                name: "sui-components".into(),
                entry: TargetEntry::Real(r("packages/components/ui-components/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into()],
                out_file: "sui-components.js".into(),
            },
            BundleTarget {
                name: "sui-components/sui-chrome".into(),
                entry: TargetEntry::Real(r("apps/shell/src/components/index.ts")),
                externals: vec!["@ship/sui-runtime".into(), "@ship/sui-dom".into()],
                out_file: "sui-components/sui-chrome.js".into(),
            },
            BundleTarget {
                name: "sui-marketing".into(),
                // Membership is TRACED, not declared here: the site-owned entry
                // `apps/site/src/marketing/marketing-bundle.ts` side-effect-imports
                // each marketing tag, and rolldown follows that import graph. Add
                // or remove a tag THERE — never in this Rust file.
                entry: TargetEntry::Real(r("apps/site/src/marketing/marketing-bundle.ts")),
                externals: vec!["@ship/sui-runtime".into(), "@ship/sui-dom".into()],
                out_file: "sui-marketing.js".into(),
            },
            BundleTarget {
                // The arcade pack — the heavy lazy chunk for the marketing
                // site's `<sui-stage>` games. The pack's `index.ts`
                // side-effect-registers every `<sui-*>` arcade tag (stage,
                // sprite, shape, collider, emitter, input, sfx, hud) and pulls
                // the pixi + arcade-engine graph. Unlike the framework
                // singletons, pixi + arcade-engine get BUNDLED here (this is the
                // ~800 KB lazy chunk); only the runtime singletons stay external
                // (forced by `SINGLETONS` in rolldown_bundle.rs — one instance).
                name: "sui-arcade".into(),
                entry: TargetEntry::Real(r("packages/components/ship/arcade/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into(), "@ship/sui-dom".into()],
                out_file: "sui-arcade.js".into(),
            },
            BundleTarget {
                // The pdf pack — a lazy chunk wrapping the pdf.js (`pdfjs-dist`)
                // engine. The pack's `index.ts` side-effect-registers `<ship-pdf>`
                // and bundles BOTH the engine AND the pdf.js worker module
                // (`pdfjs-dist/build/pdf.worker.mjs`) — the worker registers a
                // main-thread `WorkerMessageHandler` (`globalThis.pdfjsWorker`)
                // that pdf.js uses with no separate URL / fetch / CDN, so the pack
                // renders offline / same-origin. Like arcade, the engine is
                // BUNDLED (this is the heavy chunk) and only the runtime singletons
                // stay external.
                name: "sui-pdf".into(),
                entry: TargetEntry::Real(r("packages/components/ship/pdf/src/index.ts")),
                externals: vec!["@ship/sui-runtime".into(), "@ship/sui-dom".into()],
                out_file: "sui-pdf.js".into(),
            },
        ]
    }

    /// One bundle per discovered view. Two layouts under each source root:
    /// `views/<name>/page.sui` and `.ship/views/<name>/index.sui`. First-seen
    /// wins (repo_root before workspace; `views/` before `.ship/views/`).
    fn discover_view_bundles(&self) -> Vec<BundleTarget> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut candidates: Vec<(PathBuf, &str)> = Vec::new();
        for root in &self.source_roots {
            candidates.push((root.join("views"), "page.sui"));
            candidates.push((root.join(".ship").join("views"), "index.sui"));
        }
        for (dir, leaf) in candidates {
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
        let l = BundleLayout::new(PathBuf::from("/repo"), None);
        assert_eq!(l.assets_dir_for("sui-runtime"), PathBuf::from("/repo/crates/deck/deck-core/assets"));
        assert_eq!(l.assets_dir_for("sui-components/sui-rail"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("views/git/view"), PathBuf::from("/repo/.deck"));
        assert_eq!(l.assets_dir_for("ship-something"), PathBuf::from("/repo/apps/shipd/assets"));
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
}
