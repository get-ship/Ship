//! Design-token reference validation.
//!
//! Walks the style block looking for `${name.path}` references. If a
//! token source is configured (`CompileOpts::tokens`), each reference
//! must exist in the source — otherwise emits SAIL/E0010.
//!
//! When no token source is configured, the analyzer falls back to a
//! permissive pass that records the references but does not error,
//! so OSS users without the Ship token catalog can still build.

use crate::parse::ast::SailFile;
use crate::{Diagnostic, Severity};

/// Set of valid token paths. The compiler doesn't ship a default; the
/// caller passes the configured set (e.g., from Ship's tokens.css).
pub type TokenSet = std::collections::HashSet<String>;

pub fn analyze(file: &SailFile, tokens: Option<&TokenSet>, diagnostics: &mut Vec<Diagnostic>) {
    let Some(style) = &file.style else { return };
    for r in extract_token_refs(&style.raw) {
        if let Some(set) = tokens {
            if !set.contains(&r) {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "SAIL/E0010".into(),
                    message: format!("unknown design token `{r}`"),
                    span: None,
                    help: Some(
                        "check the configured token source; tokens are referenced as `${name.path}`".into(),
                    ),
                });
            }
        }
    }
}

/// Scan CSS for `${...}` references and return the inner text, trimmed.
pub fn extract_token_refs(css: &str) -> Vec<String> {
    let bytes = css.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
            let start = i + 2;
            let mut j = start;
            let mut depth = 1i32;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                if depth == 0 { break; }
                j += 1;
            }
            if depth == 0 {
                out.push(css[start..j].trim().to_string());
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn diag(src: &str, tokens: Option<TokenSet>) -> Vec<Diagnostic> {
        let f = parse::parse(src).unwrap();
        let mut d = Vec::new();
        analyze(&f, tokens.as_ref(), &mut d);
        d
    }

    #[test]
    fn extract_finds_token_refs() {
        let refs = extract_token_refs(".x { color: ${color.fg}; gap: ${space.3}; }");
        assert_eq!(refs, vec!["color.fg".to_string(), "space.3".to_string()]);
    }

    #[test]
    fn no_token_source_emits_no_diagnostics() {
        let d = diag(
            "<template><div/></template><style>.x { color: ${color.foo}; }</style>",
            None,
        );
        assert!(d.is_empty(), "got {d:?}");
    }

    #[test]
    fn unknown_token_with_source_errors() {
        let mut tokens = TokenSet::new();
        tokens.insert("color.bg".into());
        let d = diag(
            "<template><div/></template><style>.x { color: ${color.foo}; }</style>",
            Some(tokens),
        );
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, Severity::Error);
        assert_eq!(d[0].code, "SAIL/E0010");
        assert!(d[0].message.contains("color.foo"));
    }

    #[test]
    fn known_token_with_source_passes() {
        let mut tokens = TokenSet::new();
        tokens.insert("color.fg".into());
        let d = diag(
            "<template><div/></template><style>.x { color: ${color.fg}; }</style>",
            Some(tokens),
        );
        assert!(d.is_empty(), "got {d:?}");
    }
}
