//! Shared loading + tokenization helpers for the rule implementations.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

// ── Regex builders (Lazy so they're compiled once per process) ─────────────────

static SCRIPT_OPEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)<script(\s[^>]*)?>").unwrap());
static STYLE_OPEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)<style(\s[^>]*)?>").unwrap());
static STYLE_CLOSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)</style\s*>").unwrap());
static SIGNAL_MACRO_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\bsignal\s*\(\s*['"]([A-Za-z_][A-Za-z0-9_]*)['"]"#).unwrap());
static DATA_SIGNALS_ATTR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bdata-signals[-:]([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap());
static DATA_SIGNALS_OBJ_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\bdata-signals\s*=\s*['"](\{[^'"]*\})['"]"#).unwrap());
static OBJECT_KEY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:^|[\{,]\s*)['"]?([A-Za-z_][A-Za-z0-9_]*)['"]?\s*:"#).unwrap());
static QUOTED_ATTR_VALUE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"=\s*"([^"]*)""#).unwrap());
static SIGNAL_REF_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|[^A-Za-z0-9_])\$([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static SIGNALS_OK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{#-?\s*@lint:signals-ok:\s*([^#]*?)\s*-?#\}").unwrap());

pub fn script_open_regex() -> &'static Regex { &SCRIPT_OPEN_RE }
pub fn style_open_regex() -> &'static Regex { &STYLE_OPEN_RE }
pub fn style_close_regex() -> &'static Regex { &STYLE_CLOSE_RE }
pub fn signal_macro_regex() -> &'static Regex { &SIGNAL_MACRO_RE }
pub fn data_signals_attr_regex() -> &'static Regex { &DATA_SIGNALS_ATTR_RE }
pub fn data_signals_object_regex() -> &'static Regex { &DATA_SIGNALS_OBJ_RE }
pub fn quoted_attr_value_regex() -> &'static Regex { &QUOTED_ATTR_VALUE_RE }
pub fn signal_ref_regex() -> &'static Regex { &SIGNAL_REF_RE }
pub fn signals_ok_regex() -> &'static Regex { &SIGNALS_OK_RE }

pub fn extract_object_keys(obj: &str) -> Vec<String> {
    OBJECT_KEY_RE
        .captures_iter(obj)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}
