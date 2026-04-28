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
//!
//! Span anchoring: comment stripping replaces comment bytes with spaces
//! so byte offsets in the stripped body line up with the original. Each
//! diagnostic's span points at the FIRST relevant call site for the
//! family — narrowed to just the call name. v1 fallback: when only the
//! first call is anchored, multi-call leak counts still aggregate
//! correctly; the LSP highlights the most likely culprit.

use std::collections::HashMap;

use crate::parse::ast::SailFile;
use crate::{Diagnostic, Severity, Span};

const PHANTOM_EVENT: &str = "sail:cleanup";

pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let Some(script) = file.script.as_ref() else { return };
    let stripped = strip_comments_preserve_offsets(&script.raw);
    let body_offset = script.body_byte_offset;

    let mut adds: HashMap<String, Vec<Call>> = HashMap::new();
    let mut removes: HashMap<String, usize> = HashMap::new();
    let mut set_intervals: Vec<Call> = Vec::new();
    let mut clear_intervals = 0usize;
    let mut set_timeouts: Vec<Call> = Vec::new();
    let mut clear_timeouts = 0usize;
    let mut phantom_cleanup: Option<Call> = None;

    for call in find_calls(&stripped) {
        match call.name {
            "addEventListener" => match call.first_arg.as_deref() {
                Some(PHANTOM_EVENT) => {
                    if phantom_cleanup.is_none() {
                        phantom_cleanup = Some(call);
                    }
                }
                Some(ev) => {
                    let key = ev.to_string();
                    adds.entry(key).or_default().push(call);
                }
                None => {}
            },
            "removeEventListener" => {
                if let Some(ev) = call.first_arg {
                    *removes.entry(ev).or_default() += 1;
                }
            }
            "setInterval" => set_intervals.push(call),
            "clearInterval" => clear_intervals += 1,
            "setTimeout" => set_timeouts.push(call),
            "clearTimeout" => clear_timeouts += 1,
            _ => {}
        }
    }

    if let Some(call) = phantom_cleanup {
        emit(diagnostics,
            "addEventListener('sail:cleanup', ...) — runtime never dispatches this event".into(),
            "the runtime tears down via the `cleanups` array. Push a closure: `cleanups.push(() => …)` instead of listening for a phantom event".into(),
            file, body_offset, &call);
    }
    let mut names: Vec<&String> = adds.keys().collect();
    names.sort();
    for ev in names {
        let calls = &adds[ev];
        let added = calls.len();
        let removed = removes.get(ev).copied().unwrap_or(0);
        if added > removed {
            emit(diagnostics,
                format!("listener for `{ev}` added {added} time(s) but only removed {removed} time(s)"),
                format!("pair every `addEventListener('{ev}', h)` with `removeEventListener('{ev}', h)` — typically via `cleanups.push(() => el.removeEventListener('{ev}', h))`"),
                file, body_offset, &calls[0]);
        }
    }
    if set_intervals.len() > clear_intervals {
        emit(diagnostics,
            format!("setInterval called {} time(s) but clearInterval only {} time(s)", set_intervals.len(), clear_intervals),
            "save the timer id and call `clearInterval(id)` from `unmount {}` or via `cleanups.push(() => clearInterval(id))`".into(),
            file, body_offset, &set_intervals[0]);
    }
    if set_timeouts.len() > clear_timeouts {
        emit(diagnostics,
            format!("setTimeout called {} time(s) but clearTimeout only {} time(s)", set_timeouts.len(), clear_timeouts),
            "save the timer id and call `clearTimeout(id)` from `unmount {}` or via `cleanups.push(() => clearTimeout(id))`".into(),
            file, body_offset, &set_timeouts[0]);
    }
}

#[derive(Debug, Clone)]
struct Call {
    name: &'static str,
    /// Body-relative byte offset of the identifier start.
    name_start: usize,
    /// Body-relative byte offset just past the identifier.
    name_end: usize,
    first_arg: Option<String>,
}

fn emit(
    diagnostics: &mut Vec<Diagnostic>,
    message: String,
    help: String,
    file: &SailFile,
    body_offset: usize,
    call: &Call,
) {
    let span: Option<Span> = Some(
        file.source_map
            .span(body_offset + call.name_start, body_offset + call.name_end),
    );
    diagnostics.push(Diagnostic {
        severity: Severity::Error,
        code: "SAIL/E0501".to_string(),
        message,
        span,
        help: Some(help),
    });
}

/// Strip `//` and `/* … */` comments, replacing each comment byte with
/// a space so the resulting string is the same length as the input and
/// indices in the stripped string map 1:1 onto the original body.
fn strip_comments_preserve_offsets(src: &str) -> String {
    let b = src.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < b.len() {
        if is_quote(b[i]) {
            let end = skip_string(b, i);
            out.extend_from_slice(&b[i..end]);
            i = end;
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                out.push(b' ');
                i += 1;
            }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            let end = i + 2 + b[i + 2..]
                .windows(2)
                .position(|w| w == b"*/")
                .map(|p| p + 2)
                .unwrap_or(b.len() - i - 2);
            for k in i..end.min(b.len()) {
                if b[k] == b'\n' { out.push(b'\n'); } else { out.push(b' '); }
            }
            i = end.min(b.len());
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).expect("ascii preserved")
}

/// Yield [`Call`] entries for each call to a target identifier, with
/// the byte range of the identifier and its first string argument.
fn find_calls(src: &str) -> Vec<Call> {
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
        let name_end = i;
        let name = &src[start..name_end];
        let mut j = i;
        while j < b.len() && b[j].is_ascii_whitespace() { j += 1; }
        if j >= b.len() || b[j] != b'(' { continue; }
        let Some(&t) = TARGETS.iter().find(|&&t| t == name) else { continue };
        out.push(Call {
            name: t,
            name_start: start,
            name_end,
            first_arg: parse_first_string_arg(src, j + 1),
        });
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
        assert!(e[0].span.is_some(), "expected span on listener-leak diagnostic");
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
