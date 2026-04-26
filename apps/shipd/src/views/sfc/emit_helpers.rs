//! Helpers for [`super::emit`].
//!
//! Pulled out so the main emit module stays focused on the JS module
//! template. These helpers are pure: attribute-name conversion,
//! prop-decoder code generation, and string escaping for inclusion
//! inside a JS template literal.

use super::script::PropDecl;

/// `fetchUrl` → `fetch-url`. Standard kebab-case attribute mapping.
/// First character never emits a leading dash, even when uppercase
/// (`XMLHttp` → `x-m-l-http`, not `-x-m-l-http`).
pub fn to_attr_name(camel: &str) -> String {
    let mut out = String::with_capacity(camel.len() + 4);
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 { out.push('-'); }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Render the per-component prop-decoder block. Reads each documented
/// attribute via `getAttribute(...)`, falls back to the default value
/// when the attribute is absent, and coerces non-string defaults
/// (numbers + booleans) using a small `__coerce` helper.
pub fn build_prop_decoders(props: &[PropDecl]) -> String {
    if props.is_empty() {
        return "      const props = {};".to_string();
    }
    let mut out = String::from("      const props = {};\n");
    for p in props {
        let attr = to_attr_name(&p.name);
        let default_js = p.default.as_deref().unwrap_or("undefined");
        out.push_str(&format!(
            "      {{\n        const __raw = this.getAttribute(\"{attr}\");\n        props[\"{name}\"] = __raw == null ? ({default_js}) : __coerce(__raw, ({default_js}));\n      }}\n",
            attr = attr,
            name = p.name,
            default_js = default_js,
        ));
    }
    out.push_str(
        "      function __coerce(raw, def) {\n\
\x20\x20\x20\x20\x20\x20\x20\x20if (typeof def === 'number') { const n = Number(raw); return Number.isFinite(n) ? n : def; }\n\
\x20\x20\x20\x20\x20\x20\x20\x20if (typeof def === 'boolean') return raw === '' || raw === 'true' || raw === String(true);\n\
\x20\x20\x20\x20\x20\x20\x20\x20return raw;\n\
\x20\x20\x20\x20\x20\x20}",
    );
    out
}

/// Escape a string for inclusion inside a JS backtick template
/// literal. Only the three characters that change interpretation
/// inside a template literal need escaping: backslash, backtick, and
/// the dollar sign that starts an expression placeholder.
pub fn escape_template_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '$' => out.push_str("\\$"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_to_kebab_attr() {
        assert_eq!(to_attr_name("fetchUrl"), "fetch-url");
        assert_eq!(to_attr_name("count"), "count");
        // First char never emits a leading dash; subsequent uppercase do.
        assert_eq!(to_attr_name("XMLHttp"), "x-m-l-http");
    }

    #[test]
    fn escape_handles_backtick_and_dollar() {
        assert_eq!(escape_template_literal("a`b$c\\d"), "a\\`b\\$c\\\\d");
    }

    #[test]
    fn empty_props_decoder_emits_empty_object() {
        assert!(build_prop_decoders(&[]).contains("const props = {};"));
    }
}
