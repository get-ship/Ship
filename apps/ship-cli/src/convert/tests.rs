use super::*;
use crate::mcp::McpFile;
use tempfile::TempDir;

// ── getship URL detection ────────────────────────────────────────────────────

#[test]
fn is_getship_url_matches_valid_urls() {
    assert!(is_getship_url("https://getship.dev/p/rust-expert"));
    assert!(is_getship_url("https://getship.dev/profiles/cli-lane"));
    assert!(is_getship_url("https://www.getship.dev/p/test"));
    assert!(!is_getship_url("https://example.com/p/test"));
    assert!(!is_getship_url("/some/local/path"));
}

#[test]
fn extract_profile_id_from_url() {
    assert_eq!(
        extract_profile_id("https://getship.dev/p/rust-expert").unwrap(),
        "rust-expert"
    );
    assert_eq!(
        extract_profile_id("https://getship.dev/profiles/cli-lane").unwrap(),
        "cli-lane"
    );
}

#[test]
fn extract_profile_id_single_segment() {
    assert_eq!(
        extract_profile_id("https://getship.dev/my-profile").unwrap(),
        "my-profile"
    );
}

#[test]
fn extract_profile_id_empty_path_fails() {
    assert!(extract_profile_id("https://getship.dev/").is_err());
}
