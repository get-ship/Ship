//! `DeckResponse` — the transport-agnostic result every deck handler produces.
//!
//! deck-core never speaks HTTP. Handler logic returns a `DeckResponse`
//! (status + headers + body); a server adapter (deck-axum, a future
//! deck-bun, …) maps it onto its own response type. This is the port; the
//! adapters are the plugs.

use crate::DaemonMode;

/// Cache-control family. Bundles are immutable-in-prod; view artifacts must
/// revalidate. Picked per-response so the adapter needs no mode awareness.
#[derive(Clone, Copy, Debug)]
pub enum CachePolicy {
    /// Framework bundles (runtime/dom/components/islands) + per-icon lucide SVGs.
    Asset,
    /// View artifacts (template/setup/css/payload/host).
    View,
    /// Never cache (errors, not-found).
    None,
}

impl CachePolicy {
    fn header(self, mode: DaemonMode) -> &'static str {
        match self {
            CachePolicy::None => "no-cache",
            CachePolicy::Asset => match mode {
                DaemonMode::Development => "no-cache",
                DaemonMode::Production => "public, immutable, max-age=31536000",
            },
            CachePolicy::View => match mode {
                DaemonMode::Development => "no-cache",
                DaemonMode::Production => "must-revalidate, max-age=0",
            },
        }
    }
}

/// A framework-agnostic HTTP-shaped response. Adapters translate it 1:1.
#[derive(Clone, Debug)]
pub struct DeckResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub cache_control: &'static str,
    /// Strong ETag (Production only); `None` omits the header.
    pub etag: Option<String>,
    /// Additional headers beyond content-type/cache/etag (e.g. `X-Sui-View`).
    pub extra_headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl DeckResponse {
    /// Core constructor. ETag is computed for cacheable bodies in Production
    /// only — matching the daemon's prior per-handler behavior.
    pub fn new(
        status: u16,
        content_type: &'static str,
        cache: CachePolicy,
        mode: DaemonMode,
        body: impl Into<Vec<u8>>,
    ) -> Self {
        let body = body.into();
        let etag = match (cache, mode) {
            (CachePolicy::None, _) | (_, DaemonMode::Development) => None,
            (_, DaemonMode::Production) => Some(compute_etag(&body)),
        };
        Self {
            status,
            content_type,
            cache_control: cache.header(mode),
            etag,
            extra_headers: Vec::new(),
            body,
        }
    }

    pub fn js_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/javascript; charset=utf-8", CachePolicy::Asset, mode, body)
    }
    pub fn js_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/javascript; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn css_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "text/css; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn html_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "text/html; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn text_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "text/plain; charset=utf-8", CachePolicy::View, mode, body)
    }
    pub fn json_asset(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/json; charset=utf-8", CachePolicy::Asset, mode, body)
    }
    pub fn json_view(body: impl Into<Vec<u8>>, mode: DaemonMode) -> Self {
        Self::new(200, "application/json; charset=utf-8", CachePolicy::View, mode, body)
    }

    /// 404 with a plain-text reason (no cache, no etag).
    ///
    /// In Production the caller-supplied `msg` is suppressed and a generic
    /// `"not found"` body is returned — `msg` routinely carries internal detail
    /// (absolute on-disk paths, resolver attempt lists, build hints) that must
    /// never reach prod clients. In Development the reason is surfaced verbatim
    /// to aid debugging. Mirrors `render_error`'s mode-gated redaction.
    pub fn not_found(msg: impl Into<String>, mode: DaemonMode) -> Self {
        let body = match mode {
            DaemonMode::Production => "not found".to_string(),
            DaemonMode::Development => msg.into(),
        };
        let mut r = Self::new(404, "text/plain; charset=utf-8", CachePolicy::None, mode, body.into_bytes());
        r.cache_control = "no-cache";
        r
    }

    /// 500 — in Production the message is hidden; in Development it's surfaced.
    pub fn render_error(msg: impl Into<String>, mode: DaemonMode) -> Self {
        let body = match mode {
            DaemonMode::Production => "render error".to_string(),
            DaemonMode::Development => msg.into(),
        };
        Self::new(500, "text/plain; charset=utf-8", CachePolicy::None, mode, body.into_bytes())
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.extra_headers.push((name, value.into()));
        self
    }
}

/// Strong ETag derived from content bytes (fast non-cryptographic hash).
pub fn compute_etag(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    format!("\"{}\"", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_dev_surfaces_reason() {
        let r = DeckResponse::not_found("/abs/path/views/foo missing", DaemonMode::Development);
        assert_eq!(r.status, 404);
        assert_eq!(r.cache_control, "no-cache");
        assert!(r.etag.is_none());
        assert_eq!(r.body, b"/abs/path/views/foo missing");
    }

    #[test]
    fn not_found_prod_redacts_reason() {
        // Caller-supplied detail (absolute paths, build hints, resolver attempt
        // lists) must never reach prod clients — only a generic body ships.
        let r = DeckResponse::not_found("/abs/path/views/foo missing", DaemonMode::Production);
        assert_eq!(r.status, 404);
        let body = std::str::from_utf8(&r.body).unwrap();
        assert_eq!(body, "not found");
        assert!(!body.contains("/abs/path"), "prod 404 must not echo on-disk paths");
        // 404s are never cached and never carry an ETag, in either mode.
        assert_eq!(r.cache_control, "no-cache");
        assert!(r.etag.is_none());
    }
}
