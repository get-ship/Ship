//! Low-level CSS walking helpers for [`super::style`].
//!
//! Brace-balanced close-finder and top-level comma splitter — both
//! string- and comment-aware. The top-level scoping pass in
//! `super::style` orchestrates these across nested at-rules.

/// Find the index of the `}` that matches the `{` at `open`. Skips
/// braces inside CSS strings and block comments.
pub fn find_matching_close(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 1i32;
    let mut i = open + 1;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') { i += 1; }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if b == b'"' || b == b'\'' {
            let q = b;
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
                i += 1;
            }
            i = (i + 1).min(bytes.len());
            continue;
        }
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Split a selector list on top-level commas. Commas inside `(...)`,
/// `[...]`, or string literals are not split — so `:is(.a, .b)` stays
/// as one selector.
pub fn split_top_level_commas(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth_paren = 0i32;
    let mut depth_brack = 0i32;
    let mut start = 0usize;
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' || b == b'\'' {
            let q = b;
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
                i += 1;
            }
            i = (i + 1).min(bytes.len());
            continue;
        }
        match b {
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'[' => depth_brack += 1,
            b']' => depth_brack -= 1,
            b',' if depth_paren == 0 && depth_brack == 0 => {
                out.push(input[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(input[start..].to_string());
    out
}
