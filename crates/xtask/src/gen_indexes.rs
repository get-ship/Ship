//! Regenerate the two citation indexes under `.artifacts/specs/op-surface-audit/_indexes/`.
//!
//! - `emit-sites.md`    — every `EventEnvelope::new(` / `.emit(` call site in L0+L1+apps
//! - `substrate-fns.md` — every `pub fn` / `pub async fn` / `pub const fn` in ship-substrate
//!                        and ship-platform (delegated to `gen_indexes_substrate`)
//!
//! Usage:
//!   cargo xtask gen-indexes          # regenerate both indexes
//!   cargo xtask gen-indexes --check  # diff vs on-disk; exit 1 if drift

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;

use crate::gen_indexes_substrate as substrate;

// ── Output paths ─────────────────────────────────────────────────────────────

const EMIT_SITES_PATH: &str =
    ".artifacts/specs/op-surface-audit/_indexes/emit-sites.md";

// ── Source roots ─────────────────────────────────────────────────────────────

const EMIT_ROOTS: &[&str] = &[
    "crates/core/ship-substrate/src",
    "crates/core/ship-substrate/crates/ship-kernel/src",
    "crates/core/ship-substrate/crates/ship-supervisor/src",
    "crates/core/ship-platform/src",
    "crates/core/ship-platform/crates/ship-analytics/src",
    "crates/core/ship-substrate/crates/ship-host/src",
    "apps/shipd/src",
    "apps/ship-cli/src",
];

// ── Emit-site patterns ────────────────────────────────────────────────────────

const EMIT_PATTERNS: &[&str] = &[
    "EventEnvelope::new(",
    ".emit_event_envelope(",
    ".emit_event(",
    "publish_event(",
    ".emit(",
    "kernel.publish(",
    "router.emit(",
    "router.publish(",
    "mailbox.send(",
];

// ── Data structures ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct EmitSite {
    rel_path: String,
    line: usize,
    event_type: String,
    containing_fn: String,
    pattern: String,
    in_all: bool,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let event_types_src = repo_root
        .join("crates/core/ship-substrate/crates/ship-types/src/event_types/mod.rs");
    let (const_map, all_set) = load_event_types(&event_types_src)?;

    let emit_sites = collect_emit_sites(repo_root, &const_map, &all_set)?;
    let substrate_fns = substrate::collect(repo_root)?;

    let emit_md = render_emit_sites(&emit_sites, &all_set);
    let fns_md = substrate::render(&substrate_fns);

    if check {
        run_check(repo_root, &emit_md, &fns_md)
    } else {
        write_file(repo_root, EMIT_SITES_PATH, &emit_md)?;
        write_file(repo_root, substrate::index_path(), &fns_md)?;
        let distinct = emit_sites.iter().map(|s| s.event_type.as_str()).collect::<BTreeSet<_>>().len();
        eprintln!("Wrote {EMIT_SITES_PATH}");
        eprintln!("  {} emit sites, {} distinct event types", emit_sites.len(), distinct);
        eprintln!("Wrote {}", substrate::index_path());
        eprintln!("  {} pub fns", substrate_fns.len());
        Ok(())
    }
}

// ── event_types.rs loader ─────────────────────────────────────────────────────

fn load_event_types(path: &Path) -> Result<(BTreeMap<String, String>, BTreeSet<String>)> {
    let src = std::fs::read_to_string(path)
        .context(format!("read {}", path.display()))?;

    let const_re = Regex::new(r#"^pub const (\w+): &str = "([^"]+)";"#)?;
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut in_all: BTreeSet<String> = BTreeSet::new();
    let mut in_all_section = false;

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub const ALL:") {
            in_all_section = true;
        }
        if in_all_section {
            let name = trimmed.trim_end_matches(',').trim();
            if let Some(val) = map.get(name) {
                in_all.insert(val.clone());
            }
            if trimmed.starts_with("];") {
                in_all_section = false;
            }
        }
        if let Some(caps) = const_re.captures(trimmed) {
            map.insert(caps[1].to_string(), caps[2].to_string());
        }
    }

    Ok((map, in_all))
}

// ── Emit-site collection ──────────────────────────────────────────────────────

fn collect_emit_sites(
    repo_root: &Path,
    const_map: &BTreeMap<String, String>,
    all_set: &BTreeSet<String>,
) -> Result<Vec<EmitSite>> {
    let value_to_const: BTreeMap<&str, &str> = const_map
        .iter()
        .map(|(k, v)| (v.as_str(), k.as_str()))
        .collect();

    let mut sites: Vec<EmitSite> = Vec::new();

    for &root_str in EMIT_ROOTS {
        let root = repo_root.join(root_str);
        let mut files: Vec<PathBuf> = Vec::new();
        substrate::walk_rs_files(&root, &mut files)?;

        for file_path in &files {
            let rel_path = file_path
                .strip_prefix(repo_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let content = std::fs::read_to_string(file_path)
                .context(format!("read {}", file_path.display()))?;

            if is_mostly_test_content(&content) {
                continue;
            }

            let found = extract_emit_sites(&content, &rel_path, const_map, all_set, &value_to_const)?;
            sites.extend(found);
        }
    }

    sites.sort_by(|a, b| a.rel_path.cmp(&b.rel_path).then(a.line.cmp(&b.line)));
    Ok(sites)
}

fn is_mostly_test_content(content: &str) -> bool {
    let mut cfg_test_lines = 0usize;
    let mut total_lines = 0usize;
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        total_lines += 1;
        if t.contains("#[cfg(test)]") || t.contains("#[test]") || t.contains("mod tests") {
            cfg_test_lines += 1;
        }
    }
    total_lines > 0 && cfg_test_lines * 100 / total_lines > 15
}

fn extract_emit_sites(
    content: &str,
    rel_path: &str,
    const_map: &BTreeMap<String, String>,
    all_set: &BTreeSet<String>,
    value_to_const: &BTreeMap<&str, &str>,
) -> Result<Vec<EmitSite>> {
    let mut sites = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut current_fn = String::from("<module>");
    let fn_re = Regex::new(r"\bfn\s+(\w+)\s*[<(]")?;
    let str_lit_re = Regex::new(r#"EventEnvelope::new\s*\(\s*(?:"([^"]+)"|([A-Z_][A-Z0-9_]*))"#)?;
    let const_ref_re = Regex::new(r#"(?:event_types::)?([A-Z_][A-Z0-9_]{3,})"#)?;
    let mut in_cfg_test = 0usize;

    for (idx, &line) in lines.iter().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim();

        if trimmed.contains("#[cfg(test)]") {
            in_cfg_test = in_cfg_test.saturating_add(1);
        }
        if trimmed == "}" && line.starts_with('}') && in_cfg_test > 0 {
            in_cfg_test = in_cfg_test.saturating_sub(1);
        }
        if in_cfg_test > 0 {
            continue;
        }

        if let Some(caps) = fn_re.captures(trimmed) {
            current_fn = caps[1].to_string();
        }

        let matched_pattern = EMIT_PATTERNS.iter().find(|&&p| line.contains(p));
        let Some(pattern) = matched_pattern else {
            continue;
        };

        let window = lines[idx..std::cmp::min(idx + 3, lines.len())].join(" ");

        let event_type = if pattern == &"EventEnvelope::new(" {
            if let Some(caps) = str_lit_re.captures(&window) {
                if let Some(lit) = caps.get(1) {
                    lit.as_str().to_string()
                } else if let Some(ident) = caps.get(2) {
                    let name = ident.as_str();
                    let stripped = name.strip_prefix("event_types::").unwrap_or(name);
                    const_map.get(stripped).cloned().unwrap_or_else(|| name.to_string())
                } else {
                    "<dynamic>".to_string()
                }
            } else {
                extract_const_ref(&window, const_map, &const_ref_re)
                    .unwrap_or_else(|| "<dynamic>".to_string())
            }
        } else {
            extract_any_event_type(&window, const_map, &const_ref_re, value_to_const)
                .unwrap_or_else(|| "<dynamic>".to_string())
        };

        let in_all = all_set.contains(event_type.as_str());

        sites.push(EmitSite {
            rel_path: rel_path.to_string(),
            line: lineno,
            event_type,
            containing_fn: current_fn.clone(),
            pattern: (*pattern).to_string(),
            in_all,
        });
    }

    Ok(sites)
}

fn extract_const_ref(
    window: &str,
    const_map: &BTreeMap<String, String>,
    re: &Regex,
) -> Option<String> {
    for caps in re.captures_iter(window) {
        let name = caps[1].to_string();
        if let Some(val) = const_map.get(&name) {
            return Some(val.clone());
        }
    }
    None
}

fn extract_any_event_type(
    window: &str,
    const_map: &BTreeMap<String, String>,
    const_ref_re: &Regex,
    value_to_const: &BTreeMap<&str, &str>,
) -> Option<String> {
    let lit_re = Regex::new(r#""([a-z][a-z0-9._]+)""#).ok()?;
    for caps in lit_re.captures_iter(window) {
        let candidate = caps[1].to_string();
        if candidate.contains('.') || value_to_const.contains_key(candidate.as_str()) {
            return Some(candidate);
        }
    }
    extract_const_ref(window, const_map, const_ref_re)
}

// ── Emit-sites renderer ───────────────────────────────────────────────────────

fn render_emit_sites(sites: &[EmitSite], all_set: &BTreeSet<String>) -> String {
    let mut out = String::new();

    let distinct_types: BTreeSet<&str> = sites.iter().map(|s| s.event_type.as_str()).collect();
    let not_in_all: Vec<&str> = distinct_types
        .iter()
        .filter(|&&t| t != "<dynamic>" && !all_set.contains(t))
        .copied()
        .collect();
    let pattern_list = EMIT_PATTERNS.join("`, `");

    out.push_str("# Emit-site index\n\n");
    out.push_str("Generated by `cargo xtask gen-indexes`. Every row cites file:line. \
Use this as the citation source for any downstream event-classification or registry work.\n\n");
    out.push_str(&format!("**Total emit-sites:** {}\n", sites.len()));
    out.push_str(&format!(
        "**Total distinct event-types:** {} ({} in ALL · {} not-in-ALL · {} dynamic)\n",
        distinct_types.len(),
        distinct_types.iter().filter(|&&t| all_set.contains(t)).count(),
        not_in_all.len(),
        distinct_types.iter().filter(|&&t| t == "<dynamic>").count(),
    ));

    if !not_in_all.is_empty() {
        out.push_str("\n**[incomplete-event-registry] — emitted but missing from `ALL`:**\n");
        for t in &not_in_all {
            out.push_str(&format!("- `{t}`\n"));
        }
    }

    out.push_str(&format!(
        "\nPatterns scanned: `{pattern_list}`. \
Test files (`tests.rs`, `*_tests.rs`, `tests_*.rs`, `tests/`) and `#[cfg(test)]` blocks excluded.\n"
    ));

    // Group by event type (sorted).
    let mut by_type: BTreeMap<&str, Vec<&EmitSite>> = BTreeMap::new();
    for site in sites {
        by_type.entry(&site.event_type).or_default().push(site);
    }

    out.push_str("\n## By event-type\n");

    for (etype, group) in &by_type {
        out.push_str(&format!("\n### `{etype}`\n"));
        for site in group.iter() {
            let in_all_mark = if site.in_all { "" } else { " ⚠ not-in-ALL" };
            out.push_str(&format!(
                "- `{}:{}` in `fn {}` (`{}`){}\n",
                site.rel_path, site.line, site.containing_fn, site.pattern, in_all_mark
            ));
        }
    }

    out
}

// ── Check mode ────────────────────────────────────────────────────────────────

fn run_check(repo_root: &Path, emit_md: &str, fns_md: &str) -> Result<()> {
    let mut drift = false;
    drift |= check_one(repo_root, EMIT_SITES_PATH, emit_md)?;
    drift |= check_one(repo_root, substrate::index_path(), fns_md)?;

    if drift {
        eprintln!("Index drift detected. Run `cargo xtask gen-indexes` and commit the result.");
        std::process::exit(1);
    }
    eprintln!("Indexes are up-to-date.");
    Ok(())
}

fn check_one(repo_root: &Path, rel: &str, generated: &str) -> Result<bool> {
    let path = repo_root.join(rel);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing == generated {
        return Ok(false);
    }
    eprintln!("DRIFT: {rel}");
    let old_lines = existing.lines().count();
    let new_lines = generated.lines().count();
    let delta = new_lines as i64 - old_lines as i64;
    eprintln!("  on-disk: {old_lines} lines  generated: {new_lines} lines  ({delta:+})");
    Ok(true)
}

// ── File writer ───────────────────────────────────────────────────────────────

fn write_file(repo_root: &Path, rel: &str, content: &str) -> Result<()> {
    let path = repo_root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context(format!("create_dir_all {}", parent.display()))?;
    }
    std::fs::write(&path, content)
        .context(format!("write {}", path.display()))?;
    Ok(())
}
