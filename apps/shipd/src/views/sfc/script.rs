//! Script-block analysis for `.ship` SFCs.
//!
//! The script block is **not transformed** here — it is preserved
//! verbatim for inlining into the emitted custom-element class. This
//! module's job is *schema extraction*: given the raw JS, return the
//! prop list (with defaults), the per-instance signal names, and a
//! count of `onConnect` / `onDisconnect` calls (so the emitter can
//! decide whether to wire the lifecycle plumbing).
//!
//! ## Why a hand-rolled scanner instead of a JS AST parser
//!
//! The spike report locked in `oxc_parser`. Pulling oxc into `shipd`
//! adds many transitive crates for what amounts to two pattern matches
//! on top-level declarations. The scanner here recognizes the exact
//! shapes the SFC contract documents:
//!
//! - `const { name = default, ... } = props`     — prop schema
//! - `const $name = signal(initial)`             — signal binding
//! - `onConnect(...)` / `onDisconnect(...)`      — lifecycle hooks
//!
//! Anything more elaborate (computed defaults that span multiple
//! statements, reassignment, dynamic destructure) is intentionally not
//! supported: the framework contract says "use the documented shapes."
//! When `oxc_parser` becomes worth its weight (e.g. for the LSP's
//! completion provider), this module's `extract` function can be
//! re-implemented against an AST without changing its caller surface.
//!
//! Low-level byte scanning helpers (comment/string stripper, brace
//! matcher, destructure splitter) live in [`super::script_scan`] to
//! keep this file focused.

use super::script_scan::{
    find_matching_open, find_prev_nonspace, parse_destructure, strip_comments_and_strings,
};

/// A single prop declared via `const { name = default } = props`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropDecl {
    /// The destructured identifier (`fetchUrl`, `count`, …).
    pub name: String,
    /// Default value source, verbatim, if one was given (`'Search...'`,
    /// `0`, `[]`). `None` when the prop has no default.
    pub default: Option<String>,
}

/// Result of analyzing a script block.
#[derive(Debug, Clone, Default)]
pub struct ScriptSchema {
    pub props: Vec<PropDecl>,
    /// Names of `const $x = signal(...)` bindings, **without** the
    /// leading `$`. (`$query` → `"query"`.)
    pub signals: Vec<String>,
    /// Count of top-level `onConnect(` invocations.
    pub on_connect_calls: usize,
    /// Count of top-level `onDisconnect(` invocations.
    pub on_disconnect_calls: usize,
}

/// Extract the prop / signal / lifecycle schema from a script body.
pub fn extract(source: &str) -> ScriptSchema {
    let cleaned = strip_comments_and_strings(source);
    ScriptSchema {
        props: extract_props(&cleaned, source),
        signals: extract_signals(&cleaned),
        on_connect_calls: count_calls(&cleaned, "onConnect"),
        on_disconnect_calls: count_calls(&cleaned, "onDisconnect"),
    }
}

fn extract_props(cleaned: &str, original: &str) -> Vec<PropDecl> {
    let bytes = cleaned.as_bytes();
    let needle = b"= props";
    let mut out = Vec::new();
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let after = bytes.get(i + needle.len()).copied().unwrap_or(b'\n');
            if after.is_ascii_alphanumeric() || after == b'_' || after == b'.' {
                i += 1;
                continue;
            }
            if let Some(close_brace) = find_prev_nonspace(bytes, i.saturating_sub(1))
                && bytes[close_brace] == b'}'
                && let Some(open_brace) = find_matching_open(bytes, close_brace)
            {
                let inner = &original[open_brace + 1..close_brace];
                parse_destructure(inner, &mut out);
            }
            i += needle.len();
            continue;
        }
        i += 1;
    }
    out
}

fn extract_signals(cleaned: &str) -> Vec<String> {
    let bytes = cleaned.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if i + 5 < bytes.len() && &bytes[i..i + 6] == b"const " {
            let mut j = i + 6;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
            if j < bytes.len() && bytes[j] == b'$' {
                let name_start = j + 1;
                let mut k = name_start;
                while k < bytes.len()
                    && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                {
                    k += 1;
                }
                if k > name_start {
                    let mut m = k;
                    while m < bytes.len() && bytes[m].is_ascii_whitespace() { m += 1; }
                    if m < bytes.len()
                        && bytes[m] == b'='
                        && let Some(rest) = cleaned.get(m + 1..)
                        && rest.trim_start().starts_with("signal(")
                    {
                        let name = std::str::from_utf8(&bytes[name_start..k])
                            .unwrap_or("")
                            .to_string();
                        if !name.is_empty() && !out.contains(&name) {
                            out.push(name);
                        }
                    }
                }
                i = k;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn count_calls(cleaned: &str, name: &str) -> usize {
    let bytes = cleaned.as_bytes();
    let nb = name.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i + nb.len() < bytes.len() {
        if &bytes[i..i + nb.len()] == nb {
            let prev_ok = i == 0
                || (!bytes[i - 1].is_ascii_alphanumeric()
                    && bytes[i - 1] != b'_'
                    && bytes[i - 1] != b'$'
                    && bytes[i - 1] != b'.');
            let mut j = i + nb.len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
            let follow_ok = j < bytes.len() && bytes[j] == b'(';
            if prev_ok && follow_ok {
                count += 1;
            }
            i += nb.len();
            continue;
        }
        i += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_props_with_defaults() {
        let src = "const { count = 0, label = 'Hi', items = [] } = props";
        let s = extract(src);
        let names: Vec<_> = s.props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["count", "label", "items"]);
        assert_eq!(s.props[0].default.as_deref(), Some("0"));
        assert_eq!(s.props[1].default.as_deref(), Some("'Hi'"));
        assert_eq!(s.props[2].default.as_deref(), Some("[]"));
    }

    #[test]
    fn extracts_props_no_default() {
        let src = "const { fetchUrl } = props";
        let s = extract(src);
        assert_eq!(s.props.len(), 1);
        assert_eq!(s.props[0].name, "fetchUrl");
        assert!(s.props[0].default.is_none());
    }

    #[test]
    fn extracts_signals() {
        let src = "const $query = signal('')\nconst $items = signal([])\nconst $loading=signal(false)";
        let s = extract(src);
        assert_eq!(s.signals, vec!["query", "items", "loading"]);
    }

    #[test]
    fn counts_lifecycle_hooks() {
        let src = "onConnect(el => {})\nonDisconnect(el => {})\nonConnect(el => {})";
        let s = extract(src);
        assert_eq!(s.on_connect_calls, 2);
        assert_eq!(s.on_disconnect_calls, 1);
    }

    #[test]
    fn ignores_strings_and_comments() {
        let src = r#"
            // const { fake = 1 } = props
            /* const $bogus = signal(0) */
            const x = "const { y = 1 } = props"
            const $real = signal(0)
        "#;
        let s = extract(src);
        assert!(s.props.is_empty(), "got props: {:?}", s.props);
        assert_eq!(s.signals, vec!["real"]);
    }

    #[test]
    fn full_typeahead_script_extracts_correctly() {
        let src = r#"
            const { placeholder = 'Search...', fetchUrl, minChars = 1, debounceMs = 200 } = props
            const $query = signal('')
            const $items = signal([])
            const $loading = signal(false)
            const $activeIndex = signal(-1)
            const $open = signal(false)
            onConnect(el => el.querySelector('input').addEventListener('keydown', onKey))
        "#;
        let s = extract(src);
        let names: Vec<_> = s.props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["placeholder", "fetchUrl", "minChars", "debounceMs"]);
        assert_eq!(s.signals, vec!["query", "items", "loading", "activeIndex", "open"]);
        assert_eq!(s.on_connect_calls, 1);
        assert_eq!(s.on_disconnect_calls, 0);
    }
}
