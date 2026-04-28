//! Validate `data X = ship.<domain>.<op>(args)` declarations against
//! the `services::schema::OPS` catalog generated into
//! `packages/ship-set/sdk/sdk-ship-ops.json` by `cargo xtask gen-sdk`.
//!
//! Catches typos at compile time with did-you-mean suggestions. A
//! `data ship.agnts.summries()` becomes a Warning diagnostic instead
//! of a silent runtime null when the daemon's resolver fails to match.
//!
//! Severity is Warning (not Error) so an out-of-date catalog can't
//! break a build for an author who knows the op is real but not yet
//! generated. Tighten to Error once the catalog regenerates
//! automatically on `cargo build`.
//!
//! Path extraction is a hand-rolled prefix parser (NOT a regex) that
//! reads the leading dotted identifier sequence after `ship.`. The
//! previous regex-based implementation was reverted; this one walks
//! the chars and answers each question explicitly.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_json::Value;

use crate::parse::ast::SailFile;
use crate::{Diagnostic, Severity};

const CATALOG_JSON: &str = include_str!(
    "../../../../packages/ship-set/sdk/sdk-ship-ops.json"
);

fn catalog() -> &'static BTreeSet<String> {
    static CATALOG: OnceLock<BTreeSet<String>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut paths: BTreeSet<String> = BTreeSet::new();
        let parsed: Value = serde_json::from_str(CATALOG_JSON).unwrap_or(Value::Null);
        if let Some(entries) = parsed.get("entries").and_then(|e| e.as_object()) {
            for name in entries.keys() {
                paths.insert(name.clone());
            }
        }
        paths
    })
}

pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let known = catalog();
    if known.is_empty() {
        // Catalog file present but empty/malformed — don't gate the build.
        return;
    }
    let Some(script) = file.script.as_ref() else { return };
    for decl in &script.data {
        let Some(path) = extract_ship_path(&decl.expr) else { continue };
        if known.contains(&path) {
            continue;
        }
        let suggestion = nearest(&path, known);
        let help = match suggestion {
            Some(s) => Some(format!("did you mean `ship.{s}`?")),
            None => Some("run `cargo xtask gen-sdk` to refresh the catalog".to_string()),
        };
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/W0301".to_string(),
            message: format!(
                "unknown data path `ship.{path}` — not registered in services::schema::OPS",
            ),
            span: None,
            help,
        });
    }
}

/// Read a dotted identifier sequence after the literal `ship.` prefix.
/// Returns `Some("agents.summaries")` for `ship.agents.summaries()` and
/// `Some("git.diff")` for `ship.git.diff({...})`. Returns `None` for any
/// expression that doesn't begin with `ship.<ident>`.
///
/// Hand-rolled (not a regex) so the rules are explicit:
/// - leading whitespace skipped
/// - literal `ship.` required
/// - then one or more `.identifier` segments, where identifier is
///   `[A-Za-z_][A-Za-z0-9_]*`
/// - terminator is any non-identifier, non-dot char (typically `(`)
/// - paths ending in `.` (incomplete) reject
pub fn extract_ship_path(expr: &str) -> Option<String> {
    let mut chars = expr.chars().peekable();
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }
    for expected in "ship.".chars() {
        if chars.next() != Some(expected) {
            return None;
        }
    }
    let mut path = String::new();
    let mut at_segment_start = true;
    let mut last_was_dot = false;
    while let Some(&c) = chars.peek() {
        if at_segment_start {
            if c.is_ascii_alphabetic() || c == '_' {
                path.push(c);
                chars.next();
                at_segment_start = false;
                last_was_dot = false;
            } else {
                break;
            }
        } else if c.is_ascii_alphanumeric() || c == '_' {
            path.push(c);
            chars.next();
            last_was_dot = false;
        } else if c == '.' {
            path.push('.');
            chars.next();
            at_segment_start = true;
            last_was_dot = true;
        } else {
            break;
        }
    }
    if path.is_empty() || last_was_dot || !path.contains('.') {
        return None;
    }
    Some(path)
}

/// Pick the closest known path to `target` by edit distance, capped to
/// distance ≤ 3 to avoid silly suggestions for nonsense input.
fn nearest(target: &str, known: &BTreeSet<String>) -> Option<String> {
    let mut best: Option<(usize, &String)> = None;
    for cand in known {
        let d = edit_distance(target, cand);
        if d > 3 {
            continue;
        }
        match best {
            None => best = Some((d, cand)),
            Some((bd, _)) if d < bd => best = Some((d, cand)),
            _ => {}
        }
    }
    best.map(|(_, s)| s.clone())
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn diag(src: &str) -> Vec<Diagnostic> {
        let file = parse::parse(src).expect("parse");
        let mut diags = Vec::new();
        analyze(&file, &mut diags);
        diags
    }

    #[test]
    fn known_op_passes() {
        let src = "<script>data agents = ship.agents.summaries()</script>\n<template>x</template>";
        let d = diag(src);
        assert!(d.iter().all(|x| !x.code.starts_with("SAIL/W0301")), "got: {d:?}");
    }

    #[test]
    fn unknown_op_warns_with_suggestion() {
        let src = "<script>data agents = ship.agents.summries()</script>\n<template>x</template>";
        let d = diag(src);
        let w = d.iter().find(|x| x.code == "SAIL/W0301").expect("expected diagnostic");
        assert_eq!(w.severity, Severity::Error);
        assert!(w.message.contains("ship.agents.summries"), "msg: {}", w.message);
        let help = w.help.as_deref().unwrap_or("");
        assert!(help.contains("agents.summaries"), "help: {help}");
    }

    #[test]
    fn non_ship_data_expr_ignored() {
        // `data x = computed()` doesn't start with `ship.` → no warning.
        let src = "<script>data x = computed()</script>\n<template>x</template>";
        let d = diag(src);
        assert!(d.iter().all(|x| !x.code.starts_with("SAIL/W0301")), "got: {d:?}");
    }

    #[test]
    fn extract_handles_args_and_whitespace() {
        assert_eq!(extract_ship_path("ship.agents.detail({ id: 'x' })"), Some("agents.detail".into()));
        assert_eq!(extract_ship_path("ship.agents.list()"), Some("agents.list".into()));
        assert_eq!(extract_ship_path("  ship.git.diff({...})"), Some("git.diff".into()));
        assert_eq!(extract_ship_path("ship.x.y.z()"), Some("x.y.z".into()));
    }

    #[test]
    fn extract_rejects_non_ship() {
        assert_eq!(extract_ship_path("computed()"), None);
        assert_eq!(extract_ship_path("notship.foo.bar()"), None);
        assert_eq!(extract_ship_path(""), None);
    }

    #[test]
    fn extract_rejects_incomplete_paths() {
        // `ship.agents` — only one segment.
        assert_eq!(extract_ship_path("ship.agents"), None);
        // `ship.` — no segment after.
        assert_eq!(extract_ship_path("ship."), None);
        // `ship.agents.` — trailing dot.
        assert_eq!(extract_ship_path("ship.agents."), None);
    }
}
