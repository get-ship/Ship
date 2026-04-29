//! Hand-rolled HTML parser scoped to `.html.set` files.
//!
//! After `preprocess::preprocess` has run, the source is well-formed
//! enough that we don't need a full HTML5 implementation:
//!  - All non-void self-closes have been expanded.
//!  - All brace attributes are quoted.
//!
//! Span-tracking is the reason we don't reach for `html5ever`: every
//! IR node, every attribute, every text run carries a byte range so
//! diagnostics point at the user's source.
//!
//! The Parser implementation is split into `parser.rs` to keep both
//! files under the 300-line cap. This file owns the public surface
//! (types + `parse_fragment`).

use crate::diagnostics::{CompileError, Span};

mod parser;

pub(super) const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
    "source", "track", "wbr",
];

/// Tags whose body is treated as raw text (no nested elements,
/// closing handled by the literal `</tag>` match).
pub(super) const RAW_TEXT_TAGS: &[&str] = &["script", "style"];

/// Parse a `.html.set` source fragment into a flat list of top-level
/// nodes. The fragment is parsed at "body" level — no document/html
/// wrapping, no doctype handling.
pub fn parse_fragment(source: &str) -> Result<Vec<Node>, CompileError> {
    let mut p = parser::Parser::new(source);
    p.parse_children(None)
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(Text),
    Comment(Comment),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attr>,
    pub children: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: String,
    pub value: String,
    pub span: Span,
    /// Span of just the value (between quotes when quoted, or the
    /// raw value byte range when unquoted/boolean).
    pub value_span: Span,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub data: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub data: String,
    pub span: Span,
}

impl Element {
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.clone())
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|a| a.name == name)
    }
}

/// Decode the small set of HTML entities the preprocess step emits.
/// Anything else passes through verbatim.
pub fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&nbsp;", "\u{a0}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_element_and_text() {
        let nodes = parse_fragment("<h1>Hi</h1>").unwrap();
        assert_eq!(nodes.len(), 1);
        let Node::Element(el) = &nodes[0] else {
            panic!()
        };
        assert_eq!(el.tag, "h1");
        assert_eq!(el.children.len(), 1);
    }

    #[test]
    fn parses_attrs_quoted_and_brace() {
        let nodes = parse_fragment(r#"<x a="1" b="{count()}" c />"#).unwrap();
        let Node::Element(el) = &nodes[0] else {
            panic!()
        };
        assert_eq!(el.attributes.len(), 3);
        assert_eq!(el.get_attribute("a"), Some("1".into()));
        assert_eq!(el.get_attribute("b"), Some("{count()}".into()));
        assert!(el.has_attribute("c"));
    }

    #[test]
    fn parses_self_closing_void_tag() {
        let nodes = parse_fragment("<br/>after").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn diagnoses_mismatched_close() {
        let err = parse_fragment("<a></b>").unwrap_err();
        assert!(err.message.contains("</b>"));
    }

    #[test]
    fn comment_round_trip() {
        let nodes = parse_fragment("<!-- hi --> ").unwrap();
        assert!(matches!(nodes[0], Node::Comment(_)));
    }
}
