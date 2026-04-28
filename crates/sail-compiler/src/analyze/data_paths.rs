//! Validate `data X = ship.<domain>.<op>(...)` declarations against the
//! `services::schema::OPS` catalog generated into
//! `packages/ship-set/sdk/sdk-ship-ops.json` by `cargo xtask gen-sdk`.
//!
//! Catches typos at compile time. A `data ship.agnts.summries()` becomes
//! a Warning diagnostic with a did-you-mean hint instead of a silent
//! runtime null when the daemon's resolver fails to match.
//!
//! Severity is Warning (not Error) so an out-of-date catalog can't break
//! a build — authors who know the path is real but not yet generated can
//! ship while the SDK regen catches up. Make it Error once the catalog
//! is regenerated automatically on `cargo build`.

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
            severity: Severity::Warning,
            code: "SAIL/W0301".to_string(),
            message: format!(
                "unknown data path `ship.{path}` — not registered in services::schema::OPS",
            ),
            span: None,
            help,
        });
    }
}

/// Extract the dotted path immediately after `ship.` from a data expression.
/// Returns `Some("agents.summaries")` for `ship.agents.summaries()` or
/// `ship.agents.summaries({...})`. Returns `None` if the expr doesn't begin
/// with `ship.<domain>.<op>(`.
fn extract_ship_path(expr: &str) -> Option<String> {
    let trimmed = expr.trim_start();
    let rest = trimmed.strip_prefix("ship.")?;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
            end = i + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let path = &rest[..end];
    // Must contain at least one dot (`domain.op`) and not end in one.
    if !path.contains('.') || path.ends_with('.') {
        return None;
    }
    Some(path.to_string())
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
        let w = d.iter().find(|x| x.code == "SAIL/W0301").expect("expected warning");
        assert_eq!(w.severity, Severity::Warning);
        assert!(w.message.contains("ship.agents.summries"), "msg: {}", w.message);
        let help = w.help.as_deref().unwrap_or("");
        assert!(help.contains("agents.summaries"), "help: {help}");
    }

    #[test]
    fn non_ship_data_expr_ignored() {
        let src = "<script>data x = computed()</script>\n<template>x</template>";
        let d = diag(src);
        assert!(d.iter().all(|x| !x.code.starts_with("SAIL/W0301")), "got: {d:?}");
    }

    #[test]
    fn extracts_path_with_args() {
        assert_eq!(extract_ship_path("ship.agents.detail({ id: 'x' })"), Some("agents.detail".into()));
        assert_eq!(extract_ship_path("ship.agents.list()"), Some("agents.list".into()));
        assert_eq!(extract_ship_path("  ship.git.diff({...})"), Some("git.diff".into()));
        assert_eq!(extract_ship_path("computed()"), None);
        assert_eq!(extract_ship_path("ship.agents"), None); // no parens but also one segment is ok? "agents" alone has no dot
        assert_eq!(extract_ship_path("ship.x.y.z()"), Some("x.y.z".into()));
    }
}
