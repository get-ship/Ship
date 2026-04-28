//! cleanup_pattern — listener / timer leak detector. Fires `SAIL/E0501`.
//!
//! Counts paired call families in the script block: `addEventListener`
//! vs `removeEventListener` (keyed by string event name), `setInterval`
//! vs `clearInterval`, `setTimeout` vs `clearTimeout`. When add > remove,
//! the component leaks across remounts. The phantom
//! `addEventListener('sail:cleanup', …)` is always an error — the runtime
//! never dispatches it. Heuristic — string contents and comments are
//! ignored, but `setTimeout(fn, ms)` fire-and-forget is flagged by
//! design (treat it as a leak class in mount{}).

use std::collections::HashMap;

use crate::parse::ast::SailFile;
use crate::{Diagnostic, Severity};

const PHANTOM_EVENT: &str = "sail:cleanup";

pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let Some(script) = file.script.as_ref() else { return };
    let stripped = strip_comments(&script.raw);

    let mut adds: HashMap<String, usize> = HashMap::new();
    let mut removes: HashMap<String, usize> = HashMap::new();
    let mut set_intervals = 0usize;
    let mut clear_intervals = 0usize;
    let mut set_timeouts = 0usize;
    let mut clear_timeouts = 0usize;
    let mut phantom_cleanup = false;

    for (call, arg) in find_calls(&stripped) {
        match call {
            "addEventListener" => match arg.as_deref() {
                Some(PHANTOM_EVENT) => phantom_cleanup = true,
                Some(ev) => *adds.entry(ev.to_string()).or_default() += 1,
                None => {}
            },
            "removeEventListener" => {
                if let Some(ev) = arg {
                    *removes.entry(ev).or_default() += 1;
                }
            }
            "setInterval" => set_intervals += 1,
            "clearInterval" => clear_intervals += 1,
            "setTimeout" => set_timeouts += 1,
            "clearTimeout" => clear_timeouts += 1,
            _ => {}
        }
    }

    if phantom_cleanup {
        emit(diagnostics,
            "addEventListener('sail:cleanup', ...) — runtime never dispatches this event".into(),
            "the runtime tears down via the `cleanups` array. Push a closure: `cleanups.push(() => …)` instead of listening for a phantom event".into());
    }
    let mut names: Vec<&String> = adds.keys().collect();
    names.sort();
    for ev in names {
        let added = adds[ev];
        let removed = removes.get(ev).copied().unwrap_or(0);
        if added > removed {
            emit(diagnostics,
                format!("listener for `{ev}` added {added} time(s) but only removed {removed} time(s)"),
                format!("pair every `addEventListener('{ev}', h)` with `removeEventListener('{ev}', h)` — typically via `cleanups.push(() => el.removeEventListener('{ev}', h))`"));
        }
    }
    if set_intervals > clear_intervals {
        emit(diagnostics,
            format!("setInterval called {set_intervals} time(s) but clearInterval only {clear_intervals} time(s)"),
            "save the timer id and call `clearInterval(id)` from `unmount {}` or via `cleanups.push(() => clearInterval(id))`".into());
    }
    if set_timeouts > clear_timeouts {
        emit(diagnostics,
            format!("setTimeout called {set_timeouts} time(s) but clearTimeout only {clear_timeouts} time(s)"),
            "save the timer id and call `clearTimeout(id)` from `unmount {}` or via `cleanups.push(() => clearTimeout(id))`".into());
    }
}

fn emit(diagnostics: &mut Vec<Diagnostic>, message: String, help: String) {
    diagnostics.push(Diagnostic {
        severity: Severity::Error,
        code: "SAIL/E0501".to_string(),
        message,
        span: None,
        help: Some(help),
    });
}

/// Strip `//` and `/* … */` comments; preserve string literals.
fn strip_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < b.len() {
        if is_quote(b[i]) { let end = skip_string(b, i); out.push_str(&src[i..end]); i = end; continue; }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' { i += 1; }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') { i += 1; }
            i = (i + 2).min(b.len());
            continue;
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

/// Yield `(name, first_string_arg)` for each call to a target identifier.
fn find_calls(src: &str) -> Vec<(&'static str, Option<String>)> {
    const TARGETS: &[&str] = &[
        "addEventListener", "removeEventListener",
        "setInterval", "clearInterval", "setTimeout", "clearTimeout",
    ];
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if is_quote(b[i]) { i = skip_string(b, i); continue; }
        if !(b[i].is_ascii_alphabetic() || b[i] == b'_' || b[i] == b'$') { i += 1; continue; }
        let start = i;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'$') { i += 1; }
        let name = &src[start..i];
        let mut j = i;
        while j < b.len() && b[j].is_ascii_whitespace() { j += 1; }
        if j >= b.len() || b[j] != b'(' { continue; }
        let Some(&t) = TARGETS.iter().find(|&&t| t == name) else { continue };
        out.push((t, parse_first_string_arg(src, j + 1)));
        i = j + 1;
    }
    out
}

/// First argument of a call as a string literal, or `None`.
fn parse_first_string_arg(src: &str, pos: usize) -> Option<String> {
    let b = src.as_bytes();
    let mut i = pos;
    while i < b.len() && b[i].is_ascii_whitespace() { i += 1; }
    if i >= b.len() || !is_quote(b[i]) { return None; }
    let q = b[i];
    i += 1;
    let start = i;
    while i < b.len() && b[i] != q {
        if b[i] == b'\\' && i + 1 < b.len() { i += 2; continue; }
        i += 1;
    }
    if i >= b.len() { return None; }
    Some(src[start..i].to_string())
}

fn is_quote(c: u8) -> bool { c == b'"' || c == b'\'' || c == b'`' }

/// Skip past a string literal starting at `pos` (the opening quote).
/// Returns the index after the closing quote (or end of input).
fn skip_string(b: &[u8], pos: usize) -> usize {
    let q = b[pos];
    let mut i = pos + 1;
    while i < b.len() && b[i] != q {
        if b[i] == b'\\' && i + 1 < b.len() { i += 2; continue; }
        i += 1;
    }
    (i + 1).min(b.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn run(src: &str) -> Vec<Diagnostic> {
        let file = parse::parse(src).expect("parse");
        let mut d = Vec::new();
        analyze(&file, &mut d);
        d
    }

    fn errors(d: &[Diagnostic]) -> Vec<&Diagnostic> {
        d.iter().filter(|x| x.code == "SAIL/E0501").collect()
    }

    #[test]
    fn add_with_matching_remove_passes() {
        let d = run(r#"<script>mount { el.addEventListener("keydown", h); cleanups.push(() => el.removeEventListener("keydown", h)) }</script><template>x</template>"#);
        assert!(errors(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn add_without_remove_errors() {
        let d = run(r#"<script>mount { el.addEventListener("click", onClick) }</script><template>x</template>"#);
        let e = errors(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("click"));
        assert!(e[0].message.contains("added 1") && e[0].message.contains("removed 0"));
    }

    #[test]
    fn phantom_sail_cleanup_errors_regardless() {
        let d = run(r#"<script>mount { el.addEventListener("sail:cleanup", h); el.removeEventListener("sail:cleanup", h) }</script><template>x</template>"#);
        assert!(errors(&d).iter().any(|x| x.message.contains("sail:cleanup")), "got {d:?}");
    }

    #[test]
    fn set_interval_with_clear_interval_passes() {
        let d = run(r#"<script>mount { const id = setInterval(tick, 1000); cleanups.push(() => clearInterval(id)) }</script><template>x</template>"#);
        assert!(errors(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn fire_and_forget_set_timeout_errors() {
        let d = run(r#"<script>mount { setTimeout(() => focusFirst(), 50) }</script><template>x</template>"#);
        let e = errors(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("setTimeout"));
    }

    #[test]
    fn balanced_multiple_pairs_pass() {
        let d = run(r#"<script>mount { el.addEventListener('a', x); el.addEventListener('b', y); cleanups.push(() => { el.removeEventListener('a', x); el.removeEventListener('b', y) }) }</script><template>x</template>"#);
        assert!(errors(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn comments_and_strings_dont_count() {
        // String literal mention + `// addEventListener` line must
        // both be ignored; the only real call here is balanced.
        let d = run(r#"<script>const note = "el.addEventListener('keydown', h)"
// el.addEventListener('click', h)
mount { el.addEventListener("input", h); cleanups.push(() => el.removeEventListener("input", h)) }</script><template>x</template>"#);
        assert!(errors(&d).is_empty(), "got {d:?}");
    }
}
