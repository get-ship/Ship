//! Shared kernel-control operations behind the `actor.*` commands. The logic
//! lives here — NOT inline in REST handlers — so both transports (REST today,
//! Connect command) stay thin over it, and it survives the REST deletion (#78).
//!
//! These are operational/internal ops (operator/debug surface, not agent-facing
//! capability — see the audience axis, #95). Behaviour-preserving C1.

use serde_json::json;

use ship_substrate::events::types::event_types;
use ship_substrate::events::EventEnvelope;
use ship_substrate::projections::actor::actor_status;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;

/// Why an actor-input delivery failed — each transport maps this to its own
/// error shape (HTTP status / `CommandError`).
#[derive(Debug)]
pub enum ActorInputError {
    /// Text was empty after trimming.
    Empty,
    /// The actor isn't currently registered in the kernel (source of truth).
    NotOnMesh,
    /// Envelope build or emit failed.
    Service(String),
}

/// Deliver a text turn to a live actor's mailbox: validate the actor is
/// registered in the kernel *now* (the projection's status can lag), then emit a
/// `STUDIO_CHAT_TURN` targeted at it with the caller's emit context.
pub async fn deliver_actor_input(
    state: &ApiState,
    auth: &AuthContext,
    id: &str,
    text: &str,
) -> Result<(), ActorInputError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ActorInputError::Empty);
    }
    // Kernel mailbox is the source of truth for "can I reach this actor"; the
    // projection's status field can lag behind un-applied events.
    if !state.kernel().lock().await.has_actor(id) {
        return Err(ActorInputError::NotOnMesh);
    }
    let workspace_id = actor_status(id).ok().flatten().and_then(|(_, w)| w);
    let payload = json!({ "actor_id": id, "text": text });
    let envelope = EventEnvelope::new(event_types::STUDIO_CHAT_TURN, id, &payload)
        .map(|e| e.with_context(workspace_id.as_deref(), Some(id)).with_target(id))
        .map_err(|e| ActorInputError::Service(format!("envelope: {e}")))?;
    let emit_ctx = auth.to_daemon_emit_context();
    state
        .kernel()
        .lock()
        .await
        .emit(envelope, &emit_ctx)
        .await
        .map_err(|e| ActorInputError::Service(format!("emit: {e}")))?;
    Ok(())
}
