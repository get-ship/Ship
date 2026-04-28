//! Template diagnostic rules.
//!
//! Each rule is a self-contained module with a single `analyze(file,
//! diagnostics, …)` entry point. Rules read the parsed AST and the
//! [`SymbolTable`](super::symbols::SymbolTable); they don't speak to
//! the schema source or run the type checker. Diagnostics are
//! Warnings — they nudge authors at compile time but don't fail the
//! build, since the corpus may legitimately diverge from a rule's
//! heuristics until we tighten coverage.
//!
//! Rule code bands: `SAIL/W01xx` reactivity; `SAIL/W02xx` cross-
//! component.
//!
//! Shared infrastructure lives in this file: the per-component prop
//! catalog (built once at first call from the embedded
//! `sdk-framework.json` manifest) and a pair of small visitor helpers
//! used by every rule that walks the template tree.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use crate::Diagnostic;
use crate::Manifest;
use crate::parse::ast::{Node, PropBinding, SailFile, Template};

use super::symbols::SymbolTable;

pub mod cleanup_pattern;
pub mod prop_exists;
pub mod r1_signal_call;
pub mod r5_bind_oninput;

/// Run every rule against `file`, pushing diagnostics into `diagnostics`.
/// Rule order is fixed but doesn't matter for correctness.
pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let symbols = SymbolTable::from_file(file);
    r1_signal_call::analyze(file, &symbols, diagnostics);
    r5_bind_oninput::analyze(file, diagnostics);
    prop_exists::analyze(file, diagnostics);
    cleanup_pattern::analyze(file, diagnostics);
}

/// Visit every text-interpolation expression in the template (`{expr}`
/// and `{@html expr}` positions). Attribute interpolation is *not*
/// included — sail's runtime auto-wraps reactives in attr position, so
/// rules scoped to "is this a runtime null" only care about text.
pub(crate) fn for_each_text_expr<F>(tpl: &Template, mut f: F)
where
    F: FnMut(&crate::parse::ast::Expr),
{
    fn walk(n: &Node, f: &mut impl FnMut(&crate::parse::ast::Expr)) {
        match n {
            Node::Interpolation(e) | Node::RawHtml(e) => f(e),
            Node::Element { children, .. } | Node::Component { children, .. } => {
                children.iter().for_each(|c| walk(c, f));
            }
            Node::If { then_, elifs, else_, .. } => {
                then_.iter().for_each(|c| walk(c, f));
                elifs.iter().for_each(|(_, b)| b.iter().for_each(|c| walk(c, f)));
                if let Some(b) = else_ { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::For { body, empty, .. } => {
                body.iter().for_each(|c| walk(c, f));
                if let Some(e) = empty { e.iter().for_each(|c| walk(c, f)); }
            }
            Node::Await { loading, .. } => loading.iter().for_each(|c| walk(c, f)),
            Node::Slot { fallback, .. } => fallback.iter().for_each(|c| walk(c, f)),
            Node::SlotFill { body, .. } => body.iter().for_each(|c| walk(c, f)),
            Node::Text(_) | Node::Host { .. } => {}
        }
    }
    tpl.root.iter().for_each(|n| walk(n, &mut f));
}

/// Visit every `Node::Component` in the template (`<ship-foo …/>` or
/// any kebab-tagged custom element). Rules that care about prop
/// declarations or component-tag-specific semantics consume this.
pub(crate) fn for_each_component<F>(tpl: &Template, mut f: F)
where
    F: FnMut(&str, &[PropBinding]),
{
    fn walk(n: &Node, f: &mut impl FnMut(&str, &[PropBinding])) {
        match n {
            Node::Component { tag, props, children, .. } => {
                f(tag, props);
                children.iter().for_each(|c| walk(c, f));
            }
            Node::Element { children, .. } => children.iter().for_each(|c| walk(c, f)),
            Node::If { then_, elifs, else_, .. } => {
                then_.iter().for_each(|c| walk(c, f));
                elifs.iter().for_each(|(_, b)| b.iter().for_each(|c| walk(c, f)));
                if let Some(b) = else_ { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::For { body, empty, .. } => {
                body.iter().for_each(|c| walk(c, f));
                if let Some(e) = empty { e.iter().for_each(|c| walk(c, f)); }
            }
            Node::Await { loading, .. } => loading.iter().for_each(|c| walk(c, f)),
            Node::Slot { fallback, .. } => fallback.iter().for_each(|c| walk(c, f)),
            Node::SlotFill { body, .. } => body.iter().for_each(|c| walk(c, f)),
            Node::Text(_) | Node::Interpolation(_) | Node::RawHtml(_) | Node::Host { .. } => {}
        }
    }
    tpl.root.iter().for_each(|n| walk(n, &mut f));
}

/// Catalog of every `<ship-*>` component the framework ships, keyed by
/// kebab-cased tag and mapped to the kebab-cased attribute names that
/// correspond to declared `prop` signals. Built once at first call by
/// reading the embedded `sdk-framework.json` manifest, which the
/// xtask gen-sdk pipeline keeps in lockstep with the corpus under
/// `packages/ship-set-sail/components/`.
pub(crate) fn component_catalog() -> &'static HashMap<String, BTreeSet<String>> {
    static CATALOG: OnceLock<HashMap<String, BTreeSet<String>>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut out: HashMap<String, BTreeSet<String>> = HashMap::new();
        for c in &embedded_manifest().components {
            let props: BTreeSet<String> = c.props.iter().map(|p| kebab_of(&p.name)).collect();
            out.insert(c.tag.clone(), props);
        }
        out
    })
}

/// Embedded SDK framework manifest. The compiler ships with the JSON
/// the xtask gen-sdk pipeline writes to
/// `packages/ship-set/sdk/sdk-framework.json`; reading it once here
/// gives every consumer in the crate the same component catalog
/// without re-parsing the corpus at runtime.
pub(crate) fn embedded_manifest() -> &'static Manifest {
    static MANIFEST: OnceLock<Manifest> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        serde_json::from_str(EMBEDDED_FRAMEWORK_JSON)
            .expect("embedded sdk-framework.json must parse")
    })
}

const EMBEDDED_FRAMEWORK_JSON: &str =
    include_str!("../../../../../packages/ship-set/sdk/sdk-framework.json");

pub(crate) fn kebab_of(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 { out.push('-'); }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod corpus_probe {
    //! One-shot scan of the shipped component corpus that prints the
    //! current rule hits. Always passes — output shows up under
    //! `cargo test -- --nocapture` and is what the job report quotes.
    //!
    //! Reads `.set` files from disk relative to `CARGO_MANIFEST_DIR`
    //! at test time. The compiler crate doesn't ship the corpus
    //! through `include_str!` any more (the per-component prop
    //! catalog now travels via the embedded `sdk-framework.json`).
    use std::path::PathBuf;

    use crate::parse;
    use crate::tag_for_stem;

    fn corpus_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root")
            .join("packages/ship-set-sail/components")
    }

    #[test]
    fn probe_corpus() {
        let mut by_code: std::collections::BTreeMap<&'static str, usize> = Default::default();
        let mut hits: Vec<(String, String, String)> = Vec::new();
        let mut entries: Vec<_> = std::fs::read_dir(corpus_dir())
            .expect("read corpus dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("set"))
            .collect();
        entries.sort();
        for path in entries {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let tag = tag_for_stem(stem);
            let src = std::fs::read_to_string(&path).expect("read .set");
            let Ok(file) = parse::parse(&src) else { continue };
            let mut diags = Vec::new();
            super::analyze(&file, &mut diags);
            for d in diags.iter().filter(|d| {
                matches!(
                    d.code.as_str(),
                    "SAIL/W0101" | "SAIL/W0105" | "SAIL/W0201" | "SAIL/E0501"
                )
            }) {
                let code: &'static str = match d.code.as_str() {
                    "SAIL/W0101" => "SAIL/W0101",
                    "SAIL/W0105" => "SAIL/W0105",
                    "SAIL/W0201" => "SAIL/W0201",
                    "SAIL/E0501" => "SAIL/E0501",
                    _ => continue,
                };
                *by_code.entry(code).or_default() += 1;
                hits.push((tag.clone(), d.code.clone(), d.message.clone()));
            }
        }
        eprintln!("\n=== rules::corpus_probe — {} total hits ===", hits.len());
        for (c, n) in &by_code {
            eprintln!("  {c}: {n}");
        }
        for (file, code, msg) in hits.iter().take(80) {
            eprintln!("  [{code}] {file}: {msg}");
        }
    }
}
