//! `ship compile` — thin CLI wrapper around ship-platform's compile service.
//! Terminal presentation only: dry-run print, --with UX, ✓ line, session gitignore.

pub(crate) mod output;

#[cfg(test)]
mod tests_worktree;

use anyhow::Result;
use std::path::Path;

// ── Entry point ──────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Source root — the source directory on disk.
    pub space_root: &'a Path,
    /// Output root — where compiled files go. Defaults to `space_root`.
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
    /// `effortLevel` written to the worktree's `.claude/settings.json` when
    /// neither the active agent profile nor the library overrides it.
    /// `None` for the manual `ship compile` path (preserves legacy
    /// behavior); the dispatch path passes `Some("medium")`. Mirrors
    /// `agent_def::compile::CompileOptions::default_effort` — the
    /// wrapper passes it through verbatim.
    pub default_effort: Option<String>,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    use ship_agents::services::compile as svc;

    let extra_skills = opts.extra_skills.clone();

    let summary = svc::compile_project(&svc::CompileOptions {
        space_root: opts.space_root.to_path_buf(),
        output_root: opts.output_root.map(|p| p.to_path_buf()),
        harness: opts.harness.map(str::to_string),
        active_agent: opts.active_agent.map(str::to_string),
        extra_skills: opts.extra_skills,
        plugins_installed: opts.plugins_installed,
        scoped_launch: opts.scoped_launch,
        dry_run: opts.dry_run,
        default_effort: opts.default_effort,
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
        output::ensure_session_gitignored(opts.output_root.unwrap_or(opts.space_root))?;
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
