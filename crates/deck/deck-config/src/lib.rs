//! deck-config — parses `deck.jsonc` into [`DeckConfig`].
//!
//! `deck.jsonc` is the single source of truth for Deck's consumer bundle
//! manifest (peer of `package.json`). Read at build-time and daemon-startup,
//! never per-request. Framework singletons (sui-runtime/dom/sdk-client/
//! components) are hardcoded inside the bundler — NOT declared here.
//! `deck.jsonc` targets are consumer bundles only.
//!
//! Resolution: host-supplied [`DeckConfig`] (shipd calls
//! [`DeckConfig::load`]) > root `deck.jsonc` > built-in defaults
//! (conventional `views/` roots + singletons, so a zero-config host
//! like deck-demo works).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ─── types ────────────────────────────────────────────────────────────────────

/// Output kind for a [`TargetDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    /// A reusable component pack (default).
    #[default]
    ComponentPack,
    /// A lazily-loaded component pack (heavy chunk, code-split).
    LazyPack,
    /// A full application entry.
    App,
}

/// One consumer bundle target declared in `deck.jsonc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetDecl {
    /// Logical name, e.g. `sui-components/sui-chrome`.
    pub name: String,
    /// Entry module path relative to the workspace root.
    pub entry: PathBuf,
    /// Module specifiers that stay external (not bundled).
    #[serde(default)]
    pub externals: Vec<String>,
    /// Output kind. Defaults to `component-pack`.
    #[serde(default)]
    pub kind: TargetKind,
    /// Override the output filename. Defaults to `<name>.js`.
    #[serde(default)]
    pub out_file: Option<String>,
}

/// One scaffold declaration — a pre-wired HTML shell that Deck injects
/// consumer scripts into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldDecl {
    /// Logical scaffold name, e.g. `shell`.
    pub name: String,
    /// URL route pattern, e.g. `/` or `/space/{id}`.
    pub route: String,
    /// The scaffold template identifier.
    pub source: String,
    /// URL prefix under which the scaffold's assets are served.
    #[serde(default)]
    pub asset_prefix: String,
    /// Global JS variables injected into the scaffold's HTML.
    #[serde(default)]
    pub inject_globals: BTreeMap<String, String>,
}

/// Identifies the first-party application embedded in this host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstPartyApp {
    /// Source root of the embedded app, e.g. `apps/shell`.
    pub root: PathBuf,
    /// Logical label, e.g. `app:shell`.
    pub label: String,
}

/// The parsed contents of `deck.jsonc`.
///
/// All fields carry `#[serde(default)]` so a minimal (or missing) `deck.jsonc`
/// falls back gracefully to [`DeckConfig::defaults`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeckConfig {
    /// Consumer bundle targets (NOT the framework singletons — those are
    /// hardcoded inside deck-bundler and never declared here).
    pub targets: Vec<TargetDecl>,
    /// Scaffold declarations — pre-wired HTML shells.
    pub scaffolds: Vec<ScaffoldDecl>,
    /// Source roots to scan for views. Defaults to `["."]` (workspace root).
    pub source_roots: Vec<PathBuf>,
    /// Directories the bundler watches for live-reload. Defaults to `[]`.
    pub watch: Vec<PathBuf>,
    /// Root directory where host-owned assets land (e.g. `apps/shipd/assets`).
    pub host_assets_root: PathBuf,
    /// First-party application embedded in this host, if any.
    pub first_party_app: Option<FirstPartyApp>,
    /// Declared component packs (per-project component scope).
    ///
    /// When present and non-empty, component resolution is SCOPED to ONLY these
    /// packs: the per-view resolver's tag index is filtered down to tags a
    /// declared pack provides (matched by `package.json` `name`, e.g.
    /// `"@ship/sui-primitives"`). A tag from an undeclared pack resolves to a
    /// no-op (it never bundles) and the build emits an author diagnostic naming
    /// the tag and the pack that provides it. When absent or empty, resolution
    /// is GLOBAL — every workspace pack's tags are resolvable (zero behavior
    /// change for existing projects).
    #[serde(default)]
    pub packs: Vec<String>,
}

impl Default for DeckConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

impl DeckConfig {
    /// Built-in defaults — a zero-config host (e.g. deck-demo) gets these
    /// when no `deck.jsonc` is present or when only some fields are declared.
    pub fn defaults() -> Self {
        Self {
            targets: Vec::new(),
            scaffolds: Vec::new(),
            source_roots: vec![PathBuf::from(".")],
            watch: Vec::new(),
            host_assets_root: PathBuf::from(".deck"),
            first_party_app: None,
            packs: Vec::new(),
        }
    }

    /// Load `<workspace_root>/deck.jsonc`.
    ///
    /// Falls back to [`DeckConfig::defaults`] when the file is absent (zero-
    /// config hosts) or cannot be parsed (logs a warning — never panics).
    /// An unreadable file (permission error) is treated the same as absent.
    pub fn load(workspace_root: &Path) -> Self {
        let path = workspace_root.join("deck.jsonc");
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                // File absent or unreadable — zero-config host, use defaults.
                return Self::defaults();
            }
        };
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(&raw);
        match serde_json::from_str::<Self>(&stripped) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "deck-config: failed to parse deck.jsonc, using defaults"
                );
                Self::defaults()
            }
        }
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Minimal example deck.jsonc that exercises all declared fields.
    /// The shell-chrome target was removed 2026-06-06: cut 1 deleted
    /// `apps/shell/src/components/index.ts`, and the per-tag resolver in
    /// `sui_plugin.rs` already covers shell chrome via the catalog's
    /// tag-source walk — the chrome target was dead. The `targets` list
    /// is now 3 (marketing, arcade, pdf); tests in this file assert that
    /// count. The chrome sources still live in `apps/shell/src/components/`
    /// and views pick them up via `@sui-tag/<tag>`.
    const EXAMPLE_JSONC: &str = r#"{
  // Consumer bundle targets (NOT the framework singletons).
  "targets": [
    {
      "name": "sui-marketing",
      "kind": "component-pack",
      "entry": "apps/site/src/marketing/marketing-bundle.ts",
      "externals": ["@ship/sui-runtime", "@ship/sui-dom"]
    },
    {
      "name": "sui-arcade",
      "kind": "lazy-pack",
      "entry": "packages/components/ship/arcade/src/index.ts",
      "externals": ["@ship/sui-runtime", "@ship/sui-dom"]
    },
    {
      "name": "sui-pdf",
      "kind": "lazy-pack",
      "entry": "packages/components/ship/pdf/src/index.ts",
      "externals": ["@ship/sui-runtime", "@ship/sui-dom"],
      "outFile": "sui-pdf.js" // explicit out_file override
    }
  ],
  "scaffolds": [
    {
      "name": "shell",
      "route": "/",
      "source": "hub-scaffold",
      "assetPrefix": "/shell/_assets/"
    },
    {
      "name": "space",
      "route": "/space/{id}",
      "source": "space-scaffold",
      "assetPrefix": "/space/_assets/",
      "injectGlobals": { "DECK_APP_ID": "$id" }
    }
  ],
  "sourceRoots": ["."],
  "watch": ["packages", "apps/shell/src"],
  "hostAssetsRoot": ".deck",
  "firstPartyApp": { "root": "apps/shell", "label": "app:shell" }
}"#;

    #[test]
    fn parse_example_jsonc() {
        let cfg: DeckConfig =
            serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");

        assert_eq!(cfg.targets.len(), 3);

        let arcade = &cfg.targets[1];
        assert_eq!(arcade.kind, TargetKind::LazyPack);

        let pdf = &cfg.targets[2];
        assert_eq!(pdf.out_file.as_deref(), Some("sui-pdf.js"));

        assert_eq!(cfg.scaffolds.len(), 2);
        let space = &cfg.scaffolds[1];
        assert_eq!(space.route, "/space/{id}");
        assert_eq!(space.inject_globals.get("DECK_APP_ID").map(String::as_str), Some("$id"));

        assert_eq!(cfg.source_roots, vec![PathBuf::from(".")]);
        assert_eq!(cfg.watch, vec![PathBuf::from("packages"), PathBuf::from("apps/shell/src")]);
        assert_eq!(cfg.host_assets_root, PathBuf::from(".deck"));

         let app = cfg.first_party_app.as_ref().unwrap();
        assert_eq!(app.root, PathBuf::from("apps/shell"));
        assert_eq!(app.label, "app:shell");
    }

    #[test]
    fn defaults_round_trip() {
        let d = DeckConfig::defaults();
        // Serialize and deserialize — must be lossless.
        let json = serde_json::to_string(&d).unwrap();
        let d2: DeckConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
        assert!(d.targets.is_empty());
        assert!(d.scaffolds.is_empty());
        assert_eq!(d.source_roots, vec![PathBuf::from(".")]);
        assert!(d.watch.is_empty());
        assert_eq!(d.host_assets_root, PathBuf::from(".deck"));
        assert!(d.first_party_app.is_none());
        assert!(d.packs.is_empty(), "defaults declare no packs → global resolution");
    }

    /// `packs` is optional and additive — an absent field parses to empty,
    /// which the bundler reads as "global" (every workspace pack resolvable).
    #[test]
    fn packs_absent_parses_to_empty() {
        let cfg: DeckConfig =
            serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");
        assert!(cfg.packs.is_empty(), "no packs field ⇒ empty ⇒ global behavior");
    }

    /// A declared `packs` array parses into the scoped set.
    #[test]
    fn packs_present_parses_declared_set() {
        let json = r#"{ "packs": ["@ship/sui-primitives", "deck-demo-app"] }"#;
        let cfg: DeckConfig =
            serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(json)).unwrap();
        assert_eq!(cfg.packs, vec!["@ship/sui-primitives", "deck-demo-app"]);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        // No deck.jsonc in this directory.
        let cfg = DeckConfig::load(tmp.path());
        assert_eq!(cfg, DeckConfig::defaults());
    }

    #[test]
    fn parse_error_returns_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("deck.jsonc"), b"{ invalid json !!!").unwrap();
        // Should not panic; must return defaults with a warn log (not testable here).
        let cfg = DeckConfig::load(tmp.path());
        assert_eq!(cfg, DeckConfig::defaults());
    }

    #[test]
    fn target_kind_kebab_case_roundtrip() {
        let json = r#"{"name":"x","entry":"x.ts","kind":"lazy-pack","externals":[]}"#;
        let t: TargetDecl = serde_json::from_str(json).unwrap();
        assert_eq!(t.kind, TargetKind::LazyPack);
        let out = serde_json::to_string(&t).unwrap();
        assert!(out.contains("lazy-pack"));
    }
}
