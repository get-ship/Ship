//! shipd's [`AppRegistry`] impl — reads `apps.jsonc` (the host's app
//! orchestration list) and loads each app's `apps/<app>/deck.jsonc` into an
//! [`AppServeConfig`].
//!
//! `apps.jsonc` is the host (shipd) config; deck (L1) knows nothing about it.
//! Per the app-registration port discipline (sibling to `DataPort`): deck
//! declares the RESULT it needs (the `Vec<AppServeConfig>` to mount); shipd
//! owns the MECHANISM and binds the file locations HERE — deck names no path.
//!
//! ## `apps.jsonc` — orchestration only
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
//! Each entry names an app and the dir holding its `deck.jsonc` (`root`). It
//! carries NO per-app `source`/`viewOutput`/`render`/`injectGlobals` — those
//! moved to the app's `deck.jsonc` (the app defines the app). `route` here is
//! an orchestration build hint (the Justfile build orchestrator reads it to
//! pick the framework-bundling root app); shipd serves using the route in each
//! app's `deck.jsonc`, which is authoritative. `name` is the orchestration
//! handle; the app's own `deck.jsonc` `name` is authoritative for serving.
//!
//! ## `apps/<app>/deck.jsonc` — the app's serve config
//!
//! ```jsonc
//! { "name": "hub", "route": "/", "source": "apps/hub/.deck", "render": "csr" }
//! ```
//!
//! Parsed by [`AppServeConfig::parse_deck_jsonc`] — deck-config owns the schema
//! + the JSONC parser; shipd owns the file locations.

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

/// One orchestration entry: an app's handle (`name`) + the dir holding its
/// `deck.jsonc` (`root`). `route` and any other per-app field present in the
/// JSON are IGNORED here (serde drops unknown fields) — the app's `deck.jsonc`
/// is the source of truth.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppEntry {
    name: String,
    root: String,
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

/// Read the orchestration list and load each app's `deck.jsonc`.
///
/// Missing `apps.jsonc`: empty list (serve no app — a clean 404, not a panic).
/// A present-but-malformed `apps.jsonc`, or any declared app whose `deck.jsonc`
/// is missing/malformed, is an error so the dev sees the failure at startup,
/// not at first request.
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
        let deck_path = project_root.join(&entry.root).join("deck.jsonc");
        let cfg = load_app_deck(&deck_path)
            .with_context(|| format!("app '{}' ({})", entry.name, deck_path.display()))?;
        if cfg.name != entry.name {
            tracing::warn!(
                orchestration = %entry.name,
                deck = %cfg.name,
                "apps.jsonc name differs from deck.jsonc name — using deck.jsonc (authoritative)"
            );
        }
        apps.push(cfg);
    }
    Ok(apps)
}

/// Load one app's `deck.jsonc` into its serve config.
fn load_app_deck(path: &Path) -> Result<AppServeConfig> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    AppServeConfig::parse_deck_jsonc(&raw).with_context(|| format!("parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deck::RenderMode;

    /// Scaffold a temp project: `apps.jsonc` (orchestration) plus a
    /// `deck.jsonc` for each `(root, body)`. Returns the project root.
    fn scaffold(apps_jsonc: &str, decks: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("apps.jsonc"), apps_jsonc).unwrap();
        for (root, body) in decks {
            let app_dir = dir.path().join(root);
            std::fs::create_dir_all(&app_dir).unwrap();
            std::fs::write(app_dir.join("deck.jsonc"), body).unwrap();
        }
        dir
    }

    const HUB_DECK: &str = r#"{ "name": "hub", "route": "/", "source": "apps/hub/.deck", "render": "csr" }"#;
    const SPACES_DECK: &str = r#"{
        "name": "spaces", "route": "/space/{id}", "source": "apps/spaces/.deck",
        "render": "csr", "injectGlobals": { "DECK_APP_ID": "$id" }
    }"#;

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

    #[test]
    fn loads_per_app_deck_jsonc() {
        let apps_jsonc = r#"{
            "apps": [
                { "name": "hub", "root": "apps/hub", "route": "/" },
                { "name": "spaces", "root": "apps/spaces", "route": "/space/{id}" }
            ]
        }"#;
        let dir = scaffold(
            apps_jsonc,
            &[("apps/hub", HUB_DECK), ("apps/spaces", SPACES_DECK)],
        );
        let apps = load(&dir);
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].render, RenderMode::Csr);
        assert_eq!(apps[1].name, "spaces");
        assert_eq!(apps[1].route, "/space/{id}");
        assert_eq!(
            apps[1].inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
    }

    /// The gate's round-trip: the hub's serve config loaded from its
    /// `deck.jsonc` carries the SAME values the monolithic `apps.jsonc`
    /// declared before the split (name/route/source), with `render`
    /// defaulting to csr (the implicit mode it served in before).
    #[test]
    fn round_trip_matches_old_apps_jsonc_values() {
        // The old monolithic apps.jsonc hub entry, now reconstructed from the
        // orchestration list + per-app deck.jsonc.
        let dir = scaffold(
            r#"{ "apps": [ { "name": "hub", "root": "apps/hub", "route": "/" } ] }"#,
            &[("apps/hub", HUB_DECK)],
        );
        let apps = load(&dir);
        let want = AppServeConfig {
            name: "hub".to_string(),
            route: "/".to_string(),
            source: "apps/hub/.deck".to_string(),
            view_output: None,
            inject_globals: std::collections::BTreeMap::new(),
            render: RenderMode::Csr,
        };
        assert_eq!(apps, vec![want]);
    }

    /// A declared app whose `deck.jsonc` is missing surfaces as an error from
    /// the fallible loader (caught + degraded to empty by `serve_configs`).
    #[test]
    fn missing_deck_jsonc_is_error() {
        let dir = scaffold(
            r#"{ "apps": [ { "name": "hub", "root": "apps/hub" } ] }"#,
            &[], // no deck.jsonc written for apps/hub
        );
        let err = load_serve_configs(dir.path(), &dir.path().join("apps.jsonc"));
        assert!(err.is_err(), "missing per-app deck.jsonc must error");
        // The registry degrades that error to an empty list (clean 404).
        let reg = ShipAppRegistry::new(dir.path().to_path_buf(), dir.path().join("apps.jsonc"));
        assert!(reg.serve_configs().is_empty());
    }

    #[test]
    fn jsonc_comments_stripped_in_both_files() {
        let apps_jsonc = r#"// orchestration list
            { "apps": [ { "name": "hub", "root": "apps/hub", "route": "/", }, ], }"#;
        let hub_deck = r#"// the hub defines the hub
            { "name": "hub", "route": "/", "source": "apps/hub/.deck", "render": "csr", }"#;
        let dir = scaffold(apps_jsonc, &[("apps/hub", hub_deck)]);
        let apps = load(&dir);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].source, "apps/hub/.deck");
    }

    /// The registry is usable through the `AppRegistry` trait object — the
    /// seam deck declares.
    #[test]
    fn registry_enumerates_via_trait_object() {
        let dir = scaffold(
            r#"{ "apps": [ { "name": "hub", "root": "apps/hub", "route": "/" } ] }"#,
            &[("apps/hub", HUB_DECK)],
        );
        let reg: Box<dyn AppRegistry> =
            Box::new(ShipAppRegistry::new(dir.path().to_path_buf(), dir.path().join("apps.jsonc")));
        assert_eq!(reg.serve_configs().len(), 1);
    }
}
