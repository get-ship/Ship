//! HTML document wrapper for `/set/<view>` (production SSR + attach).
//!
//! Same boot-script shape as `ship_set_render.rs::wrap_document` —
//! both call into `ship_set_attach_boot` for the importmap,
//! components side-effect import, and attach() invocation. This
//! handler differs only in the production-specific outer chrome:
//! per-view `<title>`, `view-tokens.css`, `ship-elements.js`
//! legacy include, `id="sui-root"` + `data-view="{name}"` for
//! selector compatibility.

use super::sui_attach_boot::{
    attach_boot_script, components_side_effect_script, importmap_script,
};

/// Wrap the SSR walker fragment in the production document shell.
pub fn render_shell(name: &str, title: &str, ssr_html: &str) -> String {
    let safe_name = escape_html(name);
    let safe_title = escape_html(title);
    let importmap = importmap_script();
    let components = components_side_effect_script();
    let boot = attach_boot_script(
        &safe_name,
        "document.getElementById('sui-root')",
        "view",
    );
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{safe_title}</title>
  <link rel="stylesheet" href="/api/assets/view-tokens.css">
  <link rel="stylesheet" href="/_sui/views/{safe_name}/view.css">
  <script type="module" src="/api/assets/ship-elements.js"></script>
  {importmap}
  {components}
</head>
<body>
  <div id="sui-root" data-ssr-root data-view="{safe_name}">{ssr_html}</div>
  {boot}
</body>
</html>
"#
    )
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_inlines_view_name_and_ssr_fragment() {
        let html = render_shell(
            "agents",
            "Agents",
            "<button bind:value=\"b0\">walker</button>",
        );
        assert!(html.contains(r#"data-view="agents""#));
        assert!(html.contains("data-ssr-root"));
        assert!(html.contains(r#"<title>Agents</title>"#));
        assert!(html.contains("/_sui/views/agents/view.js"));
        assert!(html.contains("bind:value=\"b0\""));
        assert!(html.contains("import { attach } from '@ship/ui-dom'"));
        // viewName must be threaded through so attach() can call the
        // /_sui/render-fragment/<view> endpoint on control-flow swaps.
        assert!(html.contains("viewName: 'agents'"), "missing viewName in attach() opts: {html}");
        // Old parse-and-build path is gone.
        assert!(!html.contains("hydrate(template"));
        assert!(!html.contains("fetch('/_sui/views/agents/view.sui')"));
    }

    #[test]
    fn shell_escapes_special_chars_in_name() {
        let html = render_shell("a&b", "A & B", "");
        assert!(html.contains("data-view=\"a&amp;b\""));
        assert!(html.contains("<title>A &amp; B</title>"));
    }
}
