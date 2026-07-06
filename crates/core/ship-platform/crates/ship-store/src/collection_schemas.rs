//! Collection-schema hydration from the descriptor (format-migration
//! Wave 4 phase A). Pre-W4: file overlay (`load_schemas_merged` walked
//! `.ship/collections/*.jsonc` and `hydrate_registry` called it on
//! boot). Post-W4: collection definitions arrive via
//! `defineCollection` declare-forms in the space's `ship.config.ts` and
//! land in `ExtensionLibrary::collections` (a `Vec<compiler::CollectionDef>`).
//! This module maps the descriptor's collection definitions onto the
//! runtime `SchemaRegistry`'s `SchemaDoc` shape and registers them.
//!
//! Built-in ids in the overlay are skipped (the registry already seeds
//! them); a duplicate user id shadows via `insert`. Called once at
//! daemon startup against the ONE shared registry the wire and the
//! internal callers hold.

use indexmap::IndexMap;
use ship_collection::types::{FieldDef, FieldKind, SchemaDoc};

/// Convert a descriptor's [`compiler::CollectionDef`] into a runtime
/// `SchemaDoc` suitable for the `SchemaRegistry`.
///
/// Maps each `CollectionFieldDef` to a `FieldDef` (kind from
/// `CollectionFieldType`, required/values carried over; the descriptor's
/// `default`/`filterable`/`sortable` are project-shape metadata, not
/// registry concerns, so they don't survive the conversion). The
/// `id` is the same on both sides; `name` (descriptor) → `title`
/// (registry, optional).
pub fn collection_def_to_schema_doc(def: &compiler::CollectionDef) -> SchemaDoc {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    for f in &def.fields {
        let kind = match f.field_type {
            compiler::types::CollectionFieldType::String => FieldKind::String,
            compiler::types::CollectionFieldType::Enum => FieldKind::Enum,
            compiler::types::CollectionFieldType::Markdown => FieldKind::Markdown,
            compiler::types::CollectionFieldType::Date => FieldKind::Date,
            compiler::types::CollectionFieldType::Number => FieldKind::Number,
            compiler::types::CollectionFieldType::Boolean => FieldKind::Boolean,
            compiler::types::CollectionFieldType::Array => FieldKind::Array,
            compiler::types::CollectionFieldType::Object => FieldKind::Object,
        };
        let values = if f.values.is_empty() {
            None
        } else {
            Some(f.values.clone())
        };
        fields.insert(
            f.name.clone(),
            FieldDef {
                kind,
                required: f.required,
                values,
            },
        );
    }
    SchemaDoc {
        id: def.id.clone(),
        title: def.name.clone(),
        system: false,
        fields,
    }
}

/// Hydrate a runtime `SchemaRegistry` from the descriptor's
/// `ExtensionLibrary::collections`. Built-in ids are skipped (the
/// registry already seeds them); a duplicate user id shadows via
/// `insert`.
pub fn hydrate_registry(
    registry: &ship_collection::projection::SchemaRegistry,
    collections: &[compiler::CollectionDef],
) {
    for def in collections {
        if ship_collection::types::RESERVED_IDS.contains(&def.id.as_str()) {
            continue;
        }
        let doc = collection_def_to_schema_doc(def);
        registry.insert(doc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compiler::types::{CollectionDef, CollectionFieldDef, CollectionFieldType};

    #[test]
    fn collection_def_maps_into_schema_doc() {
        let def = CollectionDef {
            id: "tasks".into(),
            name: Some("Tasks".into()),
            fields: vec![
                CollectionFieldDef {
                    name: "title".into(),
                    field_type: CollectionFieldType::String,
                    required: true,
                    values: vec![],
                    default: None,
                    filterable: None,
                    sortable: None,
                },
                CollectionFieldDef {
                    name: "status".into(),
                    field_type: CollectionFieldType::Enum,
                    required: false,
                    values: vec!["open".into(), "done".into()],
                    default: None,
                    filterable: None,
                    sortable: None,
                },
            ],
        };
        let doc = collection_def_to_schema_doc(&def);
        assert_eq!(doc.id, "tasks");
        assert_eq!(doc.title.as_deref(), Some("Tasks"));
        assert_eq!(doc.fields.len(), 2);
        let status = doc.fields.get("status").unwrap();
        assert_eq!(status.kind, FieldKind::Enum);
        assert_eq!(
            status.values.as_deref(),
            Some(&["open".to_string(), "done".to_string()][..])
        );
    }
}
