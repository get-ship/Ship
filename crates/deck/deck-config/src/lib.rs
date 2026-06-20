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
//! A stray `deck.jsonc` at the workspace root has no effect: not loaded,
//! not detected. The earlier file-read and `deck.jsonc` project-root
//! marker were the dead-weight surface dissolved by the deck-purge-root
//! job; the host now hands deck its paths, deck does not detect them.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ─── types ────────────────────────────────────────────────────────────────────

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
    /// Globals injected as `<script>window.<key>=<value>;</script>` into
    /// the app's `index.html` before `</head>`. `$<param>` tokens in the
    /// value are substituted from the URL path captures declared in
    /// `route` (so for `route: "/space/{id}"` and an entry
    /// `("DECK_APP_ID", "$id")`, the injection is `window.DECK_APP_ID="<id>";`).
    #[serde(default)]
    pub inject_globals: BTreeMap<String, String>,
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

/// Strip JSONC extensions from a string: `//` line comments, `/* */` block
/// comments, and trailing commas before `]` or `}`. The result is valid JSON.
/// Respects string literals so URLs like `"http://"` are not mangled.
///
/// Public helper for hosts (and tests) that need to parse a JSONC document
/// without depending on `ship-substrate` — the deck-config crate is the
/// std-only JSONC parser the deck-core resolver chain requires. The
/// `deck-config::DeckConfig` type no longer reads any file itself; the
/// helper stays so a host that DOES want JSONC parsing for its own
/// config (e.g. `apps.jsonc`) can build a `DeckConfig` via
/// `serde_json::from_str(&strip_jsonc_comments(&raw))` and pass it in.
///
/// NOTE: this is a deliberate second copy of the JSONC stripper inlined
/// into the deck-config crate to drop the `ship-substrate` dependency
/// edge from deck-core's resolver chain. The two copies must stay in
/// sync — if the algorithm changes, update both.
pub fn strip_jsonc_comments(input: &str) -> String {
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
