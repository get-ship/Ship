//! Generate sdk-skills from the op registry (`cargo xtask gen-skills`).
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
//! Projected from `schema::OPS` + `COMMANDS` — never hand-maintained. Output:
//! `skills/sdk/ship-<domain>/` — the `sdk` subdir of the skills tree holds the
//! generated sdk-skill kind, a peer to `skills/docs/` (doc-skills) and the
//! hand-authored smart-skills at `skills/`. Each file is stamped AUTO-GENERATED.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};

use shipd::services::schema::ArgDescriptor;

/// Output root, relative to the repo: the `sdk` subdir of the skills tree.
/// Separate from the hand-authored `docs/` doc-site source (which it must not
/// collide with) and from the smart-skills at `skills/`. The doc-site collector
/// is pointed here via its skills-root list so these pages still harvest.
const OUT_REL: &str = "skills/sdk";

struct OpRow {
    name: String,
    description: String,
    args: &'static [ArgDescriptor],
    /// Reads carry a `returns` shape tag; commands leave this empty.
    returns: String,
    /// Required capability (e.g. `ship.git.write`).
    permission: String,
    /// Gate class for commands ("C1_WORKSPACE" | "C2_ELEVATED"); empty for reads.
    blast: String,
    sensitive: bool,
}

impl OpRow {
    fn is_read(&self) -> bool {
        self.blast.is_empty()
    }
}

fn domain_of(name: &str) -> &str {
    name.split('.').next().unwrap_or(name)
}

/// Audience axis (#95): which ops appear in the generated *agent* sdk-skills.
///
/// **Reads are always agent-facing** — observation is non-harmful for an
/// assistant acting within the user's workspace, so every read op is surfaced
/// (this includes `mesh.snapshot`, `actors.list`, `events.list`).
///
/// **Operational WRITES are excluded**: kernel/actor control (`actor.*`) and
/// mesh coordination (`mesh.*`). Actor control is operator/debug; mesh
/// coordination is reserved for the higher-level primitive that will wrap it
/// (agents hold no surfaced interface to raw mesh). These stay real ABI on the
/// operator surface — they're just not part of the agent capability set.
fn is_operational_command(name: &str) -> bool {
    matches!(domain_of(name), "actor" | "mesh")
}

/// A JSON placeholder value for an arg kind — seeds the `ship ops call` example.
fn placeholder(kind: &str) -> &'static str {
    let base = kind.trim_end_matches('?');
    match base {
        "number" => "0",
        "bool" => "false",
        "map" => "{}",
        _ => "\"\"",
    }
}

/// Example args object: required args with kind-typed placeholders. Empty `{}`
/// when the op takes no required args.
fn example_json(args: &[ArgDescriptor]) -> String {
    let req: Vec<String> = args
        .iter()
        .filter(|a| a.required)
        .map(|a| format!("\"{}\": {}", a.name, placeholder(a.kind)))
        .collect();
    if req.is_empty() { "{}".to_string() } else { format!("{{ {} }}", req.join(", ")) }
}

fn collect() -> BTreeMap<String, Vec<OpRow>> {
    let mut by_domain: BTreeMap<String, Vec<OpRow>> = BTreeMap::new();
    for op in shipd::services::schema::OPS.iter() {
        // Reads are always agent-facing — no audience exclusion.
        by_domain.entry(domain_of(op.name).to_string()).or_default().push(OpRow {
            name: op.name.to_string(),
            description: op.description.to_string(),
            args: op.args,
            returns: op.returns.to_string(),
            permission: op.permission.to_string(),
            blast: String::new(),
            sensitive: op.sensitive,
        });
    }
    for cmd in shipd::services::schema::COMMANDS {
        if is_operational_command(cmd.name) {
            continue;
        }
        by_domain.entry(domain_of(cmd.name).to_string()).or_default().push(OpRow {
            name: cmd.name.to_string(),
            description: cmd.description.to_string(),
            args: cmd.args,
            returns: String::new(),
            permission: cmd.permission.to_string(),
            blast: cmd.blast_radius.to_string(),
            sensitive: cmd.sensitive,
        });
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
        "<!-- AUTO-GENERATED by `cargo xtask gen-skills` from the op registry. Do not edit. -->\n\n# ship-{domain}\n\nYour interface to the `{domain}` ops. Invoke each over ConnectRPC:\n\n```\nship ops call <name> --json '<args-json>'\n```\n\nReads are safe (C0). Commands carry a blast-radius class. Open an op's reference\npage for full args, return shape, capability, and an example.\n\n",
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
fn render_op_page(domain: &str, op: &OpRow, order: usize) -> String {
    let mut s = String::new();
    let kind = if op.is_read() { "read" } else { "command" };
    let _ = write!(
        s,
        "---\ntitle: {name}\ndescription: {desc}\naudience: public\ngroup: \"Ops: {domain}\"\norder: {order}\n---\n\n",
        name = op.name,
        desc = op.description,
    );
    let _ = write!(s, "<!-- AUTO-GENERATED by `cargo xtask gen-skills`. Source: op registry. Do not edit. -->\n\n");
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
        s.push_str("| name | type | required |\n|------|------|----------|\n");
        for a in op.args {
            let req = if a.required { "yes" } else { "no" };
            let _ = write!(s, "| `{}` | `{}` | {} |\n", a.name, a.kind, req);
        }
        s.push('\n');
    }

    // Return.
    s.push_str("## Returns\n\n");
    if op.is_read() {
        let _ = write!(s, "`{}` (JSON).\n\n", op.returns);
    } else {
        s.push_str("Status JSON (the mutation result).\n\n");
    }

    // Invocation.
    let _ = write!(
        s,
        "## Invoke\n\n```\nship ops call {} --json '{}'\n```\n\nKind: {}.\n",
        op.name,
        example_json(op.args),
        kind,
    );
    s
}

pub fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out_root = root.join(OUT_REL);
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
            std::fs::write(&page, render_op_page(domain, op, i))
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
