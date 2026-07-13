//! Terminal presentation for compile output — dry-run printing and session gitignore.

use anyhow::Result;
use harness_config_compiler::{CompileOutput, get_harness};
use std::path::Path;

// ── Dry-run printing ─────────────────────────────────────────────────────────

pub(crate) fn print_dry_run(harness_id: &str, output: &CompileOutput) {
    println!("[dry-run] harness: {}", harness_id);
    if let Some(f) = get_harness(harness_id).and_then(|d| d.context_file.file_name())
        && output.context_content.is_some()
    {
        println!("  would write {}", f);
    }
    if let Some(ref p) = output.mcp_config_path {
        println!("  would write {}", p);
    }
    for path in output.skill_files.keys() {
        println!("  would write {}", path);
    }
    for path in output.rule_files.keys() {
        println!("  would write {}", path);
    }
    if output.claude_settings_patch.is_some() {
        println!("  would merge .claude/settings.json");
    }
    for path in output.agent_files.keys() {
        println!("  would write {}", path);
    }
    if output.codex_config_patch.is_some() {
        println!("  would write .codex/config.toml");
    }
    if output.codex_hooks_json.is_some() {
        println!("  would write .codex/hooks.json");
    }
    match &output.codex_policy_content {
        Some(_) => println!("  would write .codex/rules/ship.rules"),
        None => println!("  would remove .codex/rules/ship.rules (no rules)"),
    }
    if output.cursor_hooks_patch.is_some() {
        println!("  would write .cursor/hooks.json");
    }
    if output.cursor_cli_permissions.is_some() {
        println!("  would write .cursor/cli.json");
    }
    if output.cursor_environment_json.is_some() {
        println!("  would write .cursor/environment.json");
    }
    if output.opencode_config_patch.is_some() {
        println!("  would write opencode.json");
    }
}

// ── Session scratch space ────────────────────────────────────────────────────

/// Ensure `.artifacts/` is listed in the root `.gitignore`. Idempotent.
pub(crate) fn ensure_session_gitignored(root: &Path) -> Result<()> {
    const ENTRY: &str = ".artifacts/";
    let path = root.join(".gitignore");
    let existing = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    if existing.lines().any(|l| l.trim() == ENTRY) {
        return Ok(());
    }
    let updated = if existing.is_empty() || existing.ends_with('\n') {
        format!("{}{}\n", existing, ENTRY)
    } else {
        format!("{}\n{}\n", existing, ENTRY)
    };
    std::fs::write(&path, updated)?;
    Ok(())
}
