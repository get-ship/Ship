//! Lint `use` statements and bare path expressions for layer-direction violations.
//!
//! Complements `lint_layers.rs`, which checks Cargo-dep declarations. This
//! checker walks Rust source files and flags cross-layer imports that violate
//! architectural intent even when the Cargo dep is legitimate (e.g. an L1 crate
//! that legitimately depends on L0 but should not re-export L0 symbols through
//! a shared crate).
//!
//! ## Rules
//!
//! Shared crates — crates that are neither L0, L1, nor apps — may NOT import:
//! - `ship_platform::*`  (L1)
//! - `ship_substrate::*` (L0)
//! - `ship_kernel::*`    (L0)
//! - `compiler::*`       (L1 — the `compiler` crate, not `sui_compiler`)
//!
//! L0 crates (`ship-substrate`, `ship-kernel`, `ship-substrate-host`) may NOT import:
//! - `ship_platform::*`  (L1)
//!
//! Apps and xtask/test code are exempt (apps sit at the top; xtask is tooling).
//!
//! ## Exemptions
//!
//! - Lines containing `// LAYER_EXCEPTION: <reason>` are explicitly acknowledged.
//! - Code inside `#[cfg(test)]` blocks is exempt (test code may reach any layer).
//! - Doc comments (`///` and `//!`) are never flagged.
//! - Re-exports (`pub use`) obey the same rules as `use` — they are violations.

use std::path::{Path, PathBuf};

// ── Layer assignment ──────────────────────────────────────────────────────────

/// Layer classification of a crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    /// L0 — substrate / kernel. May not import ship_platform.
    L0,
    /// L1 — platform primitives. May import L0 + shared freely.
    L1,
    /// Shared leaf — may not import L0 or L1.
    Shared,
}

/// A crate entry with its source root and layer classification.
struct CrateEntry {
    /// Path from repo root to the crate's `src/` directory.
    src: &'static str,
    layer: Layer,
}

/// Forbidden import prefixes per layer.
///
/// Each entry is `(identifier_prefix_to_match, rule_description)`. The prefix
/// is matched against fully-qualified path segments in source lines, so
/// `ship_substrate::` matches both `use ship_substrate::db` and bare
/// `ship_substrate::db::open_db(...)` call sites.
const L0_FORBIDDEN: &[(&str, &str)] = &[("ship_platform", "L0 crates must not import ship_platform (L1)")];

const SHARED_FORBIDDEN: &[(&str, &str)] = &[
    (
        "ship_platform",
        "shared crates must not import ship_platform (L1)",
    ),
    (
        "ship_substrate",
        "shared crates must not import ship_substrate (L0)",
    ),
    (
        "ship_kernel",
        "shared crates must not import ship_kernel (L0)",
    ),
    (
        "compiler",
        "shared crates must not import compiler (L1) — use ship-types or inline the utility",
    ),
];

/// All crates under scrutiny. Apps and xtask are omitted (Layer::Exempt means
/// they are not walked at all). Add entries here when new crates are introduced.
const CRATES: &[CrateEntry] = &[
    // ── L0 ───────────────────────────────────────────────────────────────────
    CrateEntry { src: "crates/core/ship-substrate/src", layer: Layer::L0 },
    CrateEntry { src: "crates/core/ship-kernel/src",    layer: Layer::L0 },
    CrateEntry { src: "crates/core/ship-substrate-host/src", layer: Layer::L0 },
    // ── L1 ───────────────────────────────────────────────────────────────────
    CrateEntry { src: "crates/core/ship-platform/src",  layer: Layer::L1 },
    CrateEntry { src: "crates/core/compiler/src",       layer: Layer::L1 },
    CrateEntry { src: "crates/core/ship-supervisor/src",layer: Layer::L1 },
    CrateEntry { src: "crates/core/ship-analytics/src", layer: Layer::L1 },
    // ── Shared leaf crates ────────────────────────────────────────────────────
    CrateEntry { src: "crates/core/ship-types/src",     layer: Layer::Shared },
    CrateEntry { src: "crates/core/ship-pipelines/src", layer: Layer::Shared },
    CrateEntry { src: "crates/core/build-info/src",     layer: Layer::Shared },
    CrateEntry { src: "crates/deck/deck/src",           layer: Layer::Shared },
    CrateEntry { src: "crates/deck/deck-host/src",      layer: Layer::Shared },
    CrateEntry { src: "crates/deck/deck-axum/src",      layer: Layer::Shared },
    CrateEntry { src: "crates/deck/deck-core/src",      layer: Layer::Shared },
    CrateEntry { src: "crates/sui/compiler/src",        layer: Layer::Shared },
    CrateEntry { src: "crates/sui/signals/src",         layer: Layer::Shared },
    CrateEntry { src: "crates/sui/lint/src",            layer: Layer::Shared },
];

// ── File walker ───────────────────────────────────────────────────────────────

fn walk_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rs_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

// ── Line-level checks ─────────────────────────────────────────────────────────

/// Returns `true` if the line appears to be inside a `#[cfg(test)]` module.
/// This is a conservative heuristic: we scan back through the collected
/// `cfg_test_depth` state rather than parsing the full AST.
///
/// State: `depth` counts unmatched `{` braces after a `#[cfg(test)]` or
/// `mod tests {` marker. When depth is > 0 the line is considered test code.
fn is_in_test_block(in_test_depth: i32) -> bool {
    in_test_depth > 0
}

/// Returns `true` if `line` is a doc comment (`///` or `//!`).
fn is_doc_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("///") || t.starts_with("//!")
}

/// Returns `true` if `line` is a regular comment (`//` but not a doc comment).
fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
}

/// Returns `true` if the line contains a `LAYER_EXCEPTION` annotation.
fn has_exception(line: &str) -> bool {
    line.contains("LAYER_EXCEPTION:")
}

/// Returns `true` if `line` references the given crate prefix as a path segment.
/// Matches `use prefix::`, `pub use prefix::`, or bare `prefix::` call sites.
/// Does NOT match `sui_compiler::` when `prefix == "compiler"`.
fn line_references_prefix(line: &str, prefix: &str) -> bool {
    // Build the identifier form: hyphen → underscore.
    let id = prefix.replace('-', "_");
    // Pattern: id followed by `::` as a word boundary.
    // We require the character before `id` (if any) is not alphanumeric or `_`
    // so that `sui_compiler::` does not match `compiler`.
    let needle = format!("{id}::");
    let mut haystack = line;
    while let Some(pos) = haystack.find(needle.as_str()) {
        // Check character before match — must not be `_` or alphanumeric.
        let before_ok = pos == 0 || {
            let ch = haystack.as_bytes()[pos - 1] as char;
            !ch.is_alphanumeric() && ch != '_'
        };
        if before_ok {
            return true;
        }
        // Skip past this occurrence and keep searching.
        haystack = &haystack[pos + needle.len()..];
    }
    false
}

// ── Per-file check ────────────────────────────────────────────────────────────

struct Violation {
    file: PathBuf,
    lineno: usize,
    line: String,
    rule: &'static str,
    prefix: &'static str,
}

fn check_file(path: &Path, forbidden: &[(&'static str, &'static str)]) -> Vec<Violation> {
    let Ok(src) = std::fs::read_to_string(path) else {
        return vec![];
    };

    let mut violations = Vec::new();
    // Depth > 0 means we are inside a #[cfg(test)] or `mod tests` block.
    let mut cfg_test_depth: i32 = 0;
    let mut pending_cfg_test = false; // saw #[cfg(test)] on previous line

    for (idx, line) in src.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim_start();

        // Track #[cfg(test)] scope.
        if trimmed.contains("#[cfg(test)]") || (trimmed.starts_with("mod ") && trimmed.contains("tests")) {
            pending_cfg_test = true;
        }
        if pending_cfg_test && trimmed.contains('{') {
            cfg_test_depth += 1;
            pending_cfg_test = false;
        }
        let open = line.chars().filter(|&c| c == '{').count() as i32;
        let close = line.chars().filter(|&c| c == '}').count() as i32;
        // If we just opened the cfg_test block, the braces are already counted.
        if cfg_test_depth > 0 && !pending_cfg_test {
            cfg_test_depth += open - close;
            if cfg_test_depth < 0 {
                cfg_test_depth = 0;
            }
        }

        // Exemptions.
        if is_doc_comment(line) || is_comment(line) {
            continue;
        }
        if is_in_test_block(cfg_test_depth) {
            continue;
        }
        if has_exception(line) {
            continue;
        }

        for &(prefix, rule) in forbidden {
            if line_references_prefix(line, prefix) {
                violations.push(Violation {
                    file: path.to_path_buf(),
                    lineno,
                    line: line.to_string(),
                    rule,
                    prefix,
                });
            }
        }
    }
    violations
}

// ── Public entry point ────────────────────────────────────────────────────────

pub struct Report {
    pub violations: Vec<String>,
    pub warn_only: bool,
}

pub fn run(repo_root: &Path) -> Result<Report, String> {
    let mut all: Vec<Violation> = Vec::new();

    for entry in CRATES {
        let forbidden: &[(&str, &str)] = match entry.layer {
            Layer::L0 => L0_FORBIDDEN,
            Layer::Shared => SHARED_FORBIDDEN,
            // L1 crates have no import-level restrictions.
            Layer::L1 => continue,
        };

        let src_dir = repo_root.join(entry.src);
        if !src_dir.exists() {
            continue;
        }

        for path in walk_rs_files(&src_dir) {
            let violations = check_file(&path, forbidden);
            all.extend(violations);
        }
    }

    // First-run policy: >= 10 violations → warn-only initially.
    let warn_only = all.len() >= 10;

    let mut lines = Vec::new();
    for v in &all {
        let rel = v
            .file
            .strip_prefix(repo_root)
            .unwrap_or(&v.file)
            .display()
            .to_string();
        lines.push(format!(
            "{}:{}: [{}] {} — {}",
            rel, v.lineno, v.prefix, v.line.trim(), v.rule,
        ));
        lines.push(format!(
            "  hint: add `// LAYER_EXCEPTION: <reason>` to this line if intentional"
        ));
    }

    Ok(Report {
        violations: lines,
        warn_only,
    })
}
