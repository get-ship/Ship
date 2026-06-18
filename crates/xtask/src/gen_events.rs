//! Generate tier-filtered event artifacts from the live `ship_types::event_registry`.
//! Runtime-scope events are excluded; Space + Account events flow to proto + TS.
//! Outputs: `proto/ship/ops/v1/events.gen.proto` + `packages/cloud-types/src/events.ts`.
//! Usage: `cargo xtask gen-events [--check]`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ship_types::event_registry::{EventScope, Retention, registry};

// ── Wire event record ─────────────────────────────────────────────────────────

struct WireEvent {
    name: &'static str,
    scope: EventScope,
    retention: Retention,
    payload_type: &'static str,
}

/// Payload structs that contain `serde_json::Value`, generic `HashMap`, or
/// complex nested Rust types that cannot be mapped field-by-field to proto
/// scalar types. These are emitted as `bytes payload_json = 1` rather than
/// expanding individual fields.
const COMPLEX_PAYLOADS: &[&str] = &[
    "ArtifactUpdated",            // changed: serde_json::Value
    "PipelineRunStartedPayload",  // steps: Option<Pipeline>, pipeline: Option<Vec<PipelinePhase>>, vars: HashMap, step_specs: Vec<String>
    "PipelineRunDispatchedPayload", // harness_meta: Option<serde_json::Value>
    "AgentSessionStartedPayload", // vars: Option<HashMap<String,String>>, depends_on: Option<Vec<String>>
];

/// Collect and sort wire-tier events from the live inventory registry.
fn collect_wire_events() -> Vec<WireEvent> {
    let mut events: Vec<WireEvent> = registry()
        .filter(|d| matches!(d.primary_scope(), EventScope::Space | EventScope::Account))
        .map(|d| WireEvent {
            name: d.name,
            scope: d.primary_scope(),
            retention: d.retention,
            payload_type: d.payload_type_name,
        })
        .collect();
    // Sort alphabetically by name for stable, idempotent output.
    events.sort_by_key(|e| e.name);
    events
}

// ── Field parsing from Rust source files ─────────────────────────────────────

#[derive(Debug)]
struct StructField {
    name: String,
    rust_type: String,
}

/// Parse `pub struct <Name> { ... }` blocks from a Rust source file.
/// Returns a map from struct name → list of fields.
/// Only parses top-level `pub` fields; skips cfg_attr, comments, doc lines.
fn parse_structs(src: &str) -> BTreeMap<String, Vec<StructField>> {
    let mut result: BTreeMap<String, Vec<StructField>> = BTreeMap::new();
    let mut current: Option<(String, Vec<StructField>)> = None;
    let mut brace_depth: i32 = 0;

    for line in src.lines() {
        let trimmed = line.trim();
        // Start of a struct definition.
        if let Some(rest) = trimmed.strip_prefix("pub struct ") {
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('{')
                .trim()
                .to_string();
            if !name.is_empty() {
                current = Some((name, Vec::new()));
                brace_depth = 0;
            }
        }
        // Track brace depth.
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }
        // Inside a struct body (depth == 1).
        if brace_depth == 1 {
            if let Some((_, ref mut fields)) = current {
                // Match `pub field_name: Type,` or `pub field_name: Type`
                if let Some(rest) = trimmed.strip_prefix("pub ") {
                    if let Some(colon_pos) = rest.find(':') {
                        let fname = rest[..colon_pos].trim().to_string();
                        let ftype = rest[colon_pos + 1..]
                            .trim()
                            .trim_end_matches(',')
                            .trim()
                            .to_string();
                        // Skip reserved keywords and skip lines that look like trait/fn/mod
                        if !fname.is_empty()
                            && !fname.contains(' ')
                            && !ftype.starts_with("fn ")
                        {
                            fields.push(StructField {
                                name: fname,
                                rust_type: ftype,
                            });
                        }
                    }
                }
            }
        }
        // End of struct.
        if brace_depth <= 0 {
            if let Some((name, fields)) = current.take() {
                if !fields.is_empty() || brace_depth == 0 {
                    result.insert(name, fields);
                }
            }
            brace_depth = 0;
        }
    }
    result
}

// ── Rust type → proto scalar type mapping ────────────────────────────────────

fn rust_type_to_proto(rust_type: &str) -> &'static str {
    let inner = rust_type
        .trim_start_matches("Option<")
        .trim_end_matches('>')
        .trim();
    match inner {
        "String" | "&str" | "&'static str" => "string",
        "bool" => "bool",
        "u16" | "u32" | "u64" | "i32" | "i64" | "usize" => "int64",
        _ => "bytes", // serde_json::Value, HashMap, Vec<T>, custom types → opaque JSON blob
    }
}

/// Repeated types map to `repeated string`.
fn is_vec_of_string(rust_type: &str) -> bool {
    rust_type == "Vec<String>"
}

fn proto_field_for(field: &StructField, field_num: usize) -> String {
    let ty = if is_vec_of_string(&field.rust_type) {
        "repeated string"
    } else {
        rust_type_to_proto(&field.rust_type)
    };
    format!("  {ty} {} = {};\n", field.name, field_num)
}

// ── Dot-name → PascalCase conversion ─────────────────────────────────────────

fn pascal(name: &str) -> String {
    name.split(['.', '_', '-'])
        .map(|seg| {
            let mut c = seg.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

/// Proto enum variant name from dot-name: `agent.session.started` → `AGENT_SESSION_STARTED`
fn enum_variant(name: &str) -> String {
    name.replace('.', "_").to_uppercase()
}

/// Proto oneof field name from dot-name: `agent.session.started` → `agent_session_started`
fn oneof_field(name: &str) -> String {
    name.replace('.', "_")
}

// ── Proto generation ──────────────────────────────────────────────────────────

fn render_proto(
    events: &[WireEvent],
    structs: &BTreeMap<String, Vec<StructField>>,
) -> String {
    let mut s = String::new();
    s.push_str(
        "// @generated by `cargo xtask gen-events` from ship_types::event_registry.\n\
         // Tier filter: Space + Account scope only. Runtime-scope events are NOT present.\n\
         // Do not hand-edit — re-run the generator (`cargo xtask gen-events`).\n\
         syntax = \"proto3\";\n\n\
         package ship.events.v1;\n\n\
         // Enum of all wire-tier event types (Space + Account scope only).\n\
         enum EventType {\n\
           EVENT_TYPE_UNSPECIFIED = 0;\n",
    );
    for (i, ev) in events.iter().enumerate() {
        let variant = enum_variant(ev.name);
        let _ = write!(s, "  EVENT_TYPE_{variant} = {};\n", i + 1);
    }
    s.push_str("}\n\n");

    // EventPayload oneof
    s.push_str(
        "// Tagged union of all wire-tier event payloads.\n\
         message EventPayload {\n\
           oneof payload {\n",
    );
    for (i, ev) in events.iter().enumerate() {
        let msg_name = format!("{}Msg", pascal(ev.name));
        let field = oneof_field(ev.name);
        let _ = write!(s, "    {msg_name} {field} = {};\n", i + 1);
    }
    s.push_str("  }\n}\n\n");

    // Per-event payload messages
    for ev in events {
        let msg_name = format!("{}Msg", pascal(ev.name));
        let _ = write!(s, "// {scope} / {retention} — {name}\n",
            scope = if matches!(ev.scope, EventScope::Account) { "account" } else { "space" },
            retention = if matches!(ev.retention, Retention::Durable) { "durable" } else { "ephemeral" },
            name = ev.name,
        );
        let _ = write!(s, "message {msg_name} {{\n");

        if COMPLEX_PAYLOADS.contains(&ev.payload_type) {
            // Opaque JSON blob for complex/generic payload structs.
            s.push_str("  bytes payload_json = 1;\n");
        } else if let Some(fields) = structs.get(ev.payload_type) {
            if fields.is_empty() {
                // Empty struct — no fields needed.
            } else {
                for (i, field) in fields.iter().enumerate() {
                    s.push_str(&proto_field_for(field, i + 1));
                }
            }
        } else {
            // Payload type not found in parsed sources — emit opaque blob.
            s.push_str("  bytes payload_json = 1;\n");
        }
        s.push_str("}\n\n");
    }
    s
}

// ── TS generation ─────────────────────────────────────────────────────────────

fn scope_str(scope: &EventScope) -> &'static str {
    scope.as_str()
}

fn retention_str(retention: &Retention) -> &'static str {
    match retention {
        Retention::Durable => "durable",
        Retention::Ephemeral => "ephemeral",
    }
}

fn render_ts(events: &[WireEvent]) -> String {
    let mut s = String::new();
    s.push_str(
        "// @generated by `cargo xtask gen-events` from ship_types::event_registry.\n\
         // Tier filter: Space + Account scope only. Runtime-scope events are NOT present.\n\
         // Do not hand-edit — re-run the generator (`cargo xtask gen-events`).\n\n\
         // Discriminated union of all wire-tier event payloads.\n\
         export type WireEvent =\n",
    );
    for (i, ev) in events.iter().enumerate() {
        let sep = if i + 1 == events.len() { ";" } else { "" };
        let _ = write!(
            s,
            "  | {{ type: \"{name}\"; scope: \"{scope}\"; retention: \"{retention}\"; payload: Record<string, unknown> }}{sep}\n",
            name = ev.name,
            scope = scope_str(&ev.scope),
            retention = retention_str(&ev.retention),
        );
    }
    s.push('\n');

    // WIRE_EVENT_TYPES const
    s.push_str("// All wire-tier event type names (sorted). Runtime-safe const.\n");
    s.push_str("export const WIRE_EVENT_TYPES = [\n");
    for ev in events {
        let _ = write!(s, "  \"{}\",\n", ev.name);
    }
    s.push_str("] as const;\n\n");

    // Type alias for the union of event type name literals
    s.push_str(
        "export type WireEventType = (typeof WIRE_EVENT_TYPES)[number];\n\n",
    );

    // Per-scope sub-arrays for convenience
    let space_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.scope, EventScope::Space))
        .collect();
    let account_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.scope, EventScope::Account))
        .collect();

    s.push_str("// Space-scoped events (sync within a space / project).\n");
    s.push_str("export const SPACE_EVENT_TYPES = [\n");
    for ev in &space_events {
        let _ = write!(s, "  \"{}\",\n", ev.name);
    }
    s.push_str("] as const;\n\n");

    s.push_str("// Account-scoped events (sync to cloud).\n");
    s.push_str("export const ACCOUNT_EVENT_TYPES = [\n");
    for ev in &account_events {
        let _ = write!(s, "  \"{}\",\n", ev.name);
    }
    s.push_str("] as const;\n");

    s
}

// ── Source file parsing ───────────────────────────────────────────────────────

fn load_structs(root: &Path) -> Result<BTreeMap<String, Vec<StructField>>> {
    let sources = &[
        "crates/core/ship-substrate/crates/ship-types/src/payload.rs",
        "crates/core/ship-substrate/src/events/agent_session.rs",
        "crates/core/ship-substrate/src/events/pipeline.rs",
    ];
    let mut all: BTreeMap<String, Vec<StructField>> = BTreeMap::new();
    for rel in sources {
        let path = root.join(rel);
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let parsed = parse_structs(&src);
        for (name, fields) in parsed {
            all.entry(name).or_insert(fields);
        }
    }
    Ok(all)
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(check: bool) -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let events = collect_wire_events();
    let structs = load_structs(&root)?;

    let space_count = events.iter().filter(|e| matches!(e.scope, EventScope::Space)).count();
    let account_count = events.iter().filter(|e| matches!(e.scope, EventScope::Account)).count();
    let wire_count = events.len();

    let proto_content = render_proto(&events, &structs);
    let ts_content = render_ts(&events);

    let proto_path = root.join("proto/ship/ops/v1/events.gen.proto");
    let ts_path = root.join("packages/cloud-types/src/events.ts");

    if check {
        let mut stale = false;
        for (path, expected) in [
            (&proto_path, &proto_content),
            (&ts_path, &ts_content),
        ] {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing != *expected {
                eprintln!("gen-events: out of date: {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-events` and commit the result.");
            std::process::exit(1);
        }
        eprintln!(
            "gen-events: up-to-date ({wire_count} wire events: {space_count} space + {account_count} account)."
        );
        return Ok(());
    }

    std::fs::write(&proto_path, &proto_content)
        .with_context(|| format!("writing {}", proto_path.display()))?;
    std::fs::write(&ts_path, &ts_content)
        .with_context(|| format!("writing {}", ts_path.display()))?;

    eprintln!(
        "gen-events: wrote {wire_count} wire events ({space_count} space + {account_count} account) → proto + TS.",
    );
    eprintln!("  proto: {}", proto_path.display());
    eprintln!("  ts:    {}", ts_path.display());
    Ok(())
}
