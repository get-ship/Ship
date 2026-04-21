//! Shared loading + tokenization helpers for the rule implementations.
//!
//! Views are Jinja templates. A real HTML parser trips on `{% %}` / `{{ }}`
//! blocks, so the linter works on cleaned-up line arrays and narrow regexes
//! and leaves template-correctness to minijinja at render time.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use regex::Regex;

pub struct Source {
    pub path: PathBuf,
    /// File content with Jinja comments stripped (blanked, not removed) so
    /// line numbers still match the on-disk file.
    pub lines: Vec<String>,
    /// Names extracted from `{#- @lint:signals-ok: foo,bar -#}` directives.
    pub signals_ok: HashSet<String>,
}

pub fn read_source(path: &Path) -> Option<Source> {
    let raw = std::fs::read_to_string(path).ok()?;
    let signals_ok = parse_signals_ok(&raw);
    let lines = strip_jinja_comments(&raw);
    Some(Source {
        path: path.to_path_buf(),
        lines,
        signals_ok,
    })
}

/// Extract names from `{#- @lint:signals-ok: foo, bar -#}` directives
/// anywhere in the file. Matches both the short (`{#`) and trim-whitespace
/// (`{#-`) forms.
fn parse_signals_ok(raw: &str) -> HashSet<String> {
    let re = signals_ok_regex();
    let mut out = HashSet::new();
    for cap in re.captures_iter(raw) {
        let list = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        for name in list.split(',') {
            let n = name.trim();
            if !n.is_empty() {
                out.insert(n.to_string());
            }
        }
    }
    out
}

/// Blank out Jinja comments in a source string while preserving line numbers.
///
/// A real parser would track comment state across `{% raw %}` and escapes;
/// that's out of scope — views don't use those patterns, and the false
/// positives we'd cause are preferable to hiding an inline `<script>` behind
/// a quoting trick.
pub fn strip_jinja_comments(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_comment = false;
    for line in raw.split('\n') {
        if !in_comment {
            if !line.contains("{#") {
                out.push(line.to_string());
                continue;
            }
            let mut buf = String::new();
            let mut rest = line;
            loop {
                if let Some(open) = rest.find("{#") {
                    buf.push_str(&rest[..open]);
                    let after = &rest[open..];
                    if let Some(rel_close) = after.find("#}") {
                        rest = &after[rel_close + 2..];
                        continue;
                    } else {
                        in_comment = true;
                        break;
                    }
                } else {
                    buf.push_str(rest);
                    break;
                }
            }
            out.push(buf);
        } else if let Some(close) = line.find("#}") {
            in_comment = false;
            out.push(line[close + 2..].to_string());
        } else {
            out.push(String::new());
        }
    }
    out
}

// ── Regex builders ────────────────────────────────────────────────────────────
//
// Compiling a Regex isn't free, but the linter is a short-lived CLI — we
// accept the per-call cost rather than pulling in `once_cell` just for this.

pub fn script_open_regex() -> Regex {
    Regex::new(r"(?i)<script(\s[^>]*)?>").unwrap()
}

pub fn style_open_regex() -> Regex {
    Regex::new(r"(?i)<style(\s[^>]*)?>").unwrap()
}

pub fn style_close_regex() -> Regex {
    Regex::new(r"(?i)</style\s*>").unwrap()
}

pub fn signal_macro_regex() -> Regex {
    // `signal('name', …)` / `signal("name", …)` — the SDK macro.
    Regex::new(r#"\bsignal\s*\(\s*['"]([A-Za-z_][A-Za-z0-9_]*)['"]"#).unwrap()
}

pub fn data_signals_attr_regex() -> Regex {
    // `data-signals-foo=` or `data-signals:foo=` (both forms occur).
    Regex::new(r"\bdata-signals[-:]([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap()
}

pub fn data_signals_object_regex() -> Regex {
    // `data-signals="{ foo: 1, bar: 'x' }"`.
    Regex::new(r#"\bdata-signals\s*=\s*['"](\{[^'"]*\})['"]"#).unwrap()
}

pub fn object_key_regex() -> Regex {
    Regex::new(r#"(?:^|[\{,]\s*)['"]?([A-Za-z_][A-Za-z0-9_]*)['"]?\s*:"#).unwrap()
}

pub fn quoted_attr_value_regex() -> Regex {
    Regex::new(r#"=\s*"([^"]*)""#).unwrap()
}

pub fn signal_ref_regex() -> Regex {
    Regex::new(r"(?:^|[^A-Za-z0-9_])\$([A-Za-z_][A-Za-z0-9_]*)").unwrap()
}

pub fn signals_ok_regex() -> Regex {
    // `{#- @lint:signals-ok: foo, bar -#}`. `-` is optional either side.
    Regex::new(r"\{#-?\s*@lint:signals-ok:\s*([^#]*?)\s*-?#\}").unwrap()
}

/// Loose match for top-level keys in a flat `{…}` object literal — used to
/// pull signal names out of `data-signals="{foo: 1}"`. Good enough for the
/// flat payloads seen in views; anything richer is a structural smell.
pub fn extract_object_keys(obj: &str) -> Vec<String> {
    let re = object_key_regex();
    re.captures_iter(obj)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
