//! MCP — third-party integration as a Ship op (`mcp.call`).
//!
//! The Model Context Protocol is a JSON-RPC 2.0 protocol. This module is the
//! v1 projection: the MCP server is a *protocol*; what it surfaces in our
//! world is a *single* synthetic Ship op, `mcp.call`, dispatched through
//! `gateway.call` like every other effect.
//!
//! Two denials, two axes, both surfaces as `{ denied }` to the model
//! (invariant: denial is data):
//!
//!   1. **Service grant** — `Capability.grants` must contain `"mcp"`. Lacks
//!      ⇒ C2 service denial at the gate (the only enforcement point).
//!   2. **Per-server allow-list** — `Capability.mcp.allow` is the second
//!      deny-by-default axis. Lacks the entry for the requested server ⇒
//!      denial at the registry layer (data, not exception).
//!
//! Per-tool blast tier lives on the registry, not the static descriptor
//! table — the registry is dynamic, the table is not. v1 freezes the blast
//! at C2 (conservative default); v2 reads MCP `annotations`.
//!
//! See `jobs/s-mcp.md` for the full design. v1 is the "smallest first
//! impl slice that proves the design" (design §5): no stdio, no HTTP, no
//! JSON-RPC framing — `FakeMcpClient` stands in. The trait is the seam.
//!
//! `#[allow(dead_code)]` at module level: the spec'd public surface
//! (transports, content variants, error variants, the `list_tools`
//! method, `servers`) is consumed by v2's stdio client. The v1 fake
//! client is a stand-in; this allow keeps `cargo build` clean without
//! weakening the v2 contract.
// The v1 public surface — `McpServerSpec.transport`, `McpTransport`'s
// stdio fields, the `McpToolDef` metadata fields, the `McpError`
// variants, `McpClient::list_tools`, and `McpRegistry::servers` — is
// consumed by v2's StdioMcpClient / LiveMcpRegistry. v1's
// `FakeMcpClient` is a stand-in; the v2 impl will wire them up. The
// compiler can't see that future, so we silence it here rather than
// `_`-prefix the names (which would weaken the contract for v2).
#[allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::gate::Blast;
// ─────────────────────────────────────────────────────────────────────────
// Spec types — what the runtime knows about a configured MCP server.
// Populated at agent boot from the agent spec's `mcp` block.
// ─────────────────────────────────────────────────────────────────────────
/// What the runtime knows about a configured MCP server. Populated at
/// agent boot from the spec's `mcp` block; never from a tool call's
/// args (the model cannot install servers — that is an operator action).
#[derive(Debug, Clone)]
pub struct McpServerSpec {
    /// Stable id used in the agent's `mcp.call({server, ...})` and in
    /// `Capability.mcp.allow` ("sentry", "fs", …).
    pub id: String,
    /// Transport. v1: stdio only; HTTP is a v2.
    pub transport: McpTransport,
    /// Tools this server exposes (frozen at boot in v1).
    pub tools: Vec<McpToolDef>,
}

#[derive(Debug, Clone)]
pub enum McpTransport {
    /// Local child process. The child inherits NOTHING from the agent
    /// (no env, no agent auth, no socket path) — we own its argv + env.
    /// The v1 fake client ignores transport; v2's `StdioMcpClient` reads
    /// it to spawn.
    Stdio {
        cmd: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
    },
    // Http { url: String, auth: McpAuth },   // v2
}

/// One MCP tool as projected onto the Ship op surface. v1 freezes blast
/// at C2; v2 derives from `annotations.destructive` / `readOnlyHint`.
#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub server: String,
    pub name: String,
    pub title: Option<String>,
    pub description: String,
    /// Intrinsic blast tier — v1 always `C2Elevated`. v2 reads
    /// MCP `annotations` to pick C0/C1/C2.
    pub blast: Blast,
}

/// One content item in a tool call's result. The verifier only accepts
/// a small set of well-typed items (text / image / audio / resource_link
/// / resource); anything else ⇒ `Unverified` (nothing accepted blindly).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpContent {
    Text { text: String },
    Image { mime: String, data: String },
    Audio { mime: String, data: String },
    ResourceLink { uri: String, name: Option<String> },
    Resource { uri: String, mime: Option<String>, text: Option<String> },
}

/// The result of a single MCP tool call. Mirrors the JSON-RPC `result`
/// envelope minus the framing (no `jsonrpc`/`id`): `isError` is the
/// MCP-defined "the tool itself returned an error result" flag, distinct
/// from transport failure (which surfaces as `McpError`). v1 client
/// returns `isError = false`; the canned result is a single `text` item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCallResult {
    pub server: String,
    pub tool: String,
    pub is_error: bool,
    pub content: Vec<McpContent>,
}

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// The MCP server is not configured on this agent.
    #[error("MCP server `{0}` is not configured")]
    UnknownServer(String),
    /// The tool is not declared by the configured server.
    #[error("MCP tool `{tool}` is not declared on server `{server}`")]
    UnknownTool { server: String, tool: String },
    /// JSON-RPC framing / I/O / parse failure. Distinct from `isError=true`.
    #[error("MCP transport error: {0}")]
    Transport(String),
    /// The MCP server requested a host capability we deny by policy
    /// (sampling, elicitation). v1 client refuses; logs the attempt.
    #[error("MCP server request denied by policy: {0}")]
    PolicyDenied(String),
}

// ─────────────────────────────────────────────────────────────────────────
// Client trait — JSON-RPC to one MCP server. Real impls (stdio v2 /
// HTTP v2) sit behind the same surface; v1 is a canned fake.
// ─────────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait McpClient: Send + Sync {
    /// List the tools this server exposes. v1 freezes the list at boot;
    /// v2 honors `notifications/tools/list_changed`.
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, McpError>;

    /// Invoke `name` with `args` and return the result. The v1 fake
    /// returns a canned `text` result that passes spine verify; the
    /// real client (v2) frames the JSON-RPC request and parses the
    /// response.
    async fn call_tool(
        &self,
        name: &str,
        args: Value,
    ) -> Result<McpCallResult, McpError>;
}

// ─────────────────────────────────────────────────────────────────────────
// Fake client — canned tools + canned text result. Sufficient to prove
// the projection, the gate, the second deny-by-default axis, and the
// verify posture. The trait is the seam: v2's StdioMcpClient drops in
// here without touching the gate.
// ─────────────────────────────────────────────────────────────────────────

/// A canned client for v1. One `text` result per call (verifier-accepted),
/// and one canned tool per the configured server's `tools` list. No I/O.
pub struct FakeMcpClient {
    pub server_id: String,
    pub tools: Vec<McpToolDef>,
}

#[async_trait]
impl McpClient for FakeMcpClient {
    async fn list_tools(&self) -> Result<Vec<McpToolDef>, McpError> {
        Ok(self.tools.clone())
    }

    async fn call_tool(
        &self,
        name: &str,
        args: Value,
    ) -> Result<McpCallResult, McpError> {
        // Refuse tools we never declared — surfaces as a clean error
        // (not a synthetic "ok" with garbage). Real clients do the same
        // via `tools/list` cross-check before `tools/call`.
        if !self.tools.iter().any(|t| t.name == name) {
            return Err(McpError::UnknownTool {
                server: self.server_id.clone(),
                tool: name.to_string(),
            });
        }
        // Canned text result. The verifier accepts a well-typed `text`
        // item with `isError=false`; the body is a tiny echo of the
        // call so the test can assert on the shape (server, tool, args
        // all present) without being brittle about literals.
        let body = format!(
            "fake-mcp:{server}:{tool}:{args}",
            server = self.server_id,
            tool = name,
            // `serde_json::Value` is not Display; compact is stable + small.
            args = args.to_string(),
        );
        Ok(McpCallResult {
            server: self.server_id.clone(),
            tool: name.to_string(),
            is_error: false,
            content: vec![McpContent::Text { text: body }],
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Registry trait — frozen, in-process, populated at boot. Consulted by
// the gateway for the per-call blast (registry is dynamic; the static
// descriptor table is not) and for the per-server / per-tool allow
// check (the second deny-by-default axis).
// ─────────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait McpRegistry: Send + Sync {
    /// Intrinsic blast tier for a (server, tool) pair. The gateway
    /// consults this for `mcp.call`. `None` ⇒ op unknown at this server.
    /// v1 returns C2 for every known tool; v2 reads annotations.
    fn blast_of(&self, server: &str, tool: &str) -> Option<Blast>;

    /// The second deny-by-default axis: is this (server, tool) on the
    /// agent's `mcp.allow` / `mcp.allow_tools` list? Called after the
    /// service-grant gate passes; failure here is a registry-layer
    /// denial, returned to the model as `{ denied }` (data, not
    /// exception).
    fn is_allowed(&self, server: &str, tool: &str) -> bool;

    /// The client for a configured server, or `None` for an unconfigured
    /// one. The gateway uses this to dispatch after both gates pass.
    fn client_of(&self, server: &str) -> Option<Arc<dyn McpClient>>;

    /// All configured server ids. Useful for diagnostics / spec wiring.
    fn servers(&self) -> Vec<String>;
}

// ─────────────────────────────────────────────────────────────────────────
// In-memory registry — built from `(Vec<McpServerSpec>, Capability.mcp.allow,
// Capability.mcp.allow_tools)`. Pure data, no I/O. The v1's only registry;
// the v2 "live" registry is the same trait with a different constructor
// (calls `list_tools` at boot, caches, refreshes on `listChanged`).
// ─────────────────────────────────────────────────────────────────────────

pub struct InMemoryMcpRegistry {
    /// `server_id -> (spec, client)`. Built once at agent boot.
    by_id: HashMap<String, McpServerSpec>,
    clients: HashMap<String, Arc<dyn McpClient>>,
    /// Per-server allow-list from the agent's `Capability.mcp.allow`.
    allow: HashSet<String>,
    /// Optional finer per-(server, tool) filter from
    /// `Capability.mcp.allow_tools`. Empty map = no extra filter (all
    /// tools of an allowed server are reachable).
    allow_tools: HashMap<String, HashSet<String>>,
}

impl InMemoryMcpRegistry {
    /// Build the registry from the agent's `mcp` configuration. The
    /// `servers` list is the set the spec author configured; the `allow`
    /// list is what THIS agent may reach (intersected with `servers` by
    /// `is_allowed`). The intersection is explicit: a server that's
    /// configured but not allowed is invisible to the agent even if
    /// the `mcp` service is granted.
    pub fn new(
        servers: Vec<McpServerSpec>,
        allow: Vec<String>,
        allow_tools: HashMap<String, Vec<String>>,
    ) -> Self {
        let mut by_id = HashMap::new();
        let mut clients = HashMap::new();
        for spec in servers {
            // One fake client per server, holding its canned tool list.
            // v2 swaps the fake for `StdioMcpClient::new(spec.clone())`.
            let tools = spec.tools.clone();
            let client: Arc<dyn McpClient> =
                Arc::new(FakeMcpClient { server_id: spec.id.clone(), tools });
            clients.insert(spec.id.clone(), client);
            by_id.insert(spec.id.clone(), spec);
        }
        Self {
            by_id,
            clients,
            allow: allow.into_iter().collect(),
            allow_tools: allow_tools
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
        }
    }
}

impl McpRegistry for InMemoryMcpRegistry {
    fn blast_of(&self, server: &str, tool: &str) -> Option<Blast> {
        let spec = self.by_id.get(server)?;
        // If the tool isn't on the server, the call is unknown — `None`
        // tells the gateway to surface `UnknownOp`/`McpError::UnknownTool`.
        let def = spec.tools.iter().find(|t| t.name == tool)?;
        Some(def.blast)
    }

    fn is_allowed(&self, server: &str, tool: &str) -> bool {
        // Server must be both CONFIGURED and on this agent's allow-list.
        // (An unconfigured server can never be called — no client to
        // dispatch through.) The configure-but-not-allowed case is the
        // exact second deny-by-default the design calls out.
        if !self.by_id.contains_key(server) {
            return false;
        }
        if !self.allow.contains(server) {
            return false;
        }
        // Per-tool filter: an entry for this server restricts the
        // reachable tool set. An absent entry means "all tools".
        if let Some(tools) = self.allow_tools.get(server) {
            tools.contains(tool)
        } else {
            true
        }
    }

    fn client_of(&self, server: &str) -> Option<Arc<dyn McpClient>> {
        self.clients.get(server).cloned()
    }

    fn servers(&self) -> Vec<String> {
        let mut v: Vec<String> = self.by_id.keys().cloned().collect();
        v.sort();
        v
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Tests — the design's behavioral assertions. Assert semantics, never
// exact strings (per the README convention).
// ─────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(server: &str, name: &str) -> McpToolDef {
        McpToolDef {
            server: server.into(),
            name: name.into(),
            title: None,
            description: String::new(),
            blast: Blast::C2Elevated,
        }
    }

    fn spec(id: &str, names: &[&str]) -> McpServerSpec {
        McpServerSpec {
            id: id.into(),
            transport: McpTransport::Stdio {
                cmd: "echo".into(),
                args: vec![],
                env: vec![],
            },
            tools: names.iter().map(|n| tool(id, n)).collect(),
        }
    }

    // `is_allowed` is the second deny-by-default axis. A configured server
    // missing from the allow-list is invisible; an unconfigured server is
    // never reachable; a per-tool filter narrows the surface.
    #[test]
    fn is_allowed_is_the_second_deny_by_default_axis() {
        let reg = InMemoryMcpRegistry::new(
            vec![spec("fs", &["read", "write"]), spec("net", &["fetch"])],
            vec!["fs".into()],
            HashMap::new(),
        );
        // Server is configured AND allowed.
        assert!(reg.is_allowed("fs", "read"));
        assert!(reg.is_allowed("fs", "write"));
        // Server is configured but NOT in this agent's allow-list: denied.
        assert!(!reg.is_allowed("net", "fetch"));
        // Server is not configured at all: denied.
        assert!(!reg.is_allowed("nope", "x"));
    }

    // Per-tool filter narrows the reachable set on a per-server basis.
    #[test]
    fn per_tool_filter_narrows_an_allowed_server() {
        let reg = InMemoryMcpRegistry::new(
            vec![spec("fs", &["read", "write", "delete"])],
            vec!["fs".into()],
            HashMap::from([("fs".into(), vec!["read".into(), "write".into()])]),
        );
        assert!(reg.is_allowed("fs", "read"));
        assert!(reg.is_allowed("fs", "write"));
        assert!(!reg.is_allowed("fs", "delete"));
    }

    // Blast lookup is dynamic + per-(server, tool) — the registry is the
    // source of truth for `mcp.call`'s tier, not the static descriptor
    // table.
    #[test]
    fn blast_lookup_lives_on_the_registry() {
        let reg = InMemoryMcpRegistry::new(
            vec![spec("fs", &["read"])],
            vec!["fs".into()],
            HashMap::new(),
        );
        // Known tool on a known server: C2 (v1 default).
        assert_eq!(reg.blast_of("fs", "read"), Some(Blast::C2Elevated));
        // Unknown tool: None — the gateway surfaces this as unknown.
        assert_eq!(reg.blast_of("fs", "nope"), None);
        // Unknown server: None.
        assert_eq!(reg.blast_of("nope", "read"), None);
    }

    // The fake client returns a well-typed result that the spine verifier
    // (text content, isError=false) MUST accept. This is the
    // "nothing accepted blindly" invariant for MCP.
    #[test]
    fn fake_client_returned_result_passes_spine_verify() {
        // A minimal hand-rolled verifier, mirroring the posture the
        // real spine will use. The point: the canned shape (text
        // content, isError=false) is accepted; an isError=true result
        // is refused.
        fn verify(r: &McpCallResult) -> Result<(), &'static str> {
            if r.is_error {
                return Err("MCP result isError=true (refused)");
            }
            if r.content.is_empty() {
                return Err("MCP result has no content items");
            }
            for item in &r.content {
                match item {
                    McpContent::Text { text } if !text.is_empty() => {}
                    _ => return Err("MCP content has a non-text or empty item"),
                }
            }
            Ok(())
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let client = FakeMcpClient {
                server_id: "fs".into(),
                tools: vec![tool("fs", "read")],
            };
            let r = client
                .call_tool("read", serde_json::json!({"path": "/x"}))
                .await
                .expect("fake client succeeds on a declared tool");
            verify(&r).expect("canned text result must pass spine verify");
        });
    }

    // The fake client refuses tools it never declared — surfaces as
    // `McpError::UnknownTool` (data, not a panic).
    #[test]
    fn fake_client_refuses_undeclared_tool() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let client = FakeMcpClient {
                server_id: "fs".into(),
                tools: vec![tool("fs", "read")],
            };
            let r = client.call_tool("nope", serde_json::json!({})).await;
            assert!(matches!(r, Err(McpError::UnknownTool { .. })));
        });
    }
}
