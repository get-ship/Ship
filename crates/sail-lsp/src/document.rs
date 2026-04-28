//! In-memory document state — source text + parse tree + diagnostics.
//!
//! # Tree-sitter integration shape (v0 stub)
//!
//! `Documents::parser` is a real `tree_sitter::Parser`, but no language
//! is set yet — `tree-sitter-set` is being scaffolded in a parallel job
//! and its generated parser isn't wired here. As a result every
//! `parse(...)` call returns `None`, leaving `DocumentState::tree` at
//! `None`. The shape is here so that, once the grammar exits flux, the
//! follow-up only has to:
//!
//!   1. Add `tree-sitter-set = { path = "../../grammars/tree-sitter-set" }`
//!      to `Cargo.toml`.
//!   2. Call `parser.set_language(&tree_sitter_set::LANGUAGE.into())`
//!      in `Documents::new`.
//!   3. Switch `TextDocumentSyncKind` to `INCREMENTAL` and replay the
//!      `TextDocumentContentChangeEvent` ranges into `Tree::edit`
//!      before reparsing — `parse(text, prev_tree)` already threads
//!      the previous tree for incremental reparse.
//!
//! Until then, FULL sync is the actual transport and the tree field
//! stays `None` — no scaffold pretending to be more than it is.

use std::collections::HashMap;

use lsp_types::{Diagnostic, Url};
use tree_sitter::{Parser, Tree};

// V0 stores fields the binary doesn't read yet — `text` and
// `diagnostics` feed the next-milestone hover/completion handlers
// (see README "Next milestone"). The push-diagnostics path computes
// fresh on every change, so caching `diagnostics` is the staged shape,
// not yet a consumer.
#[allow(dead_code)]
pub struct DocumentState {
    pub text: String,
    /// Always `None` in v0 — see module docs.
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
        Self {
            inner: HashMap::new(),
            // Language is intentionally unset; see module docs.
            parser: Parser::new(),
        }
    }

    /// Insert (or replace) a document. Reparses with the current parser
    /// — until a language is set, this returns `None` and `tree` stays
    /// empty.
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
    fn upsert_stores_text_and_diagnostics() {
        let mut docs = Documents::new();
        let u = url("file:///x/widget.set");
        docs.upsert(u.clone(), "<script></script>".into(), Vec::new());
        let d = docs.get(&u).expect("present");
        assert_eq!(d.text, "<script></script>");
        assert!(d.diagnostics.is_empty());
        // No language set on parser → no tree in v0.
        assert!(d.tree.is_none());
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
