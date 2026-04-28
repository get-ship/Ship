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
//! catalog (built once at first call from embedded `.set` sources)
//! and a pair of small visitor helpers used by every rule that walks
//! the template tree.

use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

use crate::Diagnostic;
use crate::parse;
use crate::parse::ast::{Node, PropBinding, SailFile, Template};

use super::symbols::SymbolTable;

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
/// re-parsing the embedded `.set` sources.
pub(crate) fn component_catalog() -> &'static HashMap<String, BTreeSet<String>> {
    static CATALOG: OnceLock<HashMap<String, BTreeSet<String>>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut out: HashMap<String, BTreeSet<String>> = HashMap::new();
        for (tag, src) in COMPONENT_SOURCES {
            let Ok(file) = parse::parse(src) else { continue };
            let Some(script) = file.script else { continue };
            let props: BTreeSet<String> =
                script.props.iter().map(|p| kebab_of(&p.name)).collect();
            out.insert((*tag).to_string(), props);
        }
        out
    })
}

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

/// Embedded `.set` sources for every component in
/// `packages/ship-set-sail/components/`. New components are added by
/// a one-liner here; `cargo build` re-runs `include_str!`.
const COMPONENT_SOURCES: &[(&str, &str)] = &[
    ("ship-accordion", include_str!("../../../../../packages/ship-set-sail/components/ship-accordion.set")),
    ("ship-accordion-item", include_str!("../../../../../packages/ship-set-sail/components/ship-accordion-item.set")),
    ("ship-agents", include_str!("../../../../../packages/ship-set-sail/components/ship-agents.set")),
    ("ship-alert", include_str!("../../../../../packages/ship-set-sail/components/ship-alert.set")),
    ("ship-app-icon", include_str!("../../../../../packages/ship-set-sail/components/ship-app-icon.set")),
    ("ship-avatar", include_str!("../../../../../packages/ship-set-sail/components/ship-avatar.set")),
    ("ship-badge", include_str!("../../../../../packages/ship-set-sail/components/ship-badge.set")),
    ("ship-breadcrumb", include_str!("../../../../../packages/ship-set-sail/components/ship-breadcrumb.set")),
    ("ship-breadcrumb-item", include_str!("../../../../../packages/ship-set-sail/components/ship-breadcrumb-item.set")),
    ("ship-button", include_str!("../../../../../packages/ship-set-sail/components/ship-button.set")),
    ("ship-card", include_str!("../../../../../packages/ship-set-sail/components/ship-card.set")),
    ("ship-checkbox", include_str!("../../../../../packages/ship-set-sail/components/ship-checkbox.set")),
    ("ship-chip", include_str!("../../../../../packages/ship-set-sail/components/ship-chip.set")),
    ("ship-collapsible", include_str!("../../../../../packages/ship-set-sail/components/ship-collapsible.set")),
    ("ship-copy-button", include_str!("../../../../../packages/ship-set-sail/components/ship-copy-button.set")),
    ("ship-dashboard", include_str!("../../../../../packages/ship-set-sail/components/ship-dashboard.set")),
    ("ship-empty-state", include_str!("../../../../../packages/ship-set-sail/components/ship-empty-state.set")),
    ("ship-error-state", include_str!("../../../../../packages/ship-set-sail/components/ship-error-state.set")),
    ("ship-icon", include_str!("../../../../../packages/ship-set-sail/components/ship-icon.set")),
    ("ship-input", include_str!("../../../../../packages/ship-set-sail/components/ship-input.set")),
    ("ship-kbd", include_str!("../../../../../packages/ship-set-sail/components/ship-kbd.set")),
    ("ship-label", include_str!("../../../../../packages/ship-set-sail/components/ship-label.set")),
    ("ship-link", include_str!("../../../../../packages/ship-set-sail/components/ship-link.set")),
    ("ship-list-tile", include_str!("../../../../../packages/ship-set-sail/components/ship-list-tile.set")),
    ("ship-mark", include_str!("../../../../../packages/ship-set-sail/components/ship-mark.set")),
    ("ship-markdown-editor", include_str!("../../../../../packages/ship-set-sail/components/ship-markdown-editor.set")),
    ("ship-markdown-editor-bubble", include_str!("../../../../../packages/ship-set-sail/components/ship-markdown-editor-bubble.set")),
    ("ship-markdown-editor-toolbar", include_str!("../../../../../packages/ship-set-sail/components/ship-markdown-editor-toolbar.set")),
    ("ship-pagination", include_str!("../../../../../packages/ship-set-sail/components/ship-pagination.set")),
    ("ship-progress", include_str!("../../../../../packages/ship-set-sail/components/ship-progress.set")),
    ("ship-provider-blob", include_str!("../../../../../packages/ship-set-sail/components/ship-provider-blob.set")),
    ("ship-radio", include_str!("../../../../../packages/ship-set-sail/components/ship-radio.set")),
    ("ship-radio-group", include_str!("../../../../../packages/ship-set-sail/components/ship-radio-group.set")),
    ("ship-select", include_str!("../../../../../packages/ship-set-sail/components/ship-select.set")),
    ("ship-separator", include_str!("../../../../../packages/ship-set-sail/components/ship-separator.set")),
    ("ship-skeleton", include_str!("../../../../../packages/ship-set-sail/components/ship-skeleton.set")),
    ("ship-spinner", include_str!("../../../../../packages/ship-set-sail/components/ship-spinner.set")),
    ("ship-stat", include_str!("../../../../../packages/ship-set-sail/components/ship-stat.set")),
    ("ship-status-dot", include_str!("../../../../../packages/ship-set-sail/components/ship-status-dot.set")),
    ("ship-switch", include_str!("../../../../../packages/ship-set-sail/components/ship-switch.set")),
    ("ship-tab", include_str!("../../../../../packages/ship-set-sail/components/ship-tab.set")),
    ("ship-tab-panel", include_str!("../../../../../packages/ship-set-sail/components/ship-tab-panel.set")),
    ("ship-tabs", include_str!("../../../../../packages/ship-set-sail/components/ship-tabs.set")),
    ("ship-textarea", include_str!("../../../../../packages/ship-set-sail/components/ship-textarea.set")),
    ("ship-timestamp", include_str!("../../../../../packages/ship-set-sail/components/ship-timestamp.set")),
    ("ship-tooltip", include_str!("../../../../../packages/ship-set-sail/components/ship-tooltip.set")),
];

#[cfg(test)]
mod corpus_probe {
    //! One-shot scan of the embedded component corpus that prints the
    //! current rule hits. Always passes — output shows up under
    //! `cargo test -- --nocapture` and is what the job report quotes.
    use crate::Severity;
    use crate::parse;

    #[test]
    fn probe_corpus() {
        let mut by_code: std::collections::BTreeMap<&'static str, usize> = Default::default();
        let mut hits: Vec<(String, String, String)> = Vec::new();
        for (tag, src) in super::COMPONENT_SOURCES {
            let Ok(file) = parse::parse(src) else { continue };
            let mut diags = Vec::new();
            super::analyze(&file, &mut diags);
            for d in diags
                .iter()
                .filter(|d| matches!(d.severity, Severity::Warning))
                .filter(|d| matches!(d.code.as_str(), "SAIL/W0101" | "SAIL/W0105" | "SAIL/W0201"))
            {
                let code: &'static str = match d.code.as_str() {
                    "SAIL/W0101" => "SAIL/W0101",
                    "SAIL/W0105" => "SAIL/W0105",
                    "SAIL/W0201" => "SAIL/W0201",
                    _ => continue,
                };
                *by_code.entry(code).or_default() += 1;
                hits.push(((*tag).to_string(), d.code.clone(), d.message.clone()));
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
