//! Phase 10 — JSON schema emission.
//!
//! Public schema for LSP, devtools, docs. The MVP shape mirrors
//! `docs/sail/02-compiler-architecture.md § Phase 10`:
//!
//! ```json
//! { "tag": "...", "version": "1.0", "props": {...}, "events": {...} }
//! ```
//!
//! Slot and dependency surfaces land alongside the analyzer.

use crate::parse::ast::SailFile;
use serde_json::{Map, Value, json};

pub fn emit(file: &SailFile, tag: &str) -> String {
    let mut props = Map::new();
    let mut events = Map::new();

    if let Some(s) = &file.script {
        for p in &s.props {
            let mut entry = Map::new();
            if let Some(ty) = &p.ty {
                entry.insert("type".into(), Value::String(ty.clone()));
            }
            if let Some(def) = &p.default {
                entry.insert("default".into(), Value::String(def.clone()));
            }
            entry.insert("required".into(), Value::Bool(p.required));
            props.insert(p.name.clone(), Value::Object(entry));
        }
        for e in &s.events {
            let mut entry = Map::new();
            if let Some(ty) = &e.payload_ty {
                entry.insert("payload".into(), Value::String(ty.clone()));
            }
            events.insert(e.name.clone(), Value::Object(entry));
        }
    }

    let v = json!({
        "tag": tag,
        "version": "1.0",
        "props": Value::Object(props),
        "events": Value::Object(events),
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn emits_props_and_events() {
        let src = "<script>prop label: string = 'x'\nevent select: { id: string }</script><template><div></div></template>";
        let file = parse::parse(src).unwrap();
        let s = emit(&file, "sail-x");
        assert!(s.contains("\"tag\": \"sail-x\""));
        assert!(s.contains("\"label\""));
        assert!(s.contains("\"select\""));
    }

    #[test]
    fn empty_schema_is_well_formed() {
        let file = parse::parse("<template><div/></template>").unwrap();
        let s = emit(&file, "sail-empty");
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["tag"], "sail-empty");
    }
}
