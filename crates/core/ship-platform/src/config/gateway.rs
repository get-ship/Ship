//! Gateway-resolution product logic for the config service.
//!
//! The daemon's registry route used to read `library/settings.jsonc` and
//! `machine/credentials.jsonc` directly and apply default/trim logic inline.
//! That precedence is product logic — it lives here, over the typed config
//! documents, so the daemon route is thin transport that just calls these.

use super::{Credentials, DiskBackedConfig, library::LibrarySettings};

/// Default Ship Cloud gateway base URL when none is configured.
pub const DEFAULT_GATEWAY: &str = "https://api.getship.dev";

/// Resolve the effective gateway base URL.
///
/// Precedence: `library/settings.jsonc:cloud.base_url`, else [`DEFAULT_GATEWAY`].
/// Always returned without a trailing slash.
pub fn resolve_gateway_base_url() -> String {
    let configured = LibrarySettings::load()
        .cloud
        .and_then(|c| c.base_url)
        .filter(|u| !u.trim().is_empty());
    normalize_base_url(configured.as_deref())
}

/// Normalize a configured base URL (or `None`) into a trailing-slash-free
/// string, falling back to the default. Split out so it is unit-testable
/// without touching disk.
fn normalize_base_url(configured: Option<&str>) -> String {
    configured
        .unwrap_or(DEFAULT_GATEWAY)
        .trim()
        .trim_end_matches('/')
        .to_string()
}

/// Read the stored Ship Cloud account token from machine-local credentials.
///
/// Returns `None` when the machine is not logged in.
pub fn account_token() -> Option<String> {
    Credentials::load().account_token().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_falls_back_to_default() {
        assert_eq!(normalize_base_url(None), DEFAULT_GATEWAY);
    }

    #[test]
    fn normalize_trims_trailing_slash() {
        assert_eq!(
            normalize_base_url(Some("https://example.com/")),
            "https://example.com"
        );
    }

    #[test]
    fn normalize_preserves_path_without_trailing_slash() {
        assert_eq!(
            normalize_base_url(Some("https://example.com/api")),
            "https://example.com/api"
        );
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_base_url(Some("  https://example.com  ")),
            "https://example.com"
        );
    }
}
