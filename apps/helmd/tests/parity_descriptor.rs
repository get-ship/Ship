//! Descriptor-level parity capture: enumerate the resolved descriptor's
//! agents + skills + rules id sets so a before/after diff proves the resolver
//! merge converges on identical surfaces.
//!
//! Run:
//!   HELMD_DESCRIPTOR_OUT=/tmp/descriptor-before.txt cargo test -p helmd --test parity_descriptor -- --nocapture
use std::path::PathBuf;

use ship_resolver::roll_up_blocking;
use stack_model::ComposedLibrary;

fn write_library_snapshot(label: &str, lib: &ComposedLibrary, out: &mut String) {
    out.push_str(&format!("# {label}\n"));
    let mut agents: Vec<String> = lib.library.agent_profiles.iter().map(|p| p.agent.id.clone()).collect();
    agents.sort_unstable();
    out.push_str(&format!("agents[{}]: {}\n", agents.len(), agents.join(",")));
    let mut skills: Vec<String> = lib.library.skills.iter().map(|s| s.id.clone()).collect();
    skills.sort_unstable();
    out.push_str(&format!("skills[{}]: {}\n", skills.len(), skills.join(",")));
    let mut rules: Vec<String> = lib.library.rules.iter().map(|r| r.file_name.clone()).collect();
    rules.sort_unstable();
    out.push_str(&format!("rules[{}]: {}\n", rules.len(), rules.join(",")));
    let mut mcp: Vec<String> = lib.library.mcp_servers.iter().map(|m| m.id.clone()).collect();
    mcp.sort_unstable();
    out.push_str(&format!("mcp[{}]: {}\n", mcp.len(), mcp.join(",")));
    let mut collections: Vec<String> = lib.library.collections.iter().map(|c| c.id.clone()).collect();
    collections.sort_unstable();
    out.push_str(&format!("collections[{}]: {}\n", collections.len(), collections.join(",")));
    let mut triggers: Vec<String> = lib.library.triggers.iter().map(|t| t.id.clone()).collect();
    triggers.sort_unstable();
    out.push_str(&format!("triggers[{}]: {}\n", triggers.len(), triggers.join(",")));
    let mut pipelines: Vec<String> = lib.library.pipelines.iter().map(|p| p.name.clone()).collect();
    pipelines.sort_unstable();
    out.push_str(&format!("pipelines[{}]: {}\n", pipelines.len(), pipelines.join(",")));
    out.push_str(&format!("skill_paths: {:?}\n", lib.library.skill_paths));
    out.push_str(&format!("skills_dir: {:?}\n", lib.library.skills_dir));
    out.push_str(&format!("rules_dir: {:?}\n", lib.library.rules_dir));
}

#[test]
fn snapshot_resolved_descriptor_for_worktree() {
    let root: PathBuf = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|p| p.join("ship.config.ts").exists())
        .map(|p| p.to_path_buf())
        .expect("could not locate ship.config.ts in cwd ancestors");
    eprintln!("[descriptor] worktree root: {}", root.display());

    // No cache, no library dir for the source-only baseline.
    let composed = roll_up_blocking(&root, None, None).expect("roll_up");
    let mut out = String::new();
    out.push_str(&format!("# worktree: {}\n", root.display()));
    write_library_snapshot("library", &composed, &mut out);

    if let Ok(path) = std::env::var("HELMD_DESCRIPTOR_OUT") {
        std::fs::write(&path, &out).expect("write descriptor snapshot");
        eprintln!("[descriptor] wrote {}", path);
    } else {
        print!("{out}");
    }

    // Sanity: agents + skills + rules are non-empty (the close-out scope).
    assert!(
        !composed.library.agent_profiles.is_empty(),
        "agents should populate"
    );
}
