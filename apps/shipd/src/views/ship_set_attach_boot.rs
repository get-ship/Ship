//! Shared `<script type="module">` boot block for ship-set views.
//!
//! Both `/set/<view>` (production) and `/_ship-set/render/<view>`
//! (debug) need the exact same attach() invocation: import the
//! compiled view bundle, import `{ attach }` from the dom package,
//! create the SDK client, look up the `[data-ssr-root]` element, call
//! attach(...) with viewName threaded through. Phase E-2 added the
//! viewName threading; the production handler missed it on the first
//! pass, which broke control-flow swap until caught in the cleanup
//! arc. Centralizing the boot block here so future regressions can't
//! diverge the two call sites.
//!
//! The shared importmap is also factored out — every ship-set page
//! loads the same `@ship-set/runtime`, `@ship-set/dom`,
//! `@ship-set/sdk-client` mappings. Extracting prevents accidental
//! drift between debug and production resolution paths.

use std::fmt::Write;

/// Inline importmap that resolves the bare specifiers our runtime
/// bundles import from. Authors don't write these — they live inside
/// `@ship-set/dom` etc. Returned as a `<script type="importmap">…`
/// block, ready to drop into `<head>`.
pub fn importmap_script() -> &'static str {
    r#"<script type="importmap">
  {
    "imports": {
      "@ship-set/runtime":     "/_ship-set/runtime.js",
      "@ship-set/dom":         "/_ship-set/dom.js",
      "@ship-set/sdk-client":  "/_ship-set/sdk-client.js"
    }
  }
  </script>"#
}

/// Side-effect import that registers every `<set-*>` ported
/// component on page load. Must run before any view template tries
/// to instantiate one. Safe to drop into `<head>` after the importmap.
pub fn components_side_effect_script() -> &'static str {
    r#"<script type="module" src="/_ship-set/components.js"></script>"#
}

/// The shared attach() boot block. Threads `viewName` through so
/// Phase E-2's control-flow swap can POST to
/// `/_ship-set/render-fragment/<view>`. The error logger label is
/// per-handler so `console.error` traces back to the call site
/// (`[ship-set/view]` for prod, `[ship-set/render]` for debug).
///
/// `view_name` is HTML-escaped by the caller (see callers' helpers);
/// `error_label` is a plain ASCII identifier baked into the source.
/// `root_selector` lets callers pick `getElementById('ship-set-root')`
/// for prod compatibility or `querySelector('[data-ssr-root]')` for
/// the simpler debug shell.
pub fn attach_boot_script(
    view_name_escaped: &str,
    root_selector: &str,
    error_label: &str,
) -> String {
    let mut s = String::with_capacity(640);
    write!(
        &mut s,
        r#"<script type="module">
    import * as viewModule from '/_ship-set/views/{view_name_escaped}/view.js';
    import {{ attach }} from '@ship-set/dom';
    import {{ route, navigate }} from '@ship-set/runtime';
    import {{ createClient }} from '@ship-set/sdk-client';

    const ship = createClient();
    const routeSignals = route();

    const root = {root_selector};
    try {{
      const dispose = attach(root, {{ module: viewModule, ship, route: routeSignals, navigate, viewName: '{view_name_escaped}' }});
      window.__shipSetDispose = dispose;
    }} catch (err) {{
      console.error('[ship-set/{error_label}] attach failed:', err);
    }}
  </script>"#
    )
    .expect("write to String never fails");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_script_threads_view_name_and_error_label() {
        let s = attach_boot_script("agents", "document.getElementById('ship-set-root')", "view");
        assert!(s.contains("/_ship-set/views/agents/view.js"));
        assert!(s.contains("viewName: 'agents'"));
        assert!(s.contains("[ship-set/view]"));
        assert!(s.contains("document.getElementById('ship-set-root')"));
        // Reactive route plumbing: imports `route` + `navigate` from
        // @ship-set/runtime and instantiates a RouteSignals via route().
        assert!(s.contains("from '@ship-set/runtime'"));
        assert!(s.contains("route()"));
        assert!(s.contains("route: routeSignals"));
    }

    #[test]
    fn debug_variant_uses_querySelector() {
        let s = attach_boot_script("counter", "document.querySelector('[data-ssr-root]')", "render");
        assert!(s.contains("viewName: 'counter'"));
        assert!(s.contains("[ship-set/render]"));
        assert!(s.contains("[data-ssr-root]"));
    }

    #[test]
    fn importmap_lists_all_three_packages() {
        let s = importmap_script();
        assert!(s.contains("@ship-set/runtime"));
        assert!(s.contains("@ship-set/dom"));
        assert!(s.contains("@ship-set/sdk-client"));
    }
}
