//! CLI shim for the view linter.
//!
//! All rule logic lives in `crates/core/lint`. This module handles:
//! - CLI arg resolution (project root, view filter, rule list, output format)
//! - Output formatting for human/JSON
//! - Exit code on errors

use anyhow::Result;
use std::path::Path;

pub use lint::{ALL_RULES, Finding, Severity};

/// Output format for the `ship lint views` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Entrypoint wired from the CLI dispatcher.
pub fn run_views(
    project_root: &Path,
    view_name: Option<&str>,
    rules: &[String],
    format: OutputFormat,
) -> Result<()> {
    let views_root = ship_substrate::ShipHome::project_ship_dir(project_root).join("views");
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
            anyhow::bail!("no view named '{}' under {}", name, views_root.display());
        }
        match format {
            OutputFormat::Human => println!("no views to lint"),
            OutputFormat::Json => println!("[]"),
        }
        return Ok(());
    }

    let mut findings: Vec<Finding> = Vec::new();
    for (_view_name, view_dir) in &targets {
        let mut fs = lint::lint_view_dir(view_dir);
        // Filter to only enabled rules.
        if enabled.len() < ALL_RULES.len() {
            fs.retain(|f| enabled.contains(&f.rule));
        }
        findings.extend(fs);
    }

    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.rule.cmp(b.rule))
    });

    let has_error = findings.iter().any(|f| f.severity == Severity::Error);
    emit(&findings, format, targets.len())?;

    if has_error {
        std::process::exit(1);
    }
    Ok(())
}

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

fn discover_views(
    views_root: &Path,
    filter: Option<&str>,
) -> Result<Vec<(String, std::path::PathBuf)>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(views_root) {
        Ok(e) => e,
        Err(e) => anyhow::bail!("failed to read {}: {}", views_root.display(), e),
    };
    let mut dirs: Vec<std::path::PathBuf> = entries
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
            && f != name {
                continue;
            }
        if lint::collect_view_files(&dir).is_empty() {
            continue;
        }
        out.push((name, dir));
    }
    Ok(out)
}

fn emit(findings: &[Finding], format: OutputFormat, view_count: usize) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let payload: Vec<serde_json::Value> = findings
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "file": f.file.display().to_string(),
                        "line": f.line,
                        "col": f.col,
                        "rule": f.rule,
                        "severity": f.severity.as_str(),
                        "message": f.message,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        OutputFormat::Human => {
            if findings.is_empty() {
                println!(
                    "✓ {} view{} — clean",
                    view_count,
                    if view_count == 1 { "" } else { "s" }
                );
                return Ok(());
            }
            let mut errors = 0usize;
            let mut warns = 0usize;
            for f in findings {
                let label = match f.severity {
                    Severity::Error => { errors += 1; "error" }
                    Severity::Warn => { warns += 1; "warn" }
                };
                println!(
                    "{}:{}:{} {}[{}]: {}",
                    f.file.display(), f.line, f.col, label, f.rule, f.message,
                );
            }
            println!();
            println!(
                "{} error{}, {} warning{} across {} view{}",
                errors, if errors == 1 { "" } else { "s" },
                warns, if warns == 1 { "" } else { "s" },
                view_count, if view_count == 1 { "" } else { "s" },
            );
        }
    }
    Ok(())
}
