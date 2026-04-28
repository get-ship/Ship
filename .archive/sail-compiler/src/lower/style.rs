//! Phase 6 — CSS scoping + `:host` rewrite + token resolution.
//!
//! Ported from `apps/shipd/src/views/sfc/style.rs`. Adds:
//! - `${token.path}` → `var(--ship-token-path)` rewrite (no validation
//!   in MVP — analyzer phase will check tokens against the configured
//!   token source and emit diagnostics for unknowns).
//!
//! Honors `<style global>` (caller decides) and `<style src="…">`
//! (caller is responsible for reading and prepending the external CSS;
//! this module only scopes what it's given).

use crate::parse::ast::StyleBlock;

/// Scope a `<style>` block for the given tag. Returns the scoped CSS,
/// or the verbatim input if `style.global` is set.
pub fn scope_style(style: &StyleBlock, tag: &str) -> String {
    let with_tokens = rewrite_tokens(&style.raw);
    if style.global {
        return with_tokens;
    }
    scope_css(&with_tokens, tag)
}

/// Public entry: prefix every top-level selector with the host
/// attribute selector for `tag`.
pub fn scope_css(css: &str, tag: &str) -> String {
    let prefix = format!("[data-component=\"{tag}\"]");
    let mut out = String::with_capacity(css.len() + 64);
    rewrite(css, &prefix, &mut out, 0);
    out
}

/// Rewrite `${name.path}` → `var(--ship-name-path)`. Untyped — the
/// analyzer will validate token references in a later phase.
pub fn rewrite_tokens(css: &str) -> String {
    let bytes = css.as_bytes();
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
            // Find matching `}`.
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
                let inner = css[start..j].trim();
                let var_name = inner.replace('.', "-");
                out.push_str(&format!("var(--ship-{var_name})"));
                i = j + 1;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn rewrite(input: &str, prefix: &str, out: &mut String, depth_limit: usize) {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    let mut prelude_start = 0usize;

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
        if b == b'{' {
            let prelude = &input[prelude_start..i];
            let trimmed = prelude.trim();
            if trimmed.starts_with("@media")
                || trimmed.starts_with("@supports")
                || trimmed.starts_with("@container")
                || trimmed.starts_with("@layer")
            {
                let body_start = i + 1;
                let body_end = match find_matching_close(bytes, i) {
                    Some(e) => e,
                    None => { out.push_str(&input[prelude_start..]); return; }
                };
                out.push_str(&input[prelude_start..i + 1]);
                let inner = &input[body_start..body_end];
                if depth_limit < 4 {
                    rewrite(inner, prefix, out, depth_limit + 1);
                } else {
                    out.push_str(inner);
                }
                out.push('}');
                i = body_end + 1;
                prelude_start = i;
                continue;
            }
            let selectors = split_top_level_commas(trimmed);
            let leading_ws_end = prelude.len() - prelude.trim_start().len();
            out.push_str(&prelude[..leading_ws_end]);
            for (idx, sel) in selectors.iter().enumerate() {
                if idx > 0 { out.push_str(",\n"); }
                emit_prefixed_selector(out, sel.trim(), prefix);
            }
            out.push_str(" {");
            let body_start = i + 1;
            let body_end = match find_matching_close(bytes, i) {
                Some(e) => e,
                None => { out.push_str(&input[body_start..]); return; }
            };
            out.push_str(&input[body_start..body_end]);
            out.push('}');
            i = body_end + 1;
            prelude_start = i;
            continue;
        }
        if b == b'}' {
            out.push_str(&input[prelude_start..=i]);
            i += 1;
            prelude_start = i;
            continue;
        }
        if b == b';' {
            out.push_str(&input[prelude_start..=i]);
            i += 1;
            prelude_start = i;
            continue;
        }
        i += 1;
    }
    if prelude_start < bytes.len() {
        out.push_str(&input[prelude_start..]);
    }
}

fn emit_prefixed_selector(out: &mut String, sel: &str, prefix: &str) {
    let s = sel.trim();
    if s.is_empty() {
        out.push_str(prefix);
        return;
    }
    let s_no_comments = strip_css_comments(s);
    let s_check = s_no_comments.trim();
    if s_check.starts_with('&') || s_check.starts_with(prefix) {
        out.push_str(s);
        return;
    }
    if s_check.starts_with(":host") {
        let rewritten = rewrite_host_pseudo(s_check, prefix);
        let leading_len = s.len() - s.trim_start().len();
        out.push_str(&s[..leading_len]);
        out.push_str(&rewritten);
        return;
    }
    out.push_str(prefix);
    out.push(' ');
    out.push_str(s);
}

fn rewrite_host_pseudo(s: &str, prefix: &str) -> String {
    let bytes = s.as_bytes();
    debug_assert!(s.starts_with(":host"));
    let mut i = 5usize;
    let mut compound = String::new();
    if bytes.get(i) == Some(&b'(') {
        let start = i + 1;
        let mut depth = 1;
        let mut j = start;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth == 0 { break; }
            j += 1;
        }
        if depth == 0 {
            compound.push_str(&s[start..j]);
            i = j + 1;
        }
    }
    let rest = &s[i..];
    if compound.is_empty() {
        format!("{prefix}{rest}")
    } else {
        format!("{prefix}{compound}{rest}")
    }
}

fn strip_css_comments(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn find_matching_close(bytes: &[u8], open: usize) -> Option<usize> {
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

fn split_top_level_commas(input: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_simple_selector() {
        let out = scope_css("button { color: red; }", "my-comp");
        assert!(out.contains("[data-component=\"my-comp\"] button {"));
    }

    #[test]
    fn host_alone_rewrites_to_prefix() {
        let out = scope_css(":host { display: none; }", "x-y");
        assert!(out.contains("[data-component=\"x-y\"] {") || out.contains("[data-component=\"x-y\"]{"));
        assert!(!out.contains(":host"));
    }

    #[test]
    fn host_with_attribute_compounds() {
        let out = scope_css(":host([disabled]) { opacity: 0.5; }", "my-tag");
        assert!(out.contains("[data-component=\"my-tag\"][disabled]"));
    }

    #[test]
    fn comma_inside_is_not_split() {
        let out = scope_css(":is(.a, .b) span { color: red }", "tag");
        assert_eq!(out.matches("[data-component=\"tag\"]").count(), 1);
        assert!(out.contains(":is(.a, .b) span"));
    }

    #[test]
    fn at_media_recurses() {
        let out = scope_css("@media (min-width: 400px) { .x { color: red } }", "tag");
        assert!(out.contains("@media (min-width: 400px)"));
        assert!(out.contains("[data-component=\"tag\"] .x"));
    }

    #[test]
    fn token_interpolation_rewrites() {
        let css = ".x { color: ${color.surface}; padding: ${space.3}; }";
        let out = rewrite_tokens(css);
        assert!(out.contains("var(--ship-color-surface)"));
        assert!(out.contains("var(--ship-space-3)"));
        assert!(!out.contains("${"));
    }

    #[test]
    fn token_in_at_media_query() {
        let out = rewrite_tokens("@media ${breakpoint.md} { .x {} }");
        assert!(out.contains("var(--ship-breakpoint-md)"));
    }

    #[test]
    fn scope_style_skips_global() {
        use crate::parse::ast::StyleBlock;
        let s = StyleBlock {
            raw: "button { color: red }".into(),
            body_byte_offset: 0,
            global: true,
            src: None,
        };
        let out = scope_style(&s, "tag");
        assert!(!out.contains("[data-component"));
    }

    #[test]
    fn scope_style_applies_when_not_global() {
        use crate::parse::ast::StyleBlock;
        let s = StyleBlock {
            raw: ".x { color: ${color.fg}; }".into(),
            body_byte_offset: 0,
            global: false,
            src: None,
        };
        let out = scope_style(&s, "tag");
        assert!(out.contains("[data-component=\"tag\"] .x"));
        assert!(out.contains("var(--ship-color-fg)"));
    }
}
