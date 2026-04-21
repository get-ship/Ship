//! `ship lint views` — enforce view-authoring discipline.
//!
//! Three rules, independently toggleable:
//! - `no-inline-script` (error): `<script>` tags are forbidden in views.
//! - `inline-style-budget` (warn > 50 lines, error > 150 lines): total
//!   content-line count inside `<style>` blocks across a view directory.
//! - `undefined-signal` (warn): `$name` references that have no matching
//!   `signal('name', ...)` declaration or `data-signals:name=` attribute
//!   within the same view, excluding Datastar internals and per-view
//!   `@lint:signals-ok` allowlist entries.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod report;
pub mod rules;
pub mod source;

#[cfg(test)]
mod tests;

pub use report::{Finding, OutputFormat, Severity};

/// Rule identifiers. Kept as string constants so they survive round-trips
/// through JSON output and `--rule` CLI args without a type dance.
pub const RULE_NO_INLINE_SCRIPT: &str = "no-inline-script";
pub const RULE_INLINE_STYLE_BUDGET: &str = "inline-style-budget";
pub const RULE_UNDEFINED_SIGNAL: &str = "undefined-signal";

pub const ALL_RULES: &[&str] = &[
    RULE_NO_INLINE_SCRIPT,
    RULE_INLINE_STYLE_BUDGET,
    RULE_UNDEFINED_SIGNAL,
];

/// Entrypoint wired from the CLI dispatcher.
///
/// - `project_root` points at the directory that contains `.ship/`.
/// - `view_name` filters to a single `.ship/views/<view_name>/` tree.
/// - `rules` restricts which rules run (empty = all).
pub fn run_views(
    project_root: &Path,
    view_name: Option<&str>,
    rules: &[String],
    format: OutputFormat,
) -> Result<()> {
    let views_root = project_root.join(".ship").join("views");
    if !views_root.exists() {
        anyhow::bail!(
            ".ship/views/ not found at {}. Nothing to lint.",
            views_root.display()
        );
    }

    let enabled = normalize_rules(rules)?;
    let targets = discover_views(&views_root, view_name)?;
    if targets.is_empty() {
        if let Some(name) = view_name {
            anyhow::bail!(
                "no view named '{}' under {}",
                name,
                views_root.display()
            );
        }
        report::print_empty(format);
        return Ok(());
    }

    let mut findings: Vec<Finding> = Vec::new();
    for view in &targets {
        findings.extend(rules::lint_view(view, &enabled));
    }

    // Stable ordering — file then line — makes diffs across runs comparable.
    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.rule.cmp(&b.rule))
    });

    let has_error = findings.iter().any(|f| f.severity == Severity::Error);
    report::emit(&findings, format, &targets)?;

    if has_error {
        std::process::exit(1);
    }
    Ok(())
}

/// Resolve a list of `--rule` values into the concrete set we will run.
/// Empty input means "all rules". Unknown rules error out — silently
/// ignoring them would hide typos like `--rule no-script`.
fn normalize_rules(raw: &[String]) -> Result<Vec<&'static str>> {
    if raw.is_empty() {
        return Ok(ALL_RULES.to_vec());
    }
    let mut out: Vec<&'static str> = Vec::new();
    for r in raw {
        let matched = ALL_RULES
            .iter()
            .find(|allowed| *allowed == &r.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown rule '{}'. Valid: {}",
                    r,
                    ALL_RULES.join(", ")
                )
            })?;
        if !out.contains(matched) {
            out.push(*matched);
        }
    }
    Ok(out)
}

/// A single view target — the directory plus the `.html.j2` files inside it
/// (index + partials). Elements/SDK templates live elsewhere; only
/// author-written views are considered.
#[derive(Debug, Clone)]
pub struct ViewTarget {
    pub name: String,
    pub dir: PathBuf,
    pub files: Vec<PathBuf>,
}

fn discover_views(views_root: &Path, filter: Option<&str>) -> Result<Vec<ViewTarget>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(views_root) {
        Ok(e) => e,
        Err(e) => anyhow::bail!("failed to read {}: {}", views_root.display(), e),
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    dirs.sort();

    for dir in dirs {
        let name = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() || name.starts_with('_') {
            continue;
        }
        if let Some(f) = filter
            && f != name
        {
            continue;
        }
        let files = collect_view_files(&dir);
        if files.is_empty() {
            continue;
        }
        out.push(ViewTarget {
            name,
            dir,
            files,
        });
    }
    Ok(out)
}

/// Recursively collect every `.html.j2` file under a view directory.
/// Views may nest sub-directories (`pipelines/focus/`, `pipelines/run/`,
/// `partials/`, …); the signal-budget and style-budget rules span the
/// whole tree.
fn collect_view_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_view_files_inner(dir, &mut out);
    out.sort();
    out
}

fn collect_view_files_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_view_files_inner(&p, out);
        } else if p
            .to_str()
            .map(|s| s.ends_with(".html.j2"))
            .unwrap_or(false)
        {
            out.push(p);
        }
    }
}
