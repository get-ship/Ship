---
group: MCP Tools
order: 5
title: MCP Internals
description: ShipServer, tool router, transports (stdio + HTTP), per-connection actor store, notification relay, tool gating.
audience: internal
---

# MCP Internals

`apps/mcp/`. The MCP server speaks the Model Context Protocol over stdio or HTTP, exposes Ship's operations as tools, and relays kernel events back to the client as MCP notifications. Built on the `rmcp` crate + Ship's internal `mcp-framework` abstraction.

## Layout

```
apps/mcp/src/
  lib.rs               — ShipMcpApp (mcp_framework::McpApp impl), public exports
  server/              — ShipServer + handler + tool_gate + notification_relay
  tools/               — one module per tool group (job, session, workspace, skills, etc.)
  http.rs              — Streamable HTTP transport
  studio_server.rs     — Studio-specific MCP server (additional capabilities)
  requests/            — typed request / parameter structs
  resources.rs         — MCP resources (static URIs the client can subscribe to)
  resource_resolver.rs — resolve ship:// URIs
  network_client.rs    — HTTP client for talking to shipd from an MCP server
  push/                — push-notification outbound path
  util.rs              — shared helpers
```

## Transports

Two supported:

- **stdio** — `ShipServer + stdio()` transport. Used by providers that spawn MCP servers (Claude Code, Cursor, Gemini). Entry: `server::run_server()`.
- **HTTP** — `http::run_http_server(port)` + `http::run_studio_http_server(port)`. Streamable HTTP with SSE for server→client notifications. Used by Studio (browser) and any other client that can't or won't spawn a stdio process.

The stdio server is what `ship mcp serve` launches by default. `ship mcp serve --http --port N` launches the HTTP variant.

Both transports wrap the same `ShipServer` instance behind the `rmcp` `ServiceExt::serve(transport)` builder.

## ShipServer

```rust
pub struct ShipServer {
    tool_router: ToolRouter<Self>,
    pub active_project: Arc<Mutex<Option<PathBuf>>>,
    pub notification_peer: Arc<Mutex<Option<Peer<RoleServer>>>>,
    pub subscriptions: Arc<Mutex<HashSet<String>>>,   // subscribed resource URIs
    relay: Arc<Mutex<RelayState>>,                    // event relay state
    pub actor_store: Arc<Mutex<Option<ActorStore>>>,  // this connection's actor scope
    pub mcp_provider: Arc<Mutex<Option<String>>>,     // "claude-code", "cursor", ...
    pub session_id: Arc<Mutex<Option<String>>>,       // current session (auto-started)
    last_activity: Arc<AtomicU64>,                    // epoch ms, for heartbeat
    heartbeat_counter: Arc<AtomicU64>,                // tool-call tick counter
}
```

One `ShipServer` instance per connection. Mutex-wrapped state means a single server struct can be shared across the rmcp handler task and the notification relay task.

### Per-connection actor store

`actor_store: Option<ActorStore>` is the scoped event handle for this MCP connection. Populated when the connection initializes — the daemon registers the MCP client as an actor with the `ActorConfig` derived from the client's declared `mcp_provider` and `session_id`. Events emitted via MCP tools go through this `ActorStore`, inheriting the namespace scoping and actor_id stamping.

See [Persistence](/reference/runtime/persistence/) for what `ActorStore` enforces.

### Notification peer

`notification_peer: Option<Peer<RoleServer>>` is the rmcp peer handle — how the server sends notifications back to the client. Populated when rmcp calls `Handler::initialize`. All outbound notifications (events, status, subscription updates) go through this peer.

### Subscriptions

Clients subscribe to resource URIs via `resources/subscribe`. The server tracks the subscription set and only sends `resources/updated` notifications for URIs the client cares about. Subscription list is flat — no hierarchical matching.

### Heartbeat

Every N tool calls (counter increments in the tool handler), the server emits a `session.heartbeat` event through the actor store. This keeps `last_activity_at` fresh on the session record so the session liveness computation stays accurate even during long periods of pure tool-use.

`last_activity: AtomicU64` is the epoch millis of the last tool call — sampled for heartbeat cadence.

## Tools

`tools/` contains one module per logical group. Each module declares tool handlers via `#[tool(...)]` macros. `ShipServer` composes them via the `#[tool_router]` macro on its impl block.

| Module | Tools |
|---|---|
| `tools::job` | `start_session`, `log_progress`, `end_session`, pipeline dispatch. |
| `tools::session` | session open/close, progress. |
| `tools::session_files` | attach files to a session. |
| `tools::workspace` | workspace listing, creation, queries. |
| `tools::workspace_ops` | workspace mutations (start, archive, cleanup). |
| `tools::skills` | skill listing, `set_skill_var`, skill metadata. |
| `tools::project` | `open_project`, `close_project`, project info. |
| `tools::studio` | Studio-specific tools (canvas, annotations). |
| `tools::studio_inbox` | inbox queue for async user interactions. |
| `tools::studio_push` | push notifications to connected Studios. |
| `tools::git_info` | git status, branch, diff summary. |
| `tools::event` (unstable) | raw event emission for trusted callers. |
| `tools::view` (unstable) | view.open, view.patch, view.append, view.signal. |
| `tools::dispatch` (unstable) | direct agent dispatch (bypasses pipeline orchestrator). |

Each handler is a plain async fn with typed parameters (from `requests/`) and typed return. The tool macro generates the schema announcement + dispatch glue.

## Tool gate

`server/tool_gate.rs`. Pre-dispatch filter that validates the caller has permission to invoke the tool. Checks:

- Is the tool available in the current mode (stable vs unstable)?
- Does the session have the required role/capability?
- Is the workspace context set for tools that need it?

Rejections return a typed error to the client before the tool handler runs. See `role_gate_integration_tests.rs` for the coverage.

## Handler

`server/handler.rs`. The glue between rmcp's `Handler` trait and `ShipServer`. Implements:

- `initialize` — populate `notification_peer`, `mcp_provider`, start the actor registration flow with the daemon.
- `list_tools` — delegate to `tool_router`.
- `call_tool` — delegate to `tool_router` after the tool gate passes.
- `list_resources`, `read_resource`, `subscribe_resource`, `unsubscribe_resource` — `resources.rs` + `resource_resolver.rs` handle `ship://` URIs.
- `list_prompts` — not implemented; Ship doesn't expose static prompts.

## Notification relay

`server/notification_relay.rs`. Long-lived task per connection. Subscribes to the kernel event log via the connection's actor's mailbox, filters by the client's subscription set, serializes each relevant event as an MCP `resources/updated` notification, and sends on the `notification_peer`.

Lifecycle:

1. Connection initializes → `start_relay(server, kernel, actor_store)` spawns the task.
2. Client subscribes / unsubscribes → subscription set updates via the handler; relay picks it up on next iteration.
3. Connection closes → `notification_peer` goes `None`, relay task exits.

Relay doesn't do buffering — events too fast for the client just drop at `try_send`. Same invariant as kernel delivery: no backpressure on the router.

## Network client

`network_client.rs`. When the MCP server needs to call back to shipd (the stateful parts — pipelines, jobs, workspaces), it uses an HTTP client. Reads `~/.ship/daemon.token`, reads port from `~/.ship/shipd.port`, issues authenticated requests.

This pattern means the MCP server itself doesn't hold domain state — it's another transport. All real state is in `shipd`. A tool like `start_session` does:

```
MCP tool call → ShipServer → network_client → shipd HTTP → runtime → event log
```

Three hops because MCP and shipd are separate processes. MCP could in theory collapse into shipd as an embedded Axum route (as `mcp_apps_bridge.rs` in shipd already does for HTTP MCP). The stdio MCP server stays separate because providers invoke it as a subprocess.

## Resources

`resources.rs`, `resource_resolver.rs`. MCP resources are URIs the client can `read` or `subscribe`. Ship resources all start with `ship://`:

- `ship://project/current` — active project info.
- `ship://workspaces` — workspace list.
- `ship://workspace/{id}` — single workspace.
- `ship://session/{id}` — session detail.
- `ship://jobs` — job list.

Resolver functions handle the URI → JSON payload mapping. Subscribing a resource causes the notification relay to send `resources/updated` whenever that resource's underlying data changes (via kernel event observation).

## StudioServer

`studio_server.rs`. Variant `ShipServer` exposed to Studio with an expanded tool set (view-authoring tools, canvas annotations, feedback). Factored into its own server to keep the stable stdio MCP surface minimal while giving Studio what it needs.

Served over HTTP via `run_studio_http_server(port)`. Browser Studio connects via Streamable HTTP + SSE.

## Push module

`push/`. Outbound path for server-initiated notifications that aren't tied to an MCP subscription — primarily "the user responded" events from webhooks (Telegram, Slack). The human gateway emits events; push module routes them to any connected MCP client that's registered interest in that correlation id.

## Feature gates

`unstable` feature enables `tools::event`, `tools::view`, `tools::dispatch`. These bypass normal constraints (raw event emission, direct dispatch without orchestrator) and should not ship in stable.

## Tests

- `role_gate_integration_tests.rs` — tool gating by role.
- `tool_gate_tests.rs` — unit tests for gate logic.
- `inbox_routing_tests.rs` — studio_inbox queuing.
- `project_skills_tests.rs` — skill tool handlers.
- `resource_tests.rs` — resource URI resolution.
- `skill_file_tests.rs` — skill file I/O.
- `studio_server_tests.rs` — Studio server surface.
- `server_tests.rs` — general server behavior.
- Per-tool module tests alongside each `tools::*.rs` as `*_tests.rs`.

## Invariants

- One `ShipServer` per connection. No shared state across connections except via the kernel.
- Every outbound notification goes through `notification_peer` — no other path.
- Tool handlers don't touch `actor_store` directly; they go through `network_client` to the daemon, or through framework helpers that emit via the actor store. Keeps the "stateful in shipd, transport in MCP" split clean.
- `tool_gate` runs before every tool. A gate bypass would be a bug.
- The heartbeat counter and `last_activity` are only written from tool handlers — relay task reads but doesn't modify.

## Related

- [Commands](/reference/mcp-tools/commands/) — user-facing tool reference.
- [Daemon](/reference/runtime/daemon/) — where the real state is.
- [Persistence](/reference/runtime/persistence/) — `ActorStore` enforcement for MCP event emission.
- [Scopes](/reference/runtime/scopes/) — role-to-scope mapping that feeds the tool gate.
