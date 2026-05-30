//! Bundle target resolution for the Rust bundler (#163).
//!
//! `TARGETS` + `discoverSeparateBundles` + `discoverViewBundles` +
//! `assetsDirFor` for the bundler. A `BundleTarget` names what to build, where
//! its entry is, what stays external, and where the output lands. The Rolldown
//! build (`rolldown_bundle.rs`) iterates these; nothing here touches the
//! filesystem beyond discovery reads.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// `<bundle separate />` top-of-file directive opts a component dir into its own
/// code-split bundle (vs living in its package umbrella aggregate).
const BUNDLE_SEPARATE_NEEDLE: &str = "<bundle separate";

/// Roots scanned for `<tag>/` dirs that may opt into separate bundles.
///
/// `sui/terminal/src` is here because `<sui-terminal>` is a real island: it
/// carries `<bundle separate />` (its xterm graph is too heavy for any umbrella)
/// and is mounted live by the shell's `<sui-agent-attach>`. The daemon serves it
/// at `/_sui/sui-components/sui-terminal.js`; without this root deck would never
/// build it and the daemon would 404 the island.
const SEPARATE_BUNDLE_ROOTS: [&str; 3] = [
    "packages/components/ui-components/src",
    "apps/shell/src/components",
    "packages/components/sui/terminal/src",
];

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
    /// `sui-*`/`views/*` → `.deck/`; everything else → `apps/shipd/assets`.
    pub fn assets_dir_for(&self, name: &str) -> PathBuf {
        if DECK_OWN_BUNDLES.contains(&name) {
            return self.deck_own_assets();
        }
        if name.starts_with("sui-") || name.starts_with("views/") {
            return self.deck_out();
        }
        self.lit_assets()
    }

    /// All targets: the static framework + aggregates, plus discovered
    /// separate-bundle islands and view bundles.
    pub fn all_targets(&self) -> Vec<BundleTarget> {
        let mut out = self.static_targets();
        out.extend(self.discover_separate_bundles());
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
        ]
    }

    /// The islands manifest: every separate-bundle component tag mapped to its
    /// bundle name. Derived from the SAME `<bundle separate />` scan that drives
    /// `discover_separate_bundles` — this is the one source of truth for island
    /// membership, surfaced to deck-core as `.deck/islands.json`.
    ///
    /// Each separate-bundle dir is one bundle (named after the dir). Every `.sui`
    /// file inside contributes its file-stem as a tag pointing at that bundle, so
    /// a family dir (e.g. `sui-command-bar/` holding `sui-command-bar.sui` +
    /// `sui-command-palette.sui`) maps each member tag to the one shared bundle.
    pub fn island_manifest(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        for target in self.discover_separate_bundles() {
            // `name` is `sui-components/<dir>`; the bundle is `<dir>`.
            let Some(bundle) = target.name.strip_prefix("sui-components/") else {
                continue;
            };
            if let TargetEntry::Virtual { sources } = &target.entry {
                for src in sources {
                    if let Some(tag) = src.file_stem().and_then(|s| s.to_str()) {
                        map.insert(tag.to_string(), bundle.to_string());
                    }
                }
            }
        }
        map
    }

    /// Walk each separate-bundle root for `sui-*/` dirs whose `.sui` files carry
    /// `<bundle separate />`; each becomes a virtual aggregate of its `.sui`s.
    fn discover_separate_bundles(&self) -> Vec<BundleTarget> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for rel in SEPARATE_BUNDLE_ROOTS {
            let components_dir = self.repo_root.join(rel);
            let Ok(entries) = std::fs::read_dir(&components_dir) else { continue };
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if !name.starts_with("sui-") || seen.contains(&name) {
                    continue;
                }
                if !e.path().is_dir() {
                    continue;
                }
                let mut sui_paths = sui_files_in(&e.path());
                if sui_paths.is_empty() {
                    continue;
                }
                sui_paths.sort();
                let separate = sui_paths.iter().any(|p| {
                    std::fs::read_to_string(p)
                        .map(|s| s.contains(BUNDLE_SEPARATE_NEEDLE))
                        .unwrap_or(false)
                });
                if !separate {
                    continue;
                }
                seen.insert(name.clone());
                out.push(BundleTarget {
                    name: format!("sui-components/{name}"),
                    entry: TargetEntry::Virtual { sources: sui_paths },
                    externals: vec!["@ship/sui-runtime".into(), "@ship/sui-dom".into()],
                    out_file: format!("sui-components/{name}.js"),
                });
            }
        }
        out
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

fn sui_files_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "sui"))
        .collect()
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
    fn discovers_views_and_separate_bundles() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        // A view.
        write(&root.join("views/git/page.sui"), "<div>git</div>\n");
        // A separate-bundle component + a non-separate one.
        write(
            &root.join("packages/components/ui-components/src/sui-rail/sui-rail.sui"),
            "<bundle separate />\n<div>rail</div>\n",
        );
        write(
            &root.join("packages/components/ui-components/src/sui-plain/sui-plain.sui"),
            "<div>plain</div>\n",
        );

        let l = BundleLayout::new(root, None);
        let targets = l.all_targets();
        let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();

        assert!(names.contains(&"sui-runtime"), "static framework target present");
        assert!(names.contains(&"views/git/view"), "discovered view");
        assert!(names.contains(&"sui-components/sui-rail"), "separate-bundle island");
        assert!(
            !names.contains(&"sui-components/sui-plain"),
            "non-separate component must NOT get its own bundle"
        );
    }

    #[test]
    fn island_manifest_maps_family_tags_to_one_bundle() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let dir = "packages/components/ui-components/src";
        // A family bundle: two member tags share one dir/bundle.
        write(&root.join(format!("{dir}/sui-command-bar/sui-command-bar.sui")), "<bundle separate />\n");
        write(&root.join(format!("{dir}/sui-command-bar/sui-command-palette.sui")), "<div/>\n");
        // A solo island.
        write(&root.join(format!("{dir}/sui-canvas/sui-canvas.sui")), "<bundle separate />\n");
        // A non-separate component must not appear.
        write(&root.join(format!("{dir}/sui-plain/sui-plain.sui")), "<div/>\n");

        let m = BundleLayout::new(root, None).island_manifest();
        assert_eq!(m.get("sui-command-bar").map(String::as_str), Some("sui-command-bar"));
        assert_eq!(m.get("sui-command-palette").map(String::as_str), Some("sui-command-bar"));
        assert_eq!(m.get("sui-canvas").map(String::as_str), Some("sui-canvas"));
        assert!(!m.contains_key("sui-plain"), "non-separate tag must not be an island");
    }
}
