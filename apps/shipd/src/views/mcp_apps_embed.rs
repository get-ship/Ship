//! MCP Apps bridge — declares the `_meta.ui` envelope Ship attaches to
//! view-session tool results and resource records, and owns the CSP policy
//! applied to the iframe shell. See
//! `.artifacts/phase-1-agent-canvas-views-spec.md` §"MCP Apps bridge".
//!
//! Phase 1 bridge is server-side only. The client-side
//! `postMessage` ↔ datastar mapping is implemented by MCP Apps hosts that
//! consume `ui://ship/view-session/{id}` — they receive `ui/render` /
//! `ui/signal` messages derived from the daemon's datastar SSE stream. The
//! `MESSAGE_MAPPING` constant documents the contract so downstream hosts
//! and our web SDK stay in sync.

use serde_json::{Value, json};

/// Stable resource URI scheme for an agent-opened canvas. Hosts subscribe
/// to the corresponding `/api/v1/view-sessions/{id}` SSE stream to receive
/// `datastar-patch-*` events; rendering of the shell itself is handled by
/// the host (e.g. a SUI view in the daemon's own UI, or an external MCP
/// Apps client).
pub const RESOURCE_URI_PREFIX: &str = "ui://ship/view-session/";

/// Content-Security-Policy applied to iframe shell responses. Script-src is
/// limited to `self` so only daemon-served assets (datastar.js,
/// ship-elements.js) can execute. Matches the `_meta.ui.csp` shape returned
/// to MCP Apps hosts.
pub const IFRAME_CSP: &str =
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
     connect-src 'self'; img-src 'self' data:";

/// postMessage ↔ datastar mapping. Documented as a static table so any
/// future bridge implementation (web SDK, studio, host plugins) stays
/// aligned. Columns: inbound MCP Apps message, outbound daemon action
/// (or reverse for daemon → host).
pub const MESSAGE_MAPPING: &str = "\
inbound: ui/initialize        → subscribe to /api/v1/view-sessions/{id}\n\
inbound: ui/context-update    → merge into session.context; re-render shell\n\
inbound: tools/call           → existing MCP tool call path\n\
outbound: datastar-patch-elements → ui/render { selector, fragment }\n\
outbound: datastar-patch-signals  → ui/signal { name, value }\n";

/// Build the `_meta.ui` object for a newly-opened view session. Hosts
/// resolve `resourceUri` to fetch the rendered HTML; `csp` mirrors what
/// the daemon serves so isolation is consistent across renderers.
pub fn view_session_meta(view_id: &str) -> Value {
    json!({
        "ui": {
            "resourceUri": format!("{RESOURCE_URI_PREFIX}{view_id}"),
            "csp": {
                "default-src": ["'self'"],
                "script-src": ["'self'"],
                "style-src": ["'self'", "'unsafe-inline'"],
                "connect-src": ["'self'"],
                "img-src": ["'self'", "data:"]
            },
            "permissions": []
        }
    })
}

/// Parse a `ui://ship/view-session/{id}` URI. Returns `Some(view_id)` when
/// the URI matches, `None` otherwise. Used by MCP resource resolvers so
/// external hosts can read the iframe HTML without hitting the daemon's
/// REST surface directly.
pub fn parse_view_session_uri(uri: &str) -> Option<&str> {
    uri.strip_prefix(RESOURCE_URI_PREFIX)
        .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_session_meta_includes_resource_uri_and_csp() {
        let meta = view_session_meta("01HZZXYZ");
        assert_eq!(
            meta["ui"]["resourceUri"],
            "ui://ship/view-session/01HZZXYZ"
        );
        assert_eq!(meta["ui"]["csp"]["script-src"][0], "'self'");
    }

    #[test]
    fn parse_view_session_uri_matches_ulid_shape() {
        assert_eq!(parse_view_session_uri("ui://ship/view-session/01HZZXYZ"), Some("01HZZXYZ"));
        assert_eq!(parse_view_session_uri("ui://ship/view-session/"), None);
        assert_eq!(parse_view_session_uri("ship://something"), None);
        // Non-alphanumeric (path traversal attempt) rejected.
        assert_eq!(parse_view_session_uri("ui://ship/view-session/../etc"), None);
    }

    #[test]
    fn csp_constant_is_self_restricted() {
        assert!(IFRAME_CSP.contains("script-src 'self'"));
        assert!(!IFRAME_CSP.contains("'unsafe-eval'"));
        assert!(!IFRAME_CSP.contains("*"));
    }

    #[test]
    fn message_mapping_covers_render_and_signal() {
        assert!(MESSAGE_MAPPING.contains("datastar-patch-elements"));
        assert!(MESSAGE_MAPPING.contains("datastar-patch-signals"));
        assert!(MESSAGE_MAPPING.contains("ui/initialize"));
    }
}
