//! View linter — enforce view-authoring discipline.
//!
//! All rule IDs are category-prefixed (`<category>-<descriptor>`). The
//! catalog and stability policy live in
//! `docs/ship-contributing/references/docs/lint-rule-catalog.md`.
//!
//! Categories in use:
//! - `style` — inline `<script>` / `<style>` budget rules.
//! - `views` — Datastar signal hygiene.
//! - `pkg`   — view-manifest pack declarations.
//! - `ast`   — minijinja-AST checks (macros, imports, filters,
//!            context fields, routes, ship-ops).

use std::path::{Path, PathBuf};

pub mod report;
pub mod rules;
pub mod source;

#[cfg(test)]
mod tests;

pub use report::{Category, Finding, Severity};

pub const RULE_NO_INLINE_SCRIPT: &str = "style-no-inline-script";
pub const RULE_INLINE_STYLE_BUDGET: &str = "style-inline-budget";
pub const RULE_UNDEFINED_SIGNAL: &str = "views-undefined-signal";
pub const RULE_MISSING_PACK_DECLARATION: &str = "pkg-missing-pack-declaration";
pub const RULE_STALE_PACK_DECLARATION: &str = "pkg-stale-pack-declaration";
pub const RULE_THEME_TOKEN_REF: &str = "style-theme-token-ref";

pub use rules::ast::{
    RULE_AST_PARSE_ERROR, RULE_AST_UNDECLARED_SIGNAL, RULE_AST_UNDEFINED_CONTEXT,
    RULE_AST_UNKNOWN_FILTER, RULE_AST_UNKNOWN_MACRO, RULE_AST_UNKNOWN_ROUTE,
    RULE_AST_UNUSED_IMPORT,
};
pub use rules::ast::RULE_AST_UNKNOWN_SHIP_OP;
pub use rules::rule_theme_token_ref;

pub const ALL_RULES: &[&str] = &[
    RULE_NO_INLINE_SCRIPT,
    RULE_INLINE_STYLE_BUDGET,
    RULE_UNDEFINED_SIGNAL,
    RULE_MISSING_PACK_DECLARATION,
    RULE_STALE_PACK_DECLARATION,
    RULE_THEME_TOKEN_REF,
    RULE_AST_UNKNOWN_MACRO,
    RULE_AST_UNUSED_IMPORT,
    RULE_AST_UNKNOWN_FILTER,
    RULE_AST_UNDEFINED_CONTEXT,
    RULE_AST_UNDECLARED_SIGNAL,
    RULE_AST_UNKNOWN_ROUTE,
    RULE_AST_PARSE_ERROR,
    RULE_AST_UNKNOWN_SHIP_OP,
];

/// A single view target — the directory plus the `.html.j2` files inside it.
#[derive(Debug, Clone)]
pub struct ViewTarget {
    pub name: String,
    pub dir: PathBuf,
    pub files: Vec<PathBuf>,
}

/// Lint a single view's source string. Returns findings sorted by line.
///
/// `name` is a label for display purposes only (no filesystem access).
/// All rules that require declaration scanning operate on this single source.
/// Pack rules are skipped because they require a manifest file on disk.
pub fn lint_view_source(name: &str, source: &str) -> Vec<Finding> {
    use rules::lint_source_in_memory;
    let mut findings = lint_source_in_memory(name, source);
    findings.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    findings
}

/// Lint a whole view directory (all `.html.j2` files).
/// This is the main entry point for batch CLI and daemon-side scanning.
pub fn lint_view_dir(view_dir: &Path) -> Vec<Finding> {
    let name = view_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let files = collect_view_files(view_dir);
    if files.is_empty() {
        return Vec::new();
    }
    let target = ViewTarget { name, dir: view_dir.to_path_buf(), files };
    let mut findings = rules::lint_view(&target, ALL_RULES);
    findings.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.rule.cmp(&b.rule))
    });
    findings
}

/// Recursively collect every `.html.j2` file under a view directory.
pub fn collect_view_files(dir: &Path) -> Vec<PathBuf> {
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
