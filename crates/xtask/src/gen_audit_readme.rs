//! Regenerate `.artifacts/specs/module-audits/README.md` from per-module audit docs.
//!
//! This subcommand walks the four layer directories (L0, L1, shared, apps) and
//! extracts metadata (module name, status, pass date, scope path, flagged callouts)
//! from each `*.md` file to mechanically rebuild the README's coverage tables.
//! Non-table content (intro, legend, process, questions) is preserved.
//!
//! Usage: `cargo xtask gen-audit-readme`

use std::path::Path;
use anyhow::{Context, Result};

const AUDIT_ROOT: &str = ".artifacts/specs/module-audits";
const README_PATH: &str = ".artifacts/specs/module-audits/README.md";

#[derive(Clone)]
struct AuditRow {
    module: String,
    path: String,
    status: String,
    pass_date: String,
    doc_file: String,
}

/// Extract metadata from the first ~10 lines of an audit doc (header only).
/// This extracts: module name (from H1), status (from **Module status**: line),
/// pass date (from blockquote with Pass date:), and flagged callouts.
fn extract_metadata(content: &str, filename: &str, layer: &str) -> Result<AuditRow> {
    let mut module = String::new();
    let mut status = String::new();
    let mut pass_date = String::new();
    let mut path = String::new();
    let mut flagged = String::new();

    let lines: Vec<&str> = content.lines().collect();

    // Line 0: Extract module name from H1 title.
    if lines.len() > 0 && lines[0].starts_with("# ") {
        let h1_text = &lines[0][2..];
        // Try to extract text within backticks.
        if let Some(start) = h1_text.find('`') {
            if let Some(end) = h1_text[start + 1..].find('`') {
                module = h1_text[start + 1..start + 1 + end].to_string();
            }
        }
    }

    // Fallback: module name from filename if H1 extraction failed.
    if module.is_empty() {
        module = filename.replace(".md", "").to_string();
    }

    // Line 1-3 (blockquote header): Extract pass date from "Pass date: <date>".
    // Example: "> Pass date: 2026-05-29 · scope: L0 ship-substrate ..."
    if lines.len() > 1 && lines[1].starts_with(">") {
        if let Some(pos) = lines[1].find("Pass date:") {
            let after = &lines[1][pos + 10..];
            if let Some(word) = after.split_whitespace().next() {
                pass_date = word.to_string();
            }
        }
        // Also extract path from "scope: <path>"
        if let Some(pos) = lines[1].find("scope:") {
            let after = &lines[1][pos + 6..];
            let word = after.split(|c: char| c.is_whitespace() || c == '·').next().unwrap_or("");
            if !word.is_empty() {
                path = word.to_string();
            }
        }
    }

    // Find "**Module status**:" line (usually line 4-5).
    // Example: "**Module status:** draft — ready for user-review."
    for i in 0..std::cmp::min(10, lines.len()) {
        if lines[i].contains("**Module status**:") {
            if let Some(after) = lines[i].split("**Module status**:").nth(1) {
                status = after.trim().to_string();
                // Truncate at period or "|" for cleaner output.
                if let Some(pos) = status.find('.') {
                    status.truncate(pos);
                }
                break;
            }
        }
    }

    // Scan first 10 lines for flagged callouts (bolded patterns like **(2 SF)**).
    for i in 0..std::cmp::min(10, lines.len()) {
        let line = lines[i];
        if line.contains("**(") && line.contains(")**") {
            let mut pos = 0;
            while let Some(start) = line[pos..].find("**(") {
                let abs_start = pos + start;
                if let Some(end) = line[abs_start + 3..].find(")**") {
                    let callout = &line[abs_start + 2..abs_start + 3 + end + 1];
                    if !flagged.contains(callout) {
                        if !flagged.is_empty() {
                            flagged.push(' ');
                        }
                        flagged.push_str(callout);
                    }
                    pos = abs_start + 3 + end + 2;
                } else {
                    break;
                }
            }
        }
    }

    // Fallback: default status and pass_date if not found.
    if status.is_empty() {
        status = "pending".to_string();
    }
    if pass_date.is_empty() {
        pass_date = "—".to_string();
    }

    // Fallback: path from filename stem if not found in doc.
    if path.is_empty() {
        path = format!("{}/", filename.replace(".md", ""));
    }

    // Build the final status string with flagged callouts.
    let final_status = if flagged.is_empty() {
        status
    } else {
        format!("{} {}", status, flagged)
    };

    Ok(AuditRow {
        module,
        path,
        status: final_status,
        pass_date,
        doc_file: format!("{}/{}", layer, filename),
    })
}

/// Walk a layer directory and extract rows from all non-underscore .md files.
fn collect_layer(layer_dir: &Path) -> Result<Vec<AuditRow>> {
    let mut rows = Vec::new();

    let layer_name = layer_dir.file_name().and_then(|n| n.to_str()).unwrap_or("?");

    if !layer_dir.is_dir() {
        return Ok(rows);
    }

    for entry in std::fs::read_dir(layer_dir).context(format!("read {}", layer_dir.display()))? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip files starting with underscore.
        if filename.starts_with('_') {
            continue;
        }

        // Only process .md files.
        if !filename.ends_with(".md") {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .context(format!("read {}", path.display()))?;

        let row = extract_metadata(&content, filename, layer_name)?;
        rows.push(row);
    }

    // Sort rows by module name for deterministic output.
    rows.sort_by(|a, b| a.module.cmp(&b.module));

    Ok(rows)
}

/// Render a markdown table row.
fn render_row(row: &AuditRow) -> String {
    format!(
        "| {} | `{}` | {} | {} | [{}]({}) |",
        row.module,
        row.path,
        row.status,
        row.pass_date,
        row.doc_file,
        row.doc_file
    )
}

/// Regenerate the README: preserve non-table content, replace tables in place.
pub fn run(repo_root: &Path) -> Result<()> {
    let audit_root = repo_root.join(AUDIT_ROOT);
    let readme_path = repo_root.join(README_PATH);

    // Collect rows from all four layer directories.
    let l0_rows = collect_layer(&audit_root.join("L0-substrate"))?;
    let l1_rows = collect_layer(&audit_root.join("L1-platform"))?;
    let shared_rows = collect_layer(&audit_root.join("shared"))?;
    let apps_rows = collect_layer(&audit_root.join("apps"))?;

    // Read existing README to preserve structure, or create new one if doesn't exist.
    let existing = std::fs::read_to_string(&readme_path).unwrap_or_default();

    // Split on section headings and reconstruct with regenerated tables.
    let mut output = String::new();

    // Copy intro up to first table.
    if let Some(pos) = existing.find("## L0 — Substrate") {
        output.push_str(&existing[..pos]);
    } else {
        output.push_str(&existing);
        output.push('\n');
    }

    // L0 table with fence markers.
    output.push_str("## L0 — Substrate\n\n");
    output.push_str("The event log + persistence + path resolution + actor supervisor. No domain logic.\n\n");
    output.push_str("<!-- gen-audit-readme: L0-substrate -->\n");
    output.push_str("| Module | Path | Status | Pass date | Doc |\n");
    output.push_str("|---|---|---|---|---|\n");
    for row in &l0_rows {
        output.push_str(&render_row(row));
        output.push('\n');
    }
    output.push_str("<!-- /gen-audit-readme -->\n\n");

    // L1 table with fence markers.
    output.push_str("## L1 — Platform\n\n");
    output.push_str("Domain services on top of substrate state.\n\n");
    output.push_str("<!-- gen-audit-readme: L1-platform -->\n");
    output.push_str("| Module | Path | Status | Pass date | Doc |\n");
    output.push_str("|---|---|---|---|---|\n");
    for row in &l1_rows {
        output.push_str(&render_row(row));
        output.push('\n');
    }
    output.push_str("<!-- /gen-audit-readme -->\n\n");

    // L2 placeholder (TBD).
    output.push_str("## L2 — Application\n\n");
    output.push_str("Built-in apps consuming the same SDK as L3.\n\n");
    output.push_str("> Note: per the architecture skill, L2 is `ship-application/` — that crate\n");
    output.push_str("> doesn't exist on disk under that name. Investigation needed during the L2\n");
    output.push_str("> pass to reconcile docs with reality.\n\n");
    output.push_str("<!-- gen-audit-readme: L2-application -->\n");
    output.push_str("| Module | Path | Status | Pass date | Doc |\n");
    output.push_str("|---|---|---|---|---|\n");
    output.push_str("| _(none audited yet — disk reality TBD)_ | | pending | — | — |\n");
    output.push_str("<!-- /gen-audit-readme -->\n\n");

    // Apps table with fence markers.
    output.push_str("## Apps — top-level binaries / surfaces\n\n");
    output.push_str("<!-- gen-audit-readme: apps -->\n");
    output.push_str("| Module | Path | Status | Pass date | Doc |\n");
    output.push_str("|---|---|---|---|---|\n");
    for row in &apps_rows {
        output.push_str(&render_row(row));
        output.push('\n');
    }
    output.push_str("<!-- /gen-audit-readme -->\n\n");

    // Shared table with fence markers.
    output.push_str("## Shared — layer-neutral / cross-cutting crates\n\n");
    output.push_str("<!-- gen-audit-readme: shared -->\n");
    output.push_str("| Module | Path | Status | Pass date | Doc |\n");
    output.push_str("|---|---|---|---|---|\n");
    for row in &shared_rows {
        output.push_str(&render_row(row));
        output.push('\n');
    }
    output.push_str("<!-- /gen-audit-readme -->\n\n");

    // Append remaining sections (Citation sources, Process, Open questions).
    if let Some(pos) = existing.find("## Citation sources") {
        output.push_str(&existing[pos..]);
    }

    // Write the regenerated README.
    std::fs::write(&readme_path, &output)
        .context(format!("write {}", readme_path.display()))?;

    eprintln!("Wrote {}", readme_path.display());
    eprintln!(
        "  L0: {} rows, L1: {} rows, Apps: {} rows, Shared: {} rows",
        l0_rows.len(),
        l1_rows.len(),
        apps_rows.len(),
        shared_rows.len()
    );

    Ok(())
}
