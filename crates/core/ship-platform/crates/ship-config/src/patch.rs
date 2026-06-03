//! Patch conversion + snapshot builder for the config Watch resolver.
//!
//! Mirrors `ship-collection/src/patch.rs`: maps `ship_reactive` patch types to
//! the shared proto wire types, and builds the settings snapshot the resolver
//! diffs on each `config.changed` emission.

use ship_ops_rpc::{
    Patch as ProtoPatch, PatchFrame as ProtoPatchFrame, PatchOp as ProtoPatchOp, WatchEvent,
};

use crate::settings;

fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ProtoPatchOp {
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

/// Snapshot of all daemon settings as a JSON object `{ settings: { key: value } }`.
///
/// Re-run on each `config.changed` event; the resolver diffs successive
/// snapshots into patch frames. Defaults are merged in so a Watch subscriber
/// sees the same shape `List` returns.
pub(crate) async fn settings_snapshot() -> serde_json::Value {
    match settings::get_all_settings() {
        Ok(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map.into_iter().collect();
            serde_json::json!({ "settings": obj })
        }
        Err(_) => serde_json::json!({ "settings": {} }),
    }
}
