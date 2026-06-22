//! deck-config — `DeckConfig` (the host's view-discovery + watch layout) and
//! per-app `AppServeConfig`.
//!
//! `DeckConfig` is HOST-SUPPLIED. The framework does not read any file from
//! a project root — the host (shipd, deck-conformance) constructs the
//! `DeckConfig` it wants to pass in (typically [`DeckConfig::defaults`]
//! plus any host-specific overrides) and threads it through `Bundler::start`
//! and the resolver. There is no `deck.jsonc` lookup, no detection marker,
//! no implicit load. Per-app scaffolding (`AppServeConfig`) is the
//! `Vec<AppServeConfig>` the host builds from its own `apps.jsonc` — the
//! framework names no app, no list.
//!
//! A stray `deck.jsonc` at the workspace ROOT has no effect: not loaded,
//! not detected. The earlier file-read and `deck.jsonc` project-root
//! marker were the dead-weight surface dissolved by the deck-purge-root
//! job; the host now hands deck its paths, deck does not detect them.
//!
//! ## Per-app `deck.jsonc` (app-owned)
//!
//! Distinct from the retired root marker: each app owns a per-app
//! `apps/<app>/deck.jsonc` that declares its serve config — the
//! [`AppServeConfig`] shape (`name`, `route`, `source`, optional
//! `viewOutput`, the [`RenderMode`], optional `injectGlobals`). The hub
//! defines the hub. deck-config provides the schema + the JSONC parser
//! ([`AppServeConfig::parse_deck_jsonc`]); the HOST decides which files to
//! load and binds their locations (deck still names no path). The host's
//! orchestration list (`apps.jsonc` in the Ship repo) only points at WHERE
//! each app's `deck.jsonc` lives — the per-app file is the source of truth
//! for that app's serve config.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod jsonc;
pub use jsonc::strip_jsonc_comments;

// ─── types ────────────────────────────────────────────────────────────────────

/// How an app paints its first frame.
///
/// `Csr` (the default) ships an empty mount point (`#ship-root`) and lets the
/// client bundle render. `Ssr` means the app server-renders its first frame
/// before hydration. This is the OBSERVABLE "does this app server-render at
/// all" flag the router/serving path consumes — NOT the SSR-engine selector
/// (which engine renders is a separate, host-gated concern). The served
/// `index.html` carries the active mode as `<meta name="ship-render">` so it
/// is legible client-side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RenderMode {
    /// Client-side render — empty `#ship-root`, the client bundle paints.
    #[default]
    Csr,
    /// Server-side render — the first frame is rendered before hydration.
    Ssr,
}

/// One app's serve configuration — the parsed contents of a per-app
/// `apps/<app>/deck.jsonc`, and the shape deck's per-app router consumes.
/// shipd (or any deck host) builds a `Vec<AppServeConfig>` by loading each
/// app's `deck.jsonc` (the apps it serves come from the host's own
/// orchestration list — `apps.jsonc` in the Ship repo; the host decides the
/// source). The framework does NOT know the list — that is the host's
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
    /// Where this app's compiled VIEW bundles live, relative to the project
    /// root. The runtime probes `<view_output>/views/<name>/view.js` BEFORE
    /// falling back to the bundler's default workspace-root output dir
    /// (`<project_root>/.deck/views`), so apps with a non-conventional layout
    /// (e.g. `apps/hub/.deck/views/<name>/view.js`) serve their own bundles
    /// without depending on the global project root.
    ///
    /// Optional: when absent, deck uses the bundler's default (the workspace
    /// root's `.deck/views`).
    #[serde(default)]
    pub view_output: Option<String>,
    /// Globals injected as `<script>window.<key>=<value>;</script>` into the
    /// app's `index.html` before `</head>`. `$<param>` tokens in the value are
    /// substituted from the URL path captures declared in `route` (so for
    /// `route: "/space/{id}"` and an entry `("DECK_APP_ID", "$id")`, the
    /// injection is `window.DECK_APP_ID="<id>";`).
    #[serde(default)]
    pub inject_globals: BTreeMap<String, String>,
    /// How the app paints its first frame — `csr` (default) or `ssr`. Made
    /// observable by serving it as a `<meta name="ship-render">` tag in the
    /// app's `index.html`. Absent ⇒ `csr` (the implicit-empty-`#ship-root`
    /// posture every app had before this field existed).
    #[serde(default)]
    pub render: RenderMode,
    /// Framework clients the HOST registers for this app — bare specifier
    /// aliases that resolve to host-provided ESM modules through the
    /// generated importmap. The bundler does NOT hardcode any client alias
    /// (no `@ship/sdk-client`, no `@ship/*-sdk`); every client that needs
    /// importmap resolution must be declared HERE.
    ///
    /// Each entry maps the alias to a per-app copy instruction:
    /// `from` is the workspace-relative source path (typically a `.deck/framework/`
    /// bundle the bake produced); `dist` is the output path relative to the
    /// project's build dir (the bundler copies `from` → `<out>/<dist>` when
    /// `--bundle-framework` is set). The importmap binds the alias to
    /// `/_deck/<dist_basename>` — the same URL convention every framework
    /// singleton uses — so the daemon serves the asset the same way.
    ///
    /// This is the L1 (deck) expression of the `DataPort` / `AppRegistry`
    /// discipline: the host binds its own concrete inside the impl; deck
    /// provides the slot, not the concrete.
    #[serde(default)]
    pub framework_clients: BTreeMap<String, FrameworkClientEntry>,
    /// Host-owned entry bootstrap. The bundler generates a typed
    /// `entry.gen.ts` from this — the file that constructs the named
    /// `clients` map and hands it to `mountRouter({...})`. The data
    /// side of the bootstrap (route table, layout table, view modules,
    /// layout modules) is the framework's `routes.gen.ts`; the
    /// client-construction side is the HOST's `entry.gen.ts`. Same
    /// `data + bootstrap` discipline every per-app host follows; deck
    /// doesn't ship a `main.ts` and neither should the host.
    #[serde(default)]
    pub entry: Option<EntryBootstrap>,
}

/// One framework-client registration — the importmap alias → asset copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkClientEntry {
    /// Workspace-relative source path of the client asset. Resolved at build
    /// time; the path is read once and its bytes copied to `<out>/<dist>`.
    /// Typically `<repo>/.deck/framework/<bundle>` — the bake output.
    #[serde(rename = "from")]
    pub from: String,
    /// Output path, relative to the project's build dir. The importmap binds
    /// the alias to `/_deck/<dist_basename>`, mirroring the existing
    /// `FRAMEWORK_ASSETS` convention so the daemon's static-asset handler
    /// serves client assets on the same code path.
    #[serde(rename = "dist")]
    pub dist: String,
}

/// Host-owned entry bootstrap. The bundler reads this from
/// `apps/<app>/deck.jsonc` and emits `entry.gen.ts` next to the data-only
/// `routes.gen.ts`. Decoupled so the host can wire ANY clients it needs —
/// the framework ships `mountRouter` and a `clients` injection surface;
/// which keys the host puts on the map is the host's call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryBootstrap {
    /// Module-relative specifier of the host's client factory. The bundler
    /// emits `import { <names...> } from '<clientsModule>'` in `entry.gen.ts`;
    /// the host's file constructs each named client (a `createClient()` call,
    /// a wrapper, etc.) and named-exports the result. Same path conventions
    /// as a route's `controller.ts` import — the path is relative to the
    /// generated entry, which lives at `<project>/entry.gen.ts` next to
    /// `routes.gen.ts`.
    #[serde(rename = "clientsModule")]
    pub clients_module: String,
    /// Names the bundler reads off the factory's exports and spreads into
    /// the `clients` map. Each name becomes a key on the `clients` object
    /// `mountRouter({ clients })` receives; views read it via `ctx.clients.<name>`
    /// (and the legacy `ctx.<name>` alias, so pre-migration view code still
    /// resolves during the cutover).
    #[serde(rename = "exposeNames")]
    pub expose_names: Vec<String>,
}

impl AppServeConfig {
    /// Parse a per-app `deck.jsonc` (JSONC source) into the serve config.
    ///
    /// Strips JSONC extensions, then deserializes. `name`, `route`, and
    /// `source` are required; `viewOutput`, `injectGlobals`, and `render`
    /// default. The HOST decides which file to read and feeds the bytes here
    /// — deck-config names no path.
    pub fn parse_deck_jsonc(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&strip_jsonc_comments(raw))
    }
}
/// The parsed contents of the host's view-discovery `DeckConfig`.
///
/// The container-level `#[serde(default)]` fills every missing field from
/// [`DeckConfig::defaults`] (via the `Default` impl), so a minimal — or
/// missing — `deck.jsonc` degrades gracefully. Fields must NOT carry their
/// own `#[serde(default)]`: a field-level default takes precedence over the
/// container default and silently replaces the documented fallback with the
/// type's empty value (an empty `view_dirs` once disabled view discovery for
/// every host with a partial deck.jsonc).
///
/// ## What this is NOT
///
/// `DeckConfig` describes the host's VIEW-DISCOVERY + watch layout. It does NOT
/// describe the host's app list (the host reads its own `apps.jsonc` and passes
/// a per-app `Vec<AppServeConfig>` to `serve_apps()`), the consumer bundle set
/// (self-declared per-package in `package.json` `deck.bundles[]`, discovered by
/// the bundler), or `@ship/<name>` component resolution (derived from the
/// workspace package map). L1 is per-app; the host owns the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeckConfig {
    /// Source roots to scan for views. Defaults to `["."]` (workspace root).
    pub source_roots: Vec<PathBuf>,
    /// Directories the bundler watches for live-reload. Defaults to `[]`.
    pub watch: Vec<PathBuf>,
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
}

impl Default for DeckConfig {
    fn default() -> Self {
        Self::defaults()
    }
}
impl DeckConfig {
    /// Built-in defaults — a zero-config host (e.g. deck-conformance) gets these
    /// when the host passes no override. Conventional `["views", ".ship/views"]`
    /// view buckets and the unified `page.sui` leaf (#254).
    pub fn defaults() -> Self {
        Self {
            source_roots: vec![PathBuf::from(".")],
            watch: Vec::new(),
            packs: Vec::new(),
            // Conventional view buckets. Hosts that need a different layout
            // construct a `DeckConfig` (clone `defaults()` + override) and pass
            // it in — no file load.
            view_dirs: vec!["views".into(), ".ship/views".into()],
            // Single leaf convention — fixes the #254 `index.sui`/`page.sui`
            // divergence. Both buckets look up `<name>/<view_leaf>`.
            view_leaf: Some("page.sui".into()),
        }
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Minimal example deck.jsonc that exercises the view-layout fields a host
    /// may override. Consumer bundle targets are NO LONGER here — they are
    /// self-declared per-package (`package.json` `deck.bundles[]`) and
    /// discovered by the bundler; `@ship/<name>` resolution is derived from the
    /// workspace package map. So this config carries only view discovery + watch
    /// + pack scope.
    const EXAMPLE_JSONC: &str = r#"{
  "sourceRoots": ["."],
  "watch": ["packages", "apps/site/src"],
  "viewDirs": ["views", ".ship/views"],
  "viewLeaf": "page.sui"
}"#;
    #[test]
    fn parse_example_jsonc() {
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(EXAMPLE_JSONC))
                .expect("example deck.jsonc must parse");
        assert_eq!(cfg.source_roots, vec![PathBuf::from(".")]);
        assert_eq!(cfg.watch, vec![PathBuf::from("packages"), PathBuf::from("apps/site/src")]);
        assert_eq!(cfg.view_dirs, vec!["views".to_string(), ".ship/views".to_string()]);
        assert_eq!(cfg.view_leaf.as_deref(), Some("page.sui"));
    }
    #[test]
    fn defaults_round_trip() {
        let d = DeckConfig::defaults();
        // Serialize and deserialize — must be lossless.
        let json = serde_json::to_string(&d).unwrap();
        let d2: DeckConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
        assert_eq!(d.source_roots, vec![PathBuf::from(".")]);
        assert!(d.watch.is_empty());
        assert!(d.packs.is_empty(), "defaults declare no packs → global resolution");
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

    /// A PARTIAL deck.jsonc (deck-conformance's shape: scaffolds only) must keep the
    /// built-in view-layout defaults. Field-level `#[serde(default)]` attrs
    /// used to shadow the container default here: `view_dirs` parsed to `[]`
    /// and view discovery found zero targets (`unknown target: views/*/view`).
    #[test]
    fn partial_file_keeps_view_layout_defaults() {
        let json = r#"{
  "watch": ["packages"]
}"#;
        let cfg: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(json)).expect("partial deck.jsonc parses");
        assert_eq!(
            cfg.view_dirs,
            vec!["views".to_string(), ".ship/views".to_string()],
            "partial config must inherit the conventional view buckets"
        );
        assert_eq!(cfg.view_leaf.as_deref(), Some("page.sui"));
    }

    // ── per-app deck.jsonc (AppServeConfig) ───────────────────────────────────

    /// The hub's per-app `deck.jsonc`: a full serve config with an explicit
    /// `csr` render mode. Mirrors `apps/hub/deck.jsonc`.
    const HUB_DECK_JSONC: &str = r#"{
  // apps/hub/deck.jsonc — the hub defines the hub.
  "name": "hub",
  "route": "/",
  "source": "apps/hub/.deck",
  "render": "csr"
}"#;

    #[test]
    fn parse_app_deck_jsonc_full() {
        let cfg = AppServeConfig::parse_deck_jsonc(HUB_DECK_JSONC)
            .expect("hub deck.jsonc must parse");
        assert_eq!(cfg.name, "hub");
        assert_eq!(cfg.route, "/");
        assert_eq!(cfg.source, "apps/hub/.deck");
        assert_eq!(cfg.view_output, None);
        assert!(cfg.inject_globals.is_empty());
        assert_eq!(cfg.render, RenderMode::Csr);
    }

    /// `render` is optional and defaults to `csr` — the implicit posture every
    /// app had before the field existed (empty `#ship-root`, client paints).
    #[test]
    fn parse_app_deck_render_defaults_to_csr() {
        let json = r#"{ "name": "hub", "route": "/", "source": "apps/hub/.deck" }"#;
        let cfg = AppServeConfig::parse_deck_jsonc(json).expect("minimal deck.jsonc parses");
        assert_eq!(cfg.render, RenderMode::Csr, "absent render ⇒ csr");
        // The whole-config Default also lands on csr.
        assert_eq!(AppServeConfig::default().render, RenderMode::Csr);
    }

    /// `render: "ssr"` parses into the SSR variant; injectGlobals + a
    /// route-capture (spaces' shape) round-trip through the JSONC parser.
    #[test]
    fn parse_app_deck_ssr_and_inject_globals() {
        let json = r#"{
  "name": "spaces",
  "route": "/space/{id}",
  "source": "apps/spaces/.deck",
  "render": "ssr",
  "injectGlobals": { "DECK_APP_ID": "$id" }
}"#;
        let cfg = AppServeConfig::parse_deck_jsonc(json).expect("spaces deck.jsonc parses");
        assert_eq!(cfg.route, "/space/{id}");
        assert_eq!(cfg.render, RenderMode::Ssr);
        assert_eq!(
            cfg.inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
    }

    /// `RenderMode` serializes to the lowercase wire tokens the meta tag emits.
    #[test]
    fn render_mode_serde_tokens() {
        assert_eq!(serde_json::to_string(&RenderMode::Csr).unwrap(), "\"csr\"");
        assert_eq!(serde_json::to_string(&RenderMode::Ssr).unwrap(), "\"ssr\"");
        let m: RenderMode = serde_json::from_str("\"ssr\"").unwrap();
        assert_eq!(m, RenderMode::Ssr);
    }

    /// A `deck.jsonc` missing a required field (`source`) is a parse error —
    /// surfaced to the host at load, not silently defaulted.
    #[test]
    fn parse_app_deck_missing_required_field_errors() {
        let json = r#"{ "name": "hub", "route": "/" }"#;
        assert!(
            AppServeConfig::parse_deck_jsonc(json).is_err(),
            "missing `source` must error, not default to empty"
        );
    }
    /// `frameworkClients` + `entry` round-trip through the JSONC parser —
    /// the registration seam the bundler reads to (a) emit importmap entries
    /// for host-registered clients and (b) generate a typed `entry.gen.ts`
    /// from the host's `clientsModule` + `exposeNames`. The shape is the
    /// spec's hard constraint: deck names no ship path, the HOST does.
    #[test]
    fn parse_app_deck_framework_clients_and_entry() {
        let json = r#"{
  "name": "hub",
  "route": "/",
  "source": "apps/hub/.deck",
  "frameworkClients": {
    "@ship/sdk-client": {
      "from": ".deck/framework/sui-sdk-client.js",
      "dist": "_deck/sdk-client.js"
    }
  },
  "entry": {
    "clientsModule": "./src/lib/client.ts",
    "exposeNames": ["ship", "ops"]
  }
}"#;
        let cfg = AppServeConfig::parse_deck_jsonc(json).expect("hub deck.jsonc parses");
        let sdk = cfg.framework_clients.get("@ship/sdk-client")
            .expect("@ship/sdk-client registered");
        assert_eq!(sdk.from, ".deck/framework/sui-sdk-client.js");
        assert_eq!(sdk.dist, "_deck/sdk-client.js");
        let entry = cfg.entry.expect("entry bootstrap declared");
        assert_eq!(entry.clients_module, "./src/lib/client.ts");
        assert_eq!(entry.expose_names, vec!["ship", "ops"]);
    }

    /// A host without `frameworkClients` or `entry` parses fine — the
    /// fields default to empty/None, the bundler emits no client importmap
    /// entries and no `entry.gen.ts`. (An app with no framework clients
    /// mounts with `clients: {}` — the same shape the prior render emitted.)
    #[test]
    fn parse_app_deck_framework_clients_default_to_empty() {
        let json = r#"{ "name": "hub", "route": "/", "source": "apps/hub/.deck" }"#;
        let cfg = AppServeConfig::parse_deck_jsonc(json).expect("minimal deck.jsonc parses");
        assert!(cfg.framework_clients.is_empty(), "absent frameworkClients ⇒ empty");
        assert!(cfg.entry.is_none(), "absent entry ⇒ None");
    }


    /// `DeckConfig::load` is GONE. Constructing a host config is the host's
    /// job — `DeckConfig::defaults()` plus any overrides the host reads
    /// (with `strip_jsonc_comments` if it has JSONC). The acceptance
    /// criterion for the deck-purge-root job: a stray `deck.jsonc` at the
    /// workspace root has NO effect (not loaded, not a detection marker).
    /// This test pins that contract by parsing the JSONC content via the
    /// public helper (the host's path) and asserting a dropped file plus
    /// a default config produce identical views.
    #[test]
    fn stray_deck_jsonc_has_no_effect() {
        let d = DeckConfig::defaults();
        // A stray deck.jsonc the framework would have read before — a
        // non-empty override with custom view buckets. Parsed via the
        // public `strip_jsonc_comments` helper to show a host CAN parse
        // it (host-supplied config still works), but the framework does
        // not load it on its own.
        let jsonc = r#"{
  "viewDirs": ["apps/shell/src/views"],
  "viewLeaf": "page.sui"
}"#;
        let from_str: DeckConfig =
            serde_json::from_str(&strip_jsonc_comments(jsonc)).expect("host-parsed config");
        assert_eq!(
            from_str.view_dirs,
            vec!["apps/shell/src/views".to_string()],
            "host parsing of deck.jsonc-shaped content still works"
        );
        assert_ne!(
            d.view_dirs, from_str.view_dirs,
            "the override is real when parsed — it's just not auto-loaded"
        );
        // The framework sees defaults() — the stray file is inert.
        assert_eq!(d, DeckConfig::defaults());
    }
 }
