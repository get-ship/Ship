//! Collection item commands — item CRUD over the platform `CollectionsService`
//! (`ctx.state.collections()`). Reads (get_item / query_items / schemas) are in
//! the read registry; schema mutations (create/update_schema) are a separate
//! piece that needs the full nested `CollectionSchema` typed model.
//!
//! Default workspace (C1): collection items are the user's own data. The open
//! `data` payload travels as a JSON string (`data_json`) in and out — the typed
//! `CollectionItem` envelope is real (id/collection/timestamps), the schema-
//! dynamic body is opaque, consistent with the rest of the open-payload surface.

use serde_json::Value;

use ship_platform::services::collections::service::{CreateItemArgs, Error, UpdateItemArgs};

use super::{req_str, CommandCtx, CommandError};

fn map_err(e: Error) -> CommandError {
    match e {
        Error::NotFound(m) => CommandError::not_found(m),
        Error::Conflict(m) | Error::Validation(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

fn req_data(args: &Value) -> Result<Value, CommandError> {
    let raw = req_str(args, "data_json")?;
    serde_json::from_str(&raw)
        .map_err(|e| CommandError::bad_args(format!("`data_json` is not valid JSON: {e}")))
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

/// Project a stored `Item` into the `CollectionItem` wire shape: the envelope is
/// flat-typed, the open `data` is re-serialized into the `data_json` string.
fn item_value(item: &ship_platform::collections::store::Item) -> Value {
    serde_json::json!({
        "id": item.id,
        "collection": item.collection,
        "data_json": item.data.to_string(),
        "created_by": item.created_by,
        "created_at": item.created_at,
        "updated_at": item.updated_at,
    })
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let svc = ctx.state.collections();
    match name {
        "collections.create_item" => {
            let collection = req_str(args, "collection")?;
            let id = match args.get("id").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => generate_item_id(),
            };
            let data = req_data(args)?;
            let item = svc
                .create_item(CreateItemArgs {
                    collection,
                    id,
                    data,
                    created_by: ctx.auth.agent_id.clone(),
                })
                .await
                .map_err(map_err)?;
            Ok(Some(item_value(&item)))
        }
        "collections.update_item" => {
            let collection = req_str(args, "collection")?;
            let id = req_str(args, "id")?;
            let data = req_data(args)?;
            let item = svc
                .update_item(UpdateItemArgs { collection, id, data })
                .await
                .map_err(map_err)?;
            Ok(Some(item_value(&item)))
        }
        "collections.delete_item" => {
            let collection = req_str(args, "collection")?;
            let id = req_str(args, "id")?;
            let deleted = svc.delete_item(&collection, &id).await.map_err(map_err)?;
            Ok(Some(serde_json::json!({ "id": id, "deleted": deleted })))
        }
        _ => Ok(None),
    }
}
