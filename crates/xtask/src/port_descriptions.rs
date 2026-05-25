//! ONE-TIME porter: move op descriptions from the Rust registry
//! (`schema::OPS`/`COMMANDS`) into the per-domain protos as
//! `option (ship.ops.v1.description) = "…"`. After this, the proto carries the
//! prose (the SSOT, decision #67a) and the registry's `description` field has no
//! reader left. Run once: `cargo xtask port-descriptions`. protoc (the build)
//! is the safety net — a malformed injection fails the descriptor compile.
//!
//! Skips the `*.gen.proto` umbrella (generated scaffold, retiring with #78) and
//! the non-service files. Idempotent at file granularity: a proto that already
//! declares any `description` option is left untouched.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// `RulesService` + `List` → `rules.list` (matches the registry op-name key).
fn op_name(service: &str, method: &str) -> String {
    let svc = snake(service);
    let domain = svc.strip_suffix("_service").unwrap_or(&svc);
    format!("{domain}.{}", snake(method))
}

/// op name → one-line description, from the registry (both read + command tables).
fn descriptions() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for op in shipd::services::schema::OPS.iter() {
        m.insert(op.name.to_string(), op.description.to_string());
    }
    for cmd in shipd::services::schema::COMMANDS {
        m.insert(cmd.name.to_string(), cmd.description.to_string());
    }
    m
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn run() -> Result<()> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proto/ship/ops/v1");
    let descs = descriptions();
    let mut injected = 0usize;
    let mut missing: Vec<String> = Vec::new();

    let mut files: Vec<_> = std::fs::read_dir(&proto_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let n = p.file_name().unwrap_or_default().to_string_lossy();
            n.ends_with(".proto")
                && !n.ends_with(".gen.proto")
                && !matches!(&*n, "policy.proto" | "types.proto" | "ops.proto" | "stream.proto")
        })
        .collect();
    files.sort();

    for path in files {
        let src = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        // File-level idempotency: never double-inject.
        if src.contains("ship.ops.v1.description") {
            continue;
        }
        let mut out = String::new();
        let mut service = String::new();
        let mut changed = false;
        for line in src.lines() {
            out.push_str(line);
            out.push('\n');
            let trimmed = line.trim();
            if let Some(svc) = trimmed.strip_prefix("service ").and_then(|s| s.split_whitespace().next()) {
                service = svc.to_string();
            }
            // An rpc that OPENS an option block — inject its description first.
            if let Some(rest) = trimmed.strip_prefix("rpc ")
                && trimmed.ends_with('{')
                && !service.is_empty()
            {
                let method = rest.split('(').next().unwrap_or("").trim();
                let op = op_name(&service, method);
                match descs.get(&op) {
                    Some(d) if !d.is_empty() => {
                        out.push_str(&format!(
                            "    option (ship.ops.v1.description) = \"{}\";\n",
                            escape(d)
                        ));
                        injected += 1;
                        changed = true;
                    }
                    _ => missing.push(format!("{service}.{method} (op `{op}`)")),
                }
            }
        }
        if changed {
            std::fs::write(&path, out).with_context(|| format!("write {}", path.display()))?;
        }
    }

    eprintln!("port-descriptions: injected {injected} `description` options into per-domain protos");
    if !missing.is_empty() {
        eprintln!(
            "port-descriptions: {} rpc(s) had no registry description (left blank):\n    {}",
            missing.len(),
            missing.join("\n    ")
        );
    }
    Ok(())
}
