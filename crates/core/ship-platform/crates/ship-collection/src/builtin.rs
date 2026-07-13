//! Built-in collection schemas that ship with the runtime.
//!
//! notes — user-facing

use indexmap::IndexMap;
use crate::types::{FieldDef, FieldKind, SchemaDoc};

fn field(kind: FieldKind) -> FieldDef {
    FieldDef { kind, required: false, values: None }
}

fn required(kind: FieldKind) -> FieldDef {
    FieldDef { kind, required: true, values: None }
}

pub fn notes_schema() -> SchemaDoc {
    let mut fields = IndexMap::new();
    fields.insert("title".into(), required(FieldKind::String));
    fields.insert("content".into(), field(FieldKind::Markdown));
    fields.insert("tags".into(), field(FieldKind::Array));
    fields.insert("category".into(), field(FieldKind::String));
    SchemaDoc { id: "notes".into(), title: Some("Notes".into()), system: false, fields }
}

/// All built-in schemas.
pub fn all_builtin_schemas() -> Vec<SchemaDoc> {
    vec![notes_schema()]
}
