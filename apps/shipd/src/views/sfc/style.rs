//! CSS scoping for `.ship` `<style>` blocks.
//!
//! Top-level rule splitter: tracks brace depth, identifies selector
//! lists at depth 0 (and inside `@media` / `@supports` at depth 1),
//! splits each selector list on top-level commas, and prepends
//! `[data-component="<tag>"] ` to each selector.
//!
//! Selectors that already begin with `:host`, `&`, or the host
//! attribute prefix are emitted unchanged. Comments and at-rule
//! prelude regions (`@import`, `@charset`, `@namespace`) are also
//! passed through.
//!
//! `<style global>` callers should bypass this pass entirely (the
//! caller checks the `global` flag and returns the original CSS).
//!
//! Brace-balancing and selector-list splitting helpers live in
//! [`super::style_walk`] to keep this file focused on the scoping
//! policy.

use super::style_walk::{find_matching_close, split_top_level_commas};

/// Prefix every top-level selector in `css` with the host attribute
/// selector for `tag`. Returns scoped CSS.
pub fn scope_css(css: &str, tag: &str) -> String {
    let prefix = format!("[data-component=\"{tag}\"]");
    let mut out = String::with_capacity(css.len() + 64);
    rewrite(css, &prefix, &mut out, 0);
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
            // Plain rule.
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
    if s.starts_with(":host") || s.starts_with('&') || s.starts_with(prefix) {
        out.push_str(s);
        return;
    }
    out.push_str(prefix);
    out.push(' ');
    out.push_str(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_simple_selector() {
        let css = "button { color: red; }";
        let out = scope_css(css, "my-comp");
        assert!(out.contains("[data-component=\"my-comp\"] button {"));
        assert!(out.contains("color: red;"));
    }

    #[test]
    fn prefixes_each_selector_in_list() {
        let css = ".a, .b, .c { display: none }";
        let out = scope_css(css, "x-y");
        assert_eq!(out.matches("[data-component=\"x-y\"]").count(), 3);
        assert!(out.contains(".a"));
        assert!(out.contains(".b"));
        assert!(out.contains(".c"));
    }

    #[test]
    fn prefixes_nested_descendant_combinator() {
        let css = ".outer .inner > span + em { color: blue }";
        let out = scope_css(css, "tag");
        assert_eq!(out.matches("[data-component=\"tag\"]").count(), 1);
        assert!(out.contains("[data-component=\"tag\"] .outer .inner > span + em"));
    }

    #[test]
    fn handles_at_media_recursively() {
        let css = "@media (min-width: 400px) { .x { color: red } }";
        let out = scope_css(css, "tag");
        assert!(out.contains("@media (min-width: 400px)"));
        assert!(out.contains("[data-component=\"tag\"] .x"));
    }

    #[test]
    fn skips_already_prefixed_selector() {
        let css = "[data-component=\"tag\"] .pre { color: red }";
        let out = scope_css(css, "tag");
        assert_eq!(out.matches("[data-component=\"tag\"]").count(), 1);
    }

    #[test]
    fn comma_inside_is_not_split() {
        let css = ":is(.a, .b) span { color: red }";
        let out = scope_css(css, "tag");
        assert_eq!(out.matches("[data-component=\"tag\"]").count(), 1);
        assert!(out.contains(":is(.a, .b) span"));
    }

    #[test]
    fn typeahead_style_scopes_correctly() {
        let css = "
            .typeahead { position: relative; }
            .suggestions { position: absolute; }
            .suggestions li.active { background: blue; }
        ";
        let out = scope_css(css, "app-typeahead");
        assert!(out.contains("[data-component=\"app-typeahead\"] .typeahead {"));
        assert!(out.contains("[data-component=\"app-typeahead\"] .suggestions {"));
        assert!(out.contains("[data-component=\"app-typeahead\"] .suggestions li.active {"));
    }
}
