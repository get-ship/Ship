//! Anti-theater lint — structural enforcement of R18 (no source-grep in tests).
//!
//! R18 bans **test code** from reading a Rust *source* file and asserting on its
//! text. That pattern is test-theater: it proves nothing about behavior, breaks
//! on a rename/reformat, and gives false confidence. This lint bans, inside test
//! code (`#[test]` / `#[cfg(test)]` modules, and `tests/` integration files):
//!
//!   `include_str!("….rs")` **or** `read_to_string(… ".rs" …)`
//!   combined with `.contains(` / `.lines().count()` / `.find(`
//!
//! within the same test region. A lint that reads source to *enforce* this rule
//! is fine — R18 bans TEST code asserting on source text, not tooling.
//!
//! Discipline: known stragglers (beyond what was already culled) are WARN-listed
//! and reported; any NEW occurrence is a HARD ERROR.

use std::path::{Path, PathBuf};

use crate::metadata;

/// Known stragglers (WARN, non-blocking) keyed by a `rel_path` substring. Empty:
/// the prior cull removed every source-grepping test; this lint now guards the
/// future. A NEW hit not matched here hard-fails.
const KNOWN_STRAGGLERS: &[&str] = &[];

struct Hit {
    rel: String,
    lineno: usize,
    line: String,
}

/// Does this line read a `.rs` *source* file (the input half of the anti-pattern)?
fn reads_rs_source(line: &str) -> bool {
    let has_rs = line.contains(".rs\"") || line.contains(".rs'");
    (line.contains("include_str!") && has_rs)
        || (line.contains("read_to_string") && line.contains(".rs"))
}

/// Does this line grep/assert over text (the output half of the anti-pattern)?
fn greps_text(line: &str) -> bool {
    line.contains(".contains(") || line.contains(".lines().count()") || line.contains(".find(")
}

fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//")
}

/// Inspect one contiguous test region. If it both reads a `.rs` source and greps
/// text, every source-read line in the region is a hit.
fn scan_region(rel: &str, region: &[(usize, String)], out: &mut Vec<Hit>) {
    let has_grep = region
        .iter()
        .any(|(_, l)| !is_comment(l) && greps_text(l));
    if !has_grep {
        return;
    }
    for (lineno, line) in region {
        if !is_comment(line) && reads_rs_source(line) {
            out.push(Hit {
                rel: rel.to_string(),
                lineno: *lineno,
                line: line.trim().to_string(),
            });
        }
    }
}

/// Scan a `.rs` file. Integration-test files (`tests/…`) are entirely test code;
/// other files contribute only their `#[cfg(test)]` / `#[test]` regions.
fn scan_file(path: &Path, rel: &str, is_integration_test: bool, out: &mut Vec<Hit>) {
    let Ok(src) = std::fs::read_to_string(path) else {
        return;
    };

    if is_integration_test {
        let region: Vec<(usize, String)> = src
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l.to_string()))
            .collect();
        scan_region(rel, &region, out);
        return;
    }

    // Accumulate contiguous #[cfg(test)] / #[test] regions by brace depth.
    let mut depth: i32 = 0;
    let mut pending = false;
    let mut region: Vec<(usize, String)> = Vec::new();
    for (idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        // A region opener must be a REAL attribute / mod line — match by
        // `starts_with`, never a bare substring, and never inside a comment.
        // (Otherwise this lint's own doc text and string literals mentioning
        // `#[cfg(test)]` would open a phantom region and flag this file.)
        let is_marker = !trimmed.starts_with("//")
            && (trimmed.starts_with("#[cfg(test")
                || trimmed.starts_with("#[test]")
                || (trimmed.starts_with("mod ") && trimmed.contains("test")));
        if is_marker {
            pending = true;
        }
        let was_in = depth > 0;
        if pending && line.contains('{') {
            depth += 1;
            pending = false;
        } else if depth > 0 {
            let open = line.chars().filter(|&c| c == '{').count() as i32;
            let close = line.chars().filter(|&c| c == '}').count() as i32;
            depth += open - close;
            if depth < 0 {
                depth = 0;
            }
        }
        let now_in = depth > 0;
        if now_in {
            region.push((idx + 1, line.to_string()));
        } else if was_in && !now_in {
            // Region just closed — include the closing line then finalize.
            region.push((idx + 1, line.to_string()));
            scan_region(rel, &region, out);
            region.clear();
        }
    }
    if !region.is_empty() {
        scan_region(rel, &region, out);
    }
}

fn walk_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            walk_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

pub fn run(repo_root: &Path) -> Result<(), String> {
    let ws = metadata::load(repo_root)?;
    let mut hits: Vec<Hit> = Vec::new();

    for m in &ws.members {
        // src/ contributes its #[cfg(test)] regions; tests/ is all test code.
        let mut files: Vec<(PathBuf, bool)> = Vec::new();
        let mut src_files = Vec::new();
        walk_rs(&m.dir.join("src"), &mut src_files);
        files.extend(src_files.into_iter().map(|p| (p, false)));
        let mut test_files = Vec::new();
        walk_rs(&m.dir.join("tests"), &mut test_files);
        files.extend(test_files.into_iter().map(|p| (p, true)));

        for (path, is_it) in files {
            let rel = path
                .strip_prefix(&ws.root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            scan_file(&path, &rel, is_it, &mut hits);
        }
    }

    let mut errors = 0;
    let mut warns = 0;
    for h in &hits {
        let known = KNOWN_STRAGGLERS.iter().any(|s| h.rel.contains(s));
        if known {
            warns += 1;
            eprintln!(
                "WARN [R18-KNOWN] {}:{}: test reads .rs source + greps it — known straggler: {}",
                h.rel, h.lineno, h.line,
            );
        } else {
            errors += 1;
            eprintln!(
                "  [R18] {}:{}: test code reads a .rs source file and asserts on its text — \
                 source-grep-in-tests is banned (R18): {}",
                h.rel, h.lineno, h.line,
            );
        }
    }

    if errors > 0 {
        return Err(format!(
            "anti-theater: {errors} new source-grep-in-test occurrence(s) (R18); {warns} known"
        ));
    }
    println!("anti-theater clean (R18): no new source-grep-in-test occurrences; {warns} known-straggler warnings");
    Ok(())
}
