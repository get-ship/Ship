//! Shared blocking HTTP client for daemon-bound calls.
//!
//! Stamps `X-Shipd-Client-Build` on every outgoing request so the daemon
//! can detect CLI→server skew. Provides `check_skew` for callers that
//! want a one-time WARN when the daemon's `X-Shipd-Build` response header
//! diverges from the CLI's own build SHA.

use std::sync::OnceLock;

/// Header name the daemon stamps on every response (set by shipd's
/// `build_header_middleware`).
const SERVER_HEADER: &str = "x-shipd-build";
/// Header name the CLI stamps on every request.
const CLIENT_HEADER: &str = "x-shipd-client-build";

/// Build a blocking reqwest client pre-configured with the CLI's build
/// identity in the default headers. Callers add `Authorization` per call.
pub fn daemon_client() -> reqwest::blocking::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    let bi = build_info::build_info!();
    if let Ok(value) = reqwest::header::HeaderValue::from_str(&bi.short()) {
        headers.insert(CLIENT_HEADER, value);
    }
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

/// Once-per-process skew warning. The first response that mismatches
/// the CLI's build SHA prints a WARN to stderr; subsequent mismatches
/// are silent (deduplication is process-local).
///
/// No-op if the response lacks `X-Shipd-Build` (older daemon, or non-Ship
/// service responding).
pub fn check_skew(headers: &reqwest::header::HeaderMap) {
    static WARNED: OnceLock<()> = OnceLock::new();

    let Some(server_value) = headers.get(SERVER_HEADER).and_then(|v| v.to_str().ok()) else {
        return;
    };
    let bi = build_info::build_info!();
    let client_short = bi.short();
    if server_value == client_short {
        return;
    }
    // Strip dirty suffix for the comparison so a daemon built clean from
    // the same SHA as a dirty CLI doesn't trip the warning.
    let server_sha = server_value.split('+').next().unwrap_or("").to_string();
    let client_sha = bi.git_sha;
    if server_sha == client_sha {
        return;
    }

    if WARNED.set(()).is_ok() {
        eprintln!(
            "warn: shipd build ({server_value}) differs from ship CLI build ({client_short})"
        );
        eprintln!("      run `ship doctor` for details, or restart the daemon to pick up new builds");
    }
}
