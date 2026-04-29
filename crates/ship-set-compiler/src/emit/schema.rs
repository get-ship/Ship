//! Emit a JSON-serializable description of a file's directive
//! declarations: `<data>`, `<state>`, `<component>`/`<prop>`. Used by
//! tooling (HMR, error overlay, devserver) that needs a quick
//! semantic summary without the JS bundle.

use crate::ir::*;
use serde_json::{json, Value};

pub fn emit_schema(program: &Program) -> Value {
    let data: Vec<Value> = program
        .data
        .iter()
        .map(|d| json!({ "name": d.name, "source": d.source.source, "live": d.live }))
        .collect();
    let state: Vec<Value> = program
        .state
        .iter()
        .map(|s| json!({ "name": s.name, "value": s.value.source }))
        .collect();
    let component = program.component.as_ref().map(component_value);

    json!({ "data": data, "state": state, "component": component })
}

fn component_value(c: &ComponentDecl) -> Value {
    let props: Vec<Value> = c.props.iter().map(prop_value).collect();
    json!({ "tag": c.tag, "props": props })
}

fn prop_value(p: &PropDecl) -> Value {
    let kind = match &p.kind {
        PropKind::String => json!({ "type": "string" }),
        PropKind::Bool => json!({ "type": "bool" }),
        PropKind::Number => json!({ "type": "number" }),
        PropKind::Union(values) => json!({ "type": "union", "values": values }),
    };
    let mut entry = json!({
        "name": p.name,
        "default": p.default,
        "reflect": p.reflect,
        "bindable": p.bindable,
    });
    let merged = entry.as_object_mut().unwrap();
    for (k, v) in kind.as_object().unwrap() {
        merged.insert(k.clone(), v.clone());
    }
    Value::Object(merged.clone())
}
