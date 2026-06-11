//! HTTP-coupled session bits — `POST /api/v1/runtime/session` issuance.
//!
//! Pure session helpers (generation/parsing/lookup/registry) moved to
//! `ship_auth::session`. This file holds only the axum-typed
//! handler + its response shape.

// Re-export the platform's pure helpers so existing
// `crate::auth::session::*` import paths continue to resolve.
pub use ship_auth::session::{
    BROWSER_SESSION_TTL_SECS, COOKIE_NAME, generate_browser_session_id, is_browser_session_id,
    issue_browser_session, issue_browser_session_with_ttl, lookup_browser_session,
    parse_session_cookie,
};


