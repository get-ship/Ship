//! HTML document wrapper for `/set/<view>` (production SSR + attach).
//!
//! Phase F extracted this from `ship_set_view::render_shell` so the
//! handler module stays under the 300-LOC cap. The wrapper is
//! deliberately the same shape as
//! `ship_set_render.rs::wrap_document` — same importmap, same boot
//! script — except this one preserves the legacy `id="ship-set-root"`
//! and `data-view="{name}"` attributes plus the
//! `<script type="module" src="/api/assets/ship-elements.js">`
//! include so the production URL stays compatible with anything that
//! grepped them.
//!
//! TODO(phase-G): merge with `ship_set_render::wrap_document` once the
//! F-1 debug endpoint and this handler can share a single render path.

/// Wrap the SSR walker fragment in the production document shell.
pub fn render_shell(name: &str, title: &str, ssr_html: &str) -> String {
    let safe_name = escape_html(name);
    let safe_title = escape_html(title);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{safe_title}</title>
  <link rel="stylesheet" href="/api/assets/view-tokens.css">
  <link rel="stylesheet" href="/_ship-set/views/{safe_name}/view.css">
  <script type="module" src="/api/assets/ship-elements.js"></script>
  <script type="importmap">
  {{
    "imports": {{
      "@ship-set/runtime":     "/_ship-set/runtime.js",
      "@ship-set/dom":         "/_ship-set/dom.js",
      "@ship-set/sdk-client":  "/_ship-set/sdk-client.js"
    }}
  }}
  </script>
  <!-- Side-effect import: registers every `<set-*>` ported component
       on page load, before any view template renders. -->
  <script type="module" src="/_ship-set/components.js"></script>
</head>
<body>
  <div id="ship-set-root" data-ssr-root data-view="{safe_name}">{ssr_html}</div>
  <script type="module">
    import * as viewModule from '/_ship-set/views/{safe_name}/view.js';
    import {{ attach }} from '@ship-set/dom';
    import {{ createClient }} from '@ship-set/sdk-client';

    const navigate = (path) => {{ window.location.href = path; }};
    const ship = createClient();
    const route = {{ params: {{}}, query: {{}} }};

    const root = document.getElementById('ship-set-root');
    try {{
      const dispose = attach(root, {{ module: viewModule, ship, route, navigate }});
      window.__shipSetDispose = dispose;
    }} catch (err) {{
      console.error('[ship-set/view] attach failed:', err);
    }}
  </script>
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
            "<div data-set-bind=\"b0\">walker</div>",
        );
        assert!(html.contains(r#"data-view="agents""#));
        assert!(html.contains("data-ssr-root"));
        assert!(html.contains(r#"<title>Agents</title>"#));
        assert!(html.contains("/_ship-set/views/agents/view.js"));
        assert!(html.contains("data-set-bind=\"b0\""));
        assert!(html.contains("import { attach } from '@ship-set/dom'"));
        // Old parse-and-build path is gone.
        assert!(!html.contains("hydrate(template"));
        assert!(!html.contains("fetch('/_ship-set/views/agents/view.html.set')"));
    }

    #[test]
    fn shell_escapes_special_chars_in_name() {
        let html = render_shell("a&b", "A & B", "");
        assert!(html.contains("data-view=\"a&amp;b\""));
        assert!(html.contains("<title>A &amp; B</title>"));
    }
}
