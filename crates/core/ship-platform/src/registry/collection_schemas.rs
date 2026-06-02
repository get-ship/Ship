//! File-overlay collection-schema loader (relocated from the deleted
//! `collections/schema.rs` in the C-2 cutover).
//!
//! This is the SCHEMA AUTHORING source: user schemas authored as
//! `.ship/collections/<id>.jsonc` files, composed across the local space plus
//! its locked deps. It is a registry/compose concern — the file overlay and the
//! DataStore-backed `SchemaRegistry` (in `ship-collection`) are two faces of the
//! same schema set. The daemon hydrates the runtime registry from this overlay
//! at startup so file-authored schemas are queryable.
//!
//! Loaders parse straight into the new `ship_collection::types::SchemaDoc`
//! (`name`→`title` via the type's serde alias). Legacy per-field keys
//! (`filterable`, `default`, `items`, `properties`) are simply ignored on
//! deserialize — they are not part of the new field model (C-2 Gap 3).

use std::path::Path;

use ship_collection::schema::parse_schema_doc;
use ship_collection::types::SchemaDoc;

/// Parse a single schema document from a JSONC string.
fn parse_jsonc(jsonc: &str) -> anyhow::Result<SchemaDoc> {
    let stripped = ship_substrate::fs_util::strip_jsonc_comments(jsonc);
    parse_schema_doc(stripped.as_bytes()).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Load all schemas from a `.ship/collections/` directory.
///
/// Reads from the composed overlay (local + dep-exported schemas) when the space
/// has deps; the raw `.ship/` otherwise (depless = identical).
pub fn load_schemas(ship_dir: &Path) -> anyhow::Result<Vec<SchemaDoc>> {
    let root = crate::registry::compose::composed_root_for(ship_dir);
    let schemas_dir = root.join("collections");
    if !schemas_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut schemas = Vec::new();
    for entry in std::fs::read_dir(&schemas_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonc" || e == "json") {
            let content = std::fs::read_to_string(&path)?;
            let schema = parse_jsonc(&content)
                .map_err(|e| anyhow::anyhow!("invalid schema {}: {e}", path.display()))?;
            schemas.push(schema);
        }
    }
    Ok(schemas)
}

/// Load schemas from both the global `~/.ship/collections/` and the project's
/// `.ship/collections/`, merging them. Order: global first, then project-local.
/// Project-local schemas with the same `id` as a global schema override (shadow)
/// the global one. A warning is emitted via `tracing::warn!` when this occurs.
///
/// If `project_ship_dir` is `None` or points to the same directory as
/// `global_ship_dir`, only global schemas are loaded.
pub fn load_schemas_merged(
    global_ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) -> anyhow::Result<Vec<SchemaDoc>> {
    let mut global_schemas = load_schemas(global_ship_dir)?;

    let project_dir = match project_ship_dir {
        Some(p) if p != global_ship_dir => p,
        _ => return Ok(global_schemas),
    };

    let project_schemas = load_schemas(project_dir)?;

    for proj in project_schemas {
        if let Some(pos) = global_schemas.iter().position(|g| g.id == proj.id) {
            tracing::warn!(
                id = %proj.id,
                "project-local collection schema shadows global schema with same id"
            );
            global_schemas[pos] = proj;
        } else {
            global_schemas.push(proj);
        }
    }

    Ok(global_schemas)
}

/// Hydrate a runtime `SchemaRegistry` from the file overlay so user-authored
/// `.ship/collections/*.jsonc` schemas are queryable alongside the built-ins.
/// Called once at daemon startup against the ONE shared registry the wire and
/// the internal callers hold. Built-in ids in the overlay are skipped (the
/// registry already seeds them); a duplicate user id shadows via `insert`.
pub fn hydrate_registry(
    registry: &ship_collection::projection::SchemaRegistry,
    global_ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) {
    match load_schemas_merged(global_ship_dir, project_ship_dir) {
        Ok(schemas) => {
            for doc in schemas {
                if ship_collection::types::RESERVED_IDS.contains(&doc.id.as_str()) {
                    continue;
                }
                registry.insert(doc);
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "collection schema overlay hydration failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_jsonc_maps_name_to_title_and_ignores_legacy_keys() {
        // Legacy `.jsonc`: `name` key + a field carrying the dropped
        // `filterable`/`default` keys. The loader must map name→title and
        // silently ignore the legacy per-field keys.
        let jsonc = r#"{
            // a user collection
            "id": "tasks",
            "name": "Tasks",
            "fields": {
                "title": { "type": "string", "required": true, "filterable": true },
                "status": { "type": "enum", "values": ["open", "done"], "default": "open" }
            }
        }"#;
        let doc = parse_jsonc(jsonc).expect("parses");
        assert_eq!(doc.id, "tasks");
        assert_eq!(doc.title.as_deref(), Some("Tasks"));
        assert_eq!(doc.fields.len(), 2);
        let status = doc.fields.get("status").expect("status field");
        assert_eq!(status.kind, ship_collection::types::FieldKind::Enum);
        assert_eq!(status.values.as_deref(), Some(&["open".to_string(), "done".to_string()][..]));
    }
}
