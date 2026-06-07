//! Generate the typed TS `Database` interface from committed collection schemas.
//!
//! Reads `.ship/collections/<id>.jsonc` (parsed via the `ship_collection`
//! SSOT — never a re-implemented parser) and emits, for each collection:
//!   - a per-collection row type (`<Pascal>Row`)
//!   - an entry in the `Database` interface keyed by collection id
//!
//! Every row carries the system columns from ADR D4 (Relational Collections &
//! the Bounded Query Capability): `_id`, `_created`, `_updated`, `_actor`,
//! `_space` — present on every table, all auto-typed `string`.
//!
//! The emitted `Database` interface is the input to typed Kysely queries. This
//! generator's scope is the schema→types mapping only; the narrowed-Kysely
//! facade is a separate slice.
//!
//! Output: `packages/deck/sdk/src/generated/collections.ts` (the gitignored
//! codegen home alongside the other `gen-*` TS artifacts).
//! Usage: `cargo xtask gen-collection-types [--check]`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ship_collection::types::{FieldKind, SchemaDoc};

// ── Output path ────────────────────────────────────────────────────────────

const OUTPUT_PATH: &str = "packages/deck/sdk/src/generated/collections.ts";
const SCHEMAS_DIR: &str = ".ship/collections";

// ── System columns (ADR D4) ───────────────────────────────────────────────

/// `(column, ts_type)` for the auto-typed system columns present on every row.
const SYSTEM_COLUMNS: &[(&str, &str)] = &[
    ("_id", "string"),
    ("_created", "string"),
    ("_updated", "string"),
    ("_actor", "string"),
    ("_space", "string"),
];

// ── Public entry point ─────────────────────────────────────────────────────

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let schemas = load_schemas(&repo_root.join(SCHEMAS_DIR))?;
    let generated = render(&schemas);

    let out_path = repo_root.join(OUTPUT_PATH);
    if check {
        let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
        if existing != generated {
            eprintln!("Out of date: {}", out_path.display());
            eprintln!("Run `cargo xtask gen-collection-types` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("Collection types are up-to-date.");
        return Ok(());
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .context(format!("create_dir_all {}", parent.display()))?;
    }
    std::fs::write(&out_path, &generated)
        .context(format!("write {}", out_path.display()))?;
    eprintln!("Wrote {} ({} collections)", out_path.display(), schemas.len());
    Ok(())
}

// ── Schema loading ─────────────────────────────────────────────────────────

/// Load every `.jsonc`/`.json` schema in `dir`, parsed via the collection SSOT,
/// sorted by id for stable, idempotent output.
fn load_schemas(dir: &Path) -> Result<Vec<SchemaDoc>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .context(format!("read_dir {}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e == "jsonc" || e == "json")
        })
        .collect();
    paths.sort();

    let mut schemas = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(&path)
            .context(format!("read {}", path.display()))?;
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(&content);
        let doc = parse_doc(&stripped)
            .map_err(|e| anyhow::anyhow!("invalid schema {}: {e}", path.display()))?;
        schemas.push(doc);
    }
    schemas.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(schemas)
}

/// Parse stripped JSON into a `SchemaDoc` via the collection SSOT parser, after
/// normalizing the `json` field-type alias to `object`.
///
/// The field-type mapping treats `object` and `json` identically
/// (`Record<string, unknown>`), and ADR D4/D1 names a `Json` opt-out column —
/// but the `ship_collection::FieldKind` SSOT enum spells the free-form kind
/// `object`, not `json`. Some committed schemas author `"type": "json"`. We
/// canonicalize to the SSOT spelling here so parsing stays SSOT-driven and the
/// emitted TS is identical either way. This is a parse-time alias only; it does
/// not touch the projection/write path.
fn parse_doc(json: &str) -> Result<SchemaDoc> {
    let normalized = normalize_json_field_type(json);
    ship_collection::schema::parse_schema_doc(normalized.as_bytes())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Rewrite `"type": "json"` → `"type": "object"` in field definitions, leaving
/// the string-literal `"json"` untouched anywhere it is a value rather than a
/// declared field type.
fn normalize_json_field_type(json: &str) -> String {
    let mut value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        // Let the SSOT parser produce the real, precise error.
        Err(_) => return json.to_string(),
    };
    if let Some(fields) = value
        .get_mut("fields")
        .and_then(serde_json::Value::as_object_mut)
    {
        for field in fields.values_mut() {
            if let Some(obj) = field.as_object_mut() {
                if obj.get("type").and_then(serde_json::Value::as_str) == Some("json") {
                    obj.insert("type".into(), serde_json::Value::String("object".into()));
                }
            }
        }
    }
    value.to_string()
}

// ── TS rendering ───────────────────────────────────────────────────────────

const HEADER: &str =
    "// This file is auto-generated by `cargo xtask gen-collection-types`. Do not edit.\n\
     // Source: .ship/collections/*.jsonc — one Database entry per collection id.\n";

/// Render the full `collections.ts`: per-collection row types + `Database`.
pub fn render(schemas: &[SchemaDoc]) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');

    for doc in schemas {
        out.push_str(&render_row_type(doc));
        out.push('\n');
    }

    out.push_str(&render_database(schemas));
    out
}

/// `export interface <Pascal>Row { ... }` — system columns first, then fields.
fn render_row_type(doc: &SchemaDoc) -> String {
    let type_name = format!("{}Row", to_pascal_case(&doc.id));
    let mut out = format!("export interface {type_name} {{\n");

    for (col, ts) in SYSTEM_COLUMNS {
        out.push_str(&format!("  {col}: {ts};\n"));
    }

    for (name, field) in &doc.fields {
        let ts = field_to_ts(field.kind, field.values.as_deref());
        let optional = if field.required { "" } else { "?" };
        let key = ts_property_key(name);
        out.push_str(&format!("  {key}{optional}: {ts};\n"));
    }

    out.push_str("}\n");
    out
}

/// `export interface Database { "<id>": <Pascal>Row; ... }`.
fn render_database(schemas: &[SchemaDoc]) -> String {
    let mut out = String::from("export interface Database {\n");
    for doc in schemas {
        let type_name = format!("{}Row", to_pascal_case(&doc.id));
        out.push_str(&format!("  {}: {type_name};\n", ts_property_key(&doc.id)));
    }
    out.push_str("}\n");
    out
}

/// Map a field kind (+ enum values) to its TS type per the field-type table.
pub fn field_to_ts(kind: FieldKind, values: Option<&[String]>) -> String {
    match kind {
        FieldKind::String
        | FieldKind::Markdown
        | FieldKind::Date
        | FieldKind::File => "string".to_string(),
        FieldKind::Number => "number".to_string(),
        FieldKind::Boolean => "boolean".to_string(),
        FieldKind::Enum => match values {
            Some(vs) if !vs.is_empty() => vs
                .iter()
                .map(|v| format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(" | "),
            // Schema validation guarantees enum carries values; a defensive
            // fallback keeps the type sound rather than emitting invalid TS.
            _ => "string".to_string(),
        },
        FieldKind::Array => "string[]".to_string(),
        FieldKind::Object => "Record<string, unknown>".to_string(),
    }
}

// ── Identifier helpers ─────────────────────────────────────────────────────

/// Convert a collection id (`snake-case`/`snake_case`/`dotted`) to PascalCase.
fn to_pascal_case(id: &str) -> String {
    id.split(|c| c == '_' || c == '-' || c == '.')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Quote a TS property key when it is not a bare identifier (schema ids and
/// field names allow `-` and `.`, which are illegal bare keys).
fn ts_property_key(name: &str) -> String {
    let is_bare = !name.is_empty()
        && name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    if is_bare {
        name.to_string()
    } else {
        format!("\"{name}\"")
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(jsonc: &str) -> SchemaDoc {
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(jsonc);
        super::parse_doc(&stripped).expect("parses")
    }

    #[test]
    fn maps_scalar_field_kinds() {
        assert_eq!(field_to_ts(FieldKind::String, None), "string");
        assert_eq!(field_to_ts(FieldKind::Markdown, None), "string");
        assert_eq!(field_to_ts(FieldKind::Date, None), "string");
        assert_eq!(field_to_ts(FieldKind::File, None), "string");
        assert_eq!(field_to_ts(FieldKind::Number, None), "number");
        assert_eq!(field_to_ts(FieldKind::Boolean, None), "boolean");
        assert_eq!(field_to_ts(FieldKind::Array, None), "string[]");
        assert_eq!(
            field_to_ts(FieldKind::Object, None),
            "Record<string, unknown>"
        );
    }

    #[test]
    fn enum_becomes_string_literal_union() {
        let vals = vec!["spec".to_string(), "building".to_string(), "complete".to_string()];
        assert_eq!(
            field_to_ts(FieldKind::Enum, Some(&vals)),
            "\"spec\" | \"building\" | \"complete\""
        );
    }

    #[test]
    fn renders_representative_schema() {
        // Enum→union, array→string[], required vs optional, system columns.
        let doc = parse(
            r#"{
                "id": "tasks",
                "name": "Tasks",
                "fields": {
                    "title":   { "type": "string", "required": true },
                    "status":  { "type": "enum", "values": ["open", "done"] },
                    "tags":    { "type": "array" },
                    "weight":  { "type": "number" },
                    "done":    { "type": "boolean" },
                    "meta":    { "type": "object" }
                }
            }"#,
        );
        let ts = render(&[doc]);

        // System columns present on the row.
        assert!(ts.contains("_id: string;"), "missing _id\n{ts}");
        assert!(ts.contains("_created: string;"));
        assert!(ts.contains("_updated: string;"));
        assert!(ts.contains("_actor: string;"));
        assert!(ts.contains("_space: string;"));

        // Field mappings.
        assert!(ts.contains("title: string;"), "required field not non-optional\n{ts}");
        assert!(
            ts.contains("status?: \"open\" | \"done\";"),
            "enum→union missing\n{ts}"
        );
        assert!(ts.contains("tags?: string[];"), "array→string[] missing\n{ts}");
        assert!(ts.contains("weight?: number;"));
        assert!(ts.contains("done?: boolean;"));
        assert!(ts.contains("meta?: Record<string, unknown>;"));

        // Row type + Database wiring.
        assert!(ts.contains("export interface TasksRow {"), "row type name\n{ts}");
        assert!(ts.contains("export interface Database {"), "database iface\n{ts}");
        assert!(ts.contains("tasks: TasksRow;"), "database entry\n{ts}");
    }

    #[test]
    fn json_field_type_alias_maps_to_object() {
        // `"type": "json"` is an authoring alias for the SSOT `object` kind;
        // both render as `Record<string, unknown>`.
        let doc = parse(
            r#"{
                "id": "blobs",
                "name": "Blobs",
                "fields": {
                    "payload": { "type": "json" }
                }
            }"#,
        );
        assert_eq!(doc.fields.get("payload").unwrap().kind, FieldKind::Object);
        let ts = render(&[doc]);
        assert!(
            ts.contains("payload?: Record<string, unknown>;"),
            "json→object→unknown missing\n{ts}"
        );
    }

    #[test]
    fn quotes_non_bare_keys() {
        assert_eq!(ts_property_key("title"), "title");
        assert_eq!(ts_property_key("_id"), "_id");
        assert_eq!(ts_property_key("sui-eval-runs"), "\"sui-eval-runs\"");
        assert_eq!(ts_property_key("a.b"), "\"a.b\"");
    }

    #[test]
    fn pascal_case_handles_separators() {
        assert_eq!(to_pascal_case("systems"), "Systems");
        assert_eq!(to_pascal_case("sui-eval-runs"), "SuiEvalRuns");
        assert_eq!(to_pascal_case("architecture_observations"), "ArchitectureObservations");
    }
}
