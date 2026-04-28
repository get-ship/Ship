//! In-memory document state — source text + parse tree + diagnostics.
//!
//! # Tree-sitter integration
//!
//! `Documents::parser` carries the `tree-sitter-set` language; every
//! `upsert` reparses the new text and stores the resulting `Tree` on
//! the document. Subsequent edits pass the previous tree to
//! `Parser::parse(text, prev_tree)`, so re-parses are incremental
//! against the prior parse tree.
//!
//! FULL document sync is still the LSP transport in v0 — the editor
//! sends the entire new text on every change. Switching to INCREMENTAL
//! sync (replay `TextDocumentContentChangeEvent` ranges into
//! `Tree::edit` before reparsing) is the next milestone; the parse
//! plumbing already supports it.

use std::collections::HashMap;

use lsp_types::{Diagnostic, Url};
use tree_sitter::{Parser, Tree};

// `text` and `diagnostics` feed next-milestone hover/completion handlers
// (see README "Next milestone"). The push-diagnostics path computes fresh
// on every change, so caching `diagnostics` is the staged shape rather
// than today's consumer.
#[allow(dead_code)]
pub struct DocumentState {
    pub text: String,
    /// Tree-sitter parse tree. `Some` once the parser has consumed the
    /// document at least once; threaded into the next reparse for
    /// incremental updates.
    pub tree: Option<Tree>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Documents {
    inner: HashMap<Url, DocumentState>,
    parser: Parser,
}

impl Default for Documents {
    fn default() -> Self {
        Self::new()
    }
}

impl Documents {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_set::LANGUAGE.into())
            .expect("tree-sitter-set LANGUAGE matches the linked tree-sitter ABI");
        Self {
            inner: HashMap::new(),
            parser,
        }
    }

    /// Insert (or replace) a document. Reparses incrementally against
    /// the previously-cached tree (if any) and stores the new tree.
    pub fn upsert(
        &mut self,
        uri: Url,
        text: String,
        diagnostics: Vec<Diagnostic>,
    ) {
        let prev_tree = self.inner.get(&uri).and_then(|d| d.tree.as_ref());
        let tree = self.parser.parse(&text, prev_tree);
        self.inner.insert(
            uri,
            DocumentState {
                text,
                tree,
                diagnostics,
            },
        );
    }

    pub fn close(&mut self, uri: &Url) {
        self.inner.remove(uri);
    }

    /// Read access for next-milestone handlers (hover/completion).
    /// Currently exercised only by tests.
    #[allow(dead_code)]
    pub fn get(&self, uri: &Url) -> Option<&DocumentState> {
        self.inner.get(uri)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn upsert_stores_text_diagnostics_and_tree() {
        let mut docs = Documents::new();
        let u = url("file:///x/widget.set");
        docs.upsert(u.clone(), "<script></script>".into(), Vec::new());
        let d = docs.get(&u).expect("present");
        assert_eq!(d.text, "<script></script>");
        assert!(d.diagnostics.is_empty());
        // tree-sitter-set is wired; an empty <script></script> still
        // parses to a non-error root.
        let tree = d.tree.as_ref().expect("tree present");
        assert!(!tree.root_node().has_error(), "unexpected ERROR node");
    }

    #[test]
    fn upsert_replaces_existing_entry() {
        let mut docs = Documents::new();
        let u = url("file:///x/widget.set");
        docs.upsert(u.clone(), "first".into(), Vec::new());
        docs.upsert(u.clone(), "second".into(), Vec::new());
        assert_eq!(docs.len(), 1);
        assert_eq!(docs.get(&u).unwrap().text, "second");
    }

    #[test]
    fn close_removes_entry() {
        let mut docs = Documents::new();
        let u = url("file:///x/widget.set");
        docs.upsert(u.clone(), "x".into(), Vec::new());
        docs.close(&u);
        assert!(docs.get(&u).is_none());
        assert_eq!(docs.len(), 0);
    }
}
