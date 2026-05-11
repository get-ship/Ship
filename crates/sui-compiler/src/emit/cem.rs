#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Custom-elements-manifest entry. Conforms to
//! <https://github.com/webcomponents/custom-elements-manifest>.

use crate::ir::*;
use crate::CompileOpts;
use serde_json::{json, Value};

pub fn emit_cem(program: &Program, opts: &CompileOpts) -> Value {
    let meta = opts.component_meta.as_ref().expect("component_meta required for CEM");
    let tag = &meta.name;
    let path = opts
        .source_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{tag}.sui"));

    let attributes: Vec<Value> = program.props.iter().map(attr_entry).collect();
    let members: Vec<Value> = program.props.iter().map(member_entry).collect();

    let declaration = json!({
        "kind": "class",
        "name": class_name_for(tag),
        "tagName": tag,
        "customElement": true,
        "superclass": { "name": "HTMLElement", "package": "global" },
        "attributes": attributes,
        "members": members,
    });

    let module = json!({
        "kind": "javascript-module",
        "path": path,
        "declarations": [declaration],
        "exports": [{
            "kind": "custom-element-definition",
            "name": tag,
            "declaration": { "name": class_name_for(tag), "module": path },
        }],
    });

    json!({
        "schemaVersion": "1.0.0",
        "readme": "",
        "modules": [module],
    })
}

fn attr_entry(p: &PropDecl) -> Value {
    let mut e = json!({
        "name": p.name,
        "type": { "text": prop_type_text(&p.kind) },
    });
    if let Some(d) = p.default.as_ref() {
        e.as_object_mut().unwrap().insert("default".into(), Value::String(d.clone()));
    }
    if let Some(desc) = p.description.as_ref() {
        e.as_object_mut().unwrap().insert("description".into(), Value::String(desc.clone()));
    }
    if let PropKind::Union(values) = &p.kind {
        e.as_object_mut().unwrap().insert("values".into(), json!(values));
    }
    e
}

fn member_entry(p: &PropDecl) -> Value {
    let camel = camel_of(&p.name);
    let mut m = json!({
        "kind": "field",
        "name": camel,
        "type": { "text": prop_type_text(&p.kind) },
        "attribute": p.name,
    });
    if let Some(d) = p.default.as_ref() {
        m.as_object_mut().unwrap().insert("default".into(), Value::String(d.clone()));
    }
    if let Some(desc) = p.description.as_ref() {
        m.as_object_mut().unwrap().insert("description".into(), Value::String(desc.clone()));
    }
    if p.reflect {
        m.as_object_mut().unwrap().insert("reflects".into(), Value::Bool(true));
    }
    m
}

fn prop_type_text(k: &PropKind) -> String {
    match k {
        PropKind::String => "string".to_string(),
        PropKind::Bool => "boolean".to_string(),
        PropKind::Number => "number".to_string(),
        PropKind::Union(values) => values
            .iter()
            .map(|v| format!("'{v}'"))
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

fn class_name_for(tag: &str) -> String {
    let camel = camel_of(tag);
    let mut chars = camel.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn camel_of(attr: &str) -> String {
    let mut out = String::with_capacity(attr.len());
    let mut up_next = false;
    for c in attr.chars() {
        if c == '-' {
            up_next = true;
            continue;
        }
        if up_next {
            out.push(c.to_ascii_uppercase());
            up_next = false;
        } else {
            out.push(c);
        }
    }
    out
}
