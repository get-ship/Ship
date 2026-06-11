//! Residue lint — catches dead / copied source that rots silently.
//!
//! Three checks:
//!   (a) **Orphan `.rs`** — a source file under a crate's `src/` that no module
//!       in that crate references by name (no `mod <stem>;`, no
//!       `#[path = "…<file>"]`, no `include!("…<file>")`). Such a file compiles
//!       into nothing; it's dead weight that drifts out of sync.
//!   (b) **Byte-identical `.rs` across crates** — the same file content living in
//!       two crates. That's a copy-not-move: a fix to one silently misses the
//!       other.
//!   (c) **Generated `.js` committed to git** — a tracked `.js` file that
//!       contains a bundler preamble (`var __defProp` in the first 5 lines) or
//!       an `AUTO-GENERATED` marker (first 3 lines). Generated JS belongs in
//!       `.gitignore`, not the index; committed bundles bloat history and create
//!       a stale-output hazard. Allowlisted dirs (fixtures, scenarios, migrations,
//!       bindings) and a DOWN-ONLY pinned exception list cover the current
//!       legitimate cases.
//!
//! The orphan check is deliberately conservative (referenced-by-name, not full
//! module-path resolution) so it never false-positives a legitimately-wired file
//! into a hard error — its job is to catch the genuinely dangling, not to police
//! module style.
//!
//! Discipline: existing stragglers are WARN-listed + reported; any NEW orphan or
//! NEW cross-crate duplicate is a HARD ERROR.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::metadata;

/// Known orphan stragglers (WARN), keyed by `rel_path` substring. Empty — the
/// workspace currently has none; this guards the future.
const KNOWN_ORPHANS: &[&str] = &[];

/// Known cross-crate duplicate stragglers (WARN). Empty — the duplicate that
/// used to live at agent-def/src/testutil.rs was resolved (residue sweep); this
/// guards the future. A dup group is "known" only when every file in it matches
/// an entry.
const KNOWN_DUPS: &[&str] = &[];

/// Files this small are skipped by the dup check (empty / one-line `mod` stubs
/// collide trivially and aren't copy-not-move signal).
const MIN_DUP_BYTES: usize = 40;

// ── (c) constants ──────────────────────────────────────────────────────────────

/// Dir-path segments that unconditionally skip a tracked `.js` file from the
/// generated-JS check. Files under these dirs are intentionally generated output
/// (compiler test golden fixtures, tutorial scenario scripts, Cloudflare D1
/// migration driver shims, or tree-sitter language-binding harnesses) and must
/// not be confused with accidentally committed build artefacts.
const GENERATED_JS_SKIP_DIRS: &[&str] = &["tests/fixtures", "scenarios", "migrations", "bindings"];

/// Pinned legitimate exceptions (DOWN-ONLY): tracked `.js` files that carry
/// generation signatures but are committed by explicit operator decision.
/// Entries may only be removed (when the file is deleted or gitignored); new
/// entries require a code-change with a stated rationale update.
///
/// Format: `(repo-relative path substring, rationale)`
const GENERATED_JS_KNOWN: &[(&str, &str)] = &[
    (
        "apps/docs/public/playground/example.js",
        // Bundler preamble ("var __defProp"). Vendored playground spike —
        // operator decision to commit the bundle pending a proper gitignore
        // strategy for the docs playground.
        "vendored spike, operator decision pending",
    ),
    (
        "packages/sui/runtime/fixtures/views/counter/view.js",
        // AUTO-GENERATED marker. Compiler golden fixture for the runtime
        // integration test — intentionally tracked so regressions in the
        // compiler output surface as unexpected diffs.
        "golden fixture",
    ),
];

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

// ── (a) orphan detection ───────────────────────────────────────────────────────

/// Module names declared file-style (`mod foo;`) anywhere in `text`.
fn collect_mod_decls(text: &str, names: &mut HashSet<String>) {
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        // Find `mod <ident> ;` — a file-backed module declaration (no body).
        if let Some(rest) = find_mod_decl(t) {
            names.insert(rest);
        }
    }
}

/// Returns the module name if `t` is `… mod <ident> ;` (declaration, not a body).
fn find_mod_decl(t: &str) -> Option<String> {
    let idx = t.find("mod ")?;
    // Require `mod ` to start a token (preceded by start or whitespace/`b`-vis).
    let before_ok = idx == 0 || {
        let ch = t.as_bytes()[idx - 1] as char;
        ch.is_whitespace()
    };
    if !before_ok {
        return None;
    }
    let after = &t[idx + 4..];
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }
    let tail = after[name.len()..].trim_start();
    if tail.starts_with(';') {
        Some(name)
    } else {
        None
    }
}

/// Basenames referenced by `#[path = "…"]` or `include!("…")` anywhere in `text`.
fn collect_ref_basenames(text: &str, bases: &mut HashSet<String>) {
    for line in text.lines() {
        for marker in ["#[path", "path =", "include!"] {
            if let Some(p) = line.find(marker) {
                let rest = &line[p..];
                if let Some(q1) = rest.find('"') {
                    if let Some(q2) = rest[q1 + 1..].find('"') {
                        let s = &rest[q1 + 1..q1 + 1 + q2];
                        if let Some(base) = Path::new(s).file_name().and_then(|b| b.to_str()) {
                            bases.insert(base.to_string());
                        }
                    }
                }
            }
        }
    }
}

fn find_orphans(m: &metadata::Member, root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    walk_rs(&m.src_dir, &mut files);
    if files.is_empty() {
        return Vec::new();
    }

    let entry_srcs: HashSet<PathBuf> = m
        .target_src_paths
        .iter()
        .filter_map(|p| std::fs::canonicalize(p).ok())
        .collect();

    let mut decl_names: HashSet<String> = HashSet::new();
    let mut ref_bases: HashSet<String> = HashSet::new();
    for f in &files {
        if let Ok(txt) = std::fs::read_to_string(f) {
            collect_mod_decls(&txt, &mut decl_names);
            collect_ref_basenames(&txt, &mut ref_bases);
        }
    }

    let mut orphans = Vec::new();
    for f in &files {
        let base = f.file_name().and_then(|b| b.to_str()).unwrap_or("");
        if matches!(base, "lib.rs" | "main.rs" | "build.rs") {
            continue;
        }
        if std::fs::canonicalize(f).map(|c| entry_srcs.contains(&c)).unwrap_or(false) {
            continue;
        }
        let module_name = if base == "mod.rs" {
            f.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            base.trim_end_matches(".rs").to_string()
        };
        let referenced = decl_names.contains(&module_name) || ref_bases.contains(base);
        if !referenced {
            let rel = f.strip_prefix(root).unwrap_or(f).to_string_lossy().replace('\\', "/");
            orphans.push(rel);
        }
    }
    orphans
}

// ── (b) cross-crate byte-identical detection ───────────────────────────────────

fn hash_bytes(b: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

// ── (c) generated-JS detection ────────────────────────────────────────────────

/// Returns repo-relative paths of every git-tracked `.js` file that:
/// - is **not** under an allowlisted directory segment, AND
/// - contains a generation signature: `var __defProp` in the first 5 lines
///   (bundler preamble) OR `AUTO-GENERATED` anywhere in the first 3 lines.
fn check_generated_js(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .arg("ls-files")
        .arg("*.js")
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to spawn `git ls-files`: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git ls-files` failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut violations = Vec::new();

    for rel in stdout.lines() {
        let rel = rel.trim();
        if rel.is_empty() {
            continue;
        }
        // Skip allowlisted directory families (fixtures, scenarios, migrations,
        // bindings): they intentionally host generated or framework-provided JS.
        if GENERATED_JS_SKIP_DIRS.iter().any(|seg| rel.contains(seg)) {
            continue;
        }
        let abs = repo_root.join(rel);
        let Ok(content) = std::fs::read_to_string(&abs) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        // Bundler preamble — esbuild/Rolldown emit `var __defProp` in line 1–5.
        let has_bundler_preamble = lines.iter().take(5).any(|l| l.contains("var __defProp"));
        // Auto-generated header — sui-compiler and similar tools write
        // "AUTO-GENERATED" in a comment at the very top.
        let has_autogen_marker = lines.iter().take(3).any(|l| l.contains("AUTO-GENERATED"));
        if has_bundler_preamble || has_autogen_marker {
            violations.push(rel.to_string());
        }
    }

    Ok(violations)
}

pub fn run(repo_root: &Path) -> Result<(), String> {
    let ws = metadata::load(repo_root)?;

    let mut errors = 0usize;
    let mut warns = 0usize;

    // ── (a) orphans ──────────────────────────────────────────────────────────
    let mut all_orphans: Vec<String> = Vec::new();
    for m in &ws.members {
        all_orphans.extend(find_orphans(m, &ws.root));
    }
    all_orphans.sort();
    for rel in &all_orphans {
        if KNOWN_ORPHANS.iter().any(|s| rel.contains(s)) {
            warns += 1;
            eprintln!("WARN [RESIDUE-ORPHAN-KNOWN] {rel}: not referenced by any module — known straggler");
        } else {
            errors += 1;
            eprintln!(
                "  [RESIDUE-ORPHAN] {rel}: .rs file not reachable from any crate mod tree \
                 (no `mod`, `#[path]`, or `include!` references it) — dead source; wire it or delete it"
            );
        }
    }

    // ── (b) cross-crate duplicates ───────────────────────────────────────────
    // Collect (rel_path, crate_name, content_bytes) for src/ + tests/.
    let mut buckets: HashMap<u64, Vec<(String, String, Vec<u8>)>> = HashMap::new();
    for m in &ws.members {
        let mut files = Vec::new();
        walk_rs(&m.dir.join("src"), &mut files);
        walk_rs(&m.dir.join("tests"), &mut files);
        for f in files {
            let Ok(bytes) = std::fs::read(&f) else { continue };
            if bytes.len() < MIN_DUP_BYTES {
                continue;
            }
            let rel = f.strip_prefix(&ws.root).unwrap_or(&f).to_string_lossy().replace('\\', "/");
            buckets
                .entry(hash_bytes(&bytes))
                .or_default()
                .push((rel, m.name.clone(), bytes));
        }
    }

    let mut dup_groups: Vec<Vec<(String, String)>> = Vec::new();
    for (_h, entries) in buckets {
        if entries.len() < 2 {
            continue;
        }
        // Partition the hash bucket into true byte-identical sub-groups.
        let mut used = vec![false; entries.len()];
        for i in 0..entries.len() {
            if used[i] {
                continue;
            }
            let mut group = vec![(entries[i].0.clone(), entries[i].1.clone())];
            used[i] = true;
            for j in (i + 1)..entries.len() {
                if !used[j] && entries[j].2 == entries[i].2 {
                    used[j] = true;
                    group.push((entries[j].0.clone(), entries[j].1.clone()));
                }
            }
            // Only flag when the identical content spans ≥2 distinct crates.
            let distinct_crates: HashSet<&String> = group.iter().map(|(_, c)| c).collect();
            if group.len() >= 2 && distinct_crates.len() >= 2 {
                dup_groups.push(group);
            }
        }
    }
    dup_groups.sort();
    for group in &dup_groups {
        let known = group.iter().all(|(rel, _)| KNOWN_DUPS.iter().any(|s| rel.contains(s)));
        let listed = group
            .iter()
            .map(|(rel, c)| format!("{rel} ({c})"))
            .collect::<Vec<_>>()
            .join(" == ");
        if known {
            warns += 1;
            eprintln!("WARN [RESIDUE-DUP-KNOWN] byte-identical across crates — known straggler: {listed}");
        } else {
            errors += 1;
            eprintln!(
                "  [RESIDUE-DUP] byte-identical .rs across crates (copy-not-move) — hoist the \
                 shared content into one crate and depend on it: {listed}"
            );
        }
    }

    // ── (c) generated-JS guard ──────────────────────────────────────────────
    let mut gen_js = check_generated_js(&ws.root)
        .map_err(|e| format!("generated-js check: {e}"))?;
    gen_js.sort();

    for rel in &gen_js {
        if GENERATED_JS_KNOWN.iter().any(|(s, _)| rel.contains(s)) {
            warns += 1;
            eprintln!(
                "WARN [RESIDUE-GENERATED-JS-KNOWN] {rel}: generated JS committed — known exception"
            );
        } else {
            errors += 1;
            eprintln!(
                "  [RESIDUE-GENERATED-JS] {rel}: generated JS must be gitignored, not committed \
                 (bundler preamble `var __defProp` or `AUTO-GENERATED` marker detected in first lines)"
            );
        }
    }

    if errors > 0 {
        return Err(format!(
            "residue: {errors} new residue violation(s) (orphan/dup/generated-js); {warns} known stragglers"
        ));
    }
    println!(
        "residue clean: no new orphan/dup .rs, no unallowlisted generated-js \
         ({} members scanned; {warns} known-straggler warnings)",
        ws.members.len(),
    );
    Ok(())
}
