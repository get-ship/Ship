mod handler;
pub mod notification_relay;
mod tool_gate;

use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    Peer, RoleServer, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_router,
};
use std::sync::atomic::AtomicU64;
use std::path::PathBuf;

use crate::requests::*;
use crate::tools::{
    artifacts as artifact_tools, dispatch as dispatch_tools, event, job as job_tools, project,
    session, session_files, skills, stability, sui_lsp as sui_lsp_tools,
};
use skills::set_skill_var_tool;

// ---- Server struct ----

/// Holds event relay state. Not Debug/Clone — stored behind Arc.
struct RelayState {
    /// Shared peer list for the event relay (add/remove peers after spawn).
    peers: std::sync::Arc<tokio::sync::RwLock<Vec<notification_relay::PeerHandle>>>,
    /// Handle to the spawned relay task (kept alive for the server lifetime).
    handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct ShipServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
    pub notification_peer: std::sync::Arc<tokio::sync::Mutex<Option<Peer<RoleServer>>>>,
    /// URIs the client has subscribed to via resources/subscribe
    pub subscriptions: std::sync::Arc<tokio::sync::Mutex<std::collections::HashSet<String>>>,
    /// Event relay state — initialized on connection init.
    relay: std::sync::Arc<tokio::sync::Mutex<RelayState>>,
    /// Actor-scoped event store for this connection.
    pub actor_store: std::sync::Arc<tokio::sync::Mutex<Option<ship_substrate::events::ActorStore>>>,
    /// Canonical actor_id for this connection — the id the daemon
    /// returned from `actor_spawn`. Populated in `on_initialized`.
    /// Tool handlers read this instead of re-running synthesis fallback.
    pub canonical_actor_id: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    /// MCP harness name from clientInfo (e.g. "claude-code", "cursor").
    pub mcp_harness: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    /// Active session ID for this connection (set by auto_session_start).
    pub session_id: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    /// Epoch millis of last tool call — used for heartbeat tracking.
    last_activity: std::sync::Arc<AtomicU64>,
    /// Tool call counter for heartbeat cadence (emit every N calls).
    heartbeat_counter: std::sync::Arc<AtomicU64>,
}

impl std::fmt::Debug for ShipServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipServer").finish_non_exhaustive()
    }
}

// ---- Stable tool registration ----

#[tool_router]
impl ShipServer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // All tools register unconditionally; stability is a runtime marker
        // applied via `stability::finalize_router` (description prefix +
        // alias entries pointing at canonical handlers).
        let router = {
            let mut r = Self::tool_router();
            r.merge(Self::experimental_tool_router());
            stability::finalize_router(r)
        };
        // Detect project from CWD at startup
        let project_dir = ship_substrate::project::get_project_dir(None)
            .ok()
            .map(|ship_dir| {
                if ship_dir.file_name().and_then(|n| n.to_str()) == Some(".ship") {
                    ship_dir.parent().unwrap_or(&ship_dir).to_path_buf()
                } else {
                    ship_dir
                }
            });
        Self {
            tool_router: router,
            active_project: std::sync::Arc::new(tokio::sync::Mutex::new(project_dir)),
            notification_peer: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            subscriptions: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            relay: std::sync::Arc::new(tokio::sync::Mutex::new(RelayState {
                peers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
                handle: None,
            })),
            actor_store: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            canonical_actor_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            mcp_harness: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            last_activity: std::sync::Arc::new(AtomicU64::new(0)),
            heartbeat_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        let actor_id = self.resolve_actor_id().await;
        project::get_effective_project_dir(&actor_id, &self.active_project).await
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
    }

    /// Spawn this agent's actor in the daemon's KernelRouter.
    ///
    /// Called from `on_initialized`. Derives the actor_id from the active agent
    /// profile, falling back to `"agent.mcp"`. Creates a local `ActorStore` for
    /// event persistence but delegates mailbox ownership to the daemon.
    pub async fn spawn_agent_actor(&self) {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(_) => return,
        };

        let global_dir = match ship_substrate::project::get_global_dir() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("failed to resolve global dir: {e}");
                return;
            }
        };

        // Hint id from env (daemon-spawned agents pre-mint and propagate
        // via SHIP_ACTOR_ID). Externally-launched MCPs have no env id;
        // the daemon mints in `actor_spawn` and we adopt the response.
        let hint_id: Option<String> = ship_substrate::env::read(ship_substrate::env::Var::ActorId);

        // Compute skill-derived subscriptions
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_dir);
        let skills_list = ship_platform::agents::skill::list_skills(&ship_dir).unwrap_or_default();
        let skill_subs =
            ship_platform::agents::skill::artifact_events::skill_event_subscriptions(&skills_list);

        // Default subscription set for an agent peer. The mesh's
        // subscribe_namespaces IS the filter — anything not declared
        // here doesn't reach the agent's mailbox.
        //
        // What an agent SHOULD see (broadcast subscriptions):
        // - studio.chat_turn — user turns from the command bar
        // - studio.reply — replies from other agents (multi-agent)
        // - inbox.* / human.* — messages from external user gateways
        // - mesh.message — inter-agent direct messaging
        //
        // Mesh lifecycle (`mesh.register` / `mesh.deregister`) is
        // delivered as DIRECTED events (target_actor_id = the actor
        // joining/leaving), so an actor sees its OWN join confirmation
        // in its own TUI but doesn't get pushed about other agents'
        // lifecycle. Directed delivery routes by target — no
        // broadcast subscription needed for that path.
        //
        // What we DON'T subscribe to (deliberately):
        // - workspace.*, session.*, actor.*, supervisor.* — internal
        //   lifecycle that drowns chat turns when pushed as channel
        //   events. The dashboard + SSE views render these instead.
        //
        // Skills extend this list via their declared `in:` events
        // (visual-brainstorm.annotation_created etc.).
        let mut subscribe_namespaces = vec![
            "studio.chat_turn".to_string(),
            "studio.reply".to_string(),
            "inbox.".to_string(),
            "human.".to_string(),
            "mesh.message".to_string(),
        ];
        for ns in skill_subs {
            if !subscribe_namespaces.contains(&ns) {
                subscribe_namespaces.push(ns);
            }
        }

        // Mesh capabilities from skills
        let capabilities: Vec<String> = skills_list.iter().map(|s| s.id.clone()).collect();
        let capabilities = if capabilities.is_empty() {
            vec!["general".to_string()]
        } else {
            capabilities
        };

        let config = ship_substrate::events::ActorConfig {
            namespace: hint_id.clone().unwrap_or_default(),
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces,
            see_directed_events: false,
        };

        // Spawn the actor in the daemon's kernel. Daemon mints the
        // canonical id when no hint is sent (or accepts a ULID hint).
        // We adopt the returned id for everything downstream.
        let actor_id = match crate::network_client::actor_spawn(
            hint_id.as_deref(),
            &config,
            Some(capabilities),
        ).await {
            Ok(id) => {
                tracing::info!(actor_id = %id, "actor spawned in daemon kernel");
                id
            }
            Err(e) => {
                tracing::warn!(
                    "daemon actor spawn failed (daemon may not be running): {e}"
                );
                // Fall back to the hint id if we have one; otherwise an
                // empty string which will surface in subsequent calls.
                hint_id.clone().unwrap_or_default()
            }
        };

        // Cache the canonical id so resolve_actor_id() returns it for
        // every subsequent tool call on this connection.
        *self.canonical_actor_id.lock().await = Some(actor_id.clone());

        // Register a scoped session under the canonical id.
        let active_agent = ship_platform::config::get_active_agent(Some(ship_dir.clone())).unwrap_or(None);
        let agent_role = active_agent.as_ref().and_then(|a| a.role.as_deref());
        let workspace_id = self.session_id.lock().await.clone();
        match crate::network_client::register_session(
            &actor_id,
            agent_role,
            workspace_id.as_deref(),
        ).await {
            Ok(_token) => tracing::info!(actor_id, "scoped session registered with daemon"),
            Err(e) => {
                tracing::debug!(actor_id, "session registration skipped (daemon may not be running): {e}");
            }
        }

        // Create local ActorStore for event persistence
        match ship_substrate::events::ActorStore::open(
            &actor_id,
            &global_dir,
            config.write_namespaces.clone(),
            config.read_namespaces.clone(),
        ) {
            Ok(store) => {
                *self.actor_store.lock().await = Some(store);
            }
            Err(e) => {
                tracing::warn!(actor_id, "failed to open local ActorStore: {e}");
            }
        }
    }

    /// Wire the EventRelay for this connection using the daemon's SSE stream
    /// as the event source. Call once per MCP connection.
    pub async fn start_event_relay(&self) {
        let mut relay_state = self.relay.lock().await;

        // Only start once
        if relay_state.handle.is_some() {
            return;
        }

        let actor_id = self.resolve_actor_id().await;

        // Connect to daemon's SSE stream for this actor's events
        let sse_rx = match crate::network_client::mesh_subscribe(&actor_id) {
            Ok(rx) => rx,
            Err(e) => {
                tracing::warn!(actor_id, "daemon SSE subscribe failed: {e}");
                return;
            }
        };

        // Bridge SSE unbounded receiver into a bounded Mailbox
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        tokio::spawn(async move {
            let mut sse_rx = sse_rx;
            while let Some(event) = sse_rx.recv().await {
                if tx.send(event).await.is_err() {
                    break; // Mailbox receiver dropped
                }
            }
        });
        let mailbox = ship_substrate::events::Mailbox::from_receiver(rx);

        let relay = notification_relay::EventRelay::new();
        relay_state.peers = relay.peers();

        // Resolve workspace branch for scoping notifications.
        let branch = self
            .get_effective_project_dir()
            .await
            .ok()
            .and_then(|d| tool_gate::current_branch(&d).ok());

        // Select push adapter based on detected harness, register as peer.
        // The peer's actor identity lives on its mailbox in the kernel —
        // directed-delivery events route to that mailbox, no per-peer
        // session/actor filtering needed at the relay.
        if let Some(peer) = self.notification_peer.lock().await.clone() {
            let harness = self.mcp_harness.lock().await.clone();
            let adapter =
                crate::push::select_adapter(harness.as_deref(), peer);
            tracing::info!(
                adapter = adapter.adapter_name(),
                branch = ?branch,
                "push adapter selected"
            );
            let peer_handle = notification_relay::PeerHandle {
                adapter,
                allowed_events: std::collections::HashSet::new(),
                branch,
            };
            relay.add_peer(peer_handle).await;
        }

        relay_state.handle = Some(relay.spawn(mailbox));
    }

    /// Resolve the actor_id for this connection.
    ///
    /// Lookup order:
    /// 1. `canonical_actor_id` cached from `actor_spawn` response. The
    ///    daemon is the only id minter; once we've registered, this is
    ///    the source of truth for everything downstream (tool calls,
    ///    log lines, env propagation).
    /// 2. `SHIP_ACTOR_ID` env — pre-mint from daemon-spawned agents.
    /// 3. Empty string — caller proceeds without an id, expecting the
    ///    daemon to mint one in the next `actor_spawn` call.
    ///
    /// Synthesis (`agent.{name}.{branch}`) is gone. The daemon mints.
    async fn resolve_actor_id(&self) -> String {
        if let Some(id) = self.canonical_actor_id.lock().await.clone() {
            return id;
        }
        if let Ok(env_id) = ship_substrate::env::read_raw(ship_substrate::env::Var::ActorId)
            && !env_id.is_empty()
        {
            tracing::info!(actor_id = %env_id, "resolved actor ID from SHIP_ACTOR_ID");
            return env_id;
        }
        // No id available yet. Tool handlers that hit this before
        // `on_initialized` populates the canonical id will get an
        // empty string; the daemon's `actor_spawn` mints one when we
        // call it.
        String::new()
    }

    /// Automatically start or resume a session on MCP connection init.
    ///
    /// 1. Clean up stale draining sessions (grace window expired).
    /// 2. Resolve current workspace from git branch.
    /// 3. If a draining session exists for this workspace+agent, resume it.
    /// 4. Otherwise start a fresh session via the runtime.
    /// 5. Store the session_id for tool-call counting and shutdown drain.
    pub(crate) async fn auto_session_start(&self) {
        use ship_substrate::db::session_drain;

        // Clean up sessions stuck draining longer than 120s.
        let _ = session_drain::cleanup_stale_draining(120);

        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(_) => return,
        };
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(&project_dir);
        let branch = match tool_gate::current_branch(&project_dir) {
            Ok(b) => b,
            Err(_) => return,
        };

        let agent_id = ship_platform::config::get_active_agent(Some(ship_dir.clone()))
            .ok()
            .flatten()
            .map(|a| a.id.clone());

        let harness = self.mcp_harness.lock().await.clone();

        // Try to resume a draining session for this workspace+agent.
        let agent_key = agent_id.as_deref().unwrap_or("mcp");
        if let Ok(Some(draining)) = session_drain::find_draining_session(&branch, agent_key)
            && session_drain::resume_session(&draining.id).is_ok()
        {
            tracing::info!(
                session_id = %draining.id,
                "resumed draining session"
            );
            *self.session_id.lock().await = Some(draining.id);
            return;
        }

        // Start a fresh session via the runtime.
        match ship_platform::sessions::start_session(
            &ship_dir,
            &branch,
            None,
            agent_id,
            harness,
        ) {
            Ok(session) => {
                tracing::info!(session_id = %session.id, "auto-started session");
                *self.session_id.lock().await = Some(session.id);
            }
            Err(e) => {
                tracing::warn!("auto_session_start failed: {e}");
            }
        }
    }

    /// Drain the active session on MCP disconnect.
    ///
    /// Transitions the session to "draining" status. If another MCP connection
    /// attaches within the grace window, `auto_session_start` will resume it.
    /// Otherwise `cleanup_stale_draining` finalizes it.
    pub(crate) async fn shutdown(&self) {
        use ship_substrate::db::session_drain;

        if let Some(sid) = self.session_id.lock().await.take() {
            match session_drain::drain_session(&sid) {
                Ok(()) => tracing::info!(session_id = %sid, "session drained on disconnect"),
                Err(e) => tracing::warn!(session_id = %sid, "drain failed: {e}"),
            }
        }
    }

    async fn notify_resources_changed(&self) {
        if let Some(peer) = self.notification_peer.lock().await.as_ref() {
            let _ = peer.notify_resource_list_changed().await;
        }
    }

    /// Push a resource update notification if the client is subscribed to this URI.
    pub async fn notify_resource_updated(&self, uri: &str) {
        let subscribed = self.subscriptions.lock().await.contains(uri);
        if subscribed
            && let Some(peer) = self.notification_peer.lock().await.as_ref()
        {
            let _ = peer
                .notify_resource_updated(rmcp::model::ResourceUpdatedNotificationParam {
                    uri: uri.to_string(),
                })
                .await;
        }
    }

    #[cfg(test)]
    pub fn registered_tool_names(&self) -> Vec<String> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect()
    }

    // ---- Session ----

    #[tool(
        description = "Record a progress note for the active session. Requires an active session. \
        Set escalation to 'blocker', 'warning', or 'question' to alert the human."
    )]
    async fn log_progress(&self, Parameters(req): Parameters<LogProgressRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(b) => b,
                Err(e) => return format!("Error: {}", e),
            };
        session::log_progress(&project_dir, req, &branch)
    }

    // ---- Skills ----

    #[tool(
        description = "Set a skill variable value. Pass value_json as a JSON-encoded string \
        (e.g. '\"gitmoji\"' for strings, 'true' for bools, '42' for numbers). \
        The variable must be declared in the skill's vars.json."
    )]
    async fn set_skill_var(&self, Parameters(req): Parameters<SetSkillVarRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let result = set_skill_var_tool(&project_dir, req);
        if !result.starts_with("Error") {
            self.notify_resources_changed().await;
        }
        result
    }

    // ---- Session Files ----

    #[tool(
        description = "Write a file to .artifacts/. Fires a resource update notification \
        so subscribed clients react immediately. \
        Path is relative to .artifacts/ (e.g. 'canvas.html', 'vitest/report.html')."
    )]
    async fn write_session_file(
        &self,
        Parameters(req): Parameters<WriteSessionFileRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let path = req.path.clone();
        let result = session_files::write_session_file(&project_dir, req);
        if !result.starts_with("Error") {
            let uri = format!("ship://session/{}", path);
            self.notify_resource_updated(&uri).await;
            self.notify_resources_changed().await;
        }
        result
    }

    // ---- Mesh ----

    #[tool(description = "Send a directed message to another agent on the mesh.")]
    async fn send_mesh_message(&self, Parameters(req): Parameters<MeshSendRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_send(&actor_id, &req.to, req.body).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Broadcast a message to all agents on the mesh, optionally filtered by capability."
    )]
    async fn broadcast_mesh_message(
        &self,
        Parameters(req): Parameters<MeshBroadcastRequest>,
    ) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_broadcast(&actor_id, req.body, req.capability_filter).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Discover agents on the mesh. Optionally filter by capability or status."
    )]
    async fn discover_mesh_agents(
        &self,
        Parameters(_req): Parameters<MeshDiscoverRequest>,
    ) -> String {
        match crate::network_client::mesh_discover().await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Update this agent's status on the mesh (active, busy, idle).")]
    async fn update_mesh_status(&self, Parameters(req): Parameters<MeshStatusRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_status(&actor_id, &req.status).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    // ---- Jobs ----

    #[tool(
        description = "Create a new job. Emits job.created and returns {job_id}. \
        Jobs are event-sourced — state is derived from the event log, not stored directly."
    )]
    async fn create_job(&self, Parameters(req): Parameters<CreateJobRequest>) -> String {
        job_tools::create_job(req).await
    }

    #[tool(
        description = "Advance a job to the next status by emitting the appropriate event. \
        status: dispatched (requires worktree), gate_requested (requires gate_agent), \
        gate_passed, gate_failed (requires error), blocked (requires blocker), completed, \
        merged, failed (requires error)."
    )]
    async fn update_job(&self, Parameters(req): Parameters<UpdateJobRequest>) -> String {
        job_tools::update_job(req).await
    }

    #[tool(
        description = "Declare the current pipeline step complete. Call this \
        as your last action before exiting. If you exit without calling step_done, \
        the daemon treats the step as failed."
    )]
    async fn step_done(&self, Parameters(req): Parameters<StepDoneRequest>) -> String {
        job_tools::step_done(req).await
    }

    // ---- Collections ----

    #[tool(
        description = "Create a new item in a collection. The collection must have a schema \
        defined in .ship/collections/. Data is validated against the schema before writing."
    )]
    async fn create_doc(
        &self,
        Parameters(req): Parameters<crate::requests::collections::CreateDocRequest>,
    ) -> String {
        match crate::network_client::create_doc(&req.collection, &req.data, req.id.as_deref()).await {
            Ok(r) => r,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Update fields on an existing collection item. Only specified fields \
        are changed — unspecified fields are preserved. Validates the merged result."
    )]
    async fn update_doc(
        &self,
        Parameters(req): Parameters<crate::requests::collections::UpdateDocRequest>,
    ) -> String {
        match crate::network_client::update_doc(&req.collection, &req.item_id, &req.data).await {
            Ok(r) => r,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Get a single item from a collection by ID."
    )]
    async fn get_doc(
        &self,
        Parameters(req): Parameters<crate::requests::collections::GetDocRequest>,
    ) -> String {
        match crate::network_client::get_doc(&req.collection, &req.item_id).await {
            Ok(r) => r,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Delete an item from a collection by ID."
    )]
    async fn delete_doc(
        &self,
        Parameters(req): Parameters<crate::requests::collections::DeleteDocRequest>,
    ) -> String {
        match crate::network_client::delete_doc(&req.collection, &req.item_id).await {
            Ok(r) => r,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Query items in a collection with filters, sorting, and pagination. \
        Filter operators: $eq (implicit), $ne, $gt, $gte, $lt, $lte, $in, $exists. \
        Sort: field name to 1 (asc) or -1 (desc). Returns matching items."
    )]
    async fn query_docs(
        &self,
        Parameters(req): Parameters<crate::requests::collections::QueryDocsRequest>,
    ) -> String {
        let body = serde_json::json!({
            "collection": req.collection,
            "filter": req.filter,
            "sort": req.sort,
            "limit": req.limit,
        });
        match crate::network_client::query_docs(&req.collection, body).await {
            Ok(r) => r,
            Err(e) => format!("Error: {e}"),
        }
    }

    // ---- View client-error capture ----

    #[tool(
        description = "Retrieve client-side errors captured from Ship views (Datastar, JS, fetch). \
        Use this instead of asking the user to copy browser console output. \
        Returns recent browser-reported errors from the daemon's ring buffer: \
        JS errors, unhandled promise rejections, and Datastar fetch failures. \
        Args: since (id floor, for incremental polling), view (filter), limit \
        (default 50, max 200), clear_after (drain buffer after returning)."
    )]
    async fn get_view_errors(
        &self,
        Parameters(req): Parameters<crate::requests::ViewErrorsRequest>,
    ) -> String {
        crate::tools::view_errors::view_errors(req).await
    }

    #[tool(
        description = "Post a plain-text reply into a Ship command-bar chat actor. \
        This is how Shipwright (and other agents) address the user in the Ship UI — \
        the text renders as an agent bubble in the actor's thread. \
        Read the target `actor_id` from the `SHIP_ACTOR_ID` environment variable \
        the Ship daemon sets when spawning you; never guess it. \
        Prefer `chat_reply` over printing to stdout — stdout is logged, not shown to \
        the user. Keep replies concise; no markdown formatting (the UI is not a \
        markdown renderer)."
    )]
    async fn chat_reply(
        &self,
        Parameters(req): Parameters<crate::requests::ChatReplyRequest>,
    ) -> String {
        match crate::network_client::session_reply(&req.actor_id, &req.text).await {
            Ok(_) => "ok".to_string(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Fuzzy-search Ship's SDK surface — macros in _ship.html.j2, REST routes, \
        view manifests, collection/trigger schemas. Use this BEFORE authoring a view or asking \
        which macro exists. Prefer this over grepping the daemon source. \
        Args: query (required), limit (default 20), kind ('macro'|'route'|'view'|'collection'|'trigger')."
    )]
    async fn query_sdk(
        &self,
        Parameters(req): Parameters<crate::requests::SdkQueryRequest>,
    ) -> String {
        crate::tools::sdk_query::sdk_query(req).await
    }

    #[tool(
        description = "Generate a starter Ship view template for a given shape. Returns Jinja source \
        + manifest JSON — the caller writes them. Prefer this over authoring a view from scratch; \
        idioms stay consistent. \
        Args: name (required), collection (required for list/form/detail shapes), \
        shape ('blank'|'list'|'list-edit-delete'|'form'|'detail'|'tabs', default 'blank')."
    )]
    async fn scaffold_view(
        &self,
        Parameters(req): Parameters<crate::requests::ViewScaffoldRequest>,
    ) -> String {
        crate::tools::view_scaffold::view_scaffold(req)
    }

    // ---- Sui LSP ----

    #[tool(
        description = "Run sui-lsp diagnostics on a `.sui` file and return the same \
        findings the VS Code extension surfaces. Call BEFORE declaring a `.sui` edit done. \
        Catches parse errors, missing directive attributes, assignments inside template \
        expressions, arrow functions outside `on:` handlers, unbound identifiers, and \
        missing setup keys — feedback the agent would otherwise only see at build time. \
        Args: file (absolute path or workspace-relative)."
    )]
    async fn sui_diagnostics(
        &self,
        Parameters(req): Parameters<crate::requests::SuiDiagnosticsRequest>,
    ) -> String {
        sui_lsp_tools::sui_diagnostics(&req.file).await
    }

    #[tool(
        description = "Return hover info for an identifier or directive at a (line, col) \
        position in a `.sui` file — declaration source, setup-return type, namespace \
        contributions, or the directive contract reminder. \
        Args: file (path), line (0-based), col (0-based)."
    )]
    async fn sui_hover(
        &self,
        Parameters(req): Parameters<crate::requests::SuiHoverRequest>,
    ) -> String {
        sui_lsp_tools::sui_hover(&req.file, req.line, req.col).await
    }

    #[tool(
        description = "Report the surface a `.sui` view depends on: custom-element tags \
        (sui-*, ship-*, third-party), `on:*` event names, `bind:*` targets, identifier \
        references partitioned into setup-backed signals vs unknown-scope, namespace \
        aliases (`use:*`), and the resolved paired-controller path. Use before editing \
        a view to map blast radius — which tags to register, which signals to preserve. \
        Args: file (absolute path or workspace-relative)."
    )]
    async fn sui_dependencies(
        &self,
        Parameters(req): Parameters<crate::requests::SuiDependenciesRequest>,
    ) -> String {
        sui_lsp_tools::sui_dependencies(&req.file).await
    }

    // ---- Artifacts ----

    #[tool(
        description = "Upload an artifact to the Ship artifact store. \
        bytes_b64 is the file content base64-encoded. \
        scope: run|project|user. run_id required when scope=run. \
        Returns { id } of the stored artifact."
    )]
    async fn upload_artifact(
        &self,
        Parameters(req): Parameters<crate::requests::artifacts::ArtifactUploadRequest>,
    ) -> String {
        artifact_tools::artifact_upload(
            &req.scope,
            req.run_id.as_deref(),
            &req.bytes_b64,
            &req.name,
            req.mime.as_deref(),
            req.expires_at.as_deref(),
        )
        .await
    }

    #[tool(
        description = "Get artifact metadata (JSON record: id, name, mime, size, \
        scope, content_hash, path, created_at). Returns the metadata record only, \
        not the binary payload — read the file at `path` to retrieve bytes."
    )]
    async fn get_artifact(
        &self,
        Parameters(req): Parameters<crate::requests::artifacts::ArtifactIdRequest>,
    ) -> String {
        artifact_tools::artifact_get(&req.id).await
    }

    #[tool(
        description = "List artifacts. scope: run|project|user (default project). \
        filter_json: optional JSON object with additional query filters."
    )]
    async fn list_artifacts(
        &self,
        Parameters(req): Parameters<crate::requests::artifacts::ArtifactListRequest>,
    ) -> String {
        artifact_tools::artifact_list(req.scope.as_deref(), req.filter_json.as_deref()).await
    }

    #[tool(description = "Delete an artifact by ID.")]
    async fn delete_artifact(
        &self,
        Parameters(req): Parameters<crate::requests::artifacts::ArtifactIdRequest>,
    ) -> String {
        artifact_tools::artifact_delete(&req.id).await
    }

    #[tool(
        description = "Promote an artifact to a new scope. \
        target_scope: project|user. Requires PromoteArtifact grant."
    )]
    async fn promote_artifact(
        &self,
        Parameters(req): Parameters<crate::requests::artifacts::ArtifactPromoteRequest>,
    ) -> String {
        artifact_tools::artifact_promote(&req.id, &req.target_scope).await
    }
}

// ---- Experimental tool registration ----
//
// These tools are always registered. The `experimental` marker is applied by
// `stability::finalize_router` after the main and experimental routers merge:
// each tool's description gains an "**EXPERIMENTAL** — interface may change…"
// prefix. The historical `unstable` Cargo feature still exists for runtime-
// behavior gating but no longer controls tool registration.

#[tool_router(router = experimental_tool_router)]
impl ShipServer {
    // ---- Dispatch ----

    #[tool(
        description = "Spawn an agent: creates a git worktree, compiles harness config via \
        `ship use`, launches the agent process, and registers it on the mesh. \
        Returns the job_id for use with steer_agent, list_agents, stop_agent."
    )]
    async fn dispatch_agent(
        &self,
        Parameters(req): Parameters<DispatchAgentRequest>,
    ) -> String {
        dispatch_tools::dispatch_agent(req).await
    }

    #[tool(
        description = "List all running agents with their status, capabilities, and labels."
    )]
    async fn list_agents(&self) -> String {
        dispatch_tools::list_agents().await
    }

    #[tool(
        description = "Stop a running agent by slug. Terminates the process and \
        removes it from the registry."
    )]
    async fn stop_agent(&self, Parameters(req): Parameters<StopAgentRequest>) -> String {
        dispatch_tools::stop_agent(req).await
    }

    #[tool(
        description = "Inject a message into a running agent's stdin (Claude) or \
        via turn/steer (Codex)."
    )]
    async fn steer_agent(&self, Parameters(req): Parameters<SteerAgentRequest>) -> String {
        dispatch_tools::steer_agent(req).await
    }

    // ---- Events ----

    #[tool(
        description = "Emit a domain event. Reserved types (actor.*, session.*, skill.*, \
        workspace.*, gate.*, job.*, config.*, project.*) are rejected. \
        actor_id and workspace_id are injected from connection context — not agent-controlled."
    )]
    async fn emit_event(&self, Parameters(req): Parameters<ShipEventRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let actor_id = self.resolve_actor_id().await;
        let workspace_id =
            tool_gate::current_branch(project_dir.parent().unwrap_or(&project_dir))
                .unwrap_or_else(|_| "unknown".to_string());

        match event::emit_domain_event(
            &actor_id,
            &workspace_id,
            &req.event_type,
            req.payload,
            req.elevated.unwrap_or(false),
        )
        .await
        {
            Ok(resp) => {
                self.notify_resource_updated("ship://events").await;
                resp
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    // ---- Views (Phase 1 canvas framework) ----

    #[tool(
        description = "Open a new agent-owned view session. Returns `view_id` + `url`.\n\
        `_meta.ui.resourceUri` → `ui://ship/view-session/{view_id}` — MCP Apps \
        hosts can render the returned id natively.\n\
        Arguments: `template` (required, registered minijinja template), \
        `title` (optional), `initial_context` (optional JSON)."
    )]
    async fn open_view(&self, Parameters(req): Parameters<crate::requests::ViewOpenRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        crate::tools::view::view_open(&actor_id, req).await
    }

    #[tool(description = "Patch (morph) a selector inside an open view session with new HTML.")]
    async fn patch_view(&self, Parameters(req): Parameters<crate::requests::ViewPatchRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        crate::tools::view::view_patch(&actor_id, req).await
    }

    #[tool(description = "Append HTML at a selector inside an open view session.")]
    async fn append_view(&self, Parameters(req): Parameters<crate::requests::ViewAppendRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        crate::tools::view::view_append(&actor_id, req).await
    }

    #[tool(description = "Set a reactive signal in an open view session.")]
    async fn signal_view(&self, Parameters(req): Parameters<crate::requests::ViewSignalRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        crate::tools::view::view_signal(&actor_id, req).await
    }

    #[tool(description = "Close a view session. Only the owning agent may close.")]
    async fn close_view(&self, Parameters(req): Parameters<crate::requests::ViewCloseRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        crate::tools::view::view_close(&actor_id, req).await
    }

    // ---- View authoring (chrome edit-mode tools) ----
    //
    // Each tool dispatches to the daemon, which enforces the per-session
    // edit-mode capability set up by chrome's `enter`/`exit` calls.

    #[tool(
        description = "Read the on-disk `.sui` source of a view. Requires the calling \
        session to be in view-edit mode for the requested view. Returns the source \
        and its sha256 content hash."
    )]
    async fn view_read_source(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewName>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_read_source(session_id.as_deref(), req).await
    }

    #[tool(
        description = "Atomically write new `.sui` source for a view. Snapshots the new \
        source, runs `compile_check`, and either keeps the snapshot (on success) or \
        reverts the file and discards the snapshot (on failure). Returns the new \
        snapshot id, content hash, and any compile errors."
    )]
    async fn view_write_source(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewWriteSource>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_write_source(session_id.as_deref(), req).await
    }

    #[tool(
        description = "Bundler dry-run against the current on-disk view source. Does not \
        write. Returns `{ ok, errors? }`."
    )]
    async fn view_compile_check(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewName>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_compile_check(session_id.as_deref(), req).await
    }

    #[tool(
        description = "Return collection schemas visible in the project. Pass `id` to fetch \
        a single schema; omit for the full list."
    )]
    async fn view_collections_schema(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewCollectionsSchema>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_collections_schema(
            session_id.as_deref(),
            req,
            None,
        )
        .await
    }

    #[tool(
        description = "List sibling views in the active project. Each entry includes the \
        view name, on-disk path to its `page.sui`, and any `summary` from its manifest."
    )]
    async fn view_list_siblings(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewName>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_list_siblings(
            session_id.as_deref(),
            crate::requests::view_authoring::ViewNoArgs::default(),
            &req.view,
        )
        .await
    }

    #[tool(
        description = "Return the current theme's CSS custom properties as a `name → value` \
        map (e.g. `{ \"--bg\": \"#0a0a0a\" }`). Empty when the default `ship` theme is active."
    )]
    async fn view_theme_tokens(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewName>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_theme_tokens(
            session_id.as_deref(),
            crate::requests::view_authoring::ViewNoArgs::default(),
            &req.view,
        )
        .await
    }

    #[tool(
        description = "Server-render the view and return its serialized HTML. Use this to \
        inspect the current preview without subscribing to the SSE channel."
    )]
    async fn view_preview_snapshot(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewName>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_preview_snapshot(session_id.as_deref(), req).await
    }

    #[tool(
        description = "Pin an ephemeral annotation on a selector inside a view. Broadcasts \
        via SSE to chrome edit-mode UI. Auto-clears N seconds after the next \
        `view_write_source` on the same view."
    )]
    async fn view_annotate(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewAnnotate>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_annotate(session_id.as_deref(), req).await
    }

    #[tool(
        description = "List prior snapshots of a view, newest first. Paginate with \
        `cursor` returned from a previous page (empty when on the tail)."
    )]
    async fn view_list_snapshots(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewListSnapshotsArgs>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_list_snapshots(session_id.as_deref(), req).await
    }

    #[tool(
        description = "Restore a view to a prior snapshot. Atomic — runs `compile_check` \
        on the restored bytes; the restore itself becomes a new snapshot."
    )]
    async fn view_restore_snapshot(
        &self,
        Parameters(req): Parameters<crate::requests::view_authoring::ViewRestoreSnapshotArgs>,
    ) -> String {
        let session_id = self.session_id.lock().await.clone();
        crate::tools::view_authoring::view_restore_snapshot(session_id.as_deref(), req).await
    }
}


// ---- Server entry point ----

pub async fn run_server() -> Result<()> {
    let service = ShipServer::new();
    let handle = service.clone();
    let running = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow!("MCP Server initialization error: {:?}", e))?;
    running
        .waiting()
        .await
        .map_err(|e| anyhow!("MCP Server runtime error: {:?}", e))?;
    handle.shutdown().await;
    Ok(())
}

// ---- Tests ----

#[cfg(test)]
#[path = "../server_tests.rs"]
mod server_tests;

#[cfg(test)]
#[path = "../tool_gate_tests.rs"]
mod tool_gate_tests;

#[cfg(test)]
#[path = "../role_gate_integration_tests.rs"]
mod role_gate_integration_tests;
