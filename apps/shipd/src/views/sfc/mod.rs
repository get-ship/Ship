//! Single-file component (`.ship`) compiler.
//!
//! Pipeline (component path):
//! 1. **Block split** ([`parser`]) — pull `<script>` and `<style>` out;
//!    the remainder is the Jinja template body.
//! 2. **Schema extract** ([`script`]) — discover prop names + defaults,
//!    signal names, lifecycle-hook usage from the JS source.
//! 3. **Render template** ([`crate::views::templates`]) — turn the Jinja
//!    body into HTML. SDK macros are auto-imported (see `sdk_macros`).
//! 4. **Signal rewrite** ([`template`]) — replace `$name` with
//!    `$_<safe_tag>___SHIP_INST___<name>` inside `data-*` attributes so
//!    multiple instances on a page never share a signal namespace.
//!    `safe_tag` is the component tag with hyphens replaced by
//!    underscores (custom-element tags must contain a hyphen, but
//!    Datastar parses attribute values as JS expressions where `-`
//!    means subtraction).
//! 5. **Style scope** ([`style`]) — prepend
//!    `[data-component="<tag>"] ` to each top-level selector unless
//!    `<style global>` is set.
//! 6. **Emit JS** ([`emit`]) — produce a self-contained ES module that
//!    defines the custom element and registers it.
//!
//! View-side handling (where the file lives in `.ship/views/`) is left
//! for a follow-up — the format applies (block split + style scoping)
//! but the script block has nowhere to attach without a host element.

pub mod emit;
pub mod emit_helpers;
pub mod parser;
pub mod script;
pub mod script_scan;
pub mod style;
pub mod style_walk;
pub mod template;

#[cfg(test)]
mod emit_tests;

use crate::views::templates::render_view_source;
use parser::{split, style_is_global};
use script::extract;
use style::scope_css;
use template::rewrite_signals;

/// Compile a `.ship` component source into the JavaScript module that
/// defines and registers the custom element.
///
/// `tag` is the kebab-cased element name (e.g. `app-typeahead`).
/// `template_name` is the minijinja template identifier used for cache
/// keys + error overlays (e.g. `components/app-typeahead.ship`).
pub fn compile_component(
    tag: &str,
    template_name: &str,
    source: &str,
) -> Result<String, String> {
    let blocks = split(source);
    let schema = blocks
        .script
        .as_deref()
        .map(extract)
        .unwrap_or_default();

    // Render the Jinja template body. We pass an empty context — the
    // body cannot rely on per-request data because each browser-side
    // instance shares the same compiled HTML; per-instance values come
    // from props at runtime, not from server-side rendering.
    let context = serde_json::json!({ "props": {} });
    let rendered = render_view_source(template_name, &blocks.template, &context)
        .map_err(|e| format!("template render: {e}"))?;

    let scoped_html = rewrite_signals(&rendered, tag, &schema.signals);

    let scoped_css = match blocks.style.as_deref() {
        Some(css) if style_is_global(&blocks.style_attrs) => css.to_string(),
        Some(css) => scope_css(css, tag),
        None => String::new(),
    };

    Ok(emit::emit(&emit::EmitInputs {
        tag,
        schema: &schema,
        user_script: blocks.script.as_deref().unwrap_or(""),
        rendered_html: &scoped_html,
        scoped_css: &scoped_css,
    }))
}

/// Convert a `.ship/components/foo-bar.ship` file path stem into the
/// custom-element tag. The stem must already be kebab-cased and
/// contain at least one hyphen (custom-element naming requirement).
/// Returns `None` if the stem is not a valid tag.
pub fn tag_from_stem(stem: &str) -> Option<String> {
    if stem.is_empty() { return None; }
    if !stem.contains('-') { return None; }
    if !stem
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return None;
    }
    if !stem.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
        return None;
    }
    Some(stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_from_stem_validates_kebab() {
        assert_eq!(tag_from_stem("app-typeahead"), Some("app-typeahead".to_string()));
        assert_eq!(tag_from_stem("a-b-c"), Some("a-b-c".to_string()));
        assert_eq!(tag_from_stem("noHyphen"), None);
        assert_eq!(tag_from_stem("-leading"), None);
        assert_eq!(tag_from_stem("UPPER-CASE"), None);
        assert_eq!(tag_from_stem(""), None);
    }

    #[test]
    fn compile_minimal_component() {
        let src = "<script>const $x = signal(0)</script>\n<div data-text=\"$x\"></div>\n<style>div { color: red }</style>";
        let out = compile_component("my-x", "components/my-x.ship", src).unwrap();
        assert!(out.contains("customElements.define(TAG, ShipComponent)"));
        // Tag hyphen is sanitized to `_` in the rewritten signal namespace
        // so Datastar's expression evaluator doesn't parse `-` as subtraction.
        assert!(out.contains("$_my_x___SHIP_INST___x"));
        assert!(out.contains("[data-component=\"my-x\"] div"));
        assert!(out.contains("SIGNAL_NAMES = [\"x\"]"));
    }

    #[test]
    fn compile_full_typeahead_end_to_end() {
        // Embed the on-disk typeahead source so this test fails loudly if
        // the demo component drifts out of sync with the compiler.
        let src = include_str!("../../../../../.ship/components/app-typeahead.ship");
        let out = compile_component(
            "app-typeahead",
            "components/app-typeahead.ship",
            src,
        )
        .expect("typeahead must compile");

        // Define + tag + signals + props all present.
        assert!(out.contains("customElements.define(TAG, ShipComponent)"));
        assert!(out.contains("const TAG = \"app-typeahead\""));
        for sig in ["query", "items", "loading", "activeIndex", "open"] {
            assert!(
                out.contains(&format!("\"{sig}\"")),
                "signal name '{sig}' missing in SIGNAL_NAMES"
            );
        }
        for attr in ["placeholder", "fetch-url", "min-chars", "debounce-ms"] {
            assert!(
                out.contains(&format!("\"{attr}\"")),
                "prop attr '{attr}' missing in PROP_NAMES"
            );
        }

        // Per-instance signal scoping landed in data-* attributes. Tag
        // hyphen sanitized to `_` so the JS-side `__sigKey` (which does
        // the same sanitization) and the attribute reference agree.
        assert!(out.contains("$_app_typeahead___SHIP_INST___query"));
        assert!(out.contains("$_app_typeahead___SHIP_INST___open"));

        // Style is scoped, not global. The CSS lives inside a JS
        // template literal so double quotes appear unescaped.
        assert!(out.contains(r#"[data-component="app-typeahead"] .typeahead"#));
        assert!(out.contains(r#"[data-component="app-typeahead"] .suggestions li.active"#));
    }

    #[test]
    fn compile_global_style_skips_scoping() {
        let src = "<style global>li.active { color: red }</style>\n<div></div>";
        let out = compile_component("my-x", "components/my-x.ship", src).unwrap();
        assert!(out.contains("li.active { color: red }"));
        assert!(!out.contains("[data-component=\"my-x\"] li.active"));
    }
}
