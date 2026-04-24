//! Finding types and output rendering.

use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

use super::ViewTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warn,
    Error,
}

impl Severity {
    fn as_str(&self) -> &'static str {
        match self {
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Print a summary line when zero views were matched but the filter was
/// satisfied (e.g. empty views directory). JSON stays a valid empty array.
pub fn print_empty(format: OutputFormat) {
    match format {
        OutputFormat::Human => println!("no views to lint"),
        OutputFormat::Json => println!("[]"),
    }
}

pub fn emit(findings: &[Finding], format: OutputFormat, targets: &[ViewTarget]) -> Result<()> {
    match format {
        OutputFormat::Human => emit_human(findings, targets),
        OutputFormat::Json => emit_json(findings),
    }
}

fn emit_json(findings: &[Finding]) -> Result<()> {
    // Use a compact-but-readable array so CI log output stays grep-friendly.
    let payload: Vec<JsonFinding> = findings.iter().map(JsonFinding::from).collect();
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn emit_human(findings: &[Finding], targets: &[ViewTarget]) -> Result<()> {
    if findings.is_empty() {
        println!(
            "✓ {} view{} — clean",
            targets.len(),
            if targets.len() == 1 { "" } else { "s" }
        );
        return Ok(());
    }
    let mut errors = 0usize;
    let mut warns = 0usize;
    for f in findings {
        let label = match f.severity {
            Severity::Error => {
                errors += 1;
                "error"
            }
            Severity::Warn => {
                warns += 1;
                "warn"
            }
        };
        println!(
            "{}:{}:{} {}[{}]: {}",
            f.file.display(),
            f.line,
            f.col,
            label,
            f.rule,
            f.message,
        );
    }
    println!();
    println!(
        "{} error{}, {} warning{} across {} view{}",
        errors,
        if errors == 1 { "" } else { "s" },
        warns,
        if warns == 1 { "" } else { "s" },
        targets.len(),
        if targets.len() == 1 { "" } else { "s" },
    );
    Ok(())
}

#[derive(Serialize)]
struct JsonFinding<'a> {
    file: String,
    line: usize,
    col: usize,
    rule: &'a str,
    severity: &'a str,
    message: &'a str,
}

impl<'a> From<&'a Finding> for JsonFinding<'a> {
    fn from(f: &'a Finding) -> Self {
        Self {
            file: f.file.display().to_string(),
            line: f.line,
            col: f.col,
            rule: f.rule,
            severity: f.severity.as_str(),
            message: f.message.as_str(),
        }
    }
}
