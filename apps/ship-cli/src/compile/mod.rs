//! `ship compile` — thin CLI wrapper around ship-platform's compile service.
//! Terminal presentation only: dry-run print, --with UX, ✓ line, session gitignore.

pub(crate) mod output;

#[cfg(test)]
mod tests_worktree;

use anyhow::Result;
use std::path::Path;

// ── Entry point ──────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Source root — where .ship/ lives (may be the main repo for worktrees).
    pub project_root: &'a Path,
    /// Output root — where compiled files go. Defaults to project_root.
    pub output_root: Option<&'a Path>,
    /// Optionally restrict to a single harness.
    pub harness: Option<&'a str>,
    /// Print what would be written without touching the filesystem.
    pub dry_run: bool,
    /// Active mode id (already resolved from PathContext / ship.jsonc).
    pub active_agent: Option<&'a str>,
    /// Extra skill IDs injected at compile time via `--with`. Additive on top of
    /// the agent's declared skill refs. Each ID must exist in the loaded library.
    pub extra_skills: Vec<String>,
    /// Plugin IDs the active agent declared and that this compile installed.
    pub plugins_installed: Vec<String>,
    /// Compile for a daemon-spawned scoped launch.
    pub scoped_launch: bool,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    use ship_platform::services::compile as svc;

    let extra_skills = opts.extra_skills.clone();

    let summary = svc::compile_project(&svc::CompileOptions {
        project_root: opts.project_root.to_path_buf(),
        output_root: opts.output_root.map(|p| p.to_path_buf()),
        harness: opts.harness.map(str::to_string),
        active_agent: opts.active_agent.map(str::to_string),
        extra_skills: opts.extra_skills,
        plugins_installed: opts.plugins_installed,
        scoped_launch: opts.scoped_launch,
        dry_run: opts.dry_run,
    })?;

    if summary.harnesses.is_empty() {
        println!("No harnesses configured. Add harnesses to .ship/ship.jsonc or agent profile.");
        return Ok(());
    }

    if opts.dry_run {
        for (harness_id, output) in &summary.outputs {
            output::print_dry_run(harness_id, output);
        }
    } else {
        output::ensure_session_gitignored(opts.output_root.unwrap_or(opts.project_root))?;
        if extra_skills.is_empty() {
            println!("✓ compiled for: {}", summary.harnesses.join(", "));
        } else {
            let injected: Vec<String> =
                extra_skills.iter().map(|s| format!("+{}", s)).collect();
            println!(
                "✓ compiled for: {} ({})",
                summary.harnesses.join(", "),
                injected.join(", ")
            );
        }
    }
    Ok(())
}
