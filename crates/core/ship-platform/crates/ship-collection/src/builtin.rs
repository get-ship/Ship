//! Built-in collection schemas that ship with the runtime.
//!
//! notes — user-facing
//! feedback, annotations — system (system: true)

use indexmap::IndexMap;
use crate::types::{FieldDef, FieldKind, SchemaDoc};

fn field(kind: FieldKind) -> FieldDef {
    FieldDef { kind, required: false, values: None }
}

fn required(kind: FieldKind) -> FieldDef {
    FieldDef { kind, required: true, values: None }
}

fn enum_field(values: &[&str]) -> FieldDef {
    FieldDef {
        kind: FieldKind::Enum,
        required: false,
        values: Some(values.iter().map(|s| s.to_string()).collect()),
    }
}

fn req_enum(values: &[&str]) -> FieldDef {
    FieldDef {
        kind: FieldKind::Enum,
        required: true,
        values: Some(values.iter().map(|s| s.to_string()).collect()),
    }
}

pub fn notes_schema() -> SchemaDoc {
    let mut fields = IndexMap::new();
    fields.insert("title".into(), required(FieldKind::String));
    fields.insert("content".into(), field(FieldKind::Markdown));
    fields.insert("tags".into(), field(FieldKind::Array));
    fields.insert("category".into(), field(FieldKind::String));
    SchemaDoc { id: "notes".into(), title: Some("Notes".into()), system: false, fields }
}

pub fn feedback_schema() -> SchemaDoc {
    let mut fields = IndexMap::new();
    fields.insert(
        "target_type".into(),
        req_enum(&["pipeline_run", "agent_output", "collection_item", "file", "session"]),
    );
    fields.insert("target_id".into(), required(FieldKind::String));
    fields.insert("rating".into(), field(FieldKind::Number));
    fields.insert("annotation".into(), field(FieldKind::Markdown));
    fields.insert("tags".into(), field(FieldKind::Array));
    fields.insert("actor".into(), field(FieldKind::String));
    SchemaDoc { id: "feedback".into(), title: Some("Feedback".into()), system: true, fields }
}

pub fn annotations_schema() -> SchemaDoc {
    let mut fields = IndexMap::new();
    fields.insert("kind".into(), enum_field(&["point", "region", "action"]));
    fields.insert("view_id".into(), field(FieldKind::String));
    fields.insert("file_path".into(), field(FieldKind::String));
    fields.insert("selector".into(), field(FieldKind::String));
    fields.insert("intent".into(), field(FieldKind::String));
    fields.insert("note".into(), field(FieldKind::Markdown));
    fields.insert("payload".into(), required(FieldKind::String));
    fields.insert("author_kind".into(), field(FieldKind::String));
    SchemaDoc { id: "annotations".into(), title: Some("Annotations".into()), system: true, fields }
}

pub fn decisions_schema() -> SchemaDoc {
    let mut fields = IndexMap::new();
    fields.insert(
        "kind".into(),
        req_enum(&["correction", "operational", "mistake", "rejected", "tooling", "dependency", "product"]),
    );
    fields.insert("status".into(), req_enum(&["active", "superseded"]));
    fields.insert("title".into(), required(FieldKind::String));
    fields.insert("rationale".into(), field(FieldKind::Markdown));
    fields.insert("conviction".into(), enum_field(&["low", "medium", "high"]));
    fields.insert("tags".into(), field(FieldKind::Array));
    fields.insert("confirmations".into(), field(FieldKind::Number));
    fields.insert("supersedes".into(), field(FieldKind::String));
    fields.insert("superseded_by".into(), field(FieldKind::String));
    SchemaDoc { id: "decisions".into(), title: Some("Decisions".into()), system: true, fields }
}

/// All built-in schemas.
pub fn all_builtin_schemas() -> Vec<SchemaDoc> {
    vec![notes_schema(), feedback_schema(), annotations_schema(), decisions_schema()]
}
