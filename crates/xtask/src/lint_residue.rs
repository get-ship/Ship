//! Residue lint — catches dead / copied source that rots silently.
//!
//! Two checks:
//!   (a) **Orphan `.rs`** — a source file under a crate's `src/` that no module
//!       in that crate references by name (no `mod <stem>;`, no
//!       `#[path = "…<file>"]`, no `include!("…<file>")`). Such a file compiles
//!       into nothing; it's dead weight that drifts out of sync.
//!   (b) **Byte-identical `.rs` across crates** — the same file content living in
//!       two crates. That's a copy-not-move: a fix to one silently misses the
//!       other.
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

/// Known cross-crate duplicate stragglers (WARN). A dup group is "known" when
/// every file in it matches one of these `rel_path` substrings.
const KNOWN_DUPS: &[&str] = &[
    "crates/core/ship-platform/crates/agent-def/src/testutil.rs",
];

/// Files this small are skipped by the dup check (empty / one-line `mod` stubs
/// collide trivially and aren't copy-not-move signal).
const MIN_DUP_BYTES: usize = 40;

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

    if errors > 0 {
        return Err(format!(
            "residue: {errors} new residue violation(s) (orphan/dup); {warns} known stragglers"
        ));
    }
    println!(
        "residue clean: no new orphan/dup .rs ({} members scanned; {warns} known-straggler warnings)",
        ws.members.len(),
    );
    Ok(())
}
