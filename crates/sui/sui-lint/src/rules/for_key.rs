//! `sui-for-key` — `{#for item of items}` blocks should declare a `key=`
//! expression so the reconciler can preserve DOM identity across
//! reorder / insert / delete.
//!
//! Without `key=`, the reconciler falls back to positional matching,
//! which silently corrupts list state when items shift. The bug class
//! is common because the failure is invisible until the user reorders
//! a list with focus / selection / animation state on a row.
//!
//! Acts on the literal block header. Doesn't reach into the iter
//! expression — we only check that `key=` is present somewhere in the
//! header before the closing `}`. False positives are possible if a
//! comment inside the template contains the string `{#for`, but the
//! same false positive class affects the parser; matching the parser's
//! tolerance keeps the rule predictable.
//!
//! Suggest text points at the runtime warning that fires when keys
//! collide — same diagnostic source. See
//! `packages/sui-runtime/src/for.ts` for the runtime side.

use std::path::Path;

use crate::report::{Finding, Severity};

pub const RULE_ID: &str = "sui-for-key";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    // Walk char boundaries so multi-byte chars (em-dashes in comments,
    // entity-decoded `°`/`§`, etc.) don't trip a str-slice panic at
    // `source[i..]` when `i` lands mid-char.
    let boundaries: Vec<usize> = source
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(source.len()))
        .collect();
    let mut bi = 0;

    while bi < boundaries.len() {
        let i = boundaries[bi];
        if i >= source.len() {
            break;
        }
        if !source[i..].starts_with("{#for") {
            bi += 1;
            continue;
        }
        let header_start = i;
        // Find the closing `}` of the block header. Skip nested braces
        // and string literals so an iter expression like
        // `{#for x of items.filter(y => y.k === "}" )}` doesn't confuse us.
        let header_end = match find_block_header_end(source, i) {
            Some(end) => end,
            None => {
                // Malformed — let the parser surface that error.
                bi += 1;
                continue;
            }
        };

        let header = &source[header_start..=header_end];
        if !has_key_attribute(header) {
            let (line, col) = byte_offset_to_line_col(source, header_start);
            findings.push(Finding {
                file: path.to_path_buf(),
                line,
                col,
                rule: RULE_ID,
                severity: Severity::Warn,
                message: format!(
                    "{{#for}} block has no `key=` — the reconciler will \
                     fall back to positional matching and DOM identity \
                     (focus, selection, in-place edits) will be lost on \
                     reorder. Add `key={{item.id}}` (or whatever uniquely \
                     identifies each row)."
                ),
                suggest: Some(
                    "Add `key={EXPR}` to the for-block header, where \
                     EXPR returns a stable unique value per item — e.g. \
                     `{#for item of items key={item.id}}`."
                        .to_string(),
                ),
            });
        }

        // Advance `bi` past the header. Find the smallest boundary index
        // whose byte-offset is > header_end.
        while bi < boundaries.len() && boundaries[bi] <= header_end {
            bi += 1;
        }
    }

    findings
}

/// Scan from `start` (which must be `{`) to the matching `}` that
/// closes the block header. Returns the byte index of that `}`.
fn find_block_header_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    let mut in_str: Option<u8> = None;

    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => in_str = Some(c),
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// True when the for-block header contains a `key=` attribute. We look
/// for the whitespace-prefixed string ` key=` to avoid false matches
/// inside an iter expression that happens to use a `key` identifier
/// (e.g. `items.sortBy(i => i.key)`).
fn has_key_attribute(header: &str) -> bool {
    // Find `key=` preceded by whitespace; not inside a string literal.
    let bytes = header.as_bytes();
    let mut i = 0;
    let mut in_str: Option<u8> = None;
    while i + 4 < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            in_str = Some(c);
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() && &bytes[i + 1..i + 5] == b"key=" {
            return true;
        }
        i += 1;
    }
    false
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(src: &str) -> Vec<Finding> {
        check(Path::new("test.sui"), src)
    }

    #[test]
    fn for_with_key_attribute_passes() {
        let src = r#"<ul>{#for item of items key={item.id}}<li>{item.name}</li>{/for}</ul>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 0, "{:?}", findings);
    }

    #[test]
    fn for_without_key_warns() {
        let src = r#"<ul>{#for item of items}<li>{item.name}</li>{/for}</ul>"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`key=`"));
        assert!(findings[0].suggest.is_some());
    }

    #[test]
    fn iter_expression_using_key_identifier_does_not_count() {
        // `items.sortBy(i => i.key)` mentions "key" but not as an attribute.
        let src = r#"{#for item of items.sortBy(i => i.key)}<li>x</li>{/for}"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1, "should warn — no key= ATTRIBUTE");
    }

    #[test]
    fn two_for_blocks_each_evaluated_separately() {
        let src = r#"
            {#for a of as key={a.id}}<li>{a.name}</li>{/for}
            {#for b of bs}<li>{b.name}</li>{/for}
        "#;
        let findings = lint(src);
        assert_eq!(findings.len(), 1, "only the second block should warn");
    }

    #[test]
    fn iter_with_close_brace_in_string_does_not_break_scan() {
        let src = r#"{#for x of items.filter(i => i.s === "}" ) key={x.id}}<li>x</li>{/for}"#;
        let findings = lint(src);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn malformed_for_block_does_not_panic() {
        let src = r#"{#for item of items"#;
        let _ = lint(src);
    }
}
