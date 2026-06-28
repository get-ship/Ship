//! Patch conversion helpers — maps ship_reactive patch types to proto wire types,
//! and snapshot builders for the Watch resolver.

use std::sync::Arc;

use ship_ops_rpc::{
    Patch as ProtoPatch, PatchFrame as ProtoPatchFrame, PatchOp as ProtoPatchOp, WatchEvent,
};
use ship_substrate_core::{Query, SpaceContext};

use crate::projection::SchemaRegistry;

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

pub(crate) async fn snapshot_for_scope(
    ctx: Arc<SpaceContext>,
    schemas: Arc<SchemaRegistry>,
    scope: &str,
) -> serde_json::Value {
    if scope == "collections" {
        let list: Vec<serde_json::Value> = schemas
            .list()
            .iter()
            .map(|s| serde_json::json!({ "id": s.id, "title": s.title, "system": s.system }))
            .collect();
        return serde_json::json!({ "collections": list });
    }
    if let Some(id) = scope.strip_prefix("collection:") {
        let q = Query { table: format!("collection_{id}"), limit: 200, ..Default::default() };
        if let Ok(result) = ctx.data().query(q).await {
            let items: Vec<serde_json::Value> = result
                .rows
                .into_iter()
                .map(|r| serde_json::json!({ "id": r.id, "data": r.fields }))
                .collect();
            return serde_json::json!({ "items": items });
        }
    }
    serde_json::json!({})
}
