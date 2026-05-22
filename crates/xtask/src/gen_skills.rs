//! Generate capability-skills from the op registry (`cargo xtask gen-skills`).
//!
//! Skills-as-API: each capability domain (git, pipeline, mesh, …) becomes a
//! SKILL.md that documents its ops and tells the agent to invoke them via the
//! Connect CLI — `ship ops call <name> --json '{…}'`. The skill IS the API
//! doc; `ship ops` is the client; Connect is the transport. This is what
//! replaces the internal MCP tool surface as the agent's interface.
//!
//! Projected from `schema::OPS` + `COMMANDS` — never hand-maintained.
//! Output: `.artifacts/capability-skills/ship-<domain>/SKILL.md` (POC; the
//! production wiring writes into the skills dir + registers them).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};

struct OpRow {
    name: String,
    description: String,
    args: String,
    blast: Option<String>,
}

fn arg_summary(args: &[shipd::services::schema::ArgDescriptor]) -> String {
    if args.is_empty() {
        return "{}".to_string();
    }
    let parts: Vec<String> = args
        .iter()
        .map(|a| {
            let opt = if a.required { "" } else { "?" };
            format!("{}{opt}: {}", a.name, a.kind)
        })
        .collect();
    format!("{{ {} }}", parts.join(", "))
}

fn domain_of(name: &str) -> &str {
    name.split('.').next().unwrap_or(name)
}

/// Group reads + commands by domain.
fn collect() -> BTreeMap<String, (Vec<OpRow>, Vec<OpRow>)> {
    let mut by_domain: BTreeMap<String, (Vec<OpRow>, Vec<OpRow>)> = BTreeMap::new();
    for op in shipd::services::schema::OPS {
        by_domain
            .entry(domain_of(op.name).to_string())
            .or_default()
            .0
            .push(OpRow {
                name: op.name.to_string(),
                description: op.description.to_string(),
                args: arg_summary(op.args),
                blast: None,
            });
    }
    for cmd in shipd::services::schema::COMMANDS {
        by_domain
            .entry(domain_of(cmd.name).to_string())
            .or_default()
            .1
            .push(OpRow {
                name: cmd.name.to_string(),
                description: cmd.description.to_string(),
                args: arg_summary(cmd.args),
                blast: Some(cmd.blast_radius.to_string()),
            });
    }
    by_domain
}

fn render_skill(domain: &str, reads: &[OpRow], cmds: &[OpRow]) -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        "---\nname: ship-{domain}\ndescription: >-\n  The `{domain}` capability — call these ops via the Connect CLI\n  (`ship ops call <name> --json '{{…}}'`). Generated from the op registry.\n---\n\n",
    );
    let _ = write!(
        s,
        "# ship-{domain}\n\nYour interface to the `{domain}` ops. Invoke each over ConnectRPC with:\n\n```\nship ops call <name> --json '<args-json>'\n```\n\nArgs marked `?` are optional. Reads are safe (C0); commands carry a blast-radius class.\n\n",
    );
    if !reads.is_empty() {
        s.push_str("## Reads\n\n");
        for r in reads {
            let _ = write!(s, "- `{}` — {}\n  - args: `{}`\n  - `ship ops call {} --json '{{}}'`\n", r.name, r.description, r.args, r.name);
        }
        s.push('\n');
    }
    if !cmds.is_empty() {
        s.push_str("## Commands\n\n");
        for c in cmds {
            let blast = c.blast.as_deref().unwrap_or("");
            let _ = write!(s, "- `{}` `[{}]` — {}\n  - args: `{}`\n  - `ship ops call {} --json '<args>'`\n", c.name, blast, c.description, c.args, c.name);
        }
        s.push('\n');
    }
    s
}

pub fn run() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out_root = root.join(".artifacts/capability-skills");
    let by_domain = collect();
    let mut count = 0;
    for (domain, (reads, cmds)) in &by_domain {
        let dir = out_root.join(format!("ship-{domain}"));
        std::fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
        let path = dir.join("SKILL.md");
        std::fs::write(&path, render_skill(domain, reads, cmds))
            .with_context(|| format!("writing {}", path.display()))?;
        count += 1;
    }
    eprintln!(
        "gen-skills: {count} capability-skills → {}",
        out_root.display()
    );
    Ok(())
}
