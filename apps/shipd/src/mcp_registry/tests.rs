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

/// Parses a real v0.1 registry response (three entries covering the three
/// dominant shapes: remotes-only, npm package with env vars, github
/// repository + packages). Catches upstream-shape drift the same day the
/// registry ships it.
#[test]
fn parses_v0_1_response_shape() {
    let json = r#"{
      "servers": [
        {
          "server": {
            "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
            "name": "ac.inference.sh/mcp",
            "description": "Run 150+ AI apps — image, video, audio, LLMs, 3D and more.",
            "title": "inference.sh",
            "version": "1.0.0",
            "remotes": [{"type": "streamable-http", "url": "https://sh.inference.ac"}]
          },
          "_meta": {
            "io.modelcontextprotocol.registry/official": {
              "status": "active",
              "isLatest": true
            }
          }
        },
        {
          "server": {
            "name": "io.github.foo/agenttrust",
            "description": "AgentTrust MCP",
            "version": "1.1.1",
            "repository": {"url": "https://github.com/agenttrust/mcp-server", "source": "github"},
            "packages": [{
              "registryType": "npm",
              "identifier": "@agenttrust/mcp-server",
              "version": "1.1.1",
              "transport": {"type": "stdio"}
            }],
            "icons": [{"src": "https://app.adadvisor.ai/logo.png", "mimeType": "image/png"}]
          },
          "_meta": {
            "io.modelcontextprotocol.registry/official": {
              "status": "active",
              "isLatest": true
            }
          }
        },
        {
          "server": {
            "name": "io.github.foo/old-version",
            "version": "0.9.0"
          },
          "_meta": {
            "io.modelcontextprotocol.registry/official": {
              "status": "active",
              "isLatest": false
            }
          }
        }
      ]
    }"#;

    let resp: UpstreamResponse = serde_json::from_str(json).expect("parse v0.1 response");

    let kept: Vec<_> = resp
        .servers
        .into_iter()
        .filter_map(|e| {
            let o = e.meta.official.as_ref()?;
            if !o.is_latest {
                return None;
            }
            Some(upstream_to_server(e.server))
        })
        .collect();

    assert_eq!(kept.len(), 2, "non-latest entry must be dropped");

    let inf = kept.iter().find(|s| s.id == "mcp").expect("inference.sh entry");
    assert_eq!(inf.name, "inference.sh", "title should be preferred as display name");
    assert!(inf.description.as_deref().unwrap().starts_with("Run 150+"));

    let agent = kept.iter().find(|s| s.id == "agenttrust").expect("agenttrust entry");
    assert_eq!(agent.package_registry.as_deref(), Some("npm"));
    assert_eq!(agent.command.as_deref(), Some("stdio"));
    assert_eq!(agent.args, vec!["@agenttrust/mcp-server".to_string()]);
    assert_eq!(agent.image_url.as_deref(), Some("https://app.adadvisor.ai/logo.png"));
    assert_eq!(agent.homepage.as_deref(), Some("https://github.com/agenttrust/mcp-server"));
}

/// Hits the live registry. Ignored by default so offline builds stay green;
/// run with `cargo test -p shipd --lib mcp_registry::tests::live_upstream_returns_servers -- --ignored --nocapture`.
#[tokio::test]
#[ignore = "hits real network"]
async fn live_upstream_returns_servers() {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    match fetch_upstream(&client, REGISTRY_API).await {
        Ok(servers) => {
            eprintln!("fetched {} servers from live registry", servers.len());
            assert!(servers.len() > 10, "live registry should return more than the curated fallback (got {})", servers.len());
        }
        Err(e) => panic!("live registry fetch failed: {e:?}"),
    }
}
