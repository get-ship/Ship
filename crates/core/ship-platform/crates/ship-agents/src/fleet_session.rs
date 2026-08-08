//! Phase E — dispatch 3P harnesses through Fleet [`SessionManager`] (no second LaunchPlan).
//!
//! Opt-in: `FLEET_SESSION_SERVICE=1` (or `true` / `yes` / `embed`). When unset,
//! classic `SessionExecutor` path remains.
//!
//! E2: mission scratch / worktree env is copied onto CreateParams from
//! [`SessionContext::env`] plus SHIP_* markers.
//!
//! In-process SessionManager (product door). Requires a fleet `bridge` pin or
//! path that exports `session_create` with multi_turn/catalog fields.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use agent_schema::InvocationMode;
use agent_schema::dispatch_intent::DispatchRequestedPayload;
use bridge::dispatch_ports::{ExitCause, ExitSignal, SessionContext};
use bridge::lifecycle::ExitVocabulary;
use bridge::lifecycle::exit_handler;
use bridge::{
    CreateParams, SessionCreateEvent, SessionManager, SessionMode, SessionTransport,
    WorkspaceStrategy,
};
use tokio::sync::mpsc;

use crate::ports::AgentPorts;

type Kernel = Arc<tokio::sync::Mutex<ship_substrate::events::KernelRouter>>;

/// slug → (manager, fleet session_id) for cancel teardown.
fn fleet_live() -> &'static Mutex<HashMap<String, (Arc<SessionManager>, String)>> {
    static MAP: OnceLock<Mutex<HashMap<String, (Arc<SessionManager>, String)>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

/// True when dispatch should use Fleet SessionManager instead of SessionExecutor.
pub fn fleet_session_service_enabled() -> bool {
    match std::env::var("FLEET_SESSION_SERVICE") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "embed"
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_session_env_parsing() {
        // Safety: test-only env mutation; single-threaded unit test.
        unsafe {
            std::env::remove_var("FLEET_SESSION_SERVICE");
        }
        assert!(!fleet_session_service_enabled());
        unsafe {
            std::env::set_var("FLEET_SESSION_SERVICE", "1");
        }
        assert!(fleet_session_service_enabled());
        unsafe {
            std::env::set_var("FLEET_SESSION_SERVICE", "embed");
        }
        assert!(fleet_session_service_enabled());
        unsafe {
            std::env::remove_var("FLEET_SESSION_SERVICE");
        }
    }
}

/// Cancel a fleet-managed session by dispatch slug (E1 cancel path).
pub async fn cancel_by_slug(slug: &str) -> bool {
    let entry = {
        let mut g = fleet_live().lock().unwrap_or_else(|e| e.into_inner());
        g.remove(slug)
    };
    if let Some((manager, sid)) = entry {
        let _ = manager.cancel(&sid).await;
        return true;
    }
    false
}

/// Spawn a Fleet session for a coding-agent dispatch (after isolation prepare).
pub async fn dispatch_via_fleet_session(
    kernel: &Kernel,
    payload: &DispatchRequestedPayload,
    ports: &AgentPorts,
    session_ctx: &SessionContext,
    worktree_path: PathBuf,
) {
    let slug = &payload.slug;
    let harness = payload
        .harness
        .clone()
        .unwrap_or_else(|| "claude".to_string());

    let multi_turn = payload.keepalive
        || matches!(
            payload.invocation,
            InvocationMode::Pty | InvocationMode::Stream
        );

    let transport = match payload.invocation {
        InvocationMode::Pty | InvocationMode::Stream if bridge::product_acp(&harness) => {
            Some(SessionTransport::Acp)
        }
        _ => Some(SessionTransport::Process),
    };

    let mode = match transport {
        Some(SessionTransport::Acp) => SessionMode::Interactive,
        _ => SessionMode::Headless,
    };

    let prompt = payload
        .prompt
        .clone()
        .or_else(|| payload.spec.clone())
        .filter(|s| !s.trim().is_empty());

    // E2: preserve mission scratch / spawn env; never place-on-run.
    let mut env: HashMap<String, String> = session_ctx.env.clone();
    env.insert(
        "SHIP_WORKTREE".into(),
        worktree_path.display().to_string(),
    );
    env.insert("SHIP_SESSION_ID".into(), payload.session_id.clone());
    env.insert("SHIP_SLUG".into(), slug.clone());
    if let Some(ref p) = payload.source_spec_path {
        env.insert("SHIP_SPEC_PATH".into(), p.clone());
    }

    let params = CreateParams {
        agent: None,
        agent_json: Some(
            serde_json::json!({
                "harness": { "id": harness },
                "identity": { "id": payload.agent_id, "name": payload.agent_id },
            })
            .to_string(),
        ),
        harness: harness.clone(),
        strategy: WorkspaceStrategy::Existing,
        repo: session_ctx.repo_path.clone(),
        base_ref: session_ctx.base_ref.clone(),
        path: Some(worktree_path.clone()),
        mode,
        prompt,
        session_id: Some(payload.session_id.clone()),
        model: payload.model.clone(),
        env,
        plugin_dirs: vec![],
        permission_mode: None,
        launch_plan: None,
        transport,
        multi_turn,
        resume_harness_session_id: payload.resume_session_id.clone(),
    };

    let manager = Arc::new(SessionManager::new().with_default_catalog());
    let (tx, mut rx) = mpsc::unbounded_channel();
    let sid = match manager.create(params, tx).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                slug,
                session_id = payload.session_id,
                error = %e,
                "fleet session: Create failed"
            );
            super::dispatch::emit_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("fleet SessionManager.create: {e}"),
            )
            .await;
            return;
        }
    };

    tracing::info!(
        slug,
        session_id = %sid,
        harness = %harness,
        multi_turn,
        "fleet session: created via SessionManager (FLEET_SESSION_SERVICE)"
    );

    crate::emit::emit_process_spawned(
        kernel,
        slug,
        &payload.session_id,
        None,
        &harness,
        &[],
        &[],
        &worktree_path.to_string_lossy(),
        if multi_turn {
            "fleet-session-multi"
        } else {
            "fleet-session"
        },
        None,
    )
    .await;

    if ports.registry.take_if_cancelled(slug).await {
        let _ = manager.cancel(&sid).await;
        return;
    }

    {
        let mut g = fleet_live().lock().unwrap_or_else(|e| e.into_inner());
        g.insert(slug.clone(), (manager.clone(), sid.clone()));
    }

    // Registry entry so take_if_cancelled / live_slugs see the slug (no OS pid).
    ports
        .registry
        .register(
            slug,
            bridge::dispatch_ports::ExecutorHandle {
                pid: None,
                inner: Box::new(sid.clone()),
            },
        )
        .await;

    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    exit_handler::start(
        ports.lifecycle.clone(),
        ports.registry.clone(),
        ExitVocabulary::AgentSession {
            session_id: payload.session_id.clone(),
        },
        exit_rx,
    );

    let slug_exit = slug.clone();
    let session_id_exit = payload.session_id.clone();
    tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            match &ev {
                SessionCreateEvent::Output { text, role, .. } => {
                    tracing::debug!(slug = %slug_exit, role = %role, "{text}");
                }
                SessionCreateEvent::TurnComplete { stop_reason, .. } => {
                    tracing::debug!(slug = %slug_exit, %stop_reason, "turn complete");
                }
                SessionCreateEvent::Completed { .. } => {
                    let _ = exit_tx.send(ExitSignal {
                        slug: slug_exit.clone(),
                        session_id: session_id_exit.clone(),
                        exit_code: Some(0),
                        cause: ExitCause::AgentExited,
                    });
                    break;
                }
                SessionCreateEvent::Failed { code, message, .. } => {
                    tracing::warn!(slug = %slug_exit, %code, %message, "fleet session failed");
                    let _ = exit_tx.send(ExitSignal {
                        slug: slug_exit.clone(),
                        session_id: session_id_exit.clone(),
                        exit_code: Some(1),
                        cause: ExitCause::AgentExited,
                    });
                    break;
                }
                SessionCreateEvent::Cancelled { .. } => {
                    let _ = exit_tx.send(ExitSignal {
                        slug: slug_exit.clone(),
                        session_id: session_id_exit.clone(),
                        exit_code: Some(130),
                        cause: ExitCause::AgentExited,
                    });
                    break;
                }
                SessionCreateEvent::Started { .. } => {}
            }
        }
        let mut g = fleet_live().lock().unwrap_or_else(|e| e.into_inner());
        g.remove(&slug_exit);
    });
}
