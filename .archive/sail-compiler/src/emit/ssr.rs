//! Phase 7 — SSR emission.
//!
//! For MVP, "SSR" means the same anchored HTML the runtime uses as its
//! `HTML` constant — wrapped in a `<TAG data-component="TAG">…</TAG>`
//! host. The daemon serves this as initial paint; the runtime hydrates
//! by querying the same `data-sa` / `data-st` anchors.
//!
//! Reactive interpolations render empty in v1 (the runtime fills on
//! mount). A follow-up will pre-evaluate compile-time-knowable
//! expressions and bake initial values for `data` declarations the
//! schema source can resolve.
//!
//! See `docs/sail/02-compiler-architecture.md § Phase 7`.

use crate::parse::ast::SailFile;

/// Render initial-paint HTML for the given parsed file. Returns the
/// host element wrapped HTML so the daemon can drop it directly into
/// a page.
pub fn emit(file: &SailFile, tag: &str) -> String {
    let (_js, inner) = super::runtime::emit_with_html(file, tag);
    format!("<{tag} data-component=\"{tag}\">{inner}</{tag}>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn wraps_inner_with_host_tag() {
        let file = parse::parse("<template><span>{x}</span></template>").unwrap();
        let html = emit(&file, "sail-x");
        assert!(html.starts_with("<sail-x data-component=\"sail-x\">"));
        assert!(html.ends_with("</sail-x>"));
        assert!(html.contains("data-st=\"0\""));
    }

    #[test]
    fn static_template_passes_through() {
        let file = parse::parse("<template><h1>Hello</h1></template>").unwrap();
        let html = emit(&file, "sail-greet");
        assert!(html.contains("<h1>Hello</h1>"));
    }
}
