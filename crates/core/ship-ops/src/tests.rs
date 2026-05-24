//! Proves the unified model against a toy context: one descriptor + one registry
//! hold both a pure read and an async command, dispatched through one path.

use super::*;
use std::sync::Mutex;

/// Toy host context — stands in for the platform-service aggregate that the real
/// host (shipd's runtime) provides. Handlers reach their dependencies through it.
struct TestCtx {
    store: Mutex<Vec<String>>,
}

// A pure read: returns the current store contents. Non-capturing → coerces to fn.
fn list(_args: &Value, ctx: &TestCtx) -> Result<Value, OpError> {
    let items = ctx.store.lock().unwrap().clone();
    Ok(serde_json::json!({ "items": items }))
}

// An async command: pushes an item. Non-capturing closure → coerces to CommandFn.
const OPS: &[Op<TestCtx>] = &[
    Op {
        name: "toy.list",
        description: "List items.",
        args: &[],
        returns: "ListResponse",
        response_type: "ListResponse",
        blast: BlastRadius::C0Read,
        sensitive: false,
        handler: Handler::Read(list),
    },
    Op {
        name: "toy.add",
        description: "Add an item.",
        args: &[ArgDescriptor { name: "item", kind: "string", required: true }],
        returns: "Added",
        response_type: "Added",
        blast: BlastRadius::C1Workspace,
        sensitive: false,
        handler: Handler::Command(|args, ctx| {
            Box::pin(async move {
                let item = args
                    .get("item")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| OpError::BadArgs("item required".into()))?;
                ctx.store.lock().unwrap().push(item.to_string());
                Ok(serde_json::json!({ "ok": true }))
            })
        }),
    },
];

fn registry() -> Registry<TestCtx> {
    let mut r = Registry::new();
    r.mount(OPS).expect("mount");
    r
}

#[tokio::test]
async fn read_and_command_dispatch_through_one_path() {
    let ctx = TestCtx { store: Mutex::new(vec![]) };
    let reg = registry();

    // Command (async, effectful) and read (pure) go through the same get()→run().
    let add = reg.get("toy.add").expect("toy.add");
    add.run(&serde_json::json!({ "item": "alpha" }), &ctx).await.unwrap();

    let list_op = reg.get("toy.list").expect("toy.list");
    let out = list_op.run(&Value::Null, &ctx).await.unwrap();
    assert_eq!(out, serde_json::json!({ "items": ["alpha"] }));
}

#[test]
fn handler_variant_is_the_read_command_distinction() {
    let reg = registry();
    assert!(reg.get("toy.list").unwrap().is_read());
    assert!(!reg.get("toy.add").unwrap().is_read());
}

#[test]
fn blast_class_rides_on_the_descriptor() {
    let reg = registry();
    assert_eq!(reg.get("toy.list").unwrap().blast, BlastRadius::C0Read);
    assert_eq!(reg.get("toy.add").unwrap().blast, BlastRadius::C1Workspace);
}

#[tokio::test]
async fn bad_args_surface_as_operror() {
    let ctx = TestCtx { store: Mutex::new(vec![]) };
    let reg = registry();
    let err = reg.get("toy.add").unwrap().run(&Value::Null, &ctx).await.unwrap_err();
    assert_eq!(err, OpError::BadArgs("item required".into()));
}

#[test]
fn duplicate_mount_is_rejected() {
    let mut r = Registry::new();
    r.mount(OPS).unwrap();
    assert_eq!(r.mount(OPS), Err("toy.list"));
}

#[test]
fn blast_round_trips_through_descriptor_string() {
    for b in [BlastRadius::C0Read, BlastRadius::C1Workspace, BlastRadius::C2Elevated] {
        assert_eq!(BlastRadius::from_descriptor(b.as_descriptor()), b);
    }
    // Fail-closed: an unknown class is gated to the human.
    assert_eq!(BlastRadius::from_descriptor("nonsense"), BlastRadius::C2Elevated);
}

#[test]
fn meta_serializes_without_the_handler() {
    let reg = registry();
    let meta = reg.get("toy.add").unwrap().meta();
    let j = serde_json::to_value(meta).unwrap();
    assert_eq!(j["name"], "toy.add");
    assert_eq!(j["blast"], "C1_WORKSPACE");
    assert_eq!(j["read"], false);
    assert_eq!(j["response_type"], "Added");
}
