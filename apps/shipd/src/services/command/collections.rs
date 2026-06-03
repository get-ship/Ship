//! Collection commands over the `ship-collection` service
//! (`ctx.state.collections()`). Item CRUD + schema CRUD. Reads (list / detail /
//! items) are in the read registry.
//!
//! Item bodies + the full nested schema cross the wire as JSON strings
//! (`data_json` / `schema_json`) and are deserialized at the boundary. Schema
//! writes go through the DataStore-backed service (C-2 Gap 4) — no file write.

use serde_json::Value;

use ship_collection::error::CollectionError;

use super::{req_str, CommandCtx, CommandError};

fn map_err(e: CollectionError) -> CommandError {
    match e {
        CollectionError::NotFound(m) => CommandError::not_found(m),
        CollectionError::ReservedId(m)
        | CollectionError::Validation(m)
        | CollectionError::UnsupportedQuery(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

fn req_data_bytes(args: &Value) -> Result<Vec<u8>, CommandError> {
    let raw = req_str(args, "data_json")?;
    // Validate it parses as JSON at the boundary; pass the bytes through.
    serde_json::from_str::<Value>(&raw)
        .map_err(|e| CommandError::bad_args(format!("`data_json` is not valid JSON: {e}")))?;
    Ok(raw.into_bytes())
}

/// Time-ordered fallback id when the caller doesn't supply one (mirrors the
/// REST handler's `generate_item_id`).
fn generate_item_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).ok();
    format!("{ts:x}-{:08x}", u32::from_le_bytes(buf))
}

/// Project a stored `ItemRecord` into the `CollectionItem` wire shape: the
/// envelope is flat-typed, the open `data` is re-serialized into `data_json`.
fn item_value(item: &ship_collection::types::ItemRecord) -> Value {
    serde_json::json!({
        "id": item.id,
        "collection": item.collection,
        "data_json": serde_json::Value::Object(item.data.clone()).to_string(),
        "created_at": item.created_at,
        "updated_at": item.updated_at,
    })
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let svc = ctx
        .state
        .collections()
        .map_err(|e| CommandError::service(e.to_string()))?;
    match name {
        "collections.create_item" => {
            let collection = req_str(args, "collection")?;
            // `id` is no longer caller-assignable — the service mints a ULID.
            // The legacy fallback id is preserved only as a no-op compat read.
            let _ = match args.get("id").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => generate_item_id(),
            };
            let data = req_data_bytes(args)?;
            let item = svc.create_item(&collection, &data).await.map_err(map_err)?;
            Ok(Some(item_value(&item)))
        }
        "collections.update_item" => {
            let collection = req_str(args, "collection")?;
            let id = req_str(args, "id")?;
            let data = req_data_bytes(args)?;
            let item = svc.update_item(&collection, &id, &data).await.map_err(map_err)?;
            Ok(Some(item_value(&item)))
        }
        "collections.delete_item" => {
            let collection = req_str(args, "collection")?;
            let id = req_str(args, "id")?;
            svc.delete_item(&collection, &id).await.map_err(map_err)?;
            Ok(Some(serde_json::json!({ "id": id, "deleted": true })))
        }
        "collections.create_schema" => {
            let schema_json = req_str(args, "schema_json")?;
            // Parse to recover the id; the service re-parses + persists.
            let doc = ship_collection::schema::parse_schema_doc(schema_json.as_bytes())
                .map_err(map_err)?;
            let id = doc.id.clone();
            svc.create_schema(&id, schema_json.as_bytes()).await.map_err(map_err)?;
            Ok(Some(serde_json::json!({ "ok": true, "collection": id })))
        }
        "collections.update_schema" => {
            let collection = req_str(args, "collection")?;
            let schema_json = req_str(args, "schema_json")?;
            svc.update_schema(&collection, schema_json.as_bytes()).await.map_err(map_err)?;
            Ok(Some(serde_json::json!({ "ok": true, "collection": collection })))
        }
        _ => Ok(None),
    }
}
