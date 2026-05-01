//! GET /api/v1/mcp/servers — MCP registry proxy with in-memory caching.
//!
//! Fetches from the official MCP registry (24 h TTL). Falls back to a
//! curated list when the upstream registry is unreachable.
//! Rate-limited globally to 60 requests/minute (token bucket).

use axum::{
    Extension, Json,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

mod fallback;
pub use fallback::curated_fallback;

const REGISTRY_API: &str = "https://registry.modelcontextprotocol.io/v0.1/servers";
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_LIMIT: usize = 100;
const DEFAULT_LIMIT: usize = 20;
const MAX_QUERY_LEN: usize = 200;
const RATE_CAPACITY: f64 = 60.0;
const RATE_REFILL_PER_SEC: f64 = 1.0; // 60 req/min

pub const VETTED_IDS: &[&str] = &[
    "github",
    "filesystem",
    "brave-search",
    "memory",
    "slack",
    "linear",
    "playwright",
    "postgres",
    "puppeteer",
    "sqlite",
    "sentry",
    "fetch",
    "sequential-thinking",
    "everart",
    "google-maps",
];

// ── Public API types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "filesystem",
    "name": "Filesystem",
    "description": "Local filesystem access for MCP clients",
    "homepage": "https://github.com/modelcontextprotocol/servers",
    "tags": ["storage", "official"],
    "vendor": "modelcontextprotocol",
    "packageRegistry": "npm",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem"],
    "vetted": true,
    "imageUrl": null
}))]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub vendor: Option<String>,
    #[serde(rename = "packageRegistry")]
    pub package_registry: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub vetted: bool,
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct McpServersQuery {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub vetted: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "servers": [{
        "id": "filesystem",
        "name": "Filesystem",
        "tags": ["storage", "official"],
        "vendor": "modelcontextprotocol",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem"],
        "vetted": true
    }],
    "cached": true
}))]
pub struct McpServersResponse {
    pub servers: Vec<McpServer>,
    pub cached: bool,
}

// ── Upstream wire types (registry v0.1) ────────────────────────────────────
//
// The v0.1 response wraps each server in `{ server: {...}, _meta: {...} }`.
// Multiple rows per server exist (one per published version); we keep only
// those whose `_meta` marks them as `isLatest`.

#[derive(Debug, Deserialize)]
struct UpstreamResponse {
    servers: Vec<UpstreamEntry>,
}

#[derive(Debug, Deserialize)]
struct UpstreamEntry {
    server: UpstreamServer,
    #[serde(default, rename = "_meta")]
    meta: UpstreamMeta,
}

#[derive(Debug, Default, Deserialize)]
struct UpstreamMeta {
    #[serde(default, rename = "io.modelcontextprotocol.registry/official")]
    official: Option<UpstreamOfficialMeta>,
}

#[derive(Debug, Deserialize)]
struct UpstreamOfficialMeta {
    #[serde(default, rename = "isLatest")]
    is_latest: bool,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpstreamServer {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(default)]
    repository: Option<UpstreamRepository>,
    #[serde(default)]
    packages: Option<Vec<UpstreamPackage>>,
    #[serde(default)]
    icons: Option<Vec<UpstreamIcon>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // deserialized from upstream registry JSON but only url is accessed
struct UpstreamRepository {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpstreamPackage {
    #[serde(default, rename = "registryType")]
    registry_type: Option<String>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(default)]
    transport: Option<UpstreamTransport>,
}

#[derive(Debug, Deserialize)]
struct UpstreamTransport {
    #[serde(default, rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpstreamIcon {
    src: String,
}

// ── Registry state ─────────────────────────────────────────────────────────

pub struct McpRegistryState {
    inner: Mutex<RegistryInner>,
    client: reqwest::Client,
    upstream_url: String,
}

struct RegistryInner {
    cached_at: Option<Instant>,
    servers: Vec<McpServer>,
    bucket_tokens: f64,
    bucket_last: Instant,
}

impl McpRegistryState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            upstream_url: REGISTRY_API.to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
            inner: Mutex::new(RegistryInner {
                cached_at: None,
                servers: Vec::new(),
                bucket_tokens: RATE_CAPACITY,
                bucket_last: Instant::now(),
            }),
        })
    }

    pub async fn try_consume_rate(&self) -> bool {
        let mut g = self.inner.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(g.bucket_last).as_secs_f64();
        g.bucket_tokens = (g.bucket_tokens + elapsed * RATE_REFILL_PER_SEC).min(RATE_CAPACITY);
        g.bucket_last = now;
        if g.bucket_tokens >= 1.0 {
            g.bucket_tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub async fn get_servers(&self) -> (Vec<McpServer>, bool) {
        {
            let g = self.inner.lock().await;
            let fresh = g.cached_at.map(|t| t.elapsed() < CACHE_TTL).unwrap_or(false);
            if fresh && !g.servers.is_empty() {
                return (g.servers.clone(), true);
            }
        }

        match fetch_upstream(&self.client, &self.upstream_url).await {
            Ok(servers) => {
                let mut g = self.inner.lock().await;
                g.servers = servers.clone();
                g.cached_at = Some(Instant::now());
                (servers, false)
            }
            Err(_) => {
                let g = self.inner.lock().await;
                if !g.servers.is_empty() {
                    (g.servers.clone(), true)
                } else {
                    drop(g);
                    (curated_fallback(), false)
                }
            }
        }
    }

    #[cfg(test)]
    pub fn with_cached_servers(servers: Vec<McpServer>) -> Arc<Self> {
        Arc::new(Self {
            upstream_url: REGISTRY_API.to_string(),
            client: reqwest::Client::new(),
            inner: Mutex::new(RegistryInner {
                cached_at: Some(Instant::now()),
                servers,
                bucket_tokens: RATE_CAPACITY,
                bucket_last: Instant::now(),
            }),
        })
    }

    #[cfg(test)]
    pub fn with_failing_upstream() -> Arc<Self> {
        Arc::new(Self {
            upstream_url: "http://localhost:0".to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_millis(200))
                .build()
                .unwrap(),
            inner: Mutex::new(RegistryInner {
                cached_at: None,
                servers: Vec::new(),
                bucket_tokens: RATE_CAPACITY,
                bucket_last: Instant::now(),
            }),
        })
    }
}

// ── Upstream fetch + conversion ────────────────────────────────────────────

async fn fetch_upstream(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<McpServer>, reqwest::Error> {
    let resp = client
        .get(url)
        .query(&[("limit", "100")])
        .send()
        .await?
        .error_for_status()?
        .json::<UpstreamResponse>()
        .await?;

    // Keep only the latest published version of each server, drop anything
    // the registry has marked deleted/deprecated.
    let mut out = Vec::with_capacity(resp.servers.len());
    for entry in resp.servers {
        let Some(official) = entry.meta.official.as_ref() else {
            continue;
        };
        if !official.is_latest {
            continue;
        }
        if matches!(official.status.as_deref(), Some("deleted") | Some("deprecated")) {
            continue;
        }
        out.push(upstream_to_server(entry.server));
    }
    Ok(out)
}

fn resolve_image_url(url: Option<&str>) -> Option<String> {
    let url = url?;
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let org = path.split('/').next()?;
    if org.is_empty() || org.contains('?') {
        return None;
    }
    Some(format!("https://github.com/{org}.png"))
}

/// Short id for vetted-list matching + UI keys. Names in v0.1 look like
/// `io.github.owner/repo` or `ac.inference.sh/mcp`; we use the last
/// path segment so historical ids (`github`, `filesystem`) keep matching.
fn short_id(name: &str) -> String {
    let tail = name.rsplit('/').next().unwrap_or(name);
    tail.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect()
}

fn upstream_to_server(s: UpstreamServer) -> McpServer {
    let id = short_id(&s.name);
    let vetted = VETTED_IDS.contains(&id.as_str());
    let repo_url = s.repository.as_ref().and_then(|r| r.url.clone());
    let homepage = s.website_url.clone().or_else(|| repo_url.clone());
    let image_url = s
        .icons
        .as_ref()
        .and_then(|is| is.first())
        .map(|i| i.src.clone())
        .or_else(|| resolve_image_url(s.website_url.as_deref()))
        .or_else(|| resolve_image_url(repo_url.as_deref()));
    let first_pkg = s.packages.as_ref().and_then(|p| p.first());
    let package_registry = first_pkg.and_then(|p| p.registry_type.clone());
    let command = first_pkg.and_then(|p| p.transport.as_ref()?.kind.clone());
    let args = first_pkg
        .and_then(|p| p.identifier.clone())
        .map(|i| vec![i])
        .unwrap_or_default();
    // Display name prefers the human `title` if the registry provides one;
    // falls back to the technical `name` so there's always something to show.
    let display_name = s.title.clone().unwrap_or_else(|| s.name.clone());
    McpServer {
        id,
        name: display_name,
        description: s.description,
        homepage,
        tags: Vec::new(), // v0.1 schema dropped the tags field
        vendor: None,     // v0.1 schema dropped the vendor object
        package_registry,
        command,
        args,
        vetted,
        image_url,
    }
}

pub fn apply_filters(
    servers: Vec<McpServer>,
    query: Option<&str>,
    vetted_only: bool,
) -> Vec<McpServer> {
    servers
        .into_iter()
        .filter(|s| !vetted_only || s.vetted)
        .filter(|s| {
            let Some(q) = query else { return true };
            let q = q.to_lowercase();
            s.id.to_lowercase().contains(&q)
                || s.name.to_lowercase().contains(&q)
                || s.description.as_deref().unwrap_or("").to_lowercase().contains(&q)
                || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

// ── Handler ────────────────────────────────────────────────────────────────

#[utoipa::path(
    operation_id = "listMcpServers",
    get,
    path = "/api/v1/mcp/servers",
    params(McpServersQuery),
    responses(
        (status = 200, description = "Filtered MCP server catalog", body = McpServersResponse),
        (status = 400, description = "Query too long", body = crate::openapi_types::ErrorResponse),
        (status = 429, description = "Rate limited", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "runtime"
)]
pub async fn list_mcp_servers(
    Query(params): Query<McpServersQuery>,
    Extension(registry): Extension<Arc<McpRegistryState>>,
) -> Response {
    if !registry.try_consume_rate().await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate limit exceeded" })),
        )
            .into_response();
    }

    if let Some(q) = &params.query
        && q.len() > MAX_QUERY_LEN {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "query too long (max 200 characters)" })),
            )
                .into_response();
        }

    let vetted_only = params.vetted.as_deref() != Some("false");
    let limit = params
        .limit
        .map(|l| l.clamp(1, MAX_LIMIT))
        .unwrap_or(DEFAULT_LIMIT);

    let (all_servers, cached) = registry.get_servers().await;
    let filtered = apply_filters(all_servers, params.query.as_deref(), vetted_only);

    Json(McpServersResponse {
        servers: filtered.into_iter().take(limit).collect(),
        cached,
    })
    .into_response()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
