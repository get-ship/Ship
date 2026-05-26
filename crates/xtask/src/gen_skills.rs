//! Generate sdk-skills from the proto op surface (`cargo xtask gen-skills`).
//!
//! Skills-as-API with **progressive disclosure**: each capability domain (git,
//! pipelines, rules, …) becomes a tiny `SKILL.md` table-of-contents plus one
//! `references/docs/<op>.mdoc` page per op. The agent loads SKILL.md cheaply,
//! then opens only the single op page it needs — token-efficient by design. The
//! SKILL.md tells the agent to invoke each op via the Connect CLI
//! (`ship ops call <name> --json '{…}'`): the skill IS the API doc, `ship ops`
//! is the client, Connect is the transport.
//!
//! **doc-skill compatible**: the per-op pages carry `title:`/`description:`/
//! `group:`/`order:` frontmatter, so `apps/docs/scripts/collect-skill-docs.ts`
//! harvests the SAME files into the Astro documentation site (it scans
//! `docs/<skill>/references/docs/*.mdoc`). One generator, two consumers — the
//! agent's reference and the public API docs are the same source.
//!
//! Enumerated from the `ship.ops.v1` proto descriptor (the SSOT) — description,
//! args, return schema, policy all read from the FileDescriptorSet, never
//! hand-maintained. Output:
//! `skills/sdk/ship-<domain>/` — the `sdk` subdir of the skills tree holds the
//! generated sdk-skill kind, a peer to `skills/docs/` (doc-skills) and the
//! hand-authored smart-skills at `skills/`. Each file is stamped AUTO-GENERATED.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};

use ship_ops_rpc::policy::{
    MethodArg, domain_of, is_non_domain_service, is_operational, op_name, snake,
};

/// Output root, relative to the repo: the `sdk` subdir of the skills tree.
/// Separate from the hand-authored `docs/` doc-site source (which it must not
/// collide with) and from the smart-skills at `skills/`. The doc-site collector
/// is pointed here via its skills-root list so these pages still harvest.
const OUT_REL: &str = "skills/sdk";

struct OpRow {
    name: String,
    description: String,
    args: Vec<MethodArg>,
    /// Response message type (every op is typed under proto-SSOT).
    returns: String,
    /// Response message fields — the return schema (empty for opaque JSON).
    returns_fields: Vec<MethodArg>,
    /// Required capability (e.g. `ship.git.write`).
    permission: String,
    /// Gate class: "C0_READ" | "C1_WORKSPACE" | "C2_ELEVATED".
    blast: String,
    sensitive: bool,
    /// Proto service + method — used to locate this op's contract fixture
    /// (`proto/fixtures/<snake(service)>.json` → `[method].response_ok`).
    service: String,
    method: String,
}

impl OpRow {
    fn is_read(&self) -> bool {
        // C0 is the read tier; the proto descriptor string is `C0_SAFE`.
        self.blast == "C0_SAFE"
    }
}

/// A JSON placeholder value for an arg kind — seeds the `ship ops call` example.
fn placeholder(kind: &str) -> &'static str {
    if kind.ends_with("[]") {
        return "[]";
    }
    match kind.trim_end_matches('?') {
        "number" => "0",
        "bool" => "false",
        "map" => "{}",
        _ => "\"\"",
    }
}

/// Example args object: every request field with a kind-typed placeholder — the
/// full request shape (proto3 carries no required/optional distinction, so we
/// show all fields). Empty `{}` when the op takes no args.
fn example_json(args: &[MethodArg]) -> String {
    if args.is_empty() {
        return "{}".to_string();
    }
    let fields: Vec<String> = args
        .iter()
        .map(|a| format!("\"{}\": {}", a.name, placeholder(&a.kind)))
        .collect();
    format!("{{ {} }}", fields.join(", "))
}


/// Enumerate the agent-facing op surface FROM THE PROTO DESCRIPTOR (the SSOT) —
/// no Rust registry read. Operational ops (#95) and the umbrella scaffold are
/// excluded. Sorted by name for deterministic output.
fn collect() -> BTreeMap<String, Vec<OpRow>> {
    let mut by_domain: BTreeMap<String, Vec<OpRow>> = BTreeMap::new();
    for p in ship_ops_rpc::policy::read_policies() {
        if is_non_domain_service(&p.service) {
            continue;
        }
        let name = op_name(&p.service, &p.method);
        if is_operational(&name) {
            continue;
        }
        by_domain.entry(domain_of(&name).to_string()).or_default().push(OpRow {
            description: p.description,
            args: p.args,
            returns: p.output_type,
            returns_fields: p.returns_fields,
            permission: p.capability,
            blast: p.blast.as_descriptor().to_string(),
            sensitive: p.sensitive,
            service: p.service,
            method: p.method,
            name,
        });
    }
    for ops in by_domain.values_mut() {
        ops.sort_by(|a, b| a.name.cmp(&b.name));
    }
    by_domain
}

/// The token-efficient TOC. Frontmatter + a one-line entry per op linking to its
/// reference page — the agent reads this first, opens detail on demand.
fn render_skill_toc(domain: &str, ops: &[OpRow]) -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        "---\nname: ship-{domain}\nstable-id: ship-{domain}\ndescription: >-\n  The `{domain}` capability ({n} ops). Call any op over ConnectRPC with\n  `ship ops call <name> --json '<args>'`. This file is a table of contents;\n  open `references/docs/<op>.mdoc` for an op's args, return shape, and policy.\ntags: [sdk, api, {domain}]\nauthors: [ship]\n---\n\n",
        n = ops.len(),
    );
    let _ = write!(
        s,
        "<!-- AUTO-GENERATED by `cargo xtask gen-skills` from the proto op surface (ship.ops.v1). Do not edit. -->\n\n# ship-{domain}\n\nYour interface to the `{domain}` ops. Invoke each over ConnectRPC:\n\n```\nship ops call <name> --json '<args-json>'\n```\n\nReads are safe (C0). Commands carry a blast-radius class. Open an op's reference\npage for full args, return shape, capability, and an example.\n\n",
    );
    let (reads, cmds): (Vec<&OpRow>, Vec<&OpRow>) = ops.iter().partition(|o| o.is_read());
    if !reads.is_empty() {
        s.push_str("## Reads\n\n");
        for r in &reads {
            let _ = write!(s, "- [`{}`](references/docs/{}.mdoc) — {}\n", r.name, r.name, r.description);
        }
        s.push('\n');
    }
    if !cmds.is_empty() {
        s.push_str("## Commands\n\n");
        for c in &cmds {
            let _ = write!(
                s,
                "- [`{}`](references/docs/{}.mdoc) `[{}]` — {}\n",
                c.name, c.name, c.blast, c.description
            );
        }
        s.push('\n');
    }
    s
}

/// A per-op reference page — full detail, doc-site frontmatter so the collector
/// harvests it to the public API docs. `audience: public` (it IS the public
/// reference); the agent reads the file directly regardless of audience.
fn render_op_page(
    domain: &str,
    op: &OpRow,
    order: usize,
    fixtures: &BTreeMap<String, serde_json::Value>,
) -> String {
    let mut s = String::new();
    let kind = if op.is_read() { "read" } else { "command" };
    let _ = write!(
        s,
        "---\ntitle: {name}\ndescription: {desc}\naudience: public\ngroup: \"Ops: {domain}\"\norder: {order}\n---\n\n",
        name = op.name,
        desc = op.description,
    );
    let _ = write!(s, "<!-- AUTO-GENERATED by `cargo xtask gen-skills`. Source: proto descriptor (ship.ops.v1). Do not edit. -->\n\n");
    let _ = write!(s, "# `{}`\n\n{}\n\n", op.name, op.description);

    // Policy line.
    s.push_str("**Policy** — ");
    if op.is_read() {
        let _ = write!(s, "read (C0, safe). Capability `{}`.", op.permission);
    } else {
        let _ = write!(s, "command `[{}]`. Capability `{}`.", op.blast, op.permission);
    }
    if op.sensitive {
        s.push_str(" ⚠️ sensitive — payload may carry secrets or git content.");
    }
    s.push_str("\n\n");

    // Args table.
    s.push_str("## Arguments\n\n");
    if op.args.is_empty() {
        s.push_str("None.\n\n");
    } else {
        s.push_str("| name | type |\n|------|------|\n");
        for a in &op.args {
            let _ = write!(s, "| `{}` | `{}` |\n", a.name, a.kind);
        }
        s.push('\n');
    }

    // Return — type name + (when known) the response schema fields.
    let _ = write!(s, "## Returns\n\n`{}` (JSON).\n\n", op.returns);
    if !op.returns_fields.is_empty() {
        s.push_str("| field | type |\n|-------|------|\n");
        for f in &op.returns_fields {
            let _ = write!(s, "| `{}` | `{}` |\n", f.name, f.kind);
        }
        s.push('\n');
    }

    // Invocation.
    let _ = write!(
        s,
        "## Invoke\n\n```\nship ops call {} --json '{}'\n```\n\nKind: {}.\n\n",
        op.name,
        example_json(&op.args),
        kind,
    );

    // Realistic response example — the contract fixture (proto/fixtures), the
    // SAME data the cross-language contract tests decode. Authoritative usage.
    if let Some(example) = fixtures
        .get(&snake(&op.service))
        .and_then(|svc| svc.get(&op.method))
        .and_then(|m| m.get("response_ok"))
    {
        if let Ok(pretty) = serde_json::to_string_pretty(example) {
            let _ = write!(s, "## Example response\n\n```json\n{pretty}\n```\n");
        }
    }
    s
}

/// Load the contract fixtures keyed by service-file stem (`rules_service` →
/// its `{Method: {response_ok}}` object), so op pages can inline a realistic
/// response example — the SAME data the cross-language contract tests decode.
fn load_fixtures(root: &std::path::Path) -> BTreeMap<String, serde_json::Value> {
    let mut out = BTreeMap::new();
    let dir = root.join("proto/fixtures");
    let Ok(entries) = std::fs::read_dir(&dir) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string());
        let is_service = stem.as_deref().is_some_and(|s| s.ends_with("_service"));
        if path.extension().is_some_and(|e| e == "json") && is_service {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&path).unwrap_or_default()) {
                out.insert(stem.unwrap(), v);
            }
        }
    }
    out
}

pub fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out_root = root.join(OUT_REL);
    let fixtures = load_fixtures(&root);
    let by_domain = collect();
    let (mut skills, mut pages) = (0usize, 0usize);
    for (domain, ops) in &by_domain {
        let skill_dir = out_root.join(format!("ship-{domain}"));
        let refs_dir = skill_dir.join("references/docs");
        std::fs::create_dir_all(&refs_dir).with_context(|| format!("mkdir {}", refs_dir.display()))?;

        let toc = skill_dir.join("SKILL.md");
        std::fs::write(&toc, render_skill_toc(domain, ops))
            .with_context(|| format!("writing {}", toc.display()))?;
        skills += 1;

        for (i, op) in ops.iter().enumerate() {
            let page = refs_dir.join(format!("{}.mdoc", op.name));
            std::fs::write(&page, render_op_page(domain, op, i, &fixtures))
                .with_context(|| format!("writing {}", page.display()))?;
            pages += 1;
        }
    }
    eprintln!(
        "gen-skills: {skills} sdk-skills + {pages} op pages → {}",
        out_root.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surfaced_op_names() -> std::collections::BTreeSet<String> {
        collect().into_values().flatten().map(|r| r.name).collect()
    }

    #[test]
    fn reads_are_surfaced_not_audience_filtered() {
        // Observation is non-harmful for an assistant — reads stay agent-facing.
        // events.list is a read in OPS and is surfaced (no audience exclusion).
        // NOTE: mesh.snapshot / actors.list are declared but NOT assembled into
        // schema::OPS (their domains aren't in build_ops DOMAIN_ORDER), so they
        // can't be surfaced here yet — surfacing them is a separate OPS-assembly
        // change (the reads-are-agent-facing decision). Tracked, not done here.
        let names = surfaced_op_names();
        assert!(names.contains("events.list"), "events.list must be surfaced");
    }

    #[test]
    fn operational_writes_are_excluded() {
        // Kernel/actor control + mesh coordination are not part of the agent
        // capability set (mesh reserved for the future higher layer).
        let names = surfaced_op_names();
        for op in [
            "actor.input",
            "actor.kill_tree",
            "mesh.send",
            "mesh.broadcast",
            "mesh.status",
        ] {
            assert!(!names.contains(op), "operational write '{op}' must NOT be surfaced");
        }
    }

    #[test]
    fn is_operational_command_matches_actor_and_mesh_writes() {
        assert!(is_operational_command("actor.input"));
        assert!(is_operational_command("mesh.send"));
        assert!(!is_operational_command("rules.create"));
        assert!(!is_operational_command("agents.create"));
    }
}
