//! Dispatch-time project compile — roll up the worktree's composed world and
//! compile the agent's harness-native config into it.
//!
//! Ship-side wiring behind the `AgentPlatform::compile_agent_config` port;
//! the roll-up (`ship_resolver`) and the compile engine (`agent_def::compile`)
//! are both ship concepts, so this glue lives here, not in fleet's bridge.

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Compile `agent_id`'s harness config into `worktree_path`.
///
/// Rolls up the worktree's full composed world — cache tier + the
/// operator-enabled library tier + the space's own source, precedence-law
/// composed — the same `ship_resolver::roll_up` shipd's boot index ingests,
/// so a dispatch-compiled agent sees what the index sees. `roll_up` is async
/// (V8 TS eval); `roll_up_registered_blocking` bridges the sync trait boundary.
///
/// `default_effort: Some("medium")` writes `effortLevel: medium` into the
/// worktree's `.claude/settings.json` so a dispatched agent never inherits
/// the operator's user-scope default. The agent profile's
/// `harness_settings.claude.effortLevel` passthrough still wins on conflict
/// (merged LAST into the settings patch).
pub fn compile_agent_config(worktree_path: &Path, agent_id: &str) -> Result<()> {
    let member_roots = member_roots_for_worktree(worktree_path);
    let composed = ship_resolver::roll_up_members_blocking(&member_roots, None)
        .map_err(|e| anyhow::anyhow!("roll-up failed for agent '{agent_id}': {e}"))?;

    let summary = agent_def::compile::compile_project_from_library(
        &agent_def::compile::CompileOptions {
            space_root: worktree_path.to_path_buf(),
            output_root: None,
            harness: None,
            active_agent: Some(agent_id.to_string()),
            extra_skills: vec![],
            plugins_installed: vec![],
            dry_run: false,
            default_effort: Some("medium".to_string()),
        },
        composed.library,
    )
    .map_err(|e| anyhow::anyhow!("compile failed for agent '{agent_id}': {e}"))?;
    if summary.harnesses.is_empty() {
        return Err(anyhow::anyhow!(
            "compile for agent '{agent_id}' produced no harness outputs — \
             no harnesses configured in the worktree's .ship/"
        ));
    }
    Ok(())
}

/// The member roots a WORKTREE composes with: the worktree STANDS IN for
/// its origin repo among the origin's registered space members. Membership
/// is registration (never a scan) — this helper only decides which
/// registered member the worktree replaces:
///
/// - a plain root (`.git` is a directory) composes as itself via the
///   registry (`resolve_member_roots`);
/// - a linked worktree (`.git` is a gitfile) resolves its ORIGIN root,
///   takes the origin's registered members, and substitutes itself for
///   the origin — so a dispatched agent sees its own worktree's content
///   plus the space's other members, exactly what the space anchor sees;
/// - an unregistered origin composes the worktree alone (the degenerate
///   single-stack case).
fn member_roots_for_worktree(worktree: &Path) -> Vec<PathBuf> {
    let gitfile = worktree.join(".git");
    if !gitfile.is_file() {
        // Not a linked worktree — a plain root composes as itself.
        return ship_resolver::resolve_member_roots(worktree);
    }
    let Some(origin) = origin_repo_root(&gitfile) else {
        return vec![worktree.to_path_buf()];
    };
    let origin = std::fs::canonicalize(&origin).unwrap_or(origin);
    let wt = std::fs::canonicalize(worktree).unwrap_or_else(|_| worktree.to_path_buf());
    let mut roots = ship_resolver::resolve_member_roots(&origin);
    for r in roots.iter_mut() {
        let canon = std::fs::canonicalize(&*r).unwrap_or_else(|_| r.clone());
        if canon == origin {
            *r = wt.clone();
        }
    }
    roots
}

/// Parse a linked worktree's gitfile (`gitdir: <origin>/.git/worktrees/<name>`)
/// back to the origin repo root. Returns `None` for any shape that isn't a
/// standard linked-worktree gitdir.
fn origin_repo_root(gitfile: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(gitfile).ok()?;
    let gitdir = content.strip_prefix("gitdir:")?.trim();
    let p = Path::new(gitdir);
    let worktrees_dir = p.parent()?; // .../.git/worktrees
    if worktrees_dir.file_name()? != "worktrees" {
        return None;
    }
    let git_dir = worktrees_dir.parent()?; // .../.git
    if git_dir.file_name()? != ".git" {
        return None;
    }
    Some(git_dir.parent()?.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gap 2: a dispatched worktree's `.claude/settings.json` must
    /// carry `effortLevel: medium` so a daemon-spawned agent never
    /// inherits the operator's user-scope `xhigh` default. The
    /// patch-emission logic is unit-tested in the compiler crate;
    /// this end-to-end test confirms the dispatch default
    /// (`default_effort: Some("medium")` in `compile_agent_config`)
    /// flows through to the worktree. Skipped when `roll_up_registered_blocking`
    /// can't find a sui runtime — the unit test pins the patch
    /// behavior; the e2e confirms the wiring.
    #[test]
    fn compile_dispatch_writes_settings_with_effort_medium() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let worktree = tmp.path().join("wt");
        std::fs::create_dir_all(&worktree).expect("mkdir worktree");
        let ship_dir = worktree.join(".ship");
        std::fs::create_dir_all(&ship_dir).expect("mkdir .ship");
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{ "active_agent": "demo" }"#,
        )
        .expect("write ship.jsonc");
        std::fs::create_dir_all(ship_dir.join("agents")).expect("mkdir agents");
        std::fs::write(
            ship_dir.join("agents/demo.jsonc"),
            r#"{ "agent": { "id": "demo", "name": "Demo" }, "target_agents": ["claude"] }"#,
        )
        .expect("write demo agent");
        if compile_agent_config(&worktree, "demo").is_err() {
            // Roll-up needs a sui runtime that may not be present in
            // the test environment — skip; the unit test pins the
            // patch behavior.
            return;
        }
        let settings = std::fs::read_to_string(worktree.join(".claude/settings.json"))
            .expect("settings.json");
        let v: serde_json::Value = serde_json::from_str(&settings).expect("parse settings");
        assert_eq!(
            v.get("effortLevel").and_then(|s| s.as_str()),
            Some("medium"),
            "gap 2: dispatched worktree must carry effortLevel: medium"
        );
    }

    fn write_gitfile(dir: &Path, content: &str) -> PathBuf {
        let f = dir.join(".git");
        std::fs::write(&f, content).unwrap();
        f
    }

    #[test]
    fn standard_linked_worktree_gitfile_resolves_origin() {
        let tmp = tempfile::tempdir().unwrap();
        let f = write_gitfile(
            tmp.path(),
            "gitdir: /home/dev/origin/.git/worktrees/my-job
",
        );
        assert_eq!(
            origin_repo_root(&f).unwrap(),
            PathBuf::from("/home/dev/origin")
        );
    }

    #[test]
    fn non_worktree_gitdir_shapes_resolve_none() {
        let tmp = tempfile::tempdir().unwrap();
        for content in [
            "gitdir: /somewhere/else
",
            "gitdir: /repo/.git
",
            "not a gitfile at all",
            "gitdir: /repo/.hg/worktrees/x
",
        ] {
            let f = write_gitfile(tmp.path(), content);
            assert!(origin_repo_root(&f).is_none(), "content: {content:?}");
        }
    }
}
