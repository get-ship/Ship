//! Template-body rewrite for `.ship` SFCs.
//!
//! The job: rewrite Datastar signal references inside `data-*` attribute
//! values from `$name` to `$_<tag>___SHIP_INST___<name>` so multiple
//! instances of the same component on a page never share signal names.
//!
//! The `__SHIP_INST__` token is replaced at element-construction time
//! by the emitted custom-element wrapper class with a per-instance
//! counter (`1`, `2`, ...).
//!
//! ## Why a regex over `data-*="..."`
//!
//! Datastar's reactivity surface is exclusively the `data-*` attribute
//! family (`data-bind:`, `data-on:`, `data-show`, `data-text`, `data-class:`,
//! `data-attr:`, `data-for`, ...). Rewriting only inside those attribute
//! values is correct and conservative: free-text Jinja expressions and
//! HTML body content are never touched, so `${jsTemplate}` interpolations
//! and natural-language `$` references in copy stay intact.
//!
//! Word boundaries ensure `$query` rewrites without also catching
//! `$queryStr` — the lookbehind/lookahead are checked against
//! `[A-Za-z0-9_$]`.

use regex::Regex;
use std::sync::OnceLock;

static ATTR_RE: OnceLock<Regex> = OnceLock::new();

fn attr_re() -> &'static Regex {
    // Match `data-...="..."` or `data-...='...'`.
    ATTR_RE.get_or_init(|| {
        Regex::new(r#"(data-[A-Za-z0-9_:.\-]+)\s*=\s*("[^"]*"|'[^']*')"#).unwrap()
    })
}

/// Rewrite signal references inside `data-*` attribute values in
/// `template_html`. Each rewritten name becomes
/// `$_<tag>___SHIP_INST___<name>` — the wrapper substitutes
/// `__SHIP_INST__` with a per-instance counter at runtime.
pub fn rewrite_signals(template_html: &str, tag: &str, signals: &[String]) -> String {
    if signals.is_empty() {
        return template_html.to_string();
    }
    let re = attr_re();
    let result = re.replace_all(template_html, |caps: &regex::Captures<'_>| {
        let attr = caps.get(1).unwrap().as_str();
        let val_with_quotes = caps.get(2).unwrap().as_str();
        let q = &val_with_quotes[..1];
        let inner = &val_with_quotes[1..val_with_quotes.len() - 1];
        let rewritten = rewrite_inside(inner, tag, signals);
        format!("{attr}={q}{rewritten}{q}")
    });
    result.into_owned()
}

fn rewrite_inside(value: &str, tag: &str, signals: &[String]) -> String {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len() + 32);
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'$' {
            // Avoid `$$` Datastar escape sequences and template literal
            // `${...}` — only rewrite when followed by an identifier.
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if next.is_ascii_alphabetic() || next == b'_' {
                // Read identifier.
                let id_start = i + 1;
                let mut j = id_start;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_')
                {
                    j += 1;
                }
                let name = &value[id_start..j];
                // Check previous char isn't an identifier-continuation
                // (avoid touching `foo$bar`).
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                let prev_ok = !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$');
                if prev_ok && signals.iter().any(|s| s == name) {
                    out.push_str(&format!("$_{tag}___SHIP_INST___{name}"));
                    i = j;
                    continue;
                }
                // Not a known signal — copy verbatim.
                out.push('$');
                out.push_str(name);
                i = j;
                continue;
            }
        }
        // UTF-8: copy the full char.
        let ch_end = next_char_boundary(bytes, i);
        out.push_str(&value[i..ch_end]);
        i = ch_end;
    }
    out
}

fn next_char_boundary(bytes: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    while j < bytes.len() && (bytes[j] & 0xC0) == 0x80 {
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signals(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn rewrites_signal_in_data_bind() {
        let html = r#"<input data-bind:value="$query" />"#;
        let out = rewrite_signals(html, "app-typeahead", &signals(&["query"]));
        assert!(out.contains(r#"data-bind:value="$_app-typeahead___SHIP_INST___query""#));
    }

    #[test]
    fn does_not_rewrite_outside_data_attrs() {
        // `$query` in body text is not a Datastar reference.
        let html = r#"<span>price: $query</span><div data-text="$query"></div>"#;
        let out = rewrite_signals(html, "tag", &signals(&["query"]));
        assert!(out.contains("price: $query"), "out: {out}");
        assert!(out.contains("$_tag___SHIP_INST___query"));
    }

    #[test]
    fn skips_unknown_signals() {
        let html = r#"<div data-text="$known + $unknown"></div>"#;
        let out = rewrite_signals(html, "tag", &signals(&["known"]));
        assert!(out.contains("$_tag___SHIP_INST___known"));
        assert!(out.contains("$unknown"));
    }

    #[test]
    fn word_boundary_prevents_partial_match() {
        let html = r#"<div data-text="$query + $queryStr"></div>"#;
        let out = rewrite_signals(html, "tag", &signals(&["query"]));
        assert!(out.contains("$_tag___SHIP_INST___query +"));
        assert!(out.contains("$queryStr"));
    }

    #[test]
    fn handles_multiple_attrs_on_one_element() {
        let html = r#"<input data-bind:value="$query" data-on:input="$open = true" />"#;
        let out = rewrite_signals(html, "tag", &signals(&["query", "open"]));
        assert!(out.contains("$_tag___SHIP_INST___query"));
        assert!(out.contains("$_tag___SHIP_INST___open"));
    }

    #[test]
    fn empty_signals_list_passes_through() {
        let html = r#"<div data-text="$x"></div>"#;
        let out = rewrite_signals(html, "tag", &signals(&[]));
        assert_eq!(out, html);
    }

    #[test]
    fn single_quoted_attribute_value() {
        let html = r#"<div data-text='$x'></div>"#;
        let out = rewrite_signals(html, "tag", &signals(&["x"]));
        assert!(out.contains("$_tag___SHIP_INST___x"));
    }
}
