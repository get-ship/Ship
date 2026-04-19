//! Trigger engine — subscribes to the kernel, filters triggers, runs actions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};

use crate::events::{ActorConfig, EventEnvelope, KernelRouter};
use crate::services::ServiceHandler;

use super::actions::{ActionCtx, dispatch};
use super::def::{TriggerAction, TriggerDef};
use super::filter;
use super::render;

/// Maximum trigger chain depth. Events emitted by a trigger carry a depth
/// marker; at depth ≥ this value, triggers are not executed.
pub const MAX_TRIGGER_DEPTH: u32 = 3;

/// Per-trigger / per-event context passed to each action while an action list
/// runs.
pub struct TriggerExecutionCtx {
    pub trigger_name: String,
    pub depth: u32,
    pub event_id: String,
    pub results: Vec<Value>,
}

/// Shared state for the trigger engine. Cloneable via `Arc` — the engine is
/// typically owned by one service task and multiple reloader paths.
pub struct TriggerEngine {
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    triggers: Arc<RwLock<HashMap<String, TriggerDef>>>,
}

impl TriggerEngine {
    pub fn new(kernel: Arc<Mutex<KernelRouter>>, ship_dir: PathBuf) -> Self {
        Self {
            kernel,
            ship_dir,
            triggers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn ship_dir(&self) -> &Path {
        &self.ship_dir
    }

    /// Replace the registered trigger set by rescanning the triggers directory.
    pub async fn reload(&self) {
        let report = super::loader::load_triggers_dir(&self.ship_dir);
        let mut map = HashMap::with_capacity(report.triggers.len());
        for trigger in report.triggers {
            map.insert(trigger.name.clone(), trigger);
        }
        tracing::info!(
            count = map.len(),
            errors = report.errors.len(),
            "triggers: reloaded"
        );
        *self.triggers.write().await = map;
    }

    /// Snapshot the triggers map (useful for tests).
    pub async fn snapshot(&self) -> Vec<TriggerDef> {
        self.triggers.read().await.values().cloned().collect()
    }

    /// Process one event — match every registered trigger against it and run
    /// matching triggers sequentially. Errors are logged per-trigger; they never
    /// prevent other triggers from running.
    pub async fn dispatch_event(&self, envelope: &EventEnvelope) {
        let depth = incoming_depth(envelope);
        if depth >= MAX_TRIGGER_DEPTH {
            tracing::warn!(
                event_id = %envelope.id,
                depth,
                "triggers: recursion guard — skipping"
            );
            return;
        }
        let event_ctx = filter::build_event_context(envelope);
        let triggers = self.triggers.read().await;
        for trigger in triggers.values() {
            if trigger.trigger.event != envelope.event_type {
                continue;
            }
            if !filter::matches(trigger.trigger.filter.as_ref(), &event_ctx) {
                continue;
            }
            if let Err(e) = self.run_trigger(trigger, envelope, &event_ctx, depth + 1).await {
                tracing::warn!(
                    trigger = trigger.name,
                    event_type = envelope.event_type,
                    "triggers: execution failed: {e}"
                );
            }
        }
    }

    async fn run_trigger(
        &self,
        trigger: &TriggerDef,
        envelope: &EventEnvelope,
        event_ctx: &Value,
        depth: u32,
    ) -> Result<()> {
        let mut exec = TriggerExecutionCtx {
            trigger_name: trigger.name.clone(),
            depth,
            event_id: envelope.id.clone(),
            results: Vec::new(),
        };
        for (idx, action) in trigger.actions.iter().enumerate() {
            let render_ctx = build_render_ctx(event_ctx, &exec);
            if !should_run(action, &render_ctx)? {
                tracing::debug!(trigger = trigger.name, idx, "triggers: action skipped by when");
                exec.results.push(Value::Null);
                continue;
            }
            let action_ctx = ActionCtx {
                kernel: &self.kernel,
                ship_dir: &self.ship_dir,
                trigger_event_id: &envelope.id,
                trigger_name: &trigger.name,
                depth,
            };
            match dispatch(&action.action_type, &render_ctx, &action_ctx).await {
                Ok(result) => exec.results.push(result),
                Err(e) => {
                    return Err(anyhow!(
                        "action[{idx}] ({}) failed: {e}",
                        action.action_type.kind()
                    ));
                }
            }
        }
        Ok(())
    }
}

fn should_run(action: &TriggerAction, render_ctx: &Value) -> Result<bool> {
    match &action.when {
        Some(expr) if !expr.trim().is_empty() => render::evaluate_when(expr, render_ctx),
        _ => Ok(true),
    }
}

fn build_render_ctx(event_ctx: &Value, exec: &TriggerExecutionCtx) -> Value {
    json!({
        "event": event_ctx,
        "workspace": event_ctx.get("workspace_id").cloned().unwrap_or(Value::Null),
        "actor": event_ctx.get("actor_id").cloned().unwrap_or(Value::Null),
        "trigger": {
            "name": exec.trigger_name,
            "depth": exec.depth,
        },
        "results": Value::Array(exec.results.clone()),
    })
}

fn incoming_depth(envelope: &EventEnvelope) -> u32 {
    let payload: Value = serde_json::from_str(&envelope.payload_json).unwrap_or(Value::Null);
    payload
        .get("_trigger_depth")
        .and_then(|v| v.as_u64())
        .map(|d| d as u32)
        .unwrap_or(0)
}

/// Headless service wrapper — routes mailbox events into `TriggerEngine`.
///
/// The trigger service subscribes to every namespace present in the loaded
/// trigger set. On each mailbox event, it dispatches through the engine.
pub struct TriggerService {
    pub engine: Arc<TriggerEngine>,
    runtime: tokio::runtime::Handle,
}

impl TriggerService {
    pub fn new(engine: Arc<TriggerEngine>, runtime: tokio::runtime::Handle) -> Self {
        Self { engine, runtime }
    }
}

impl ServiceHandler for TriggerService {
    fn name(&self) -> &str {
        "triggers"
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &crate::events::ActorStore) -> Result<()> {
        let engine = self.engine.clone();
        let envelope = event.clone();
        self.runtime.spawn(async move {
            engine.dispatch_event(&envelope).await;
        });
        Ok(())
    }
}

/// Spawn the trigger service on the kernel. Reloads the trigger set once
/// before returning so triggers are live immediately.
pub async fn spawn_trigger_service(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
) -> Result<Arc<TriggerEngine>> {
    let engine = Arc::new(TriggerEngine::new(kernel.clone(), ship_dir));
    engine.reload().await;

    let runtime = tokio::runtime::Handle::current();
    let service = Box::new(TriggerService::new(engine.clone(), runtime));
    let config = ActorConfig {
        namespace: "service.triggers".to_string(),
        write_namespaces: vec!["trigger.".to_string()],
        read_namespaces: vec![],
        // Subscribe to everything so the trigger engine sees every event.
        subscribe_namespaces: vec!["".to_string()],
    };
    let mut router = kernel.lock().await;
    crate::services::spawn_service(&mut router, "service.triggers", config, service)
        .map_err(|e| anyhow!("failed to spawn trigger service: {e}"))?;
    Ok(engine)
}
