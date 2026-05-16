//! Shared static assets embedded at compile time + their serve handlers.
//!
//! Post-j2/Lit deletion: only SUI-era assets remain. SUI bundles live under
//! `crates/sui/ship-sui-server/assets/` and are served via
//! `ship_sui_server`; this module owns view-tokens.css (design tokens +
//! themes) and the aggregate CEM consumed by tooling.

use axum::http::StatusCode;

/// Shared design tokens + base styles for all daemon-served views.
/// Canonical OKLCH palette sourced from `@ship/ui-components/tokens/tokens.css`
/// so SUI components, daemon views, and theme studio can never drift.
/// View-scoped base rules live alongside in `assets/view-tokens.css`.
const VIEW_TOKENS_CSS: &str = concat!(
    include_str!("../../../../packages/ui-components/src/tokens/tokens.css"),
    "\n",
    include_str!("../../assets/view-tokens.css"),
    "\n",
    // Per-theme overrides — :root[data-theme='paper'] etc. Applied by
    // the theme picker toggling `data-theme` on <html>.
    include_str!("../../../../packages/ui-components/src/tokens/themes.css"),
);

/// Aggregated Custom Elements Manifest for SUI components (one module per
/// `<sui-*>` tag). Built by `runtime-assets/bundler.ts` from the
/// per-component CEMs in `packages/ui-components/src/<tag>/` and written
/// to `crates/sui/ship-sui-server/assets/sui-custom-elements.json`.
const SUI_ELEMENTS_MANIFEST: &str =
    include_str!("../../../../crates/sui/ship-sui-server/assets/sui-custom-elements.json");

/// GET /api/v1/runtime/assets/view-tokens.css
#[utoipa::path(
    operation_id = "serveViewTokens",
    get,
    path = "/api/assets/view-tokens.css",
    responses(
        (status = 200, description = "Design tokens + base CSS for views", content_type = "text/css"),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    security(),
    tag = "views"
)]
pub async fn serve_view_tokens() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")
        // No-cache so theme/token edits land on the next load without
        // needing a hard refresh. Production builds can add ETags.
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(VIEW_TOKENS_CSS))
        .unwrap()
}

/// GET /api/assets/sui-custom-elements.json — aggregated SUI component CEMs.
pub async fn serve_sui_elements_manifest() -> axum::response::Response {
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(axum::body::Body::from(SUI_ELEMENTS_MANIFEST))
        .unwrap()
}

/// GET /api/v1/runtime/assets/theme.css
/// Returns the active theme's CSS overrides. No-cache so theme changes apply immediately.
#[utoipa::path(
    operation_id = "serveThemeCss",
    get,
    path = "/api/assets/theme.css",
    responses(
        (status = 200, description = "Active theme CSS overrides", content_type = "text/css"),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    security(),
    tag = "views"
)]
pub async fn serve_theme_css() -> axum::response::Response {
    let css = crate::views::themes::resolve_theme_css();
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(css))
        .unwrap()
}

/// GET /api/v1/runtime/themes
/// Returns JSON list of all available themes.
#[utoipa::path(
    operation_id = "serveThemesList",
    get,
    path = "/api/v1/themes",
    responses(
        (status = 200, description = "Available themes", content_type = "application/json"),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    security(),
    tag = "views"
)]
pub async fn serve_themes_list() -> axum::response::Response {
    let themes = crate::views::themes::list_themes();
    let json = serde_json::to_string(&themes).unwrap_or_else(|_| "[]".to_string());
    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(axum::http::header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(json))
        .unwrap()
}
