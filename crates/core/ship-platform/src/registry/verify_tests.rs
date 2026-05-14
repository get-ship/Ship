use super::*;
use ship_substrate::env::Var;
use ship_substrate::testutil::EnvGuard;

// ── find_version_hash unit tests ──────────────────────────────────────

#[test]
fn find_hash_matching_version() {
    let body = serde_json::json!({
        "versions": [
            { "tag": "v1.0.0", "contentHash": "sha256:aaa" },
            { "tag": "v2.0.0", "contentHash": "sha256:bbb" },
        ]
    });
    assert_eq!(
        find_version_hash(&body, "v1.0.0"),
        Some("sha256:aaa".to_string())
    );
    assert_eq!(
        find_version_hash(&body, "v2.0.0"),
        Some("sha256:bbb".to_string())
    );
}

#[test]
fn find_hash_no_matching_version() {
    let body = serde_json::json!({
        "versions": [
            { "tag": "v1.0.0", "contentHash": "sha256:aaa" },
        ]
    });
    assert_eq!(find_version_hash(&body, "v3.0.0"), None);
}

#[test]
fn find_hash_missing_versions_field() {
    let body = serde_json::json!({ "name": "pkg" });
    assert_eq!(find_version_hash(&body, "v1.0.0"), None);
}

#[test]
fn find_hash_empty_versions_array() {
    let body = serde_json::json!({ "versions": [] });
    assert_eq!(find_version_hash(&body, "v1.0.0"), None);
}

#[test]
fn find_hash_version_without_content_hash() {
    let body = serde_json::json!({
        "versions": [
            { "tag": "v1.0.0" },
        ]
    });
    assert_eq!(find_version_hash(&body, "v1.0.0"), None);
}

#[test]
fn find_hash_versions_not_array() {
    let body = serde_json::json!({ "versions": "not-an-array" });
    assert_eq!(find_version_hash(&body, "v1.0.0"), None);
}

// ── registry_base_url unit tests ──────────────────────────────────────

#[test]
fn base_url_defaults_to_getship() {
    let _env = EnvGuard::unset(Var::RegistryUrl);
    let url = registry_base_url();
    assert_eq!(url, Some("https://getship.dev".to_string()));
}

#[test]
fn base_url_respects_env_override() {
    let _env = EnvGuard::set(Var::RegistryUrl, "http://localhost:3000");
    let url = registry_base_url();
    assert_eq!(url, Some("http://localhost:3000".to_string()));
}

#[test]
fn base_url_empty_string_opts_out() {
    let _env = EnvGuard::set(Var::RegistryUrl, "");
    let url = registry_base_url();
    assert_eq!(url, None);
}

// ── check_registry_hash unit tests ────────────────────────────────────

#[test]
fn check_skipped_when_offline() {
    let result = check_registry_hash("github.com/owner/pkg", "v1.0.0", "sha256:abc", true);
    assert_eq!(result, None);
}

#[test]
fn check_returns_none_when_registry_unavailable() {
    let _env = EnvGuard::set(Var::RegistryUrl, "http://127.0.0.1:1");
    let result = check_registry_hash("github.com/owner/pkg", "v1.0.0", "sha256:abc", false);
    assert_eq!(result, None);
}

#[test]
fn check_returns_none_when_opted_out() {
    let _env = EnvGuard::set(Var::RegistryUrl, "");
    let result = check_registry_hash("github.com/owner/pkg", "v1.0.0", "sha256:abc", false);
    assert_eq!(result, None);
}

// ── fetch_registry_hash integration-style tests ───────────────────────

#[test]
fn fetch_returns_none_on_unreachable_registry() {
    let _env = EnvGuard::set(Var::RegistryUrl, "http://127.0.0.1:1");
    let result = fetch_registry_hash("github.com/owner/pkg", "v1.0.0");
    assert_eq!(result, None);
}

#[test]
fn fetch_returns_none_when_opted_out() {
    let _env = EnvGuard::set(Var::RegistryUrl, "");
    let result = fetch_registry_hash("github.com/owner/pkg", "v1.0.0");
    assert_eq!(result, None);
}
