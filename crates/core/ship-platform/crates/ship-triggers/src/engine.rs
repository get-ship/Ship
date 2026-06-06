//! Hook engine — subscribes to the kernel, filters hooks, runs actions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use tokio::sync::{Mutex, RwLock};

use ship_substrate::events::{ActorConfig, EventEnvelope, KernelRouter};
use ship_substrate::services::ServiceHandler;

use super::actions::{ActionCtx, dispatch};
use super::def::{HookAction, HookDef};
use super::filter;
use super::render;
use super::sources::SourceManager;
use ship_config::resolve_hooks_max_depth;

// Historical hook chain depth (events emitted by a hook carry a depth
// marker; at depth ≥ this value, hooks are not executed). The runtime
// value is resolved from the `hooks.max_depth` KV setting (default 3) via
// `ship_config::resolve_hooks_max_depth`.

/// Per-hook / per-event context passed to each action while an action list
/// runs.
pub struct HookExecutionCtx {
    pub hook_name: String,
    pub depth: u32,
    pub event_id: String,
    pub results: Vec<Value>,
}

/// Shared state for the hook engine. Cloneable via `Arc` — the engine is
/// typically owned by one service task and multiple reloader paths.
pub struct HookEngine {
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    hooks: Arc<RwLock<HashMap<String, HookDef>>>,
    /// External source adapters. `None` when the engine is created without
    /// source support (e.g. in tests that only exercise the internal-event path).
    sources: Option<Arc<SourceManager>>,
    /// Shared collection schema registry. Defaults to a fresh registry seeded
    /// with built-ins; the daemon injects the runtime's SHARED registry via
    /// `with_schemas` so a wire-created schema is visible to a trigger insert
    /// (C-2 UD-7).
    schemas: Arc<ship_collection::projection::SchemaRegistry>,
}

impl HookEngine {
    pub fn new(kernel: Arc<Mutex<KernelRouter>>, ship_dir: PathBuf) -> Self {
        Self {
            kernel,
            ship_dir,
            hooks: Arc::new(RwLock::new(HashMap::new())),
            sources: None,
            schemas: Arc::new(ship_collection::projection::SchemaRegistry::new()),
        }
    }

    pub fn new_with_sources(
        kernel: Arc<Mutex<KernelRouter>>,
        ship_dir: PathBuf,
        sources: Arc<SourceManager>,
    ) -> Self {
        Self {
            kernel,
            ship_dir,
            hooks: Arc::new(RwLock::new(HashMap::new())),
            sources: Some(sources),
            schemas: Arc::new(ship_collection::projection::SchemaRegistry::new()),
        }
    }

    /// Inject the daemon's shared collection schema registry so trigger inserts
    /// see schemas created over the collection wire (C-2 UD-7).
    pub fn with_schemas(
        mut self,
        schemas: Arc<ship_collection::projection::SchemaRegistry>,
    ) -> Self {
        self.schemas = schemas;
        self
    }

    pub fn ship_dir(&self) -> &Path {
        &self.ship_dir
    }

    /// Access the webhook source (for wiring into the HTTP router).
    pub fn webhook_source(
        &self,
    ) -> Option<Arc<super::sources::webhook::WebhookSource>> {
        self.sources.as_ref().map(|s| s.webhook.clone())
    }

    /// Replace the registered hook set by rescanning the triggers directory.
    /// Also notifies all source adapters of the updated definitions.
    pub async fn reload(&self) {
        // Load triggers from the composed overlay (local + dep exports) when the
        // space has deps; raw `.ship/` otherwise. The watcher still watches the
        // local dir for hot-reload — dep triggers are static (from the cache).
        let root = ship_substrate::path_layout::composed_root_for(&self.ship_dir);
        let report = super::loader::load_triggers_dir(&root);
        let mut map = HashMap::with_capacity(report.hooks.len());
        for hook in report.hooks {
            map.insert(hook.name.clone(), hook);
        }
        tracing::info!(count = map.len(), errors = report.errors.len(), "hooks: reloaded");
        let defs: Vec<HookDef> = map.values().cloned().collect();
        *self.hooks.write().await = map;

        if let Some(sources) = &self.sources {
            sources.reload(&defs);
        }
    }

    /// Snapshot the hooks map (useful for tests).
    pub async fn snapshot(&self) -> Vec<HookDef> {
        self.hooks.read().await.values().cloned().collect()
    }

    /// Process one event — match every registered hook against it and run
    /// matching hooks sequentially. Errors are logged per-hook; they never
    /// prevent other hooks from running.
    pub async fn dispatch_event(&self, envelope: &EventEnvelope) {
        let depth = incoming_depth(envelope);
        let max_depth = resolve_hooks_max_depth();
        if depth >= max_depth {
            tracing::warn!(
                event_id = %envelope.id,
                depth,
                "hooks: recursion guard — skipping"
            );
            return;
        }
        let event_ctx = filter::build_event_context(envelope);
        let hooks = self.hooks.read().await;
        for hook in hooks.values() {
            let Some(expected_type) = hook.trigger.dispatch_event_type(&hook.name) else {
                continue;
            };
            if expected_type != envelope.event_type {
                continue;
            }
            if !filter::matches(hook.trigger.filter.as_ref(), &event_ctx) {
                continue;
            }
            if let Err(e) = self.run_hook(hook, envelope, &event_ctx, depth + 1).await {
                tracing::warn!(
                    hook = hook.name,
                    event_type = envelope.event_type,
                    "hooks: hook execution failed: {e}"
                );
            }
        }
    }

    async fn run_hook(
        &self,
        hook: &HookDef,
        envelope: &EventEnvelope,
        event_ctx: &Value,
        depth: u32,
    ) -> Result<()> {
        let mut exec = HookExecutionCtx {
            hook_name: hook.name.clone(),
            depth,
            event_id: envelope.id.clone(),
            results: Vec::new(),
        };
        for (idx, action) in hook.actions.iter().enumerate() {
            let render_ctx = build_render_ctx(event_ctx, &exec);
            if !should_run(action, &render_ctx)? {
                tracing::debug!(hook = hook.name, idx, "hooks: action skipped by when");
                exec.results.push(Value::Null);
                continue;
            }
            let action_ctx = ActionCtx {
                kernel: &self.kernel,
                ship_dir: &self.ship_dir,
                schemas: &self.schemas,
                trigger_event_id: &envelope.id,
                trigger_space: envelope.space_id(),
                hook_name: &hook.name,
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

fn should_run(action: &HookAction, render_ctx: &Value) -> Result<bool> {
    match &action.when {
        Some(expr) if !expr.trim().is_empty() => render::evaluate_when(expr, render_ctx),
        _ => Ok(true),
    }
}

fn build_render_ctx(event_ctx: &Value, exec: &HookExecutionCtx) -> Value {
    json!({
        "event": event_ctx,
        "workspace": event_ctx.get("workspace_id").cloned().unwrap_or(Value::Null),
        "actor": event_ctx.get("actor_id").cloned().unwrap_or(Value::Null),
        "hook": {
            "name": exec.hook_name,
            "depth": exec.depth,
        },
        "results": Value::Array(exec.results.clone()),
    })
}

fn incoming_depth(envelope: &EventEnvelope) -> u32 {
    let payload: Value = serde_json::from_str(&envelope.payload_json).unwrap_or(Value::Null);
    payload
        .get("_hook_depth")
        .and_then(|v| v.as_u64())
        .map(|d| d as u32)
        .unwrap_or(0)
}

/// Headless service wrapper — routes mailbox events into `HookEngine`.
///
/// The hook service subscribes to every namespace present in the loaded hook
/// set. On each mailbox event, it dispatches through the engine.
pub struct HookService {
    pub engine: Arc<HookEngine>,
    runtime: tokio::runtime::Handle,
}

impl HookService {
    pub fn new(engine: Arc<HookEngine>, runtime: tokio::runtime::Handle) -> Self {
        Self { engine, runtime }
    }
}

impl ServiceHandler for HookService {
    fn name(&self) -> &str {
        "hooks"
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &ship_substrate::events::ActorStore) -> Result<()> {
        let engine = self.engine.clone();
        let envelope = event.clone();
        self.runtime.spawn(async move {
            engine.dispatch_event(&envelope).await;
        });
        Ok(())
    }
}

/// Spawn the hook service on the kernel with source adapters enabled.
/// Reloads the hook set once before returning so hooks are live immediately.
///
/// `schemas` is the daemon's ONE shared collection schema registry (the same
/// `Arc` the collection wire is mounted with). Threading it here is what makes
/// a wire-created user schema visible to a trigger insert (C-2 UD-7).
pub async fn spawn_hook_service(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    schemas: Arc<ship_collection::projection::SchemaRegistry>,
) -> Result<Arc<HookEngine>> {
    let source_mgr = Arc::new(SourceManager::new(ship_dir.clone()));
    source_mgr.start_all(kernel.clone());

    let engine = Arc::new(
        HookEngine::new_with_sources(kernel.clone(), ship_dir, source_mgr)
            .with_schemas(schemas),
    );
    engine.reload().await;

    let runtime = tokio::runtime::Handle::current();
    let service = Box::new(HookService::new(engine.clone(), runtime));
    let config = ActorConfig {
        namespace: "service.hooks".to_string(),
        write_namespaces: vec!["hook.".to_string(), "external.".to_string()],
        read_namespaces: vec![],
        // Subscribe to everything so the hook engine sees every event.
        subscribe_namespaces: vec!["".to_string()],
        see_directed_events: false,
    };
    let mut router = kernel.lock().await;
    ship_substrate::services::spawn_service(&mut router, "service.hooks", config, service)
        .map_err(|e| anyhow!("failed to spawn hook service: {e}"))?;
    Ok(engine)
}
