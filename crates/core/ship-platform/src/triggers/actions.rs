//! Action dispatchers — wire each [`HookActionType`] to the runtime primitive
//! that performs it (collections CRUD, event emission, pipeline trigger).

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::collections::schema::{CollectionSchema, load_schemas};
use crate::collections::{service as collection_service, store as collection_store};
use ship_substrate::data::DataService;
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use super::def::HookActionType;
use super::render;

/// Context the executor passes to each action dispatcher.
pub struct ActionCtx<'a> {
    pub kernel: &'a Arc<Mutex<KernelRouter>>,
    pub ship_dir: &'a Path,
    pub trigger_event_id: &'a str,
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

    let schema = load_schema(ctx.ship_dir, collection)?;
    let mut svc = open_docs(ctx.ship_dir)?;
    collection_store::ensure_collection_table(&mut svc, &schema)?;
    let id = ship_substrate::gen_ulid();
    let item = collection_service::create_item(
        &mut svc,
        ctx.kernel,
        &schema,
        &id,
        &data_value,
        Some(&format!("hook:{}", ctx.hook_name)),
    )
    .await?;
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
    let schema = load_schema(ctx.ship_dir, collection)?;
    let mut svc = open_docs(ctx.ship_dir)?;
    collection_store::ensure_collection_table(&mut svc, &schema)?;
    let id = resolve_filter_id(&filter_value)?;
    let item =
        collection_service::update_item(&mut svc, ctx.kernel, &schema, &id, &data_value).await?;
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
    let emit_ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: envelope.space_id.clone(),
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
    let emit_ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: None,
        session_id: None,
    };
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

fn open_docs(ship_dir: &Path) -> Result<DataService> {
    collection_store::open_data_db(ship_dir)
}

fn load_schema(ship_dir: &Path, collection: &str) -> Result<CollectionSchema> {
    let schemas = load_schemas(ship_dir)?;
    schemas
        .into_iter()
        .find(|s| s.id == collection)
        .ok_or_else(|| anyhow!("no schema for collection '{collection}'"))
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
