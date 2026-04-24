//! HTTP client for forwarding mesh/session operations to the shipd daemon.

use anyhow::{Result, anyhow};
use serde_json::Value;
use std::sync::OnceLock;
use tokio::sync::Mutex;

/// Cached session token — set after register_session succeeds.
static SESSION_TOKEN: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn session_token_store() -> &'static Mutex<Option<String>> {
    SESSION_TOKEN.get_or_init(|| Mutex::new(None))
}

/// Get the shipd port from ~/.ship/daemon.port
fn get_network_port() -> Result<u16> {
    let global_dir = runtime::project::get_global_dir()?;
    let port_file = global_dir.join("daemon.port");
    let port_str = std::fs::read_to_string(&port_file)
        .map_err(|_| anyhow!("ship daemon is not running — start it with: ship daemon start"))?;
    port_str
        .trim()
        .parse::<u16>()
        .map_err(|e| anyhow!("failed to parse network port: {e}"))
}

/// Read the daemon bearer token from ~/.ship/daemon.token.
fn get_network_token() -> Result<String> {
    let global_dir = runtime::project::get_global_dir()?;
    let token = std::fs::read_to_string(global_dir.join("daemon.token"))
        .map_err(|_| anyhow!("ship daemon token not found — is the daemon running?"))?;
    Ok(token.trim().to_string())
}

/// Return the session token if registered, otherwise fall back to daemon token.
async fn get_effective_token() -> Result<String> {
    let store = session_token_store().lock().await;
    if let Some(ref t) = *store {
        return Ok(t.clone());
    }
    drop(store);
    get_network_token()
}

fn base_url() -> Result<String> {
    Ok(format!("http://127.0.0.1:{}/api", get_network_port()?))
}

/// POST JSON to a REST endpoint, return the response body.
async fn post(path: &str, body: Value) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

/// GET from a REST endpoint.
async fn get(path: &str) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

pub async fn mesh_register(agent_id: &str, capabilities: Vec<String>) -> Result<String> {
    post("/mesh/register", serde_json::json!({
        "agent_id": agent_id,
        "capabilities": capabilities,
    })).await
}

pub async fn mesh_send(from: &str, to: &str, body: Value) -> Result<String> {
    post("/mesh/send", serde_json::json!({
        "from": from,
        "to": to,
        "body": body,
    })).await
}

pub async fn mesh_broadcast(from: &str, body: Value, capability_filter: Option<String>) -> Result<String> {
    post("/mesh/broadcast", serde_json::json!({
        "from": from,
        "body": body,
        "capability_filter": capability_filter,
    })).await
}

pub async fn mesh_discover() -> Result<String> {
    get("/mesh/discover").await
}

pub async fn mesh_status(agent_id: &str, status: &str) -> Result<String> {
    post("/mesh/status", serde_json::json!({
        "agent_id": agent_id,
        "status": status,
    })).await
}

/// Spawn an actor in the daemon's KernelRouter with the given config.
/// The daemon stashes the mailbox for SSE delivery via /mesh/events/{actor_id}.
pub async fn actor_spawn(
    actor_id: &str,
    config: &runtime::events::ActorConfig,
    capabilities: Option<Vec<String>>,
) -> Result<String> {
    post("/actor/spawn", serde_json::json!({
        "actor_id": actor_id,
        "config": config,
        "capabilities": capabilities,
    })).await
}

/// Create a job via the daemon's REST API. The daemon handles pipeline
/// enrichment, persistence, kernel routing, and dispatch.
pub async fn create_job_via_daemon(body: serde_json::Value) -> Result<String> {
    post("/runtime/jobs", body).await
}

/// List running agents via the daemon's REST API.
pub async fn list_agents_via_daemon() -> Result<String> {
    get("/runtime/agents").await
}

/// Stop a running agent by slug via the daemon.
pub async fn stop_agent_via_daemon(slug: &str) -> Result<String> {
    post(&format!("/runtime/agents/{slug}/stop"), serde_json::json!({})).await
}

/// Inject a message into a running agent via the daemon.
pub async fn steer_agent_via_daemon(slug: &str, message: &str) -> Result<String> {
    post(
        &format!("/runtime/agents/{slug}/steer"),
        serde_json::json!({ "message": message }),
    ).await
}

/// POST with explicit `x-ship-agent-id` header so the daemon's auth layer
/// attributes the call to the calling agent (ownership enforcement). Used
/// by the `view.*` tool family.
async fn post_as_agent(path: &str, agent_id: &str, body: Value) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&token)
        .header("x-ship-agent-id", agent_id)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

async fn delete_as_agent(path: &str, agent_id: &str) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .delete(&url)
        .bearer_auth(&token)
        .header("x-ship-agent-id", agent_id)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

pub async fn view_open(
    agent_id: &str,
    template: &str,
    title: Option<&str>,
    initial_context: Option<&Value>,
) -> Result<String> {
    let mut body = serde_json::json!({ "template": template });
    if let Some(t) = title {
        body["title"] = serde_json::Value::String(t.to_string());
    }
    if let Some(c) = initial_context {
        body["initial_context"] = c.clone();
    }
    post_as_agent("/runtime/view-sessions", agent_id, body).await
}

pub async fn view_patch(agent_id: &str, view_id: &str, selector: &str, html: &str) -> Result<String> {
    post_as_agent(
        &format!("/runtime/view-sessions/{view_id}/patch"),
        agent_id,
        serde_json::json!({ "selector": selector, "html": html }),
    )
    .await
}

pub async fn view_append(agent_id: &str, view_id: &str, selector: &str, html: &str) -> Result<String> {
    post_as_agent(
        &format!("/runtime/view-sessions/{view_id}/append"),
        agent_id,
        serde_json::json!({ "selector": selector, "html": html }),
    )
    .await
}

pub async fn view_signal(agent_id: &str, view_id: &str, name: &str, value: &Value) -> Result<String> {
    post_as_agent(
        &format!("/runtime/view-sessions/{view_id}/signal"),
        agent_id,
        serde_json::json!({ "name": name, "value": value }),
    )
    .await
}

pub async fn view_close(agent_id: &str, view_id: &str) -> Result<String> {
    delete_as_agent(&format!("/runtime/view-sessions/{view_id}"), agent_id).await
}

/// Fetch the iframe shell HTML for a view session. Used by the MCP resource
/// resolver when an external host asks for `ui://ship/view-session/{id}`.
pub async fn view_session_raw(view_id: &str) -> Result<String> {
    get(&format!("/runtime/view-sessions/{view_id}/raw")).await
}

/// Download an artifact's bytes as a UTF-8 string. Used by `view.open` when
/// the caller targets a canvas artifact instead of a manifest template.
pub async fn artifact_download(artifact_id: &str) -> Result<String> {
    get(&format!("/artifacts/{artifact_id}")).await
}

/// Route a pre-built EventEnvelope through the daemon's KernelRouter.
/// This ensures events reach daemon subscribers (job-dispatch, workspace-sync, etc).
pub async fn event_route(
    envelope: &runtime::events::EventEnvelope,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<String> {
    post("/events/route", serde_json::json!({
        "envelope": envelope,
        "workspace_id": workspace_id,
        "session_id": session_id,
    })).await
}

/// Broadcast an agent reply onto a session's event stream. The daemon
/// emits it as a `ProviderEvent::Message`, which the command bar renders
/// as an agent bubble. Used by the `chat_reply` MCP tool.
pub async fn session_reply(session_id: &str, text: &str) -> Result<String> {
    post(
        &format!("/sessions/{session_id}/reply"),
        serde_json::json!({ "text": text }),
    )
    .await
}

/// Register a scoped session with the daemon. On success, stores the session
/// token so all subsequent requests use it instead of the daemon token.
///
/// If the daemon is not running or registration fails, returns an error
/// but does not prevent the MCP server from operating (falls back to daemon token).
pub async fn register_session(
    agent_id: &str,
    role: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<String> {
    let url = format!("{}/session/register", base_url()?);
    let daemon_token = get_network_token()?;
    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&daemon_token)
        .json(&serde_json::json!({
            "agent_id": agent_id,
            "role": role,
            "workspace_id": workspace_id,
        }))
        .send()
        .await
        .map_err(|e| anyhow!("session register failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("session register error ({}): {}", status, text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("failed to parse session register response: {e}"))?;
    let session_token = parsed["session_token"]
        .as_str()
        .ok_or_else(|| anyhow!("missing session_token in response"))?
        .to_string();

    // Store the session token for subsequent requests.
    *session_token_store().lock().await = Some(session_token.clone());
    tracing::info!(agent_id, "registered scoped session with daemon");

    Ok(session_token)
}

/// Open an SSE stream for mesh events targeted at this agent.
/// Returns a receiver that yields EventEnvelopes as they arrive.
/// The background task runs until the connection drops or the receiver is dropped.
pub fn mesh_subscribe(
    agent_id: &str,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<runtime::events::EventEnvelope>> {
    let url = format!("{}/mesh/events/{}", base_url()?, agent_id);
    let token = get_network_token()?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let resp = match reqwest::Client::new()
            .get(&url)
            .bearer_auth(&token)
            .header("Accept", "text/event-stream")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("mesh SSE connect failed: {e}");
                return;
            }
        };

        let status = resp.status();
        tracing::info!("mesh SSE connected, status={status}");
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("mesh SSE rejected: {status} {body}");
            return;
        }

        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        let mut buf = String::new();

        tracing::info!("mesh SSE entering read loop");
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("mesh SSE read error: {e}");
                    break;
                }
            };
            let chunk_str = String::from_utf8_lossy(&chunk);
            tracing::info!(len = chunk.len(), "mesh SSE chunk received");
            buf.push_str(&chunk_str);

            // Parse SSE frames: lines ending with \n\n
            while let Some(pos) = buf.find("\n\n") {
                let frame = buf[..pos].to_string();
                buf = buf[pos + 2..].to_string();

                // Extract data lines from the SSE frame
                let data: String = frame
                    .lines()
                    .filter_map(|line| line.strip_prefix("data:").or_else(|| line.strip_prefix("data: ")))
                    .collect::<Vec<_>>()
                    .join("\n");

                if data.is_empty() {
                    continue;
                }

                match serde_json::from_str::<runtime::events::EventEnvelope>(&data) {
                    Ok(event) => {
                        tracing::info!(event_type = %event.event_type, "mesh SSE event parsed");
                        if tx.send(event).is_err() {
                            tracing::warn!("mesh SSE receiver dropped");
                            return;
                        }
                    }
                    Err(e) => {
                        tracing::debug!(data = %data, "mesh SSE parse skip: {e}");
                    }
                }
            }
        }
        tracing::info!("mesh SSE stream ended");
    });

    Ok(rx)
}

// ── Collection operations ────────────────────────────────────────────────────

async fn delete(path: &str) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .delete(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

async fn put(path: &str, body: Value) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let token = get_effective_token().await?;
    let resp = reqwest::Client::new()
        .put(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("ship daemon unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship daemon error ({}): {}", status, text));
    }
    Ok(text)
}

pub async fn create_doc(collection: &str, data: &Value, id: Option<&str>) -> Result<String> {
    let mut body = serde_json::json!({ "data": data });
    if let Some(id) = id {
        body["id"] = serde_json::json!(id);
    }
    post(&format!("/collections/{collection}/items"), body).await
}

pub async fn update_doc(collection: &str, item_id: &str, data: &Value) -> Result<String> {
    put(
        &format!("/collections/{collection}/items/{item_id}"),
        serde_json::json!({ "data": data }),
    ).await
}

pub async fn get_doc(collection: &str, item_id: &str) -> Result<String> {
    get(&format!("/collections/{collection}/items/{item_id}")).await
}

pub async fn delete_doc(collection: &str, item_id: &str) -> Result<String> {
    delete(&format!("/collections/{collection}/items/{item_id}")).await
}

pub async fn query_docs(collection: &str, body: Value) -> Result<String> {
    post(&format!("/collections/{collection}/query"), body).await
}

// ── Artifact operations ───────────────────────────────────────────────────────

/// Upload an artifact via multipart POST. `bytes_b64` is base64-encoded content.
pub async fn artifact_upload(
    scope: &str,
    run_id: Option<&str>,
    bytes_b64: &str,
    name: &str,
    mime: Option<&str>,
    expires_at: Option<&str>,
) -> Result<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bytes_b64)
        .map_err(|e| anyhow!("base64 decode failed: {e}"))?;

    let port = get_network_port()?;
    let token = get_effective_token().await?;

    let mut url = format!("http://127.0.0.1:{port}/api/artifacts?scope={scope}");
    if let Some(rid) = run_id {
        url.push_str(&format!("&run_id={rid}"));
    }

    let mime_str = mime.unwrap_or("application/octet-stream").to_string();
    let file_part = reqwest::multipart::Part::bytes(bytes)
        .file_name(name.to_string())
        .mime_str(&mime_str)
        .map_err(|e| anyhow!("invalid mime: {e}"))?;

    let mut form = reqwest::multipart::Form::new().part("file", file_part);
    if let Some(exp) = expires_at {
        form = form.text("expires_at", exp.to_string());
    }

    let resp = reqwest::Client::new()
        .post(&url)
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("daemon unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("daemon error ({}): {}", status, text));
    }
    Ok(text)
}

/// Get artifact metadata. Returns JSON.
pub async fn artifact_get(id: &str) -> Result<String> {
    get(&format!("/artifacts/{id}/metadata")).await
}

/// List artifacts with optional scope filter and extra query params.
pub async fn artifact_list(scope: Option<&str>, filter_json: Option<&str>) -> Result<String> {
    let mut path = "/artifacts".to_string();
    let mut params = Vec::new();
    if let Some(s) = scope {
        params.push(format!("scope={s}"));
    }
    if let Some(f) = filter_json {
        // Parse as JSON object and append each k=v as query params.
        if let Ok(obj) = serde_json::from_str::<serde_json::Map<String, Value>>(f) {
            for (k, v) in &obj {
                if let Some(s) = v.as_str() {
                    params.push(format!("{k}={s}"));
                }
            }
        }
    }
    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }
    get(&path).await
}

/// Delete an artifact by ID.
pub async fn artifact_delete(id: &str) -> Result<String> {
    delete(&format!("/artifacts/{id}")).await
}

/// Promote an artifact to a new scope.
pub async fn artifact_promote(id: &str, target_scope: &str) -> Result<String> {
    post(
        &format!("/artifacts/{id}/promote"),
        serde_json::json!({ "target_scope": target_scope }),
    )
    .await
}
