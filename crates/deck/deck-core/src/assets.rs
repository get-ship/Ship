use crate::{DaemonMode, DeckResponse};
use deck_config::ScaffoldDecl;

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// deck's OWN baked product (sui-runtime/dom/sdk-client + primitives umbrella +
/// lucide). These ship inside the deck-core crate, like Next shipping its own
/// runtime. App + user content is NOT here — see `read_deck_asset`.
///
/// Used for committed assets (lucide-icons.json, live-reload.js) that remain in
/// the crate's `assets/` directory. Framework singletons are served by
/// `read_framework_asset`, which reads from `.deck/framework/` first.
fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

/// Hot-path for the 4 framework singletons (sui-runtime/dom/sdk-client/components).
/// Reads from `.deck/framework/<name>` — the canonical bake target, which is
/// updated on every `cargo run -p deck-cli -- build-assets` and by the running
/// daemon's watch-rebuild. In an installed (no-.deck) deployment the caller falls
/// back to the embedded `include_str!` constant.
fn read_framework_asset(name: &str) -> Option<String> {
    let root = crate::resolver::deck_out_root()?;
    std::fs::read_to_string(root.join("framework").join(name)).ok()
}

/// App + user build output, read from the host-supplied `.deck/` tree (deck's
/// `.next`-equivalent). deck bakes none of this — view + chrome bundles are the
/// project's content, served from where the project built them. Returns None
/// when there is no project root (the on-demand compile path then takes over).
fn read_deck_asset(rel: &str) -> Option<String> {
    let root = crate::resolver::deck_out_root()?;
    std::fs::read_to_string(root.join(rel)).ok()
}

/// Project SOURCE (not built output), read from the project root directly. The
/// dev/source fallback for scaffolds: when a scaffold has no built
/// `.deck/<source>/` artifact yet, serve its authored `<source>/index.html` in
/// place — the same built-or-source fallback `view.rs` uses for view modules.
fn read_source_asset(rel: &str) -> Option<String> {
    let root = crate::resolver::resolve_project_root()?;
    std::fs::read_to_string(root.join(rel)).ok()
}

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
const SUI_LUCIDE_ICONS_JSON: &str = include_str!("../assets/lucide-icons.json");
const LIVE_RELOAD_JS: &str = include_str!("../assets/live-reload.js");

// Aliases the runtime used to resolve client-side. Promoted to the daemon so a
// `<sui-icon name="git">` request finds `git-branch` server-side with no client
// indirection. Keep in lockstep with `packages/components/sui/primitives/src/sui-icon/registry.ts`.
const ICON_ALIASES: &[(&str, &str)] = &[
    ("git", "git-branch"),
    ("refresh", "refresh-cw"),
    ("stop", "stop-circle"),
    ("warning", "triangle-alert"),
];

/// Parse the embedded `lucide-icons.json` into a `name -> inner-svg-fragment`
/// map once at first access. `LazyLock` keeps the deserialized map
/// alongside its initializer — no separate `get_or_init` accessor needed
/// and no missing-init path.
static ICON_MAP: std::sync::LazyLock<std::collections::HashMap<String, String>> =
    std::sync::LazyLock::new(|| {
        serde_json::from_str(SUI_LUCIDE_ICONS_JSON)
            .expect("deck-core lucide-icons.json failed to deserialize at startup")
    });

fn alias_for(name: &str) -> Option<&'static str> {
    ICON_ALIASES.iter().find_map(|(k, v)| (*k == name).then_some(*v))
}

// Public helper (used by view.rs). deck bakes NO app views — per-view bundles
// live in the project's `.deck/views/<name>/view.js`; absent that, view.rs
// compiles on demand from source.

pub fn resolve_view_js(name: &str) -> Option<String> {
    // SSOT-first: if the bundler recorded this view in the asset graph, serve
    // the exact artifact it built (resolved through the one project root). Fall
    // back to the conventional `.deck/` path so a view compiled outside
    // `build_all` (the on-demand path) still resolves — never worse than the
    // old direct probe.
    if let Some(node) = crate::asset_graph::lookup(&format!("views/{name}/view"))
        && let Some(abs) = crate::asset_graph::resolve(&node)
        && let Ok(js) = std::fs::read_to_string(&abs)
    {
        return Some(js);
    }
    read_deck_asset(&format!("views/{name}/view.js"))
}

// ── Scaffold serving ──────────────────────────────────────────────────────────
//
// A `ScaffoldDecl` (from `deck.jsonc` or built-in defaults) describes one HTML
// shell: which built artifact to load (`source`), where its hashed assets are
// re-rooted (`asset_prefix`), and which `window.<key>=<value>` globals to
// inject (`inject_globals`). This single function handles every scaffold route
// (shell `GET /`, space `GET /space/{id}`, etc.).
//
// `params` are route-param substitutions — e.g. `&[("id", "my-space")]` lets
// the config value `"$id"` resolve to the actual route param. Any `$key` token
// in an `inject_globals` value is replaced with the matching param.

/// Serve an HTML scaffold defined by a `ScaffoldDecl`.
///
/// - Reads the built index.html from `<deck_out>/<source>/index.html`.
/// - Rewrites `"/assets/` → `"<asset_prefix>` so Vite-hashed asset hrefs
///   resolve under the scaffold's dedicated asset route.
/// - Injects one `<script>window.<key>=<value>;</script>` per
///   `inject_globals` entry, substituting `$<param>` tokens from `params`.
/// - Injects the daemon-mode meta tag via `inject_mode_meta`.
///
/// Reads the built `.deck/<source>/index.html`; in dev (no build artifact yet)
/// falls back to the authored source `<source>/index.html`. Returns a styled
/// 404 only when neither exists.
pub fn serve_scaffold(
    mode: DaemonMode,
    scaffold: &ScaffoldDecl,
    params: &[(&str, &str)],
) -> DeckResponse {
    let Some(html) = read_deck_asset(&format!("{}/index.html", scaffold.source))
        .or_else(|| read_source_asset(&format!("{}/index.html", scaffold.source)))
    else {
        return DeckResponse::not_found(
            format!(
                "scaffold '{}' not built (missing {}/index.html)",
                scaffold.name, scaffold.source
            ),
            mode,
        );
    };

    // Build the globals injection block.
    let inject = if scaffold.inject_globals.is_empty() {
        String::new()
    } else {
        let mut parts = String::from("<script>");
        for (key, raw_value) in &scaffold.inject_globals {
            // Validate that the key is a safe JS identifier before embedding
            // it verbatim in the emitted script. An unchecked key like
            // `x;alert(1)//` would inject arbitrary script on every page load.
            if !is_valid_js_identifier(key) {
                tracing::warn!(
                    scaffold = %scaffold.name,
                    key = %key,
                    "inject_globals key is not a valid JS identifier — skipping"
                );
                continue;
            }
            // Substitute $param tokens.
            let resolved = substitute_params(raw_value, params);
            // JSON-encode the value so it is a safe JS string literal.
            let js_value =
                serde_json::to_string(&resolved).unwrap_or_else(|_| "\"\"".to_string());
            parts.push_str(&format!("window.{key}={js_value};"));
        }
        parts.push_str("</script>");
        parts
    };

    // Rewrite asset hrefs: Vite emits `href="/assets/..."` — reroot under the
    // scaffold's dedicated asset prefix so assets from different scaffolds don't
    // collide. The prefix in `deck.jsonc` ends with `/`, so `"/assets/` →
    // `"<prefix>` gives `"/shell/_assets/"` etc.
    let prefix = if scaffold.asset_prefix.is_empty() {
        "/assets/".to_string()
    } else {
        scaffold.asset_prefix.trim_end_matches('/').to_string() + "/"
    };

    let html = html.replace("\"/assets/", &format!("\"{prefix}"));

    // Inject globals before `</head>` so the JS variables are available as
    // early as possible.  Fall back to `<head>` insertion when `</head>` is
    // absent (malformed scaffold).
    let html = if !inject.is_empty() {
        if html.contains("</head>") {
            html.replace("</head>", &format!("{inject}</head>"))
        } else {
            html.replacen("<head>", &format!("<head>{inject}"), 1)
        }
    } else {
        html
    };

    let html = crate::mode_html::inject_mode_meta(&html, mode);
    DeckResponse::html_view(html, mode)
}

/// Serve a static asset from `<deck_out>/<source>/assets/<file>`.
///
/// The `file` parameter comes from the axum path extractor and is
/// percent-decoded but NOT stripped of `..` segments. Validate it before
/// constructing the path to prevent directory traversal: a request like
/// `/space/_assets/../../../etc/passwd` would otherwise escape the assets dir.
pub fn serve_scaffold_asset(mode: DaemonMode, scaffold: &ScaffoldDecl, file: &str) -> DeckResponse {
    // Reject path-traversal patterns. Only simple filenames (no slashes, no
    // leading dots, no ".." components) are valid asset names. This mirrors the
    // guard already present in serve_sui_chunk.
    if file.contains("..") || file.contains('/') || file.contains('\\') || file.starts_with('.') {
        return DeckResponse::not_found(
            format!("invalid asset path: {file}"),
            mode,
        );
    }
    match read_deck_asset(&format!("{}/assets/{file}", scaffold.source)) {
        Some(body) if file.ends_with(".css") => DeckResponse::css_view(body, mode),
        Some(body) => DeckResponse::js_asset(body, mode),
        None => DeckResponse::not_found(
            format!("scaffold '{}' asset not found: {file}", scaffold.name),
            mode,
        ),
    }
}

/// Default `ScaffoldDecl` for the shell (hub). Used as a fallback when no
/// `deck.jsonc` scaffolds entry matches, so a zero-config host still serves.
pub fn default_shell_scaffold() -> ScaffoldDecl {
    use std::collections::BTreeMap;
    ScaffoldDecl {
        name: "shell".to_string(),
        route: "/".to_string(),
        source: "hub-scaffold".to_string(),
        asset_prefix: "/shell/_assets/".to_string(),
        inject_globals: BTreeMap::new(),
    }
}

/// Default `ScaffoldDecl` for a space window. Used as a fallback when no
/// `deck.jsonc` scaffolds entry matches.
pub fn default_space_scaffold() -> ScaffoldDecl {
    use std::collections::BTreeMap;
    let mut inject = BTreeMap::new();
    inject.insert("DECK_APP_ID".to_string(), "$id".to_string());
    ScaffoldDecl {
        name: "space".to_string(),
        route: "/space/{id}".to_string(),
        source: "space-scaffold".to_string(),
        asset_prefix: "/space/_assets/".to_string(),
        inject_globals: inject,
    }
}

/// Validate that `key` is a safe JS identifier: `[A-Za-z_][A-Za-z0-9_]*`.
///
/// This is called before using a key verbatim in `window.{key}=...` in the
/// emitted `<script>` block. An unvalidated key like `x;alert(1)//` would
/// inject arbitrary script on every page load.
fn is_valid_js_identifier(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    let mut chars = key.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Substitute `$<key>` tokens in `value` with matching entries from `params`.
fn substitute_params(value: &str, params: &[(&str, &str)]) -> String {
    let mut out = value.to_string();
    for (key, replacement) in params {
        out = out.replace(&format!("${key}"), replacement);
    }
    out
}

pub fn serve_space_asset(mode: DaemonMode, file: &str) -> DeckResponse {
    serve_scaffold_asset(mode, &default_space_scaffold(), file)
}

pub fn serve_shell_asset(mode: DaemonMode, file: &str) -> DeckResponse {
    serve_scaffold_asset(mode, &default_shell_scaffold(), file)
}

// Framework bundle handlers
//
// Hot-path: read from `.deck/framework/` (the canonical bake target, updated
// on every `build-assets` run and by the daemon's watch-rebuild).
// Production/installed fallback: the embedded `include_str!` constant (baked
// into the binary at compile time by build.rs staging `.deck/framework/` →
// `assets/`). This keeps the daemon self-contained when deployed without a
// source tree.

pub fn serve_sui_runtime(mode: DaemonMode) -> DeckResponse {
    let js = read_framework_asset("sui-runtime.js").unwrap_or_else(|| SUI_RUNTIME_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_dom(mode: DaemonMode) -> DeckResponse {
    let js = read_framework_asset("sui-dom.js").unwrap_or_else(|| SUI_DOM_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_sdk_client(mode: DaemonMode) -> DeckResponse {
    let js = read_framework_asset("sui-sdk-client.js").unwrap_or_else(|| SUI_SDK_CLIENT_JS.to_string());
    DeckResponse::js_asset(js, mode)
}
/// `GET /_sui/icons/{name}.svg` — one lucide icon, served standalone. Replaces
/// `GET /_sui/icons/{name}.svg` — one lucide icon, served standalone. Replaces
/// the retired whole-blob `/_sui/lucide-icons.json` endpoint (~410 KB fetched
/// once on first mount). Per-icon serving hands the browser HTTP cache the
/// granularity that matches usage: each `<sui-icon>` triggers one tiny GET
/// (typically <1 KB), 304-cached on second mount.
///
/// The response is a full standalone `<svg>` with lucide's canonical viewBox
/// and `stroke="currentColor"` so the icon picks up the host's `color` CSS.
/// Aliases (`git` → `git-branch` etc.) are resolved server-side so the
/// client can stay alias-unaware.
pub fn serve_sui_icon(mode: DaemonMode, name: &str) -> DeckResponse {
    // Reject path-traversal + non-lucide name shapes early — names are
    // kebab-case ASCII, identical to the file stems lucide-static ships.
    if name.is_empty()
        || name.len() > 64
        || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return DeckResponse::not_found("invalid icon name", mode);
    }
    let resolved = if ICON_MAP.contains_key(name) {
        name
    } else if let Some(via) = alias_for(name) {
        via
    } else {
        return DeckResponse::not_found(format!("icon not in lucide set: {name}"), mode);
    };
    let inner = ICON_MAP.get(resolved).expect("resolved name in map");
    // Canonical lucide attrs — kept in lockstep with `makeSvg()` in
    // sui-icon.controller.ts so client (CDN-src mode) + daemon (named mode)
    // render identically.
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" \
         stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" \
         stroke-linejoin=\"round\">{inner}</svg>"
    );
    DeckResponse::new(
        200,
        "image/svg+xml; charset=utf-8",
        crate::response::CachePolicy::Asset,
        mode,
        svg.into_bytes(),
    )
}

/// `GET /_sui/live-reload.js` — the one HMR client, shared by the view
/// host and the shell. Served in every mode (it self-gates on the
/// `ship-mode` meta tag); only subscribes when the daemon is in dev.
pub fn serve_sui_live_reload(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("live-reload.js").unwrap_or_else(|| LIVE_RELOAD_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

/// `GET /_sui/{file}` — a shared chunk emitted by the multi-entry view build
/// (#267): a deduped view bundle does `import "../../<chunk>.js"`, which the
/// browser resolves to `/_sui/<chunk>.js`. Served content-addressed from the
/// `.deck/` root. Single-segment `.js` only; the framework singletons
/// (`/_sui/runtime.js` …) match their exact routes first, so this catches only
/// the hashed shared chunks.
pub fn serve_sui_chunk(mode: DaemonMode, file: &str) -> DeckResponse {
    if file.contains("..") || file.contains('/') || !file.ends_with(".js") {
        return DeckResponse::not_found("invalid chunk", mode);
    }
    match read_deck_asset(file) {
        Some(js) => DeckResponse::js_asset(js, mode),
        None => DeckResponse::not_found(format!("chunk missing: {file}"), mode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_js_returns_baked_fallback() {
        let resp = serve_sui_runtime(DaemonMode::Development);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn icon_known_name_returns_svg() {
        let resp = serve_sui_icon(DaemonMode::Development, "search");
        assert_eq!(resp.status, 200, "lucide ships `search`");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.starts_with("<svg"));
        assert!(body.contains("viewBox=\"0 0 24 24\""));
        assert!(body.contains("stroke=\"currentColor\""));
    }

    #[test]
    fn icon_alias_resolves_server_side() {
        let direct = serve_sui_icon(DaemonMode::Development, "git-branch");
        let aliased = serve_sui_icon(DaemonMode::Development, "git");
        assert_eq!(direct.status, 200);
        assert_eq!(aliased.status, 200);
        assert_eq!(direct.body, aliased.body, "alias must serve identical bytes");
    }

    #[test]
    fn icon_unknown_returns_404() {
        let resp = serve_sui_icon(DaemonMode::Development, "definitely-not-a-real-icon");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_path_traversal_rejected() {
        let resp = serve_sui_icon(DaemonMode::Development, "../etc/passwd");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn icon_empty_name_rejected() {
        let resp = serve_sui_icon(DaemonMode::Development, "");
        assert_eq!(resp.status, 404);
    }

    // ── serve_scaffold ────────────────────────────────────────────────────────

    /// Build a minimal ScaffoldDecl in-memory for tests.
    fn make_scaffold(
        name: &str,
        asset_prefix: &str,
        inject_globals: Vec<(&str, &str)>,
    ) -> ScaffoldDecl {
        use std::collections::BTreeMap;
        let mut globals = BTreeMap::new();
        for (k, v) in inject_globals {
            globals.insert(k.to_string(), v.to_string());
        }
        ScaffoldDecl {
            name: name.to_string(),
            route: "/".to_string(),
            source: "nonexistent-scaffold".to_string(),
            asset_prefix: asset_prefix.to_string(),
            inject_globals: globals,
        }
    }

    #[test]
    fn serve_scaffold_missing_source_returns_404() {
        let scaffold = make_scaffold("test", "/test/_assets/", vec![]);
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[]);
        assert_eq!(resp.status, 404);
        let body = std::str::from_utf8(&resp.body).unwrap();
        // Styled 404, not raw text — should contain HTML.
        assert!(body.contains("nonexistent-scaffold") || body.contains("404") || body.contains("not found"),
            "404 body should mention the scaffold source: {body}");
    }

    #[test]
    fn substitute_params_replaces_dollar_token() {
        assert_eq!(substitute_params("$id", &[("id", "my-space")]), "my-space");
        assert_eq!(substitute_params("prefix-$id-suffix", &[("id", "x")]), "prefix-x-suffix");
        assert_eq!(substitute_params("no-token", &[("id", "x")]), "no-token");
        // Unknown token stays as-is (no panic, no empty string).
        assert_eq!(substitute_params("$other", &[("id", "x")]), "$other");
    }

    #[test]
    fn substitute_params_multiple_tokens() {
        let result = substitute_params("$a/$b", &[("a", "hello"), ("b", "world")]);
        assert_eq!(result, "hello/world");
    }

    #[test]
    fn default_space_scaffold_has_deck_app_id_global() {
        let scaffold = default_space_scaffold();
        assert_eq!(scaffold.inject_globals.get("DECK_APP_ID").map(String::as_str), Some("$id"));
        assert_eq!(scaffold.source, "space-scaffold");
        assert_eq!(scaffold.asset_prefix, "/space/_assets/");
    }

    #[test]
    fn default_shell_scaffold_has_no_globals() {
        let scaffold = default_shell_scaffold();
        assert!(scaffold.inject_globals.is_empty());
        assert_eq!(scaffold.source, "hub-scaffold");
        assert_eq!(scaffold.asset_prefix, "/shell/_assets/");
    }

    /// serve_scaffold with a real (tmpdir) HTML source rewrites asset hrefs
    /// and injects globals. Uses the env-var resolver (no caching — the resolver
    /// reads DECK_PROJECT_DIR on every call). The env var is restored after the
    /// test so other tests that rely on it being absent are not affected.
    #[test]
    fn serve_scaffold_rewrites_assets_and_injects_globals() {
        use crate::test_lock;
        let _lock = test_lock::lock();

        // Capture the previous value so we can restore it after the test.
        let prev = std::env::var("DECK_PROJECT_DIR").ok();

        let dir = tempfile::tempdir().unwrap();
        // deck_out_root() = DECK_PROJECT_DIR/.deck
        let dot_deck = dir.path().join(".deck");
        let scaffold_dir = dot_deck.join("my-scaffold");
        std::fs::create_dir_all(&scaffold_dir).unwrap();
        let html = r#"<!doctype html><html><head></head><body><script src="/assets/main.js"></script></body></html>"#;
        std::fs::write(scaffold_dir.join("index.html"), html).unwrap();

        // Point the resolver at the tmpdir.
        unsafe { std::env::set_var("DECK_PROJECT_DIR", dir.path()) };

        let mut globals = std::collections::BTreeMap::new();
        globals.insert("DECK_APP_ID".to_string(), "$id".to_string());
        let scaffold = ScaffoldDecl {
            name: "space".to_string(),
            route: "/space/{id}".to_string(),
            source: "my-scaffold".to_string(),
            asset_prefix: "/space/_assets/".to_string(),
            inject_globals: globals,
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[("id", "my-space")]);

        // Restore env before asserting — a panic below still cleans up.
        unsafe {
            match prev {
                Some(v) => std::env::set_var("DECK_PROJECT_DIR", v),
                None => std::env::remove_var("DECK_PROJECT_DIR"),
            }
        }

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        // Asset href rewritten.
        assert!(body.contains("/space/_assets/main.js"), "expected rewritten asset href in: {body}");
        // Global injected with substituted id.
        assert!(body.contains("window.DECK_APP_ID=\"my-space\""), "expected injected global in: {body}");
    }

    /// serve_scaffold falls back to the authored SOURCE `<source>/index.html`
    /// when there is no built `.deck/<source>/` artifact — the dev/source path
    /// (mirrors view.rs). Gates the fallback so the demo (and any host) never
    /// needs to commit a `.deck/` artifact to serve a scaffold.
    #[test]
    fn serve_scaffold_falls_back_to_source_when_unbuilt() {
        use crate::test_lock;
        let _lock = test_lock::lock();
        let prev = std::env::var("DECK_PROJECT_DIR").ok();

        let dir = tempfile::tempdir().unwrap();
        // SOURCE location: <root>/my-scaffold/index.html — NOT under .deck/.
        let src_dir = dir.path().join("my-scaffold");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("index.html"),
            r#"<!doctype html><html><head></head><body><div id="sui-root"></div></body></html>"#,
        )
        .unwrap();
        // Deliberately NO .deck/my-scaffold — only the source exists.
        unsafe { std::env::set_var("DECK_PROJECT_DIR", dir.path()) };

        let scaffold = ScaffoldDecl {
            name: "space".to_string(),
            route: "/space/{id}".to_string(),
            source: "my-scaffold".to_string(),
            asset_prefix: "/space/_assets/".to_string(),
            inject_globals: std::collections::BTreeMap::new(),
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[("id", "x")]);

        unsafe {
            match prev {
                Some(v) => std::env::set_var("DECK_PROJECT_DIR", v),
                None => std::env::remove_var("DECK_PROJECT_DIR"),
            }
        }

        assert_eq!(resp.status, 200, "source-fallback must serve the unbuilt scaffold");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("sui-root"), "served source HTML, got: {body}");
    }

    // ── serve_scaffold_asset path validation ─────────────────────────────────

    #[test]
    fn scaffold_asset_dotdot_rejected() {
        let scaffold = make_scaffold("test", "/test/_assets/", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, "../../../etc/passwd");
        assert_eq!(resp.status, 404, "'..' traversal must be rejected");
        let body = std::str::from_utf8(&resp.body).unwrap();
        assert!(body.contains("invalid asset path") || body.contains("404") || body.contains("not found"),
            "must produce a safe 404; got: {body}");
    }

    #[test]
    fn scaffold_asset_slash_in_name_rejected() {
        let scaffold = make_scaffold("test", "/test/_assets/", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, "sub/dir/file.js");
        assert_eq!(resp.status, 404, "slash in filename must be rejected");
    }

    #[test]
    fn scaffold_asset_leading_dot_rejected() {
        let scaffold = make_scaffold("test", "/test/_assets/", vec![]);
        let resp = serve_scaffold_asset(DaemonMode::Development, &scaffold, ".hidden");
        assert_eq!(resp.status, 404, "leading dot must be rejected");
    }

    // ── is_valid_js_identifier ────────────────────────────────────────────────

    #[test]
    fn valid_js_identifiers_accepted() {
        assert!(is_valid_js_identifier("DECK_APP_ID"));
        assert!(is_valid_js_identifier("myVar"));
        assert!(is_valid_js_identifier("_private"));
        assert!(is_valid_js_identifier("x"));
        assert!(is_valid_js_identifier("abc123"));
    }

    #[test]
    fn invalid_js_identifiers_rejected() {
        assert!(!is_valid_js_identifier(""));
        assert!(!is_valid_js_identifier("0starts_with_digit"));
        assert!(!is_valid_js_identifier("has-hyphen"));
        assert!(!is_valid_js_identifier("has space"));
        assert!(!is_valid_js_identifier("x;alert(1)//"));
        assert!(!is_valid_js_identifier("$dollar"));
        assert!(!is_valid_js_identifier("has.dot"));
    }

    /// A scaffold with an invalid inject_globals key must not emit that key in
    /// the <script> block. The request that used to allow script injection now
    /// skips the bad key and either produces an empty inject block or only
    /// includes valid keys.
    #[test]
    fn inject_globals_invalid_key_skipped_not_injected() {
        use crate::test_lock;
        let _lock = test_lock::lock();
        let prev = std::env::var("DECK_PROJECT_DIR").ok();

        let dir = tempfile::tempdir().unwrap();
        let dot_deck = dir.path().join(".deck").join("safe-scaffold");
        std::fs::create_dir_all(&dot_deck).unwrap();
        let html = r#"<!doctype html><html><head></head><body></body></html>"#;
        std::fs::write(dot_deck.join("index.html"), html).unwrap();

        unsafe { std::env::set_var("DECK_PROJECT_DIR", dir.path()) };

        let mut globals = std::collections::BTreeMap::new();
        // Invalid key: would inject `alert(1)` if not validated.
        globals.insert("x;alert(1)//".to_string(), "injected".to_string());
        // Valid key alongside it — should still appear.
        globals.insert("SAFE_KEY".to_string(), "safe_value".to_string());

        let scaffold = ScaffoldDecl {
            name: "test".to_string(),
            route: "/".to_string(),
            source: "safe-scaffold".to_string(),
            asset_prefix: "/test/_assets/".to_string(),
            inject_globals: globals,
        };
        let resp = serve_scaffold(DaemonMode::Development, &scaffold, &[]);

        unsafe {
            match prev {
                Some(v) => std::env::set_var("DECK_PROJECT_DIR", v),
                None => std::env::remove_var("DECK_PROJECT_DIR"),
            }
        }

        assert_eq!(resp.status, 200);
        let body = std::str::from_utf8(&resp.body).unwrap();
        // The malicious key must NOT appear in the output.
        assert!(
            !body.contains("alert(1)"),
            "invalid inject_globals key must be stripped; got: {body}"
        );
        // The valid key must still be injected.
        assert!(
            body.contains("window.SAFE_KEY"),
            "valid inject_globals key must be injected; got: {body}"
        );
    }
}
