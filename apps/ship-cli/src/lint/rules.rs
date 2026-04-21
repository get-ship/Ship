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
    RULE_INLINE_STYLE_BUDGET, RULE_NO_INLINE_SCRIPT, RULE_UNDEFINED_SIGNAL, ViewTarget,
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
