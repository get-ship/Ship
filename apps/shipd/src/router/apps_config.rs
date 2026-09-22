//! shipd's [`AppRegistry`] impl — reads `apps.jsonc` (the host's app
//! orchestration list) and, for each entry, resolves the referenced
//! package to its on-disk dir via the node-resolution [`AppSource`],
//! then loads the package's `deck.jsonc` into an [`AppServeConfig`].
//!
//! `apps.jsonc` is the host (shipd) config; deck (L1) knows nothing about it.
//! Per the app-registration port discipline (sibling to `DataPort`): deck
//! declares the RESULT it needs (the `Vec<AppServeConfig>` to mount); shipd
//! owns the MECHANISM and binds the file locations HERE — deck names no path.
//!
//! ## `apps.jsonc` — orchestration only (host-owned, AUTHORITATIVE)
//!
//! ```jsonc
//! {
//!   "apps": [
//!     { "name": "hub",    "package": "@ship/hub",    "route": "/" },
//!     { "name": "spaces", "package": "spaces",       "route": "/space/{id}" }
//!   ]
//! }
//! ```
//!
//! Each entry is the host's bind: the `name` is the orchestration handle
//! (kept for log lines + matching), `package` is the package reference
//! resolved through [`AppSource`], and `route` is where shipd mounts it.
//! The package's `deck.jsonc` no longer carries `name`/`route`/`source` —
//! those are HOST concerns.
//!
//! `route` is AUTHORITATIVE for serving. The package's `deck.jsonc` cannot
//! override it — the host composes many apps; the app does not decide where
//! it mounts.
//!
//! ## `AppSource` — pluggable package resolution
//!
//! ```text
//!                     ┌──────────────┐
//!   apps.jsonc ──→    │  AppSource   │ ──→  on-disk package dir
//!   (entry.package)   │  (enum)      │      (e.g. apps/hub)
//!                     └──────────────┘
//!                            │
//!                            ├── NodeModules { base_dir, package }   (Phase 1, dev)
//!                            ├── Cdn { url, hash }                  (Phase 2, prod)
//!                            └── Registry { url }                    (Phase 3, registry)
//! ```
//!
//! Phase 1 implements only `NodeModules`: look up `<base_dir>/node_modules/<package>`
//! (or `<base_dir>/node_modules/@scope/<name>` for scoped packages). `Cdn` and
//! `Registry` are declared-but-not-constructed — adding them later requires no
//! `apps.jsonc` or serve-chain changes; only the resolver gains a new variant.
//!
//! `base_dir` is configurable per launch (env var `SHIP_APPS_BASE_DIR` or
//! `ShipAppRegistry::with_base_dir`). In dev, `base_dir` points at the ship
//! workspace root so the resolver finds `node_modules/@ship/hub` (the
//! workspace symlink the ship root `package.json` self-declares — j-12).
//! In prod the base_dir points at the host's installed `node_modules/`.
//! The default is `<project_root>/node_modules` — i.e. a top-level
//! `node_modules` next to `apps/shipd/apps.jsonc`.
//!
//! ## `<resolved-package-dir>/deck.jsonc` — the app's NON-conventional config
//!
//! An app whose config is fully conventional needs NO `deck.jsonc` —
//! the loader returns `AppServeConfig::default()`. When the file IS
//! present, it carries only the non-conventional pieces (the rendering
//! mode, optional URL-capture globals, optional `entry.gen.ts` bootstrap).
//!
//! Parsed by [`AppServeConfig::parse_deck_jsonc`] — deck-config owns the
//! schema + the JSONC parser; shipd owns the file locations.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use deck::AppServeConfig;
use deck::config::strip_jsonc_comments;
use deck_core::AppRegistry;
use serde::Deserialize;

// ── AppSource ────────────────────────────────────────────────────────────────

/// How a package reference in `apps.jsonc` becomes an on-disk directory.
///
/// Phase 1 implements only [`AppSource::NodeModules`]. [`AppSource::Cdn`] and
/// [`AppSource::Registry`] are declared but cannot be constructed today;
/// a future phase adds their constructors and resolution logic without
/// touching `apps.jsonc` or the serve chain.
#[derive(Debug, Clone)]
pub enum AppSource {
    /// `node_modules`-style resolution from a configurable base directory.
    /// Dev: `base_dir = /path/to/ship` and the package is found at
    /// `<base_dir>/node_modules/<package>`. Scoped packages (`@ship/hub`)
    /// resolve to `<base_dir>/node_modules/@ship/hub`.
    NodeModules {
        /// Directory containing the `node_modules/` to search. In dev this
        /// is the ship workspace root; in prod it's the host's
        /// `node_modules/` parent.
        base_dir: PathBuf,
        /// Package specifier — `spaces` (unscoped) or `@ship/hub` (scoped).
        package: String,
    },
    /// CDN-fetched bundle: a single self-contained built artifact fetched
    /// by URL, integrity-checked against a hash, and cached locally.
    /// **Phase 2 — not yet constructable.** Stubbed as an enum variant so
    /// the future code path doesn't need to touch `apps.jsonc` or the
    /// serve chain.
    #[allow(dead_code)]
    Cdn {
        url: String,
        #[allow(dead_code)]
        hash: String,
    },
    /// Internal registry: a package host (e.g. Verdaccio-via-deno-npm) that
    /// ships built bundles the same way the dev `node_modules/` does, but
    /// fetched over the network. **Phase 3 — not yet constructable.**
    #[allow(dead_code)]
    Registry {
        url: String,
        #[allow(dead_code)]
        package: String,
    },
}

impl AppSource {
    /// Resolve to the package's on-disk directory.
    ///
    /// Phase 1 only implements the [`AppSource::NodeModules`] branch. `Cdn`
    /// and `Registry` return an error — the variants exist for the future
    /// pluggable resolver, not for current use.
    pub fn resolve_dir(&self) -> Result<PathBuf> {
        match self {
            AppSource::NodeModules { base_dir, package } => resolve_node_modules(base_dir, package),
            AppSource::Cdn { .. } => Err(anyhow!(
                "AppSource::Cdn is not implemented yet (Phase 2); use NodeModules"
            )),
            AppSource::Registry { .. } => Err(anyhow!(
                "AppSource::Registry is not implemented yet (Phase 3); use NodeModules"
            )),
        }
    }
}

/// Walk `<base_dir>/node_modules/<package>` (or `@scope/name` for scoped
/// packages) and return the resolved directory. Errors when the package
/// is not present in the base dir's `node_modules/` — the loader must
/// surface that to the operator (a typo in `apps.jsonc`, a missed
/// `bun install`, or a stale `node_modules/`).
fn resolve_node_modules(base_dir: &Path, package: &str) -> Result<PathBuf> {
    let node_modules = base_dir.join("node_modules");
    // Scoped: `@ship/hub` → `node_modules/@ship/hub`
    // Unscoped: `spaces` → `node_modules/spaces`
    let candidate = if let Some(scope_rest) = package.strip_prefix('@') {
        if let Some((scope, name)) = scope_rest.split_once('/') {
            node_modules.join(format!("@{scope}")).join(name)
        } else {
            bail!(
                "package {package:?}: scoped name must be `@scope/name` \
                 (missing `/` after scope)"
            );
        }
    } else {
        node_modules.join(package)
    };
    if !candidate.is_dir() {
        bail!(
            "package {package:?} not found in {}/node_modules/ \
             (expected {})",
            base_dir.display(),
            candidate.display(),
        );
    }
    Ok(candidate)
}

// ── apps.jsonc schema ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppsConfigFile {
    #[serde(default)]
    apps: Vec<AppEntry>,
}

/// One orchestration entry: the host's bind for one app. `route` is
/// AUTHORITATIVE for serving — the package's `deck.jsonc` cannot override
/// it. `name` is the orchestration handle (kept for log lines + matching;
/// the URL namespace is the resolved package dir's basename). `package`
/// is resolved through [`AppSource`] to the on-disk dir; `source`
/// overrides the default convention `<dir-basename>/.deck` when an app
/// ships a non-conventional build output (rare; default is the bundler's
/// `<project>/.deck`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppEntry {
    name: String,
    route: String,
    package: String,
    /// Optional override for the convention `<dir-basename>/.deck`. When
    /// present, must be a path relative to the resolved package dir.
    #[serde(default)]
    source: Option<String>,
}

// ── ShipAppRegistry ──────────────────────────────────────────────────────────

/// shipd's app registry. Binds the resolver's `base_dir` (where
/// `node_modules/<package>` lives) and the `apps.jsonc` orchestration
/// file, then enumerates apps by resolving each entry's package and
/// loading the package's `deck.jsonc`. Mirrors the `DataPort` discipline —
/// the host binds its own state inside the impl; deck names no path.
///
/// `package_dirs()` returns the resolved on-disk dir for each loaded
/// app, keyed by the app's authoritative name (the resolved package
/// dir's basename). The host needs this for per-app operations that
/// reach back into the package (e.g. registering the package's
/// `.deck/views/` output dir for deck's view-bundle runtime). It's
/// host-specific state and intentionally lives OUTSIDE the
/// `AppRegistry` trait — the L1 framework consumes only the
/// `Vec<AppServeConfig>` seam and must not learn the host's package
/// resolution.
pub struct ShipAppRegistry {
    /// Directory containing the `node_modules/` the resolver searches.
    /// In dev: the ship workspace root (j-12). In prod: a host-managed
    /// `node_modules/` parent. Configurable per launch so dev / staging /
    /// prod can resolve different package sources without code changes.
    base_dir: PathBuf,
    /// Path to the host's `apps.jsonc` orchestration file.
    apps_config: PathBuf,
    /// Resolved package dirs from the last `serve_configs()` call,
    /// keyed by the package dir basename. Populated lazily on
    /// `serve_configs()`; reads before the first call return an empty
    /// map (so a load failure doesn't strand the host with stale
    /// per-app bookkeeping).
    package_dirs: std::sync::Mutex<std::collections::BTreeMap<String, PathBuf>>,
}

impl ShipAppRegistry {
    /// Build a registry that resolves packages from `<base_dir>/node_modules/`.
    /// Use [`ShipAppRegistry::new`] when the project root IS the resolver
    /// base (e.g. the ship-repo's own `node_modules/`).
    pub fn new(base_dir: PathBuf, apps_config: PathBuf) -> Self {
        Self {
            base_dir,
            apps_config,
            package_dirs: std::sync::Mutex::new(std::collections::BTreeMap::new()),
        }
    }

    /// Snapshot of the resolved package dirs from the most recent
    /// `serve_configs()` call. Keyed by the package dir basename (the
    /// authoritative app name). The host uses this for per-app
    /// operations that need the package's on-disk location (e.g.
    /// registering the package's view-bundle output dir).
    pub fn package_dirs(&self) -> std::collections::BTreeMap<String, PathBuf> {
        self.package_dirs
            .lock()
            .expect("ShipAppRegistry::package_dirs mutex")
            .clone()
    }
}

impl AppRegistry for ShipAppRegistry {
    fn serve_configs(&self) -> Vec<AppServeConfig> {
        match load_serve_configs(&self.base_dir, &self.apps_config, &self.package_dirs) {
            Ok(apps) => {
                tracing::info!(
                    path = %self.apps_config.display(),
                    base_dir = %self.base_dir.display(),
                    count = apps.len(),
                    "shipd: loaded apps from apps.jsonc (node_modules resolver)"
                );
                apps
            }
            Err(e) => {
                tracing::warn!(
                    path = %self.apps_config.display(),
                    base_dir = %self.base_dir.display(),
                    error = %e,
                    "shipd: app registry failed to load — serving no apps"
                );
                Vec::new()
            }
        }
    }
}

// ── loader ───────────────────────────────────────────────────────────────────

/// Read the orchestration list, resolve each entry's package via
/// `AppSource::NodeModules`, and load the package's `deck.jsonc` (if any).
///
/// Missing `apps.jsonc`: empty list (serve no app — a clean 404, not a panic).
/// A present-but-malformed `apps.jsonc`, or any declared app whose package
/// cannot be resolved (missing from `node_modules/`, or a directory
/// problem), is an error so the dev sees the failure at startup, not at
/// first request. A missing `<package>/deck.jsonc` is NOT an error — the
/// loader returns `AppServeConfig::default()` and fills in the
/// conventional fields from the orchestration bind + the resolved
/// package dir basename.
///
/// For every loaded app the host fills in three fields the per-app file
/// no longer carries:
///
/// - `name` — derived from the resolved package dir's basename
///   (`apps/hub` ⇒ `hub`). The directory is the source of truth for the
///   URL namespace; the `apps.jsonc` `name` is for log lines + matching.
///   A mismatch is logged and the basename wins.
/// - `route` — copied from the orchestration entry. The file cannot
///   override it; the host composes many apps.
/// - `source` — convention: `<dir-basename>/.deck` (the bundler's
///   default `out_dir`). Overridable per-entry via `entry.source`.
fn load_serve_configs(
    base_dir: &Path,
    apps_config: &Path,
    package_dirs: &std::sync::Mutex<std::collections::BTreeMap<String, PathBuf>>,
) -> Result<Vec<AppServeConfig>> {
    let raw = match std::fs::read_to_string(apps_config) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(anyhow!("read {}: {e}", apps_config.display())),
    };
    let parsed: AppsConfigFile = serde_json::from_str(&strip_jsonc_comments(&raw))
        .with_context(|| format!("parse {}", apps_config.display()))?;
    let mut apps = Vec::with_capacity(parsed.apps.len());
    for entry in parsed.apps {
        let source = AppSource::NodeModules {
            base_dir: base_dir.to_path_buf(),
            package: entry.package.clone(),
        };
        let package_dir = source.resolve_dir().with_context(|| {
            format!(
                "app '{}' (package {}): resolve from {}",
                entry.name,
                entry.package,
                base_dir.display()
            )
        })?;
        // Derive the app's name from the directory basename. This is
        // the AUTHORITATIVE identity — the bundler writes built assets
        // under `/<name>/…` and the daemon serves them at that path.
        let dir_name = package_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                anyhow!(
                    "apps.jsonc entry `{}`: cannot derive directory basename from `{}`",
                    entry.name,
                    package_dir.display()
                )
            })?
            .to_string();
        if dir_name != entry.name {
            tracing::warn!(
                orchestration = %entry.name,
                directory = %dir_name,
                package = %entry.package,
                "apps.jsonc name differs from package dir basename — using basename (authoritative)",
            );
        }
        // Convention: `<package>/.deck` (absolute path, resolved from
        // the package dir on disk). The `.deck` dir IS the bundler's
        // default `out_dir`; the AppSource resolver gave us the
        // package's absolute path, so we just join `.deck` onto it.
        // The `source` field on the entry is treated as a SUFFIX
        // relative to the package dir ("hub/.deck", "dist", etc.)
        // — never an absolute path; absolute overrides are a
        // path-traversal footgun the framework deliberately avoids.
        // A leading `/` is stripped so "/.deck" still resolves to
        // the package's bundled output.
        let source_suffix = entry
            .source
            .as_deref()
            .map(|s| s.trim_start_matches('/'))
            .unwrap_or(".deck");
        let source = package_dir
            .join(source_suffix)
            .to_string_lossy()
            .to_string();
        // Load the per-app `deck.jsonc` — OPTIONAL. Absent file
        // produces the conventional defaults; the host fills in the
        // orchestration bind below.
        let deck_path = package_dir.join("deck.jsonc");
        let mut cfg = if deck_path.exists() {
            load_app_deck(&deck_path)
                .with_context(|| format!("app '{}' ({})", entry.name, deck_path.display()))?
        } else {
            AppServeConfig::default()
        };
        cfg.name = dir_name.clone();
        cfg.route = entry.route;
        cfg.source = source;
        let authoritative_name = dir_name.clone();
        apps.push(cfg);
        // Record the resolved package dir under the authoritative
        // name (the dir basename). The host uses this snapshot for
        // per-app operations that need the package's on-disk
        // location (e.g. registering the view-bundle output dir).
        // The map reflects only the most recent `serve_configs()`
        // pass.
        package_dirs
            .lock()
            .expect("ShipAppRegistry::package_dirs mutex")
            .insert(authoritative_name, package_dir);
    }
    Ok(apps)
}

/// Load one app's `deck.jsonc` into its serve config. Used for the
/// (now optional) per-app file. Absent files fall through to
/// `AppServeConfig::default()`; the host binds name/route/source
/// outside this helper.
fn load_app_deck(path: &Path) -> Result<AppServeConfig> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    AppServeConfig::parse_deck_jsonc(&raw).with_context(|| format!("parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deck::RenderMode;

    /// Scaffold a temp `node_modules` layout: one base dir containing
    /// `node_modules/<package>` (an empty package dir), and an
    /// `apps.jsonc` next to it. The loader resolves the package from
    /// the `base_dir` we pass, so the base dir lives at `dir.path()`
    /// and the node_modules at `dir.path()/node_modules/`.
    fn scaffold(apps_jsonc: &str, packages: &[(&str, Option<&str>)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        std::fs::write(dir.path().join("apps.jsonc"), apps_jsonc).unwrap();
        for (pkg, body) in packages {
            // Unscoped: `node_modules/<name>`. Scoped: `node_modules/@scope/name`.
            let pkg_dir = if let Some(scope_rest) = pkg.strip_prefix('@') {
                if let Some((scope, name)) = scope_rest.split_once('/') {
                    dir.path()
                        .join("node_modules")
                        .join(format!("@{scope}"))
                        .join(name)
                } else {
                    panic!("bad scoped package in test fixture: {pkg}");
                }
            } else {
                dir.path().join("node_modules").join(pkg)
            };
            std::fs::create_dir_all(&pkg_dir).unwrap();
            if let Some(b) = body {
                std::fs::write(pkg_dir.join("deck.jsonc"), b).unwrap();
            }
        }
        dir
    }

    fn load(base_dir: &Path) -> Vec<AppServeConfig> {
        let pkg_dirs = std::sync::Mutex::new(std::collections::BTreeMap::new());
        load_serve_configs(base_dir, &base_dir.join("apps.jsonc"), &pkg_dirs).unwrap()
    }

    #[test]
    fn empty_apps_returns_empty() {
        let dir = scaffold("{}", &[]);
        assert!(load(dir.path()).is_empty());
    }

    #[test]
    fn missing_apps_jsonc_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let apps = {
            let pkg_dirs = std::sync::Mutex::new(std::collections::BTreeMap::new());
            load_serve_configs(dir.path(), &dir.path().join("nope.jsonc"), &pkg_dirs).unwrap()
        };
        assert!(apps.is_empty());
    }

    /// A first-party app whose config is fully conventional needs NO
    /// `deck.jsonc`. The loader returns `AppServeConfig::default()` and
    /// the host binds `name`/`route`/`source` from the package dir
    /// basename + the apps.jsonc entry. This is the post-cutover hub
    /// shape.
    #[test]
    fn hub_no_deck_jsonc_loads_with_conventional_defaults() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "package": "@ship/hub", "route": "/" }
            ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", None)]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        // Host-derived (package dir basename + apps.jsonc + convention).
        assert_eq!(
            apps[0].name, "hub",
            "name derived from package dir basename"
        );
        assert_eq!(apps[0].route, "/", "route from apps.jsonc");
        assert!(
            apps[0].source.ends_with("/hub/.deck"),
            "source = convention <abs-pkg-dir>/.deck: {}",
            apps[0].source
        );
        // Conventional defaults (no file ⇒ Default).
        assert_eq!(apps[0].render, RenderMode::Csr);
        assert!(apps[0].inject_globals.is_empty());
        assert!(apps[0].entry.is_none());
    }

    /// Unscoped packages resolve via the same node_modules path layout
    /// (`node_modules/<name>`), no `@scope/` prefix.
    #[test]
    fn spaces_unscoped_resolves_via_node_modules() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "spaces", "package": "spaces", "route": "/space/{id}" }
            ]
        }"#;
        let spaces_deck = r#"{ "injectGlobals": { "DECK_APP_ID": "$id" } }"#;
        let dir = scaffold(apps_jsonc, &[("spaces", Some(spaces_deck))]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "spaces");

        assert_eq!(
            apps[0].route, "/space/{id}",
            "route from apps.jsonc, not the file"
        );
        assert!(apps[0].source.ends_with("/spaces/.deck"));
        assert_eq!(
            apps[0]
                .inject_globals
                .get("DECK_APP_ID")
                .map(String::as_str),
            Some("$id"),
        );
        assert!(apps[0].entry.is_none());
    }

    /// The legacy hub (deck-jsonc-simplify BEFORE) had `name`/`route`/
    /// `source` on the file. The cutover makes them unknown to the new
    /// schema (serde drops them silently) and the host fills them in
    /// from apps.jsonc + the package dir + convention. A legacy file
    /// is harmless — the new fields are the only ones the file can
    /// carry.
    #[test]
    fn legacy_per_app_deck_fields_are_dropped_silently() {
        let legacy_hub_deck =
            r#"{ "name": "hub", "route": "/legacy", "source": "legacy/path", "render": "csr" }"#;
        let apps_jsonc = r#"{
            "apps": [ { "name": "hub", "package": "@ship/hub", "route": "/" } ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", Some(legacy_hub_deck))]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        // Host-bound (apps.jsonc + dir basename + convention), NOT from the file.
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].route, "/", "host wins; legacy `route` is dropped");
        assert!(
            apps[0].source.ends_with("/hub/.deck"),
            "convention wins; legacy `source` is dropped: got {}",
            apps[0].source,
        );
    }

    /// Both apps load — the host reads `route` from apps.jsonc, the per-
    /// app files carry their own non-conventional config (the hub's
    /// entry bootstrap, the spaces' URL-capture).
    #[test]
    fn loads_per_app_deck_jsonc_for_hub_and_spaces() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "package": "@ship/hub", "route": "/" },
                { "name": "spaces", "package": "spaces", "route": "/space/{id}" }
            ]
        }"#;
        let hub_deck = r#"{ "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship", "ops"] } }"#;
        let spaces_deck = r#"{ "injectGlobals": { "DECK_APP_ID": "$id" } }"#;
        let dir = scaffold(
            apps_jsonc,
            &[("@ship/hub", Some(hub_deck)), ("spaces", Some(spaces_deck))],
        );
        let apps = load(dir.path());
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].route, "/");
        assert!(apps[0].entry.is_some(), "hub keeps its entry bootstrap");
        assert_eq!(apps[1].name, "spaces");
        assert_eq!(apps[1].route, "/space/{id}");
        assert_eq!(
            apps[1]
                .inject_globals
                .get("DECK_APP_ID")
                .map(String::as_str),
            Some("$id")
        );
    }

    /// The gate's round-trip: the hub's loaded config carries `name`/
    /// `route`/`source` from apps.jsonc + the package dir basename +
    /// convention, with `render` defaulting to csr (the implicit mode
    /// every app served in before) and `entry` populated from the file.
    #[test]
    fn round_trip_matches_expected_host_bound_values() {
        let apps_jsonc = r#"{
            "apps": [ { "name": "hub", "package": "@ship/hub", "route": "/" } ]
        }"#;
        let hub_deck = r#"{ "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship", "ops"] } }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", Some(hub_deck))]);
        let apps = load(dir.path());
        let want = AppServeConfig {
            name: "hub".to_string(),
            route: "/".to_string(),
            source: dir
                .path()
                .join("node_modules")
                .join("@ship")
                .join("hub")
                .join(".deck")
                .to_string_lossy()
                .to_string(),
            inject_globals: std::collections::BTreeMap::new(),
            ops_project: None,
            render: RenderMode::Csr,
            entry: Some(deck::config::EntryBootstrap {
                clients_module: "./src/lib/client.ts".to_string(),
                expose_names: vec!["ship".to_string(), "ops".to_string()],
            }),
        };
        assert_eq!(apps, vec![want]);
    }

    /// JSONC comments strip cleanly in both files; the per-app file
    /// only carries non-conventional fields post-cutover.
    #[test]
    fn jsonc_comments_stripped_in_both_files() {
        let apps_jsonc = r#"// orchestration list
            { "apps": [ { "name": "hub", "package": "@ship/hub", "route": "/", }, ], }"#;
        let hub_deck = r#"// the hub defines the hub
            { "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship"] } }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", Some(hub_deck))]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].route, "/", "comment-stripped route from apps.jsonc");
        assert!(
            apps[0].entry.is_some(),
            "comment-stripped entry from deck.jsonc"
        );
    }

    /// The registry is usable through the `AppRegistry` trait object —
    /// the seam deck declares.
    #[test]
    fn registry_enumerates_via_trait_object() {
        let apps_jsonc = r#"{
            "apps": [ { "name": "hub", "package": "@ship/hub", "route": "/" } ]
        }"#;
        let hub_deck =
            r#"{ "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship"] } }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", Some(hub_deck))]);
        let reg: Box<dyn AppRegistry> = Box::new(ShipAppRegistry::new(
            dir.path().to_path_buf(),
            dir.path().join("apps.jsonc"),
        ));
        assert_eq!(reg.serve_configs().len(), 1);
    }

    /// Mismatched name in apps.jsonc vs package dir basename: the
    /// directory basename wins (it's the URL namespace). Loader logs
    /// a warning; the loaded config carries the basename.
    #[test]
    fn apps_jsonc_name_mismatch_falls_back_to_dir_basename() {
        let apps_jsonc = r#"{
            "apps": [ { "name": "not-hub", "package": "@ship/hub", "route": "/" } ]
        }"#;
        let hub_deck =
            r#"{ "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship"] } }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", Some(hub_deck))]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "hub", "directory basename is authoritative");
        assert!(apps[0].source.ends_with("/hub/.deck"));
    }

    // ── AppSource::NodeModules resolution ────────────────────────────────

    /// `resolve_node_modules` walks `<base>/node_modules/@scope/name` for
    /// scoped packages — the Phase 1 dev path.
    #[test]
    fn resolve_node_modules_scoped_package() {
        let dir = scaffold("{}", &[("@ship/hub", None)]);
        let resolved = resolve_node_modules(dir.path(), "@ship/hub").unwrap();
        assert_eq!(
            resolved,
            dir.path().join("node_modules").join("@ship").join("hub")
        );
        assert!(resolved.is_dir());
    }

    /// Unscoped packages resolve to `<base>/node_modules/<name>` (no
    /// `@scope/` prefix).
    #[test]
    fn resolve_node_modules_unscoped_package() {
        let dir = scaffold("{}", &[("spaces", None)]);
        let resolved = resolve_node_modules(dir.path(), "spaces").unwrap();
        assert_eq!(resolved, dir.path().join("node_modules").join("spaces"));
        assert!(resolved.is_dir());
    }

    /// A missing package is an error (typo in `apps.jsonc`, missed
    /// `bun install`, or a stale `node_modules/`). The loader must
    /// surface that to the operator at startup, not at first request.
    #[test]
    fn resolve_node_modules_missing_package_errors() {
        let dir = scaffold("{}", &[]);
        let err = resolve_node_modules(dir.path(), "@ship/hub").unwrap_err();
        assert!(
            err.to_string().contains("@ship/hub"),
            "error must name the package: {err}"
        );
    }

    /// A scoped name without a `/` (e.g. `@scope` with no name) is
    /// rejected — node modules resolution requires `@scope/name`.
    #[test]
    fn resolve_node_modules_malformed_scoped_name_errors() {
        let dir = scaffold("{}", &[]);
        let err = resolve_node_modules(dir.path(), "@scope").unwrap_err();
        assert!(
            err.to_string().contains("`@scope/name`"),
            "error must explain the format: {err}"
        );
    }

    /// The `source` field on `apps.jsonc` overrides the `<dir>/.deck`
    /// convention for non-conventional layouts.
    #[test]
    fn source_override_changes_convention() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "package": "@ship/hub", "route": "/", "source": "hub/dist" }
            ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("@ship/hub", None)]);
        let apps = load(dir.path());
        assert_eq!(apps.len(), 1);
        assert!(
            apps[0].source.ends_with("/hub/dist"),
            "explicit source wins over convention: {}",
            apps[0].source
        );
    }

    /// `AppSource::Cdn` is declared but not constructable today —
    /// constructing it via `resolve_dir` returns a Phase-2 error so
    /// the future path is wired but inert.
    #[test]
    fn app_source_cdn_variant_resolve_errors() {
        let src = AppSource::Cdn {
            url: "https://example.invalid/hub.tgz".to_string(),
            hash: "deadbeef".to_string(),
        };
        let err = src.resolve_dir().unwrap_err();
        assert!(
            err.to_string().contains("Phase 2"),
            "cdn must declare it's Phase 2: {err}"
        );
    }

    /// `AppSource::Registry` is declared but not constructable today.
    #[test]
    fn app_source_registry_variant_resolve_errors() {
        let src = AppSource::Registry {
            url: "https://registry.invalid/".to_string(),
            package: "@ship/hub".to_string(),
        };
        let err = src.resolve_dir().unwrap_err();
        assert!(
            err.to_string().contains("Phase 3"),
            "registry must declare it's Phase 3: {err}"
        );
    }
}
