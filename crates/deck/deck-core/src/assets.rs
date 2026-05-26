use crate::{DaemonMode, DeckResponse};

// Per-tag island bundles are served by file existence under
// `assets/sui-components/`. The bundler (packages/deck/bundler/index.ts +
// packages/deck/build via the umbrella's sui.config.ts `lazyOutSubdir`)
// is the source of truth for what lands there. The previous
// hand-maintained `is_known_island` allowlist was retired alongside
// the workspace-catalog cut-over — origin metadata in
// `.deck/catalogs/workspace.json` makes per-tag gating
// redundant. Path-traversal is rejected explicitly in
// `serve_sui_island` so a missing-bundle 404 is the only failure mode.

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

// Public helper (used by view.rs). deck bakes NO app views — per-view bundles
// live in the project's `.deck/views/<name>/view.js`; absent that, view.rs
// compiles on demand from source.

pub fn resolve_view_js(name: &str) -> Option<String> {
    read_deck_asset(&format!("views/{name}/view.js"))
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

pub fn serve_sui_lucide_icons(mode: DaemonMode) -> DeckResponse {
    let json = read_asset("lucide-icons.json").unwrap_or_else(|| SUI_LUCIDE_ICONS_JSON.to_string());
    DeckResponse::json_asset(json, mode)
}

/// `GET /_sui/live-reload.js` — the one HMR client, shared by the view
/// host and the shell. Served in every mode (it self-gates on the
/// `ship-mode` meta tag); only subscribes when the daemon is in dev.
pub fn serve_sui_live_reload(mode: DaemonMode) -> DeckResponse {
    let js = read_asset("live-reload.js").unwrap_or_else(|| LIVE_RELOAD_JS.to_string());
    DeckResponse::js_asset(js, mode)
}

pub fn serve_sui_island(mode: DaemonMode, file: &str) -> DeckResponse {
    // Reject path-traversal explicitly — the file segment is captured
    // verbatim from the URL, so a malicious `../../etc/passwd` would
    // otherwise escape the assets dir.
    if file.contains("..") || file.contains('/') {
        return DeckResponse::not_found("invalid island name");
    }
    // The route captures the whole filename (`sui-rail.js`) because axum 0.8
    // forbids mixing path-params with literal suffixes in the same segment.
    // Strip the `.js` to recover the bundle name.
    let bundle = file.strip_suffix(".js").unwrap_or(file).to_string();
    // Single canonical path: the project's `.deck/sui-components/<bundle>.js`.
    // Code-split component + chrome bundles are app/user content, so they live
    // in the project's deck build output — never baked into deck's own crate.
    // A missing bundle is a 404 with an actionable hint (no baked fallback).
    let rel = format!("sui-components/{bundle}.js");
    match read_deck_asset(&rel) {
        Some(js) => DeckResponse::js_asset(js, mode),
        None => DeckResponse::not_found("island bundle missing — run `bun run packages/deck/bundler/index.ts --all`"),
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
    fn island_missing_returns_404() {
        // Islands are app/user content read from the project's `.deck/`; deck
        // bakes no island fallback (chrome is the shell app's, not the
        // framework's). A name with no `.deck/sui-components/<name>.js` 404s
        // with the bundler hint — there is no embedded copy to mask it.
        let resp = serve_sui_island(DaemonMode::Development, "definitely-not-a-real-bundle");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn island_path_traversal_rejected() {
        let resp = serve_sui_island(DaemonMode::Development, "../etc/passwd");
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn lucide_icons_returns_baked_fallback() {
        let resp = serve_sui_lucide_icons(DaemonMode::Development);
        assert_eq!(resp.status, 200);
    }
}
