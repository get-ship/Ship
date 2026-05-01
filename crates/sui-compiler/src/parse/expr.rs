//! Expression validator. Mirrors the rules in
//! `packages/ui-dom/src/template/expr.ts`:
//!
//!  - statements (`;` outside of strings) are forbidden.
//!  - assignments (`=` not part of a comparison or arrow) are forbidden.
//!  - arrow functions are forbidden unless `allow_arrow` is set.
//!  - empty expressions are forbidden.
//!
//! The compiler does not parse a JS AST. Validation walks the source
//! tracking string-literal state so the checks ignore characters
//! inside `'='` or `` `=>` `` literals.

use crate::diagnostics::{CompileError, Span};

pub fn validate(expr: &str, allow_arrow: bool, location: &str) -> Result<(), CompileError> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(CompileError::new("empty expression")
            .with_category(location.to_string()));
    }
    let bytes = trimmed.as_bytes();
    let mut in_string: Option<u8> = None;
    let mut template_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        let prev = if i > 0 { bytes[i - 1] } else { 0 };
        let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };

        if let Some(q) = in_string {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_string = None;
            } else if q == b'`' && c == b'$' && next == b'{' {
                template_depth += 1;
                in_string = None;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            in_string = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            brace_depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' {
            if template_depth > 0 && brace_depth == 0 {
                template_depth -= 1;
                in_string = Some(b'`');
                i += 1;
                continue;
            }
            brace_depth -= 1;
            i += 1;
            continue;
        }
        if c == b'(' {
            paren_depth += 1;
        }
        if c == b')' {
            paren_depth -= 1;
        }
        if c == b';' {
            return Err(CompileError::new(format!(
                "statements are not allowed in expressions: `{trimmed}`"
            ))
            .with_category(location.to_string()));
        }
        if c == b'=' && next == b'>' {
            if !allow_arrow {
                return Err(CompileError::new(format!(
                    "arrow functions are not allowed here: `{trimmed}`"
                ))
                .with_category(location.to_string()));
            }
            i += 2;
            continue;
        }
        if c == b'=' {
            let is_compare = matches!(prev, b'=' | b'!' | b'<' | b'>') || next == b'=';
            if !is_compare {
                return Err(CompileError::new(format!(
                    "assignments are not allowed in expressions: `{trimmed}`"
                ))
                .with_category(location.to_string()));
            }
        }
        i += 1;
    }
    if in_string.is_some() {
        return Err(CompileError::new(format!(
            "unterminated string in expression: `{trimmed}`"
        ))
        .with_category(location.to_string()));
    }
    let _ = paren_depth;
    Ok(())
}

/// Convenience wrapper that attaches a span to validation errors.
pub fn validate_at(
    expr: &str,
    allow_arrow: bool,
    location: &str,
    span: Span,
) -> Result<(), CompileError> {
    validate(expr, allow_arrow, location).map_err(|e| e.with_span(span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_call() {
        validate("count()", false, "x").unwrap();
    }

    #[test]
    fn rejects_assignment() {
        let err = validate("x = 1", false, "x").unwrap_err();
        assert!(err.message.contains("assignments"));
    }

    #[test]
    fn allows_compare() {
        validate("a == b", false, "x").unwrap();
        validate("a >= b", false, "x").unwrap();
        validate("a !== b", false, "x").unwrap();
    }

    #[test]
    fn rejects_statements() {
        let err = validate("x; y", false, "x").unwrap_err();
        assert!(err.message.contains("statements"));
    }

    #[test]
    fn rejects_arrow_by_default() {
        let err = validate("() => 1", false, "x").unwrap_err();
        assert!(err.message.contains("arrow"));
    }

    #[test]
    fn allows_arrow_when_enabled() {
        validate("(e) => x.set(e.target.value)", true, "x").unwrap();
    }

    #[test]
    fn ignores_inside_strings() {
        validate("'a; b'", false, "x").unwrap();
        validate("`a=>b`", false, "x").unwrap();
        validate("`a=${1}`", false, "x").unwrap();
    }

    #[test]
    fn rejects_empty() {
        let err = validate("   ", false, "x").unwrap_err();
        assert!(err.message.contains("empty"));
    }
}
