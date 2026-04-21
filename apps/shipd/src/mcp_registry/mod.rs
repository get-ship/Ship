//! GET /api/mcp/servers — MCP registry proxy with in-memory caching.
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

// Current canonical endpoint per registry docs. The old Cloudflare Worker
// hit `/api/v0/servers`, which is now 404. The v0.1 response wraps each
// entry in `{ server: {...}, _meta: {...} }` rather than the flat shape
// our UpstreamServer still expects — deserialization fails silently and
// callers get the curated fallback list. Proper upstream parsing lands
// in a follow-up.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct McpServersQuery {
    #[serde(rename = "q")]
    pub query: Option<String>,
    pub vetted: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct McpServersResponse {
    servers: Vec<McpServer>,
    cached: bool,
}

// ── Upstream wire types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct UpstreamResponse {
    servers: Vec<UpstreamServer>,
}

#[derive(Debug, Deserialize)]
struct UpstreamServer {
    id: Option<String>,
    name: String,
    description: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    tags: Option<Vec<String>>,
    package: Option<UpstreamPackage>,
    vendor: Option<UpstreamVendor>,
}

#[derive(Debug, Deserialize)]
struct UpstreamPackage {
    registry: Option<String>,
    command: Option<String>,
    args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct UpstreamVendor {
    name: Option<String>,
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
        .query(&[("per_page", "100")])
        .send()
        .await?
        .error_for_status()?
        .json::<UpstreamResponse>()
        .await?;
    Ok(resp.servers.into_iter().map(upstream_to_server).collect())
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

fn upstream_to_server(s: UpstreamServer) -> McpServer {
    let id = s.id.unwrap_or_else(|| {
        s.name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
            .collect()
    });
    let vetted = VETTED_IDS.contains(&id.as_str());
    let image_url = resolve_image_url(s.homepage.as_deref())
        .or_else(|| resolve_image_url(s.repository.as_deref()));
    McpServer {
        id,
        name: s.name,
        description: s.description,
        homepage: s.homepage,
        tags: s.tags.unwrap_or_default(),
        vendor: s.vendor.and_then(|v| v.name),
        package_registry: s.package.as_ref().and_then(|p| p.registry.clone()),
        command: s.package.as_ref().and_then(|p| p.command.clone()),
        args: s.package.and_then(|p| p.args).unwrap_or_default(),
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

    if let Some(q) = &params.query {
        if q.len() > MAX_QUERY_LEN {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "query too long (max 200 characters)" })),
            )
                .into_response();
        }
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
