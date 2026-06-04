use crate::{DaemonMode, DeckResponse};

const ASSETS_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// deck's OWN baked product (sui-runtime/dom/sdk-client + primitives umbrella +
/// lucide). These ship inside the deck-core crate, like Next shipping its own
/// runtime. App + user content is NOT here — see `read_deck_asset`.
fn read_asset(rel: &str) -> Option<String> {
    let path = std::path::Path::new(ASSETS_DIR).join("assets").join(rel);
    std::fs::read_to_string(path).ok()
}

/// App + user build output, read from the host-supplied `.deck/` tree (deck's
/// `.next`-equivalent). deck bakes none of this — view + chrome bundles are the
/// project's content, served from where the project built them. Returns None
/// when there is no project root (the on-demand compile path then takes over).
fn read_deck_asset(rel: &str) -> Option<String> {
    let root = crate::resolver::deck_out_root()?;
    std::fs::read_to_string(root.join(rel)).ok()
}

const SUI_RUNTIME_JS: &str = include_str!("../assets/sui-runtime.js");
const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str = include_str!("../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str = include_str!("../assets/sui-components.js");
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
/// map once at first access. Holds the deserialized map in a `OnceLock` so
/// every `/_sui/icons/{name}.svg` request is a single HashMap lookup.
fn icon_map() -> &'static std::collections::HashMap<String, String> {
    use std::sync::OnceLock;
    static MAP: OnceLock<std::collections::HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(SUI_LUCIDE_ICONS_JSON)
            .expect("deck-core lucide-icons.json failed to deserialize at startup")
    })
}

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

// ── Space + shell scaffolds (apps/spaces → .deck/space-scaffold;
//                              apps/shell → .deck/hub-scaffold) ──────────────
//
// A Tauri `--app=<id>` window navigates to `/space/<id>`; the no-`--app`
// (shell) window navigates to the daemon root `/`. The daemon serves both as
// SAME-ORIGIN HTML so the scaffold's `/_sui` + Connect resolve to this daemon
// with no proxy/origin-injection. Spaces plant a launched id; the shell
// plants nothing (the `home` space is the implicit context). Each scaffold's
// hashed asset refs are rewritten to `/{space,shell}/_assets/`.

pub fn serve_space(mode: DaemonMode, id: &str) -> DeckResponse {
    let Some(html) = read_deck_asset("space-scaffold/index.html") else {
        return DeckResponse::not_found("space scaffold not built (run apps/spaces build)", mode);
    };
    // JSON-encode the id so it is a safe string literal in the injected script.
    let id_js = serde_json::to_string(id).unwrap_or_else(|_| "\"\"".to_string());
    let inject = format!("<script>window.__SHIP_APP_ID__ = {id_js};</script>");
    let html = html
        .replacen("<head>", &format!("<head>{inject}"), 1)
        .replace("\"/assets/", "\"/space/_assets/");
    let html = crate::mode_html::inject_mode_meta(&html, mode);
    DeckResponse::html_view(html, mode)
}

pub fn serve_space_asset(mode: DaemonMode, file: &str) -> DeckResponse {
    match read_deck_asset(&format!("space-scaffold/assets/{file}")) {
        Some(body) if file.ends_with(".css") => DeckResponse::css_view(body, mode),
        Some(body) => DeckResponse::js_asset(body, mode),
        None => DeckResponse::not_found(format!("space asset not found: {file}"), mode),
    }
}

/// `GET /` — the shell scaffold (apps/shell), the Ship platform app. Tauri's
/// window loads the daemon root so the shell loads SAME-ORIGIN — `/_sui` +
/// Connect resolve to this daemon with no proxy, the same model as
/// `/space/<id>`. The shell IS the platform root; spaces get their own
/// windows at `/space/<id>`. Shell assets are served at `/shell/_assets/`
/// (symmetric with `/space/_assets/`).
pub fn serve_shell(mode: DaemonMode) -> DeckResponse {
    let Some(html) = read_deck_asset("hub-scaffold/index.html") else {
        return DeckResponse::not_found(
            "shell scaffold not built (missing hub-scaffold/index.html asset)",
            mode,
        );
    };
    let html = html.replace("\"/assets/", "\"/shell/_assets/");
    let html = crate::mode_html::inject_mode_meta(&html, mode);
    DeckResponse::html_view(html, mode)
}

pub fn serve_shell_asset(mode: DaemonMode, file: &str) -> DeckResponse {
    match read_deck_asset(&format!("hub-scaffold/assets/{file}")) {
        Some(body) if file.ends_with(".css") => DeckResponse::css_view(body, mode),
        Some(body) => DeckResponse::js_asset(body, mode),
        None => DeckResponse::not_found(format!("shell asset not found: {file}"), mode),
    }
}

// Framework bundle handlers

pub fn serve_sui_runtime(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("sui-runtime.js").unwrap_or_else(|| SUI_RUNTIME_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_dom(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("sui-dom.js").unwrap_or_else(|| SUI_DOM_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_sdk_client(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("sui-sdk-client.js").unwrap_or_else(|| SUI_SDK_CLIENT_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_components(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("sui-components.js").unwrap_or_else(|| SUI_COMPONENTS_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

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
    let resolved = if icon_map().contains_key(name) {
        name
    } else if let Some(via) = alias_for(name) {
        via
    } else {
        return DeckResponse::not_found(format!("icon not in lucide set: {name}"), mode);
    };
    let inner = icon_map().get(resolved).expect("resolved name in map");
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
}
