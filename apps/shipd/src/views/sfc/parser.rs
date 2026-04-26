//! Block splitter for `.ship` single-file components.
//!
//! Splits a `.ship` source string into three optional regions:
//! - **script**: the (single) `<script>...</script>` block, if present.
//! - **style**: the (single) `<style>...</style>` block, if present, plus
//!   the attributes of the opening tag (so `<style global>` can be
//!   detected downstream).
//! - **template body**: everything else — Jinja markup with Datastar
//!   attributes.
//!
//! The split uses two non-greedy DOTALL regexes (`<script[^>]*>...</script>`
//! and `<style[^>]*>...</style>`). This is the right choice for `.ship`
//! files because Jinja expressions like `{{ props.url }}` inside HTML
//! attributes confuse tag-aware HTML parsers; the regex approach treats
//! the template body as opaque text and only excises the recognized
//! script/style envelopes.
//!
//! Multiple `<script>` or `<style>` blocks in a single file are not part
//! of the v1 surface — only the first occurrence of each is recognized;
//! any additional occurrences remain in the template body and will be
//! served verbatim as part of the rendered HTML. (This matches the
//! "convention: script → template → style" ordering documented in the
//! spec without forcing it.)

use regex::Regex;
use std::sync::OnceLock;

static SCRIPT_RE: OnceLock<Regex> = OnceLock::new();
static STYLE_RE: OnceLock<Regex> = OnceLock::new();

fn script_re() -> &'static Regex {
    SCRIPT_RE.get_or_init(|| Regex::new(r"(?s)<script\b([^>]*)>(.*?)</script\s*>").unwrap())
}

fn style_re() -> &'static Regex {
    STYLE_RE.get_or_init(|| Regex::new(r"(?s)<style\b([^>]*)>(.*?)</style\s*>").unwrap())
}

/// A `.ship` document split into its three optional regions.
#[derive(Debug, Clone, Default)]
pub struct SplitBlocks {
    /// Body of the first `<script>` block, sans wrapping tags.
    pub script: Option<String>,
    /// Body of the first `<style>` block, sans wrapping tags.
    pub style: Option<String>,
    /// Raw attribute string from the opening `<style ...>` tag (with
    /// leading whitespace), used to detect `global`.
    pub style_attrs: String,
    /// Everything else, in source order, with the recognized script and
    /// style blocks removed.
    pub template: String,
}

/// Split a `.ship` source into its blocks. Always succeeds; missing
/// blocks return as `None`.
pub fn split(source: &str) -> SplitBlocks {
    let mut out = SplitBlocks::default();

    // Pull the first <script> block.
    let after_script = if let Some(m) = script_re().captures(source) {
        let full = m.get(0).unwrap();
        out.script = Some(m.get(2).unwrap().as_str().to_string());
        let mut combined = String::with_capacity(source.len());
        combined.push_str(&source[..full.start()]);
        combined.push_str(&source[full.end()..]);
        combined
    } else {
        source.to_string()
    };

    // Pull the first <style> block from what's left.
    let after_style = if let Some(m) = style_re().captures(&after_script) {
        let full = m.get(0).unwrap();
        out.style_attrs = m.get(1).unwrap().as_str().to_string();
        out.style = Some(m.get(2).unwrap().as_str().to_string());
        let mut combined = String::with_capacity(after_script.len());
        combined.push_str(&after_script[..full.start()]);
        combined.push_str(&after_script[full.end()..]);
        combined
    } else {
        after_script
    };

    out.template = after_style.trim_matches('\n').to_string();
    out
}

/// Returns true if the captured `<style ...>` attribute string contains
/// the `global` flag (bare or `global="true"`). Whitespace tolerant.
pub fn style_is_global(attrs: &str) -> bool {
    // Cheap word-boundary scan; we don't need a full attribute parser.
    let trimmed = attrs.trim();
    for token in trimmed.split(|c: char| c.is_whitespace() || c == '=' || c == '"' || c == '\'') {
        if token == "global" {
            return true;
        }
    }
    false
}

/// Extract the value of a `src="..."` (or `src='...'`) attribute from a
/// `<style>` opening-tag attribute string. Returns the raw attribute
/// value with surrounding quotes stripped; the caller resolves the path
/// relative to the .ship file location.
///
/// Returns `None` if no `src` attribute is present. Whitespace and
/// other attributes are tolerated.
pub fn style_src(attrs: &str) -> Option<String> {
    static SRC_RE: OnceLock<Regex> = OnceLock::new();
    let re = SRC_RE.get_or_init(|| Regex::new(r#"\bsrc\s*=\s*("([^"]*)"|'([^']*)')"#).unwrap());
    re.captures(attrs).map(|c| {
        c.get(2)
            .or_else(|| c.get(3))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_all_three_blocks() {
        let src = "<script>const x = 1</script>\n<div>hi {{ name }}</div>\n<style>.x{}</style>";
        let s = split(src);
        assert_eq!(s.script.as_deref(), Some("const x = 1"));
        assert_eq!(s.style.as_deref(), Some(".x{}"));
        assert!(s.template.contains("<div>hi {{ name }}</div>"));
        assert!(!s.template.contains("<script"));
        assert!(!s.template.contains("<style"));
    }

    #[test]
    fn missing_blocks_are_none() {
        let s = split("<div>just markup</div>");
        assert!(s.script.is_none());
        assert!(s.style.is_none());
        assert_eq!(s.template.trim(), "<div>just markup</div>");
    }

    #[test]
    fn jinja_inside_script_is_not_confused() {
        let src = "<script>const url = `${props.fetchUrl}?q=${q}`</script>\n<div>{{ x }}</div>";
        let s = split(src);
        assert!(s.script.as_deref().unwrap().contains("props.fetchUrl"));
        assert!(s.template.contains("{{ x }}"));
    }

    #[test]
    fn style_global_attribute_detected() {
        let src = "<style global>.x{}</style>";
        let s = split(src);
        assert!(style_is_global(&s.style_attrs));

        let src2 = "<style>.x{}</style>";
        let s2 = split(src2);
        assert!(!style_is_global(&s2.style_attrs));
    }

    #[test]
    fn style_with_other_attrs_does_not_falsely_match_global() {
        let src = r#"<style data-foo="globalish">.x{}</style>"#;
        let s = split(src);
        assert!(!style_is_global(&s.style_attrs));
    }

    #[test]
    fn style_src_attribute_extracted() {
        let s = split(r#"<style src="./shared.css"></style>"#);
        assert_eq!(style_src(&s.style_attrs).as_deref(), Some("./shared.css"));

        let s = split(r#"<style src='./shared.css'></style>"#);
        assert_eq!(style_src(&s.style_attrs).as_deref(), Some("./shared.css"));

        // No src present.
        let s = split(r#"<style>.x{}</style>"#);
        assert_eq!(style_src(&s.style_attrs), None);

        // Other attributes don't confuse the matcher.
        let s = split(r#"<style global src="./shared.css"></style>"#);
        assert_eq!(style_src(&s.style_attrs).as_deref(), Some("./shared.css"));
        assert!(style_is_global(&s.style_attrs));
    }

    #[test]
    fn template_strips_leading_trailing_newlines() {
        let s = split("<script>x</script>\n\n<div></div>\n\n<style>y</style>");
        assert_eq!(s.template, "<div></div>");
    }
}
