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

/// One app's serve configuration. The shape deck's per-app router consumes;
/// shipd (or any deck host) builds a `Vec<AppServeConfig>` from its
/// monorepo-level app list (`apps.jsonc` in the Ship repo; the host decides
/// the source). The framework does NOT know the list — that is the host's
/// concern.
///
/// Each app's `index.html` lives at `<source>/index.html` relative to the
/// project root (the dev's authored source tree). The bundler writes
/// `<project>/.deck/` for the built output; the dev points `source` there
/// (e.g. `apps/hub/.deck`). The framework reads via the project's resolver
/// so the path is repo-relative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppServeConfig {
    /// Logical app name. Used for log lines + the SPA-fallback lookup
    /// (which app serves a given multi-segment client route).
    pub name: String,
    /// URL route pattern. e.g. `/` (hub) or `/space/{id}` (spaces — the
    /// `{id}` is a path capture, accessible via `inject_globals` substitution).
    pub route: String,
    /// Path to the built app, relative to the project root.
    /// e.g. `apps/hub/.deck` or `apps/spaces/.deck`.
    pub source: String,
    /// URL prefix under which the app's assets are served.
    /// e.g. `/hub/_assets/`. The dev's `index.html` references
    /// `/assets/entry-…` which the framework rewrites to
    /// `<asset_prefix>entry-…` so multiple apps don't collide on
    /// `/assets/*`. Empty (default) = no rewrite; assets live at `/assets/`.
    #[serde(default)]
    pub asset_prefix: String,
    /// Globals injected as `<script>window.<key>=<value>;</script>` into
    /// the app's `index.html` before `</head>`. `$<param>` tokens in the
    /// value are substituted from the URL path captures declared in
    /// `route` (so for `route: "/space/{id}"` and an entry
    /// `("DECK_APP_ID", "$id")`, the injection is `window.DECK_APP_ID="<id>";`).
    #[serde(default)]
    pub inject_globals: BTreeMap<String, String>,
}

/// Back-compat alias — `ScaffoldDecl` was the pre-per-app-app name. The new
/// API surface uses `AppServeConfig`; the old type is preserved as a type
/// alias so deck's existing `ScaffoldDecl` callers (read from `deck.jsonc`
/// historically; now populated from `apps.jsonc` by the host) keep working
/// until the cutover commit migrates them. The two names have identical
/// shape; this is a pure rename.
#[deprecated(note = "renamed to AppServeConfig — the framework is per-app; \
    the host owns the list and passes Vec<AppServeConfig> to serve_apps()")]
pub type ScaffoldDecl = AppServeConfig;

/// Identifies the first-party application embedded in this host.
/// (RETIRED — every app is per-app now; this field was the apps/shell
/// shim. Removed in the cutover commit. The struct stays as a marker for
/// any straggling reference until they're all updated.)
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
/// The container-level `#[serde(default)]` fills every missing field from
/// [`DeckConfig::defaults`] (via the `Default` impl), so a minimal — or
/// missing — `deck.jsonc` degrades gracefully. Fields must NOT carry their
/// own `#[serde(default)]`: a field-level default takes precedence over the
/// container default and silently replaces the documented fallback with the
/// type's empty value (an empty `view_dirs` once disabled view discovery for
/// every host with a partial deck.jsonc).
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
    pub packs: Vec<String>,
    /// Per-source-root view bucket names. The resolver and bundler scan
    /// `<source_root>/<view_dir>/<name>/<view_leaf>` for each `view_dir` in
    /// this list. Defaults to the conventional `["views", ".ship/views"]`,
    /// so a zero-config host still resolves from both top-level and
    /// project-scoped view buckets. Hosts with a different layout override
    /// this list; deck no longer bakes the bucket names.
    pub view_dirs: Vec<String>,
    /// The leaf filename for view artifacts under each `view_dir/<name>/`.
    /// Defaults to `"page.sui"`. A single leaf convention across all
    /// `view_dirs` fixes the #254 leaf-name divergence (`page.sui` vs
    /// `index.sui`) that broke `.ship/views/<name>/index.sui` resolution.
    pub view_leaf: Option<String>,
    /// Root-relative public-asset directory (e.g. `apps/shell/public`). When set,
    /// deck serves single-segment root paths (`/favicon.ico`, `/ship-logo.svg`)
    /// from `<project_root>/<public_dir>/<file>`. `None` = no public-asset server.
    /// deck resolves the project root via its own resolver — never a Ship path.
    pub public_dir: Option<PathBuf>,
    /// Repo-root-relative directories searched, in order, to resolve a
    /// `@ship/<name>` component import to its package source. The resolver
    /// joins `<repo_root>/<root>/<name>` for each root and takes the first
    /// that exists. Defaults to `["packages"]` — a generic, host-agnostic
    /// layout. Hosts with a nested component layout (e.g. Ship's
    /// `packages/components/{sui,ship}`) declare the full search order here;
    /// deck no longer bakes any monorepo layout into its code.
    pub component_source_roots: Vec<PathBuf>,
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
            // Conventional view buckets. Override per-host in deck.jsonc.
            view_dirs: vec!["views".into(), ".ship/views".into()],
            // Single leaf convention — fixes the #254 `index.sui`/`page.sui`
            // divergence. Both buckets look up `<name>/<view_leaf>`.
            view_leaf: Some("page.sui".into()),
            public_dir: None,
            // Generic single-root layout — `<repo_root>/packages/<name>`.
            // Hosts with a nested component layout override in deck.jsonc;
            // deck's code bakes no monorepo-specific search order.
            component_source_roots: vec![PathBuf::from("packages")],
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
        let stripped = strip_jsonc_comments(&raw);
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


/// Strip JSONC extensions from a string: `//` line comments, `/* */` block
/// comments, and trailing commas before `]` or `}`. The result is valid JSON.
/// Respects string literals so URLs like `"http://"` are not mangled.
///
/// NOTE: this is a deliberate second copy of the JSONC stripper inlined
/// into the deck-config crate to drop the `ship-substrate` dependency
/// edge from deck-core's resolver chain. The two copies must stay in
/// sync — if the algorithm changes, update both.
fn strip_jsonc_comments(input: &str) -> String {
    // Phase 1: strip comments.
    let mut stage = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            stage.push(c);
            if escape { escape = false; } else if c == '\\' { escape = true; } else if c == '"' { in_string = false; }
            continue;
        }
        if c == '"' { in_string = true; stage.push(c); continue; }
        if c == '/' {
            match chars.peek() {
                Some('/') => { while let Some(&nc) = chars.peek() { chars.next(); if nc == '\n' { stage.push('\n'); break; } } continue; }
                Some('*') => { chars.next(); while let Some(nc) = chars.next() { if nc == '*' && chars.peek() == Some(&'/') { chars.next(); break; } } continue; }
                _ => {}
            }
        }
        stage.push(c);
    }
    // Phase 2: remove trailing commas (`,` followed only by whitespace then
    // `]` or `}`). Walk as bytes for cheap peek-ahead, but copy non-ASCII
    // bytes verbatim — `bytes[i] as char` would mangle multi-byte UTF-8
    // (e.g. an emdash 0xE2 0x80 0x94 turns into three U+00E2/U+0080/U+0094
    // chars when push'd to a String).
    let src = stage.as_bytes();
    let mut cleaned: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    let mut in_string2 = false;
    let mut escape2 = false;
    while i < src.len() {
        let b = src[i];
        if in_string2 {
            cleaned.push(b);
            if escape2 {
                escape2 = false;
            } else if b == b'\\' {
                escape2 = true;
            } else if b == b'"' {
                in_string2 = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' { in_string2 = true; cleaned.push(b); i += 1; continue; }
        if b == b',' {
            let mut j = i + 1;
            while j < src.len() && (src[j] == b' ' || src[j] == b'\t' || src[j] == b'\r' || src[j] == b'\n') {
                j += 1;
            }
            if j < src.len() && (src[j] == b']' || src[j] == b'}') {
                i += 1;
                continue;
            }
        }
        cleaned.push(b);
        i += 1;
    }
    // Safe: we only dropped ASCII commas at valid char boundaries; every
    // other byte was copied verbatim from the original UTF-8 String.
    String::from_utf8(cleaned).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
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
  "firstPartyApp": { "root": "apps/shell/src", "label": "app:shell" }
}"#;
    #[test]
    fn parse_example_jsonc() {
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
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
        assert_eq!(app.root, PathBuf::from("apps/shell/src"));
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
        // View layout defaults match the conventional buckets.
        assert_eq!(d.view_dirs, vec!["views".to_string(), ".ship/views".to_string()]);
        assert_eq!(d.view_leaf.as_deref(), Some("page.sui"));
    }

    /// `packs` is optional and additive — an absent field parses to empty,
    /// which the bundler reads as "global" (every workspace pack resolvable).
    #[test]
    fn packs_absent_parses_to_empty() {
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");
        assert!(cfg.packs.is_empty(), "no packs field ⇒ empty ⇒ global behavior");
    }

    /// A declared `packs` array parses into the scoped set.
    #[test]
    fn packs_present_parses_declared_set() {
        let json = r#"{ "packs": ["@ship/sui-primitives", "deck-demo-app"] }"#;
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(json)).unwrap();
        assert_eq!(cfg.packs, vec!["@ship/sui-primitives", "deck-demo-app"]);
    }

    /// A PARTIAL deck.jsonc (deck-demo's shape: scaffolds only) must keep the
    /// built-in view-layout defaults. Field-level `#[serde(default)]` attrs
    /// used to shadow the container default here: `view_dirs` parsed to `[]`
    /// and view discovery found zero targets (`unknown target: views/*/view`).
    #[test]
    fn partial_file_keeps_view_layout_defaults() {
        let json = r#"{
  "scaffolds": [
    { "name": "space", "route": "/space/{id}", "source": "space-scaffold" }
  ]
}"#;
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(json)).expect("partial deck.jsonc parses");
        assert_eq!(cfg.scaffolds.len(), 1);
        assert_eq!(
            cfg.view_dirs,
            vec!["views".to_string(), ".ship/views".to_string()],
            "partial config must inherit the conventional view buckets"
        );
        assert_eq!(cfg.view_leaf.as_deref(), Some("page.sui"));
        assert_eq!(cfg.component_source_roots, vec![PathBuf::from("packages")]);
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
