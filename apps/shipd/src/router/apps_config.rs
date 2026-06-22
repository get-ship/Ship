//! shipd's [`AppRegistry`] impl — reads `apps.jsonc` (the host's app
//! orchestration list) and loads each app's `apps/<app>/deck.jsonc` into an
//! [`AppServeConfig`].
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
//!     { "name": "hub",    "root": "apps/hub",    "route": "/" },
//!     { "name": "spaces", "root": "apps/spaces", "route": "/space/{id}" }
//!   ]
//! }
//! ```
//!
//! Each entry is the host's bind: the `name` is the orchestration handle,
//! `root` is the dir holding the app, and `route` is where shipd mounts it.
//! The app's `deck.jsonc` no longer carries `name`/`route`/`source` —
//! those are HOST concerns. `name` here must match the directory basename
//! (the framework derives identity from the dir; the apps.jsonc handle is
//! for log lines + matching).
//!
//! `route` is AUTHORITATIVE for serving. The app's `deck.jsonc` cannot
//! override it — the host composes many apps; the app does not decide
//! where it mounts.
//!
//! ## `apps/<app>/deck.jsonc` — the app's NON-conventional config (OPTIONAL)
//!
//! An app whose config is fully conventional needs NO `deck.jsonc` —
//! the loader returns `AppServeConfig::default()`. When the file IS
//! present, it carries only the non-conventional pieces (the rendering
//! mode, optional URL-capture globals, optional `entry.gen.ts`
//! bootstrap).
//!
//! Parsed by [`AppServeConfig::parse_deck_jsonc`] — deck-config owns the
//! schema + the JSONC parser; shipd owns the file locations.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use deck::config::strip_jsonc_comments;
use deck::AppServeConfig;
use deck_core::AppRegistry;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppsConfigFile {
    #[serde(default)]
    apps: Vec<AppEntry>,
}

/// One orchestration entry: the host's bind for one app. `route` is
/// AUTHORITATIVE for serving — the app's `deck.jsonc` cannot override it.
/// `name` is the orchestration handle (must match the directory basename,
/// since the framework derives the URL namespace from the dir); `root`
/// is the dir holding the app's authored source + optional `deck.jsonc`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppEntry {
    name: String,
    root: String,
    route: String,
}
/// shipd's app registry. Binds the project root (where `apps/<app>/` dirs live)
/// and the `apps.jsonc` orchestration file, then enumerates apps by loading
/// each one's `deck.jsonc`. Mirrors the `DataPort` discipline — the host binds
/// its own state inside the impl; deck names no path.
pub struct ShipAppRegistry {
    project_root: PathBuf,
    apps_config: PathBuf,
}

impl ShipAppRegistry {
    pub fn new(project_root: PathBuf, apps_config: PathBuf) -> Self {
        Self {
            project_root,
            apps_config,
        }
    }
}

impl AppRegistry for ShipAppRegistry {
    fn serve_configs(&self) -> Vec<AppServeConfig> {
        match load_serve_configs(&self.project_root, &self.apps_config) {
            Ok(apps) => {
                tracing::info!(
                    path = %self.apps_config.display(),
                    count = apps.len(),
                    "shipd: loaded apps from apps.jsonc + per-app deck.jsonc"
                );
                apps
            }
            Err(e) => {
                tracing::warn!(
                    path = %self.apps_config.display(),
                    error = %e,
                    "shipd: app registry failed to load — serving no apps"
                );
                Vec::new()
            }
        }
    }
}

/// Read the orchestration list and load each app's `deck.jsonc` (if any).
///
/// Missing `apps.jsonc`: empty list (serve no app — a clean 404, not a panic).
/// A present-but-malformed `apps.jsonc`, or any declared app whose `root`
/// directory is missing/malformed, is an error so the dev sees the failure
/// at startup, not at first request. A missing `apps/<root>/deck.jsonc`
/// is NOT an error — the loader returns `AppServeConfig::default()` and
/// fills in the conventional fields from the orchestration bind.
///
/// For every loaded app the host fills in three fields the per-app file
/// no longer carries:
///
/// - `name` — derived from the directory basename (`apps/hub` ⇒ `hub`).
///   This MUST match the orchestration handle (`entry.name`); a mismatch
///   is logged and the basename wins (the directory is the source of
///   truth for the URL namespace).
/// - `route` — copied from the orchestration entry. The file cannot
///   override it; the host composes many apps.
/// - `source` — convention: `<root>/.deck` (the bundler's default
///   `out_dir`). One way to do things; apps that need a different
///   layout are out of scope for the deck-jsonc-simplify cutover.
fn load_serve_configs(project_root: &Path, apps_config: &Path) -> Result<Vec<AppServeConfig>> {
    let raw = match std::fs::read_to_string(apps_config) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(anyhow::anyhow!("read {}: {e}", apps_config.display())),
    };
    let parsed: AppsConfigFile = serde_json::from_str(&strip_jsonc_comments(&raw))
        .with_context(|| format!("parse {}", apps_config.display()))?;
    let mut apps = Vec::with_capacity(parsed.apps.len());
    for entry in parsed.apps {
        let app_dir = project_root.join(&entry.root);
        // Derive the app's name from the directory basename. This is
        // the AUTHORITATIVE identity — the bundler writes built assets
        // under `/<name>/…` and the daemon serves them at that path.
        let dir_name = app_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "apps.jsonc entry `{}`: cannot derive directory basename from `{}`",
                    entry.name,
                    app_dir.display()
                )
            })?
            .to_string();
        if dir_name != entry.name {
            tracing::warn!(
                orchestration = %entry.name,
                directory = %dir_name,
                "apps.jsonc name differs from directory basename — using directory basename (authoritative)",
            );
        }
        let convention_source = format!("{}/.deck", entry.root.trim_end_matches('/'));
        // Load the per-app `deck.jsonc` — OPTIONAL. Absent file
        // produces the conventional defaults; the host fills in the
        // orchestration bind below.
        let deck_path = app_dir.join("deck.jsonc");
        let mut cfg = if deck_path.exists() {
            load_app_deck(&deck_path)
                .with_context(|| format!("app '{}' ({})", entry.name, deck_path.display()))?
        } else {
            AppServeConfig::default()
        };
        cfg.name = dir_name;
        cfg.route = entry.route;
        cfg.source = convention_source;
        apps.push(cfg);
    }
    Ok(apps)
}

/// Load one app's `deck.jsonc` into its serve config. Used for the
/// (now optional) per-app file. Absent files fall through to
/// `AppServeConfig::default()`; the host binds name/route/source
/// outside this helper.
fn load_app_deck(path: &Path) -> Result<AppServeConfig> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    AppServeConfig::parse_deck_jsonc(&raw).with_context(|| format!("parse {}", path.display()))
}
#[cfg(test)]
mod tests {
    use super::*;
    use deck::RenderMode;

    /// Scaffold a temp project: `apps.jsonc` (orchestration) plus an
    /// optional `deck.jsonc` for each `(root, body)` (None ⇒ file
    /// absent). Returns the project root.
    fn scaffold(apps_jsonc: &str, decks: &[(&str, Option<&str>)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("apps.jsonc"), apps_jsonc).unwrap();
        for (root, body) in decks {
            let app_dir = dir.path().join(root);
            std::fs::create_dir_all(&app_dir).unwrap();
            if let Some(b) = body {
                std::fs::write(app_dir.join("deck.jsonc"), b).unwrap();
            }
        }
        dir
    }

    /// New-shape hub deck: only the entry bootstrap. `name`/`route`/`source`
    /// are HOST concerns (apps.jsonc), not on the file.
    const HUB_DECK_NEW: &str = r#"{ "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship", "ops"] } }"#;
    /// New-shape spaces deck: only injectGlobals. URL-capture stays on the file.
    const SPACES_DECK_NEW: &str = r#"{ "injectGlobals": { "DECK_APP_ID": "$id" } }"#;

    fn load(dir: &tempfile::TempDir) -> Vec<AppServeConfig> {
        load_serve_configs(dir.path(), &dir.path().join("apps.jsonc")).unwrap()
    }

    #[test]
    fn empty_apps_returns_empty() {
        let dir = scaffold("{}", &[]);
        assert!(load(&dir).is_empty());
    }

    #[test]
    fn missing_apps_jsonc_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let apps = load_serve_configs(dir.path(), &dir.path().join("nope.jsonc")).unwrap();
        assert!(apps.is_empty());
    }

    /// A first-party app whose config is fully conventional needs NO
    /// `deck.jsonc`. The loader returns `AppServeConfig::default()` and
    /// the host binds `name`/`route`/`source` from apps.jsonc + the
    /// directory basename. This is the post-cutover hub shape.
    #[test]
    fn hub_no_deck_jsonc_loads_with_conventional_defaults() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "root": "apps/hub", "route": "/" }
            ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("apps/hub", None)]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        // Host-derived (apps.jsonc + dir basename + convention).
        assert_eq!(apps[0].name, "hub", "name derived from dir basename");
        assert_eq!(apps[0].route, "/", "route from apps.jsonc");
        assert_eq!(apps[0].source, "apps/hub/.deck", "source = convention <root>/.deck");
        // Conventional defaults (no file ⇒ Default).
        assert_eq!(apps[0].render, RenderMode::Csr);
        assert!(apps[0].inject_globals.is_empty());
        assert!(apps[0].entry.is_none());
    }

    /// Spaces keeps only its genuinely non-conventional bits in
    /// `deck.jsonc` (the URL-capture → `window.DECK_APP_ID` injection).
    /// `name`/`route`/`source` come from apps.jsonc + convention.
    #[test]
    fn spaces_loads_with_inject_globals_from_file_and_route_from_apps_jsonc() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "spaces", "root": "apps/spaces", "route": "/space/{id}" }
            ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("apps/spaces", Some(SPACES_DECK_NEW))]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "spaces");
        assert_eq!(apps[0].route, "/space/{id}", "route from apps.jsonc, not the file");
        assert_eq!(apps[0].source, "apps/spaces/.deck");
        assert_eq!(
            apps[0].inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id"),
        );
        assert!(apps[0].entry.is_none());
    }

    /// The legacy hub (deck-jsonc-simplify BEFORE) had `name`/`route`/
    /// `source` on the file. The cutover makes them unknown to the new
    /// schema (serde drops them silently) and the host fills them in
    /// from apps.jsonc + convention. A legacy file is harmless — the
    /// new fields are the only ones the file can carry.
    #[test]
    fn legacy_per_app_deck_fields_are_dropped_silently() {
        let legacy_hub_deck = r#"{ "name": "hub", "route": "/legacy", "source": "legacy/path", "render": "csr" }"#;
        let apps_jsonc = r#"{
            "apps": [ { "name": "hub", "root": "apps/hub", "route": "/" } ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("apps/hub", Some(legacy_hub_deck))]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        // Host-bound (apps.jsonc + dir basename + convention), NOT from the file.
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].route, "/", "host wins; legacy `route` is dropped");
        assert_eq!(apps[0].source, "apps/hub/.deck", "convention wins; legacy `source` is dropped");
    }

    /// Both apps load — the host reads `route` from apps.jsonc, the per-
    /// app files carry their own non-conventional config (the hub's
    /// entry bootstrap, the spaces' URL-capture).
    #[test]
    fn loads_per_app_deck_jsonc_for_hub_and_spaces() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "root": "apps/hub", "route": "/" },
                { "name": "spaces", "root": "apps/spaces", "route": "/space/{id}" }
            ]
        }"#;
        let dir = scaffold(
            apps_jsonc,
            &[("apps/hub", Some(HUB_DECK_NEW)), ("apps/spaces", Some(SPACES_DECK_NEW))],
        );
        let apps = load(&dir);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].route, "/");
        assert!(apps[0].entry.is_some(), "hub keeps its entry bootstrap");
        assert_eq!(apps[1].name, "spaces");
        assert_eq!(apps[1].route, "/space/{id}");
        assert_eq!(
            apps[1].inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
    }

    /// The gate's round-trip: the hub's loaded config carries `name`/
    /// `route`/`source` from apps.jsonc + the directory basename +
    /// convention, with `render` defaulting to csr (the implicit mode
    /// every app served in before) and `entry` populated from the file.
    #[test]
    fn round_trip_matches_expected_host_bound_values() {
        let dir = scaffold(
            r#"{ "apps": [ { "name": "hub", "root": "apps/hub", "route": "/" } ] }"#,
            &[("apps/hub", Some(HUB_DECK_NEW))],
        );
        let apps = load(&dir);
        let want = AppServeConfig {
            name: "hub".to_string(),
            route: "/".to_string(),
            source: "apps/hub/.deck".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
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
            { "apps": [ { "name": "hub", "root": "apps/hub", "route": "/", }, ], }"#;
        let hub_deck = r#"// the hub defines the hub
            { "entry": { "clientsModule": "./src/lib/client.ts", "exposeNames": ["ship"] } }"#;
        let dir = scaffold(apps_jsonc, &[("apps/hub", Some(hub_deck))]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].route, "/", "comment-stripped route from apps.jsonc");
        assert!(apps[0].entry.is_some(), "comment-stripped entry from deck.jsonc");
    }

    /// The registry is usable through the `AppRegistry` trait object —
    /// the seam deck declares.
    #[test]
    fn registry_enumerates_via_trait_object() {
        let dir = scaffold(
            r#"{ "apps": [ { "name": "hub", "root": "apps/hub", "route": "/" } ] }"#,
            &[("apps/hub", Some(HUB_DECK_NEW))],
        );
        let reg: Box<dyn AppRegistry> =
            Box::new(ShipAppRegistry::new(dir.path().to_path_buf(), dir.path().join("apps.jsonc")));
        assert_eq!(reg.serve_configs().len(), 1);
    }

    /// Mismatched name in apps.jsonc vs directory basename: the
    /// directory basename wins (it's the URL namespace). Loader logs
    /// a warning; the loaded config carries the basename.
    #[test]
    fn apps_jsonc_name_mismatch_falls_back_to_dir_basename() {
        let apps_jsonc = r#"{
            "apps": [ { "name": "not-hub", "root": "apps/hub", "route": "/" } ]
        }"#;
        let dir = scaffold(apps_jsonc, &[("apps/hub", Some(HUB_DECK_NEW))]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "hub", "directory basename is authoritative");
        assert_eq!(apps[0].source, "apps/hub/.deck");
    }
}
