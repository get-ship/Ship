//! Patch frame conversion and snapshot builder for the Watch resolver.

use std::sync::Arc;
use std::collections::BTreeMap;

use ship_ops_rpc::{
    Patch as ProtoPatch, PatchFrame as ProtoPatchFrame, PatchOp as ProtoPatchOp, WatchEvent,
};
use ship_substrate::ports::SpaceContext;

use crate::projection::{TABLE, ensure_artifacts_table, row_to_record};

pub(crate) fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ProtoPatchOp {
    use ship_reactive::PatchOp::*;
    match op {
        Set => ProtoPatchOp::PATCH_OP_SET,
        Merge => ProtoPatchOp::PATCH_OP_MERGE,
        Append => ProtoPatchOp::PATCH_OP_APPEND,
        Prepend => ProtoPatchOp::PATCH_OP_PREPEND,
        Insert => ProtoPatchOp::PATCH_OP_INSERT,
        Remove => ProtoPatchOp::PATCH_OP_REMOVE,
        Move => ProtoPatchOp::PATCH_OP_MOVE,
        Splice => ProtoPatchOp::PATCH_OP_SPLICE,
        Inc => ProtoPatchOp::PATCH_OP_INC,
    }
}

pub(crate) fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
    let patches: Vec<ProtoPatch> = frame
        .patches
        .into_iter()
        .map(|p| ProtoPatch {
            op: buffa::EnumValue::from(patch_op_to_proto(&p.op)),
            path: p.path,
            value: p.value.map(|v| serde_json::to_vec(&v).unwrap_or_default()),
            index: p.index.unwrap_or(0) as i32,
            from: p.from.unwrap_or(0) as i32,
            to: p.to.unwrap_or(0) as i32,
            count: p.count.unwrap_or(0) as i32,
            by: p.by.unwrap_or(0.0),
            v: 1,
            ..Default::default()
        })
        .collect();
    let proto_frame = ProtoPatchFrame { patches, ..Default::default() };
    WatchEvent {
        body: Some(ship_ops_rpc::__buffa::oneof::watch_event::Body::Patches(Box::new(proto_frame))),
        ..Default::default()
    }
}

/// Build the snapshot value for an artifacts Watch scope.
pub async fn snapshot_for_artifacts(
    ctx: Arc<SpaceContext>,
    scope_filter: &str,
    mime_filter: &str,
) -> serde_json::Value {
    let _ = ensure_artifacts_table(ctx.data()).await;

    let mut filter = BTreeMap::new();
    if !scope_filter.is_empty() {
        filter.insert("scope".into(), serde_json::Value::String(scope_filter.to_string()));
    }
    if !mime_filter.is_empty() {
        filter.insert("mime".into(), serde_json::Value::String(mime_filter.to_string()));
    }

    let q = ship_substrate::ports::Query {
        table: TABLE.to_string(),
        filter,
        limit: 500,
        ..Default::default()
    };

    let items: Vec<serde_json::Value> = ctx
        .data()
        .query(q)
        .await
        .map(|r| {
            r.rows
                .into_iter()
                .filter_map(|row| row_to_record(row).ok())
                .map(|rec| serde_json::json!({
                    "id": rec.id,
                    "name": rec.name,
                    "mime": rec.mime,
                    "size": rec.size,
                    "scope": rec.scope,
                    "created_at": rec.created_at,
                }))
                .collect()
        })
        .unwrap_or_default();

    serde_json::json!({ "items": items })
}
