//! Rule implementations.
//!
//! Each rule takes the pre-loaded source list (see `source.rs`) and returns
//! findings. Public entry is `lint_view`, which dispatches to each rule
//! conditionally based on the enabled set.

use std::collections::HashSet;
use std::path::PathBuf;

use super::report::{Finding, Severity};
use super::source::{self, Source};
use super::{
    RULE_INLINE_STYLE_BUDGET, RULE_MISSING_PACK_DECLARATION, RULE_NO_INLINE_SCRIPT,
    RULE_STALE_PACK_DECLARATION, RULE_UNDEFINED_SIGNAL, ViewTarget,
};

const STYLE_WARN_THRESHOLD: usize = 50;
const STYLE_ERROR_THRESHOLD: usize = 150;

/// Signals Datastar injects automatically — referencing them is never an error.
/// `_` is the scratch signal, `evt` is the current event in `data-on:*` exprs.
const DATASTAR_INTRINSIC_SIGNALS: &[&str] = &["_", "evt"];

pub fn lint_view(view: &ViewTarget, enabled: &[&'static str]) -> Vec<Finding> {
    let mut out = Vec::new();

    let sources: Vec<Source> = view
        .files
        .iter()
        .filter_map(|p| source::read_source(p))
        .collect();

    if enabled.contains(&RULE_NO_INLINE_SCRIPT) {
        for src in &sources {
            out.extend(rule_no_inline_script(src));
        }
    }

    if enabled.contains(&RULE_INLINE_STYLE_BUDGET) {
        out.extend(rule_inline_style_budget(view, &sources));
    }

    if enabled.contains(&RULE_UNDEFINED_SIGNAL) {
        out.extend(rule_undefined_signal(&sources));
    }

    if enabled.contains(&RULE_MISSING_PACK_DECLARATION)
        || enabled.contains(&RULE_STALE_PACK_DECLARATION)
    {
        out.extend(rule_pack_declarations(view, &sources, enabled));
    }

    out
}

// ── Rule 1 — no-inline-script ─────────────────────────────────────────────────

fn rule_no_inline_script(src: &Source) -> Vec<Finding> {
    let mut out = Vec::new();
    let re = source::script_open_regex();
    for (idx, line) in src.lines.iter().enumerate() {
        for m in re.find_iter(line) {
            out.push(Finding {
                file: src.path.clone(),
                line: idx + 1,
                col: m.start() + 1,
                rule: RULE_NO_INLINE_SCRIPT,
                severity: Severity::Error,
                message: "inline <script> block (use data-effect or extend an element)"
                    .to_string(),
            });
        }
    }
    out
}

// ── Rule 2 — inline-style-budget ──────────────────────────────────────────────

fn rule_inline_style_budget(view: &ViewTarget, sources: &[Source]) -> Vec<Finding> {
    let open_re = source::style_open_regex();
    let close_re = source::style_close_regex();

    let mut total: usize = 0;
    // Remember the first style-open location so the finding points somewhere
    // actionable rather than at the view directory in the abstract.
    let mut first_loc: Option<(PathBuf, usize, usize)> = None;

    for src in sources {
        let mut in_style = false;
        for (idx, line) in src.lines.iter().enumerate() {
            if !in_style {
                if let Some(m) = open_re.find(line) {
                    in_style = true;
                    if first_loc.is_none() {
                        first_loc = Some((src.path.clone(), idx + 1, m.start() + 1));
                    }
                    let after = &line[m.end()..];
                    if close_re.is_match(after) {
                        in_style = false;
                    }
                    continue;
                }
            } else {
                if close_re.is_match(line) {
                    in_style = false;
                    continue;
                }
                total += 1;
            }
        }
    }

    if total <= STYLE_WARN_THRESHOLD {
        return Vec::new();
    }
    let (severity, label, threshold) = if total > STYLE_ERROR_THRESHOLD {
        (Severity::Error, "exceeds", STYLE_ERROR_THRESHOLD)
    } else {
        (Severity::Warn, "exceeds warn threshold of", STYLE_WARN_THRESHOLD)
    };
    let (file, line, col) = first_loc.unwrap_or_else(|| (view.dir.clone(), 1, 1));
    vec![Finding {
        file,
        line,
        col,
        rule: RULE_INLINE_STYLE_BUDGET,
        severity,
        message: format!(
            "view '{}' has {} lines of inline <style>, {} {} lines (extract into an element or macro)",
            view.name, total, label, threshold,
        ),
    }]
}

// ── Rule 3 — undefined-signal ─────────────────────────────────────────────────

fn rule_undefined_signal(sources: &[Source]) -> Vec<Finding> {
    let (declared, allowlist) = collect_declared_signals(sources);

    let mut out = Vec::new();
    let mut reported: HashSet<(PathBuf, usize, String)> = HashSet::new();
    for src in sources {
        for (idx, line) in src.lines.iter().enumerate() {
            for (col, name) in collect_signal_refs(line) {
                if declared.contains(&name) || allowlist.contains(&name) {
                    continue;
                }
                if DATASTAR_INTRINSIC_SIGNALS.contains(&name.as_str()) {
                    continue;
                }
                let key = (src.path.clone(), idx + 1, name.clone());
                if !reported.insert(key) {
                    continue;
                }
                out.push(Finding {
                    file: src.path.clone(),
                    line: idx + 1,
                    col: col + 1,
                    rule: RULE_UNDEFINED_SIGNAL,
                    severity: Severity::Warn,
                    message: format!(
                        "signal ${} referenced but never declared (add signal('{}', …) or allow via {{#- @lint:signals-ok: {} -#}})",
                        name, name, name,
                    ),
                });
            }
        }
    }
    out
}

fn collect_declared_signals(sources: &[Source]) -> (HashSet<String>, HashSet<String>) {
    let mut declared: HashSet<String> = HashSet::new();
    let mut allowlist: HashSet<String> = HashSet::new();

    let macro_re = source::signal_macro_regex();
    let attr_re = source::data_signals_attr_regex();
    let obj_re = source::data_signals_object_regex();

    for src in sources {
        allowlist.extend(src.signals_ok.iter().cloned());
        for line in &src.lines {
            for cap in macro_re.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    declared.insert(m.as_str().to_string());
                }
            }
            for cap in attr_re.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    declared.insert(m.as_str().to_string());
                }
            }
            for cap in obj_re.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    for name in source::extract_object_keys(m.as_str()) {
                        declared.insert(name);
                    }
                }
            }
        }
    }
    (declared, allowlist)
}

/// Collect `$name` references from double-quoted attribute values on a line,
/// returning `(column, name)` pairs.
///
/// Pragmatic scope: only double-quoted attribute values. Unquoted references
/// barely occur; single-quoted ones are rare in the view corpus.
fn collect_signal_refs(line: &str) -> Vec<(usize, String)> {
    let attr_re = source::quoted_attr_value_regex();
    let ref_re = source::signal_ref_regex();
    let mut out = Vec::new();

    for cap in attr_re.captures_iter(line) {
        let Some(value_match) = cap.get(1) else {
            continue;
        };
        let value_start = value_match.start();
        let value = value_match.as_str();
        for ref_cap in ref_re.captures_iter(value) {
            let Some(name) = ref_cap.get(1) else { continue };
            // Skip `$foo` inside Jinja `{{ … }}` interpolations — those are
            // the author's value references, not Datastar signals.
            let rel_start = ref_cap.get(0).map(|m| m.start()).unwrap_or(0);
            if is_inside_jinja_expr(value, rel_start) {
                continue;
            }
            out.push((value_start + rel_start, name.as_str().to_string()));
        }
    }
    out
}

/// Coarse check — true if the nearest `{{` to the left of `pos` isn't
/// balanced by a `}}` before `pos`. Enough to avoid false positives on
/// `data-x="{{ loop.index }}$foo"` where `$foo` is outside the interpolation.
fn is_inside_jinja_expr(value: &str, pos: usize) -> bool {
    let before = &value[..pos];
    let last_open = before.rfind("{{");
    let Some(open) = last_open else { return false };
    let last_close = before.rfind("}}");
    match last_close {
        Some(close) => close < open,
        None => true,
    }
}

// ── Rules 4 & 5 — missing-pack-declaration / stale-pack-declaration ───────────

/// Known packs → heavy-island tags the pack provides. Mirror of the daemon's
/// `views::packs::pack_bundles` map in pack→tag direction. Kept in the
/// linter so the rule has no runtime dependency on the daemon crate.
const PACKS: &[(&str, &[&str])] = &[
    ("editor", &["ship-code-editor", "ship-markdown-editor"]),
    ("terminal", &["ship-terminal"]),
];

fn rule_pack_declarations(
    view: &ViewTarget,
    sources: &[Source],
    enabled: &[&'static str],
) -> Vec<Finding> {
    let mut out = Vec::new();

    // Collect every heavy-island tag referenced in the templates.
    let tag_open_re = pack_tag_regex();
    let mut used_tags: HashSet<String> = HashSet::new();
    // Track (pack, first-seen-location) so a missing declaration points at
    // the template that triggered the requirement, not the view root.
    let mut first_tag_loc: std::collections::HashMap<&'static str, (PathBuf, usize, usize)> =
        std::collections::HashMap::new();

    for src in sources {
        for (idx, line) in src.lines.iter().enumerate() {
            for cap in tag_open_re.captures_iter(line) {
                let Some(tag_match) = cap.get(1) else { continue };
                let tag = tag_match.as_str();
                used_tags.insert(tag.to_string());
                if let Some(pack) = pack_for_tag(tag) {
                    first_tag_loc
                        .entry(pack)
                        .or_insert_with(|| (src.path.clone(), idx + 1, tag_match.start() + 1));
                }
            }
        }
    }

    let declared = match read_manifest_packs(&view.dir) {
        Ok(list) => list,
        Err(_) => Vec::new(),
    };

    // Rule: missing-pack-declaration
    if enabled.contains(&RULE_MISSING_PACK_DECLARATION) {
        for &(pack, tags) in PACKS {
            let view_uses_pack = tags.iter().any(|t| used_tags.contains(*t));
            if view_uses_pack && !declared.iter().any(|d| d == pack) {
                let (file, line, col) = first_tag_loc
                    .get(pack)
                    .cloned()
                    .unwrap_or_else(|| (view.dir.clone(), 1, 1));
                let used_list = tags
                    .iter()
                    .filter(|t| used_tags.contains(**t))
                    .map(|t| format!("<{t}>"))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push(Finding {
                    file,
                    line,
                    col,
                    rule: RULE_MISSING_PACK_DECLARATION,
                    severity: Severity::Error,
                    message: format!(
                        "view '{}' uses {} but manifest does not declare `packs: [{}]` — element will not upgrade without the bundle",
                        view.name, used_list, pack,
                    ),
                });
            }
        }
    }

    // Rule: stale-pack-declaration
    if enabled.contains(&RULE_STALE_PACK_DECLARATION) {
        for pack in &declared {
            let Some(tags) = PACKS
                .iter()
                .find(|(name, _)| *name == pack.as_str())
                .map(|(_, tags)| *tags)
            else {
                // Unknown pack — not our rule's job (daemon ignores it).
                continue;
            };
            let view_uses_pack = tags.iter().any(|t| used_tags.contains(*t));
            if !view_uses_pack {
                out.push(Finding {
                    file: view.dir.join(manifest_filename(&view.dir)),
                    line: 1,
                    col: 1,
                    rule: RULE_STALE_PACK_DECLARATION,
                    severity: Severity::Warn,
                    message: format!(
                        "view '{}' declares `packs: [{}]` but no template uses its tags — drop the declaration or the view loads unused bundles",
                        view.name, pack,
                    ),
                });
            }
        }
    }

    out
}

fn pack_for_tag(tag: &str) -> Option<&'static str> {
    for &(pack, tags) in PACKS {
        if tags.contains(&tag) {
            return Some(pack);
        }
    }
    None
}

fn pack_tag_regex() -> regex::Regex {
    // `<ship-name` followed by whitespace, `>`, or `/`. Capturing just the
    // tag name keeps the match cheap. Case-insensitive for defensive parity
    // with HTML's tag name rules even though Ship tags are always lower.
    regex::Regex::new(r"(?i)<(ship-[a-z0-9-]+)(?:[\s/>])").unwrap()
}

/// Read the `packs:` list from a view manifest. Returns the declared packs
/// as lower-case strings, or an empty vec when the manifest is missing or
/// has no packs field.
///
/// Only `manifest.yaml`, `manifest.jsonc`, and `manifest.json` are
/// recognised — mirrors the daemon's resolution order so the linter can't
/// disagree with what the runtime actually reads.
fn read_manifest_packs(view_dir: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let yaml = view_dir.join("manifest.yaml");
    let jsonc = view_dir.join("manifest.jsonc");
    let json = view_dir.join("manifest.json");

    if yaml.exists() {
        let raw = std::fs::read_to_string(&yaml)?;
        let parsed: serde_yaml::Value = serde_yaml::from_str(&raw)?;
        return Ok(extract_packs(&value_from_yaml(&parsed)));
    }
    if jsonc.exists() {
        let raw = std::fs::read_to_string(&jsonc)?;
        let stripped = strip_jsonc_comments(&raw);
        let parsed: serde_json::Value = serde_json::from_str(&stripped)?;
        return Ok(extract_packs(&parsed));
    }
    if json.exists() {
        let raw = std::fs::read_to_string(&json)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)?;
        return Ok(extract_packs(&parsed));
    }
    Ok(Vec::new())
}

fn extract_packs(value: &serde_json::Value) -> Vec<String> {
    value
        .get("packs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Coerce `serde_yaml::Value` to `serde_json::Value` so both manifest
/// formats hit the same `extract_packs` path. `serde_yaml` preserves JSON
/// numeric and string shapes, so a direct `to_value(...)` round-trip works.
fn value_from_yaml(v: &serde_yaml::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

/// Minimal JSONC comment stripper — removes `//` line comments and `/* */`
/// block comments while preserving string literal contents. Sufficient for
/// the linter; the daemon uses `compiler::jsonc::strip_jsonc_comments`
/// which this tracks in behavior but does not depend on (keeps the linter
/// crate free of compiler deps).
fn strip_jsonc_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut in_string = false;
    let mut string_delim = b'"';
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            out.push(c as char);
            if c == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if c == string_delim {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            in_string = true;
            string_delim = c;
            out.push(c as char);
            i += 1;
            continue;
        }
        // Line comment
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // Block comment
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        out.push(c as char);
        i += 1;
    }
    out
}

fn manifest_filename(view_dir: &std::path::Path) -> &'static str {
    if view_dir.join("manifest.yaml").exists() {
        "manifest.yaml"
    } else if view_dir.join("manifest.jsonc").exists() {
        "manifest.jsonc"
    } else {
        "manifest.json"
    }
}
