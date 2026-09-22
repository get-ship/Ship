//! Action dispatchers — wire each [`HookActionType`] to the runtime primitive
//! that performs it (collections CRUD, event emission, pipeline trigger).

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use super::def::HookActionType;
use super::render;

/// Context the executor passes to each action dispatcher.
pub struct ActionCtx<'a> {
    pub kernel: &'a Arc<Mutex<KernelRouter>>,
    pub ship_dir: &'a Path,
    /// Shared schema registry — the SAME instance the wire + view-authoring
    /// hold, so a wire-created schema is visible to a trigger insert (C-2 UD-7).
    pub schemas: &'a Arc<SchemaRegistry>,
    pub trigger_event_id: &'a str,
    /// The space the triggering event resolved to (its `scope_key` when Space).
    /// Threaded onto the Space-scoped events this action emits
    /// (`collection.item_*`, `pipeline.run.started`) so they ride the trigger's
    /// space — mirroring the `event_emit` relay that carries `envelope.space_id()`.
    pub trigger_space: Option<&'a str>,
    pub hook_name: &'a str,
    pub depth: u32,
}

/// Dispatch one action. Returns the JSON result that gets appended to
/// `results` in the rendering context for subsequent actions.
pub async fn dispatch(
    action: &HookActionType,
    render_ctx: &Value,
    ctx: &ActionCtx<'_>,
) -> Result<Value> {
    match action {
        HookActionType::CollectionInsert {
            collection,
            data,
            template_file,
        } => collection_insert(collection, data, template_file.as_deref(), render_ctx, ctx).await,
        HookActionType::CollectionUpdate {
            collection,
            filter,
            data,
        } => collection_update(collection, filter, data, render_ctx, ctx).await,
        HookActionType::EventEmit {
            event_type,
            payload,
        } => event_emit(event_type, payload, render_ctx, ctx).await,
        HookActionType::PipelineTrigger { pipeline, args } => {
            pipeline_trigger(pipeline, args.as_ref(), render_ctx, ctx).await
        }
    }
}

async fn collection_insert(
    collection: &str,
    data: &Value,
    template_file: Option<&str>,
    render_ctx: &Value,
    ctx: &ActionCtx<'_>,
) -> Result<Value> {
    let data_value = if let Some(tf) = template_file {
        let path = ctx.ship_dir.join(tf);
        let template = std::fs::read_to_string(&path)
            .with_context(|| format!("reading template_file {}", path.display()))?;
        let rendered = render::render_string(&template, render_ctx)?;
        serde_json::from_str::<Value>(&rendered)
            .with_context(|| format!("template_file {} must render to JSON", path.display()))?
    } else {
        render::render_value(data, render_ctx)?
    };

    let svc = collection_service(ctx)?;
    ensure_table(&svc, collection).await?;
    let data_bytes = serde_json::to_vec(&data_value)
        .map_err(|e| anyhow!("collection.insert data is not serializable: {e}"))?;
    let item = svc
        .create_item(collection, &data_bytes)
        .await
        .map_err(|e| anyhow!("collection.insert failed: {e}"))?;
    Ok(json!({
        "collection": collection,
        "item_id": item.id,
        "data": item.data,
    }))
}

async fn collection_update(
    collection: &str,
    filter: &Value,
    data: &Value,
    render_ctx: &Value,
    ctx: &ActionCtx<'_>,
) -> Result<Value> {
    let filter_value = render::render_value(filter, render_ctx)?;
    let data_value = render::render_value(data, render_ctx)?;
    let id = resolve_filter_id(&filter_value)?;
    let svc = collection_service(ctx)?;
    ensure_table(&svc, collection).await?;
    let data_bytes = serde_json::to_vec(&data_value)
        .map_err(|e| anyhow!("collection.update data is not serializable: {e}"))?;
    let item = svc
        .update_item(collection, &id, &data_bytes)
        .await
        .map_err(|e| anyhow!("collection.update failed: {e}"))?;
    Ok(json!({
        "collection": collection,
        "item_id": item.id,
        "data": item.data,
    }))
}

async fn event_emit(
    event_type: &str,
    payload: &Value,
    render_ctx: &Value,
    ctx: &ActionCtx<'_>,
) -> Result<Value> {
    let rendered_payload = render::render_value(payload, render_ctx)?;
    let entity_id = render_ctx
        .get("event")
        .and_then(|e| e.get("entity_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("hook")
        .to_string();
    let mut envelope = EventEnvelope::new(event_type, &entity_id, &rendered_payload)?
        .with_causation(ctx.trigger_event_id)
        .with_actor_id(&format!("hook:{}", ctx.hook_name));
    // Propagate hook depth in payload? We store depth separately in an
    // internal registry — the envelope itself doesn't expose depth.
    // Relay: carry the envelope's own resolved space through the binding,
    // preserving its session. Daemon-internal mint (trusted relay).
    let emit_ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(
            envelope.space_id().map(ship_types::SpaceId::new),
            agent_schema::Authority::Sdk,
        ),
        session_id: envelope.session_id.clone(),
    };
    // Inject the hook depth so the engine sees a chain even when events
    // pass through the router out-of-process. Depth lives on the payload's
    // optional `_hook_depth` marker — a private convention.
    if let Value::Object(obj) = &mut envelope_payload_mut(&mut envelope)? {
        obj.insert("_hook_depth".into(), Value::from(ctx.depth));
        envelope.payload_json = serde_json::to_string(&Value::Object(obj.clone()))?;
    }
    ctx.kernel.lock().await.emit(envelope.clone(), &emit_ctx).await?;
    Ok(json!({
        "event_id": envelope.id,
        "event_type": event_type,
    }))
}

async fn pipeline_trigger(
    pipeline: &str,
    args: Option<&Value>,
    render_ctx: &Value,
    ctx: &ActionCtx<'_>,
) -> Result<Value> {
    let rendered_args = match args {
        Some(v) => render::render_value(v, render_ctx)?,
        None => Value::Null,
    };
    let pipeline_run_id = ship_substrate::gen_ulid();
    let slug = render_ctx
        .get("event")
        .and_then(|e| e.get("entity_id"))
        .and_then(|v| v.as_str())
        .map(|s| format!("hook-{}-{}", ctx.hook_name, s))
        .unwrap_or_else(|| format!("hook-{}-{}", ctx.hook_name, &pipeline_run_id));

    let vars = extract_vars(&rendered_args);
    let payload = json!({
        "pipeline_run_id": pipeline_run_id,
        "slug": slug,
        "agent": "runtime",
        "branch": format!("hook/{slug}"),
        "spec_path": format!(".artifacts/hooks/{slug}.md"),
        "plan_id": null,
        "model": null,
        "harness": null,
        "pipeline_name": pipeline,
        "vars": vars,
        "headless": true,
        "_hook_depth": ctx.depth,
    });
    let envelope = EventEnvelope::new("pipeline.run.started", &pipeline_run_id, &payload)?
        .with_causation(ctx.trigger_event_id)
        .with_actor_id(&format!("hook:{}", ctx.hook_name));
    // `pipeline.run.started` is Space-scoped: the run is space work that syncs to
    // collaborators. Bind the trigger-resolved space (same as the sibling
    // `event_emit` relay), not the keyless daemon mint.
    let emit_ctx = EmitContext::daemon_internal(ctx.trigger_space.map(ship_types::SpaceId::new));
    ctx.kernel.lock().await.emit(envelope.clone(), &emit_ctx).await?;
    Ok(json!({
        "pipeline_run_id": pipeline_run_id,
        "pipeline": pipeline,
        "event_id": envelope.id,
    }))
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn envelope_payload_mut(envelope: &mut EventEnvelope) -> Result<Value> {
    serde_json::from_str(&envelope.payload_json)
        .map_err(|e| anyhow!("event.emit payload is not JSON: {e}"))
}

/// Build a `CollectionService` over a `SpaceContext` bound to the trigger's
/// space (the per-scope `spaces/<id>/space.db` + shared kernel). The schema
/// registry is the shared one threaded through `ActionCtx` (C-2 UD-7) —
/// collection-table creation + schema validation happen inside
/// `create_item`/`update_item`.
fn collection_service(ctx: &ActionCtx<'_>) -> Result<CollectionService> {
     use ship_substrate::db::{ensure_events_db_at_path, space_db_path};
     use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};
     // A collection CRUD action is space-scoped. With no resolved trigger space
     // there is no active space to bind to — error cleanly rather than fabricate a
     // `global` partition (S2 home-space kill).
     let space_id = ctx.trigger_space.ok_or_else(|| {
         anyhow!("collection action requires a resolved trigger space; none active")
     })?;
     // Mirror the canonical SpaceContext::for_space: open the per-scope
     // spaces/<id>/space.db that the CollectionProjection wrote into.
     let space_db = space_db_path(ctx.ship_dir, space_id);
     ensure_events_db_at_path(&space_db)?;
     let data_svc = ship_substrate::data::DataService::open_at(&space_db)
         .map_err(|e| anyhow!("trigger collection action: open space.db: {e}"))?;
     let store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
     let kernel_log = Arc::new(KernelLog::new(ctx.kernel.clone()));
     let space_ctx = Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel_log));
     Ok(CollectionService { ctx: space_ctx, schemas: ctx.schemas.clone() })
 }

/// Ensure the collection's item table exists before a CRUD call. The new
/// `CollectionService::create_item`/`update_item` assume the table is present
/// (it is created by `create_schema`); file-authored / built-in collections
/// hit by a trigger may not have been created yet, so create it from the
/// registered schema here.
async fn ensure_table(svc: &CollectionService, collection: &str) -> Result<()> {
    let schema = svc
        .schemas
        .get(collection)
        .ok_or_else(|| anyhow!("no schema for collection '{collection}'"))?;
    ship_collection::projection::ensure_collection_table(svc.ctx.data(), &schema)
        .await
        .map_err(|e| anyhow!("ensure collection table '{collection}': {e}"))
}

fn resolve_filter_id(filter: &Value) -> Result<String> {
    // Only `{ "id": "..." }` is supported — everything else is rejected
    // because collection_service::update_item needs a concrete item id.
    let id = filter
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("collection.update filter must contain a string 'id'"))?;
    Ok(id.to_string())
}

fn extract_vars(args: &Value) -> Value {
    if let Value::Object(map) = args {
        let coerced: serde_json::Map<String, Value> = map
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(stringify(v))))
            .collect();
        Value::Object(coerced)
    } else {
        Value::Object(Default::default())
    }
}

fn stringify(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
