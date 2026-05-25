//! File writing — emit compiler output to the filesystem with progress reporting.

use anyhow::{Result, anyhow};
use compiler::{CompileOutput, get_harness};
use std::path::Path;

// ── File writer ──────────────────────────────────────────────────────────────

pub fn write_output(root: &Path, harness_id: &str, output: &CompileOutput) -> Result<()> {
    let desc = get_harness(harness_id)
        .ok_or_else(|| anyhow!("unknown harness: {}", harness_id))?;

    // Delegate filesystem writes to the compiler's native output module.
    compiler::write_output(root, harness_id, output)?;

    // Progress reporting: announce each written file.
    if let Some(file_name) = desc.context_file.file_name()
        && output.context_content.is_some()
    {
        println!("  {} {}", harness_id, file_name);
    }
    if let Some(ref mcp_path) = output.mcp_config_path
        && harness_id != "codex" {
            println!("  {} {}", harness_id, mcp_path);
        }
    for path in output.skill_files.keys() {
        println!("  {} {}", harness_id, path);
    }
    for path in output.rule_files.keys() {
        println!("  {} {}", harness_id, path);
    }
    if output.claude_settings_patch.is_some() {
        println!("  {} .claude/settings.json", harness_id);
    }
    for path in output.agent_files.keys() {
        println!("  {} {}", harness_id, path);
    }
    if output.codex_config_patch.is_some() {
        println!("  {} .codex/config.toml", harness_id);
    }
    if output.codex_hooks_json.is_some() {
        println!("  {} .codex/hooks.json", harness_id);
    }
    if output.codex_policy_content.is_some() { println!("  {} .codex/rules/ship.rules", harness_id) }
    if output.cursor_hooks_patch.is_some() {
        println!("  {} .cursor/hooks.json", harness_id);
    }
    if output.cursor_cli_permissions.is_some() {
        println!("  {} .cursor/cli.json", harness_id);
    }
    if output.cursor_environment_json.is_some() {
        println!("  {} .cursor/environment.json", harness_id);
    }
    if output.opencode_config_patch.is_some() {
        println!("  {} opencode.json", harness_id);
    }

    Ok(())
}

/// Dry-run: print what would be written.
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

// ── JSON merge helpers (re-exported from compiler for local use) ─────────────

#[allow(unused_imports)]
pub(crate) use compiler::{ensure_parent, merge_json, merge_json_file};

// ── Session scratch space ────────────────────────────────────────────────────

/// Ensure `.artifacts/` is listed in the root `.gitignore`.
/// Called once per `ship use` — idempotent.
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
