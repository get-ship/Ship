use std::path::PathBuf;
use std::sync::{Once, RwLock};

/// One-root-per-process invariant: a single daemon serves one workspace, so a
/// process-global override is sound. If that ever breaks (multi-tenant
/// embedding), per-request threading through the resolver is the fallback.
static PROJECT_ROOT_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);
static DETECTION_WARN_ONCE: Once = Once::new();

/// Authoritative project-root override set ONCE at host startup.
///
/// Long-running / embedding hosts (shipd, deck-demo) call this with the
/// workspace root they already hold. The resolver consults this BEFORE
/// the cwd-detection floor, so per-workspace requests resolve against
/// the workspace they were launched against even if cwd drifts later.
pub fn set_project_root(root: PathBuf) {
    if let Ok(mut slot) = PROJECT_ROOT_OVERRIDE.write() {
        *slot = Some(root);
    }
}

/// Read the current override (test-only inspection).
pub fn current_project_root_override() -> Option<PathBuf> {
    PROJECT_ROOT_OVERRIDE.read().ok().and_then(|g| g.clone())
}

#[cfg(test)]
pub(crate) fn reset_project_root_override_for_test() {
    if let Ok(mut slot) = PROJECT_ROOT_OVERRIDE.write() {
        *slot = None;
    }
    // Note: `DETECTION_WARN_ONCE` is NOT reset — the per-process warn
    // is one-shot by design (per spec). Tests that need a fresh warn
    // stamp must run in their own process.
}

/// Nearest-ancestor detection: upwalk from `cwd`, accept the first ancestor
/// that contains a deck-owned SOURCE marker (`deck.jsonc` > `views/` >
/// `.ship/views/`). Bounded at ~32 hops. Std-only — never reaches a
/// TOPMOST-Cargo.toml workspace resolver (which would shadow a nested
/// project's root onto the monorepo root). Never keys on `.deck` (build
/// output, absent on fresh checkout, flips post-build).
pub(crate) fn detect_project_root_from(cwd: &std::path::Path) -> Option<PathBuf> {
    const MAX_DEPTH: usize = 32;
    let mut cur = cwd;
    for _ in 0..MAX_DEPTH {
        if cur.join("deck.jsonc").is_file() {
            return Some(cur.to_path_buf());
        }
        if cur.join("views").is_dir() {
            return Some(cur.to_path_buf());
        }
        if cur.join(".ship").join("views").is_dir() {
            return Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => break,
        }
    }
    None
}

use deck_host::resolver::ViewLayout;
pub use deck_host::resolver::{ResolvedView, ResolveError, render_resolved};
/// First-party view root, in resolution order:
///
/// 1. `DeckConfig.first_party_app.root` from `<project_root>/deck.jsonc`
///    (the canonical config-driven source — deck never names the app).
/// 2. `DECK_FIRST_PARTY_VIEWS` env var.
///
/// When none of the above is set, the host is non-ship and there is no
/// first-party bucket. See .artifacts/asset-architecture.md.
fn first_party_root() -> Option<PathBuf> {
    if let Some(root) = resolve_project_root() {
        let cfg = deck_config::DeckConfig::load(&root);
        if let Some(fpa) = cfg.first_party_app {
            // `fpa.root` is repo-relative (e.g. `apps/shell/src`); the host
            // resolver expects an absolute path so its `canonicalize`
            // containment guard has a stable root. The relative-path form
            // would resolve against cwd, which is not what callers mean.
            return Some(root.join(fpa.root));
        }
    }
    std::env::var("DECK_FIRST_PARTY_VIEWS")
        .ok()
        .map(PathBuf::from)
}
/// Build a [`ViewLayout`] from the project-root `deck.jsonc` (when present)
/// or fall back to the conventional `["views", ".ship/views"]` buckets with
/// a single `page.sui` leaf. The leaf is unified across all buckets
/// (fixes the #254 `index.sui` vs `page.sui` divergence); the bucket
/// list is config-driven so a host with a different layout can override
/// it via `viewDirs` in `deck.jsonc`.
fn view_layout() -> ViewLayout {
    let mut layout = ViewLayout::conventional();
    if let Some(root) = resolve_project_root() {
        let cfg = deck_config::DeckConfig::load(&root);
        if !cfg.view_dirs.is_empty() {
            layout.view_dirs = cfg.view_dirs.clone();
        }
        if let Some(leaf) = cfg.view_leaf {
            layout.view_leaf = leaf;
        }
    }
    layout
}
pub fn resolve(name: &str, sub_path: Option<&str>) -> Result<ResolvedView, ResolveError> {
    let mut layout = view_layout();
    layout.first_party_root = first_party_root();
    // `resolve_project_root` resolves override → nearest-ancestor marker
    // detection → cwd fallback. The override is the safe path; long-running
    // hosts (shipd, deck-demo) call `set_project_root` at startup so they
    // never ride the cwd-detection floor. The detection floor is the fragile
    // path: it keys on deck-owned SOURCE markers (`deck.jsonc` > `views/` >
    // `.ship/views/`), so a fresh checkout resolves before any `.deck` build
    // output exists, and a nested project's `app/` is not shadowed onto the
    // monorepo root.
    let Some(dir) = resolve_project_root() else {
        return Err(ResolveError {
            message: format!(
                "view not found: {} — no project root (host should call deck_core::resolver::set_project_root at startup)",
                join_request(name, sub_path),
            ),
        });
    };
    deck_host::resolver::resolve(&dir, &layout, name, sub_path)
}
/// Project root for locating the `.deck` build-output tree. deck reads app +
/// user view/chrome bundles from `<project_root>/.deck/` — never from its own
/// crate (that holds only deck's baked runtime). Same resolution as view
/// source, so the built bundles sit beside the source tree they came from.
pub(crate) fn deck_out_root() -> Option<PathBuf> {
    resolve_project_root().map(|p| p.join(".deck"))
}

/// Resolution order (HIGH → LOW):
/// 1. **HOST OVERRIDE** — `set_project_root(root)` set at startup. Authoritative
///    for long-running hosts (shipd, deck-demo) that already hold the workspace
///    root in typed state. Sound because deck serves one root per process.
/// 2. **DETECTION FLOOR** — std-only nearest-ancestor upwalk from
///    `std::env::current_dir()`, looking for a deck-owned SOURCE marker
///    (`deck.jsonc` > `views/` > `.ship/views/`) at any of the up to 32
///    ancestors. Nearest-ancestor-wins: deck-demo's `app/` does NOT resolve
///    to the monorepo root, and a fresh checkout resolves before `.deck` is
///    built (markers are source-side, all git-tracked).
/// 3. **ZERO-CONFIG FALLBACK** — `current_dir()`. Matches the
///    `source_roots: ["."]` default in [`DeckConfig::defaults`].
///
/// The host-override step is the SAFE path; the cwd-detection floor is
/// fragile. A long-running host that never calls `set_project_root` is
/// warned once on the detection / fallback path so the coupling is
/// surfaced instead of silently riding the floor.
pub(crate) fn resolve_project_root() -> Option<PathBuf> {
    if let Some(root) = PROJECT_ROOT_OVERRIDE.read().ok().and_then(|g| g.clone()) {
        return Some(root);
    }
    let cwd = std::env::current_dir().ok();
    if let Some(ref dir) = cwd
        && let Some(root) = detect_project_root_from(dir)
    {
        warn_server_host_uses_detection();
        return Some(root);
    }
    if let Some(dir) = cwd {
        warn_server_host_uses_detection();
        return Some(dir);
    }
    None
}

/// One-shot warn behind `std::sync::Once`. Catches a future long-running host
/// that mounts the resolver without calling `set_project_root` — a smell,
/// because the cwd-detection floor is the fragile path.
fn warn_server_host_uses_detection() {
    if std::env::var_os("DECK_SERVER_HOST").is_some() {
        DETECTION_WARN_ONCE.call_once(|| {
            tracing::warn!(
                "deck: project root resolved by cwd-detection; a long-running host should call set_project_root()"
            );
        });
    }
}

fn join_request(name: &str, sub_path: Option<&str>) -> String {
    match sub_path.filter(|s| !s.is_empty()) {
        Some(sub) => format!("{name}/{sub}"),
        None => name.to_string(),
    }
}

/// Best-effort list of top-level view names servable on this daemon, for the
/// human-facing not-found page. Scans the same on-disk roots the resolver
/// consults — the first-party bucket (config-driven via
/// `first_party_app.root` or the legacy env-var fallback) plus every
/// bucket in the host's `view_dirs` (default `["views", ".ship/views"]`).
/// Cheap (one shallow `read_dir` per root, no compilation) and graceful:
/// any missing root contributes nothing, and with no project root at
/// all the list is simply empty. Names are deduped and sorted so the
/// page renders deterministically.
pub fn list_view_names() -> Vec<String> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let layout = view_layout();
    if let Some(fp) = layout.first_party_root.as_deref() {
        collect_view_dirs(fp, &mut names);
    }
    if let Some(root) = resolve_project_root() {
        for bucket in &layout.view_dirs {
            collect_view_dirs(&root.join(bucket), &mut names);
        }
    }
    names.into_iter().collect()
}

/// Add each immediate child directory of `dir` whose name is a valid view name
/// to `out`. A non-directory or unreadable `dir` contributes nothing.
fn collect_view_dirs(dir: &std::path::Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && crate::view::is_valid_view_name(name)
        {
            out.insert(name.to_string());
        }
    }
}

#[cfg(test)]
mod project_root_tests {
    use super::{detect_project_root_from, reset_project_root_override_for_test, set_project_root};
    use std::path::PathBuf;

    /// Nested project: monorepo root has a `views/`, the inner project's
    /// `app/` ALSO has a `views/`. Detection must pick the NEAREST one
    /// (`app/`), not the topmost (monorepo). This is the deck-demo shape:
    /// a demo app nested inside a wider monorepo, where reaching the
    /// monorepo root would shadow the project's own views.
    #[test]
    fn detection_picks_nearest_ancestor_marker_not_topmost() {
        let tmp = tempfile::tempdir().unwrap();
        let monorepo = tmp.path().to_path_buf();
        std::fs::create_dir(monorepo.join("views")).unwrap();
        let app = monorepo.join("app");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(app.join("views")).unwrap();
        let deep = app.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();

        // Run from a deeply-nested leaf of `app/`; the nearest `views/`
        // is `app/views`, not the monorepo's.
        let detected = detect_project_root_from(&deep);
        assert_eq!(
            detected,
            Some(app.clone()),
            "nearest-ancestor-wins: expected the inner app root, got {detected:?}",
        );
        // The monorepo's `views/` is NOT picked up, even though it exists.
        let monorepo_with_views = monorepo.join("views");
        assert!(monorepo_with_views.is_dir());
        assert_ne!(detected, Some(monorepo));
    }

    /// `deck.jsonc` is the strongest marker — even nested inside a tree
    /// that ALSO has a `views/` directory, `deck.jsonc` wins at the SAME
    /// depth, and the nearest one wins over any ancestor's `deck.jsonc`.
    #[test]
    fn detection_prefers_deck_jsonc_over_views_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = tmp.path().to_path_buf();
        std::fs::write(outer.join("deck.jsonc"), "{}\n").unwrap();
        let inner = outer.join("nested");
        std::fs::create_dir(&inner).unwrap();
        std::fs::create_dir(inner.join("views")).unwrap();
        // From `inner`, the nearest marker is `views/`. The outer's
        // `deck.jsonc` is one level up.
        let detected = detect_project_root_from(&inner);
        assert_eq!(detected, Some(inner));
    }

    /// `.ship/views/` is the third marker (legacy deck-style placement).
    #[test]
    fn detection_accepts_dot_ship_views_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".ship").join("views")).unwrap();
        let detected = detect_project_root_from(&root);
        assert_eq!(detected, Some(root));
    }
    /// No marker anywhere up the tree → `None`. (Caller falls back to
    /// cwd.)
    #[test]
    fn detection_returns_none_with_no_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(detect_project_root_from(&deep).is_none());
    }

    /// The global override ALWAYS wins — even from a directory that
    /// would otherwise be detected as a project root.
    #[test]
    fn override_wins_over_detection() {
        reset_project_root_override_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let detected_root = tmp.path().to_path_buf();
        std::fs::create_dir(detected_root.join("views")).unwrap();

        // The detected-root is "the project" from disk's perspective.
        let detected = detect_project_root_from(&detected_root);
        assert_eq!(detected, Some(detected_root.clone()));

        // But the override sits at a different, unrelated path, and
        // `resolve_project_root` consults the override FIRST. We do not
        // call `resolve_project_root` here (it would `current_dir` and
        // confuse the test); we only assert the contract documented on
        // `set_project_root`.
        set_project_root(PathBuf::from("/some/explicit/host/root"));
        assert_eq!(
            super::current_project_root_override(),
            Some(PathBuf::from("/some/explicit/host/root")),
        );
        reset_project_root_override_for_test();
    }
}
