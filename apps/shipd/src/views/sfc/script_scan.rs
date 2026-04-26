//! Low-level scanning helpers used by [`super::script`].
//!
//! Split out so the parent stays under the 300-line cap. These are
//! pure byte-level utilities: comment/string stripping (with offset
//! preservation), brace matching, destructure splitting, identifier
//! validation. None of them know what an SFC is.

use super::script::PropDecl;

/// Replace JS string / template / comment regions with same-length
/// whitespace so byte offsets match the original. Newlines are
/// preserved so line-based diagnostics stay sane.
pub fn strip_comments_and_strings(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = vec![b' '; bytes.len()];
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            out[i] = b'\n';
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                if bytes[i] == b'\n' { out[i] = b'\n'; }
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            let q = b;
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    if bytes[i + 1] == b'\n' { out[i + 1] = b'\n'; }
                    i += 2;
                    continue;
                }
                if bytes[i] == b'\n' { out[i] = b'\n'; }
                i += 1;
            }
            i = (i + 1).min(bytes.len());
            continue;
        }
        out[i] = b;
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

pub fn find_prev_nonspace(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    loop {
        if !bytes[i].is_ascii_whitespace() {
            return Some(i);
        }
        if i == 0 { return None; }
        i -= 1;
    }
}

pub fn find_matching_open(bytes: &[u8], close: usize) -> Option<usize> {
    let mut depth = 1;
    let mut i = close;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'}' => depth += 1,
            b'{' => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
    }
    None
}

/// Parse the inside of `{ a, b = 1, c: alias = 'x' }` and append each
/// recognized property to `out`. Aliases (`c: alias`) are recorded as
/// the original key (`c`) — the prop *attribute name* is what matters
/// for codegen.
pub fn parse_destructure(inner: &str, out: &mut Vec<PropDecl>) {
    let parts = split_top_level_commas(inner);
    for raw in parts {
        let part = raw.trim();
        if part.is_empty() { continue; }
        let (name_part, default) = match part.find('=') {
            Some(idx) => (
                part[..idx].trim().to_string(),
                Some(part[idx + 1..].trim().to_string()),
            ),
            None => (part.to_string(), None),
        };
        let name = name_part
            .split(':')
            .next()
            .unwrap_or(&name_part)
            .trim()
            .to_string();
        if name.is_empty() || !is_identifier(&name) {
            continue;
        }
        out.push(PropDecl { name, default });
    }
}

fn split_top_level_commas(inner: &str) -> Vec<&str> {
    let mut depth_paren = 0i32;
    let mut depth_brack = 0i32;
    let mut depth_brace = 0i32;
    let mut start = 0usize;
    let bytes = inner.as_bytes();
    let mut parts: Vec<&str> = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'[' => depth_brack += 1,
            b']' => depth_brack -= 1,
            b'{' => depth_brace += 1,
            b'}' => depth_brace -= 1,
            b',' if depth_paren == 0 && depth_brack == 0 && depth_brace == 0 => {
                parts.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&inner[start..]);
    parts
}

pub fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}
