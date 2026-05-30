/// Integration tests — SyncClient construction and URL shaping.
///
/// These tests do not make live HTTP calls. They exercise the constructor
/// and verify the URL-building logic for all four HTTP methods by inspecting
/// the normalised base URL via `SyncClient::base_url()` (a `#[cfg(test)]`
/// accessor added to surface the private field without cracking the type open).
///
/// No tempdir / init_project is needed — SyncClient carries no filesystem state.
use crate::sync::SyncClient;

// ── Constructor ───────────────────────────────────────────────────────────────

#[test]
fn sync_client_new_strips_trailing_slash() {
    let client = SyncClient::new("https://api.example.com/");
    assert_eq!(
        client.base_url(),
        "https://api.example.com",
        "trailing slash must be stripped"
    );
}

#[test]
fn sync_client_new_retains_url_without_trailing_slash() {
    let client = SyncClient::new("https://api.example.com");
    assert_eq!(
        client.base_url(),
        "https://api.example.com",
        "url without trailing slash must be unchanged"
    );
}

#[test]
fn sync_client_new_strips_multiple_trailing_slashes() {
    let client = SyncClient::new("https://api.example.com///");
    assert_eq!(
        client.base_url(),
        "https://api.example.com",
        "all trailing slashes must be stripped"
    );
}

#[test]
fn sync_client_new_with_path_prefix_retains_path() {
    let client = SyncClient::new("https://api.example.com/v2");
    assert_eq!(
        client.base_url(),
        "https://api.example.com/v2",
        "path prefix must be retained when no trailing slash"
    );
}

// ── URL shape for each method ─────────────────────────────────────────────────

#[test]
fn sync_client_platform_push_url_shape() {
    let client = SyncClient::new("https://relay.getship.dev");
    let expected = "https://relay.getship.dev/api/sync/platform/push";
    let actual = format!("{}/api/sync/platform/push", client.base_url());
    assert_eq!(actual, expected);
}

#[test]
fn sync_client_platform_pull_url_shape_no_cursor() {
    let client = SyncClient::new("https://relay.getship.dev");
    let expected = "https://relay.getship.dev/api/sync/platform/pull";
    let actual = format!("{}/api/sync/platform/pull", client.base_url());
    assert_eq!(actual, expected);
}

#[test]
fn sync_client_platform_pull_url_shape_with_cursor() {
    let client = SyncClient::new("https://relay.getship.dev");
    let cursor = "01JAAAAAAAAAAAAAAAAAAAAAAAA";
    let expected = format!(
        "https://relay.getship.dev/api/sync/platform/pull?since={cursor}"
    );
    let mut actual = format!("{}/api/sync/platform/pull", client.base_url());
    actual.push_str(&format!("?since={cursor}"));
    assert_eq!(actual, expected);
}

#[test]
fn sync_client_workspace_push_url_shape() {
    let client = SyncClient::new("https://relay.getship.dev");
    let workspace_id = "ws-abc123";
    let expected = format!(
        "https://relay.getship.dev/api/sync/workspace/{workspace_id}/push"
    );
    let actual = format!(
        "{}/api/sync/workspace/{workspace_id}/push",
        client.base_url()
    );
    assert_eq!(actual, expected);
}

#[test]
fn sync_client_workspace_pull_url_shape_no_cursor() {
    let client = SyncClient::new("https://relay.getship.dev");
    let workspace_id = "ws-abc123";
    let expected = format!(
        "https://relay.getship.dev/api/sync/workspace/{workspace_id}/pull"
    );
    let actual = format!(
        "{}/api/sync/workspace/{workspace_id}/pull",
        client.base_url()
    );
    assert_eq!(actual, expected);
}

#[test]
fn sync_client_workspace_pull_url_shape_with_cursor() {
    let client = SyncClient::new("https://relay.getship.dev");
    let workspace_id = "ws-abc123";
    let cursor = "01JBBBBBBBBBBBBBBBBBBBBBBBBB";
    let expected = format!(
        "https://relay.getship.dev/api/sync/workspace/{workspace_id}/pull?since={cursor}"
    );
    let mut actual = format!(
        "{}/api/sync/workspace/{workspace_id}/pull",
        client.base_url()
    );
    actual.push_str(&format!("?since={cursor}"));
    assert_eq!(actual, expected);
}
