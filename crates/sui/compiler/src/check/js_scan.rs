//! Shared low-level scanning helpers for the textual `.ts` checks
//! (`controller_parse`, `pure_derived`, `scope_bound`).
//!
//! Each helper is string-, comment-, and brace-aware so the callers
//! don't accidentally match keywords inside string literals or skip
//! past the end of a parenthesised expression.

/// Advance past whitespace, line comments (`//`), and block comments
/// (`/* … */`). Idempotent at non-trivia bytes.
pub(super) fn skip_trivia(bytes: &[u8], mut i: usize) -> usize {
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i = skip_line_comment(bytes, i);
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i = skip_block_comment(bytes, i);
            continue;
        }
        break;
    }
    i
}

/// Skip a `//` comment up to and including the newline (or EOF).
pub(super) fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Skip a `/* ... */` block comment, returning the offset just past `*/`.
pub(super) fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    i += 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

/// At a `(`, `{`, or `[`, return the offset just past the matching
/// closer. String / template literal contents are skipped. Returns
/// `i + 1` if `bytes[i]` isn't an opener so callers can defensively
/// step forward.
pub(super) fn skip_balanced(bytes: &[u8], i: usize, end: usize) -> usize {
    if i >= end {
        return end;
    }
    let (open, close) = match bytes[i] {
        b'(' => (b'(', b')'),
        b'{' => (b'{', b'}'),
        b'[' => (b'[', b']'),
        _ => return i + 1,
    };
    let mut depth = 0i32;
    let mut k = i;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k += 2; continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            c if c == open => { depth += 1; k += 1; }
            c if c == close => {
                depth -= 1;
                k += 1;
                if depth == 0 { return k; }
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'/' => {
                k = skip_line_comment(bytes, k);
            }
            b'/' if k + 1 < end && bytes[k + 1] == b'*' => {
                k = skip_block_comment(bytes, k);
            }
            _ => k += 1,
        }
    }
    end
}

/// Walk to the next `,` or `}` at depth zero. Used by object-literal
/// parsers to find the end of a `key: value` entry.
pub(super) fn next_top_separator(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' { k += 2; continue; }
            if c == q { in_str = None; }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { in_str = Some(c); k += 1; }
            b'(' | b'{' | b'[' => { k = skip_balanced(bytes, k, end); }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}

// The identifier predicates are crate-shared; re-export so this module's
// consumers (scope_bound_expr, symbol_table) keep importing them from here
// alongside the skip_* scanners that still live in this file.
pub(super) use crate::scan::{is_ident_cont, is_ident_start};
