//! The agent-cancel decision — `service.agents`' handling of
//! `session.cancelled`.
//!
//! Stop the running harness, tear down its worktree/tmux, and emit
//! `process.killed` for audit. The projection has already transitioned the
//! session to Cancelled via the transition table; this decision's job is to
//! stop the running work.
//!
//! The teardown MECHANISM is the slug-keyed primitive at
//! [`bridge::lifecycle::stop_by_slug`] /
//! [`bridge::lifecycle::stop_handle`]. The slug
//! is resolved from the session_id via `ProjectionReader::slug_for_session_id`
//! — keeps this decision off `crate::projections` directly.
//!
//! The Stop RPC (`AgentsService::stop`) now emits `session.cancelled`
//! (Runtime-scoped, Durable, canonical payload with slug populated) and returns
//! optimistically; this handler performs the actual process teardown on receipt
//! of that event. See apps/shipd/src/ops_rpc.rs stop_fn for the emit seam.

use std::sync::Arc;

use agent_schema::canonical_session::SessionCancelled;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::emit::persist_and_route;
use ship_substrate::events::process::ProcessKilledPayload;
use ship_types::event_types as process_event_types;
use tokio::sync::Mutex;

use crate::ports::AgentPorts;

type Kernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// Handle `session.cancelled`. Reads `session_id` from the canonical payload,
/// resolves the slug via `ProjectionReader::slug_for_session_id`, stops the
/// process via the L0 teardown core, cleans up the worktree/tmux/branch, and
/// emits `process.killed`.
pub async fn handle_agent_cancelled(kernel: &Kernel, envelope: &EventEnvelope, ports: &AgentPorts) {
    let payload: SessionCancelled = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                entity_id = %envelope.entity_id,
                "agent-session-cancel: malformed session.cancelled payload: {e}"
            );
            return;
        }
    };

    let session_id = &payload.session_id;

    // Resolve slug from session_id via the ProjectionReader port so this
    // module stays off crate::projections directly.
    let slug = match ports.projections.slug_for_session_id(session_id).await {
        Some(s) if !s.is_empty() => s,
        _ => {
            tracing::warn!(
                session_id = %session_id,
                "agent-session-cancel: no slug found for session_id — skipping process kill"
            );
            return;
        }
    };

    // Registry first (fast path), via the L0 slug-keyed teardown primitive. On a
    // registry miss, try the executor's resume path — best effort — and stop the
    // recovered handle through the same L0 teardown core.
    let pid = if let Some(pid) = bridge::lifecycle::stop_by_slug(
        &ports.lifecycle,
        &ports.registry,
        &ports.executor,
        &slug,
        "cancellation",
        "operator",
    )
    .await
    {
        pid
    } else if let Some(handle) = ports.executor.resume_handle(&slug, None, None).await {
        bridge::lifecycle::stop_handle(
            &ports.lifecycle,
            &ports.executor,
            &slug,
            handle,
            "cancellation",
            "operator",
        )
        .await
    } else {
        // No handle to stop — but a dispatch for this slug may be spawning right
        // now (cancel raced the spawn). Drop a tombstone so the in-flight
        // dispatch tears the child down after it registers instead of leaking
        // it.
        ports.registry.mark_cancelled(&slug).await;
        tracing::info!(
            slug,
            "agent-session-cancel: no live handle found; tombstoned for in-flight spawn, proceeding to cleanup"
        );
        None
    };

    // Tear down worktree + tmux + branch. Bare agent-session branches follow
    // the `job/<slug>` dispatch convention; `cleanup_job` keys the worktree/tmux
    // by slug and uses the branch only to delete it.
    let branch = format!("job/{slug}");
    ports.platform.cleanup_job(&slug, &branch).await;

    // Emit process.killed for audit.
    let killed = ProcessKilledPayload {
        slug: slug.clone(),
        pid,
        signal: "SIGTERM+SIGKILL".into(),
        reason: "cancellation".into(),
        actor: "operator".into(),
    };
    match EventEnvelope::new(process_event_types::PROCESS_KILLED, &slug, &killed) {
        Ok(env) => {
            let env = env.with_actor_id("service.agents");
            persist_and_route(kernel, env, &slug).await;
        }
        Err(e) => {
            tracing::warn!(
                slug,
                "agent-session-cancel: failed to build process.killed envelope: {e}"
            );
        }
    }
}
