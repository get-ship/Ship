use super::*;

fn make_server(id: &str, vetted: bool) -> McpServer {
    McpServer {
        id: id.to_string(),
        name: id.to_string(),
        description: None,
        homepage: None,
        tags: vec![],
        vendor: None,
        package_registry: None,
        command: None,
        args: vec![],
        vetted,
        image_url: None,
    }
}

#[tokio::test]
async fn fallback_returned_when_upstream_fails() {
    let state = McpRegistryState::with_failing_upstream();
    let (servers, cached) = state.get_servers().await;
    assert!(!cached, "stale upstream should not mark result as cached");
    assert!(!servers.is_empty(), "curated fallback must be non-empty");
    assert!(
        servers.iter().any(|s| s.id == "github"),
        "curated fallback must include github"
    );
    assert!(
        servers.iter().all(|s| s.vetted),
        "all curated fallback servers must be vetted"
    );
}

#[tokio::test]
async fn cache_hit_bypasses_upstream() {
    let cached_servers = vec![make_server("cached-server", true)];
    let state = McpRegistryState::with_cached_servers(cached_servers);
    let (servers, is_cached) = state.get_servers().await;
    assert!(is_cached, "fresh cache should report cached=true");
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].id, "cached-server");
}

#[test]
fn filter_vetted_only_removes_unvetted() {
    let servers = vec![make_server("a", true), make_server("b", false)];
    let filtered = apply_filters(servers, None, true);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "a");
}

#[test]
fn filter_by_query_matches_id_name_description_tags() {
    let mut server_with_tag = make_server("plain", true);
    server_with_tag.tags = vec!["tooling".to_string()];

    let servers = vec![
        McpServer {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            description: Some("Git hosting".to_string()),
            tags: vec![],
            ..make_server("github", true)
        },
        McpServer {
            id: "slack".to_string(),
            name: "Slack".to_string(),
            description: Some("Chat platform".to_string()),
            tags: vec![],
            ..make_server("slack", true)
        },
        server_with_tag,
    ];

    let filtered = apply_filters(servers.clone(), Some("git"), false);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "github");

    let filtered = apply_filters(servers, Some("tool"), false);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "plain");
}

#[test]
fn resolve_image_url_extracts_github_org() {
    assert_eq!(
        resolve_image_url(Some("https://github.com/modelcontextprotocol/servers")),
        Some("https://github.com/modelcontextprotocol.png".to_string())
    );
    assert_eq!(
        resolve_image_url(Some("https://linear.app/foo")),
        None
    );
    assert_eq!(resolve_image_url(None), None);
}

#[test]
fn curated_fallback_non_empty_and_all_vetted() {
    let fallback = curated_fallback();
    assert!(!fallback.is_empty());
    assert!(fallback.iter().all(|s| s.vetted));
}
