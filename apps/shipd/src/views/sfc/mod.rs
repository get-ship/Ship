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
use parser::{split, style_is_global, style_src};
use script::extract;
use std::path::Path;
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
    compile_component_with_path(tag, template_name, source, None)
}

/// Like [`compile_component`] but accepts the source-file path so
/// `<style src="./shared.css">` external imports can resolve relative to
/// the .ship file's directory. Pass `None` if no on-disk path exists
/// (tests that compile from a string in memory).
pub fn compile_component_with_path(
    tag: &str,
    template_name: &str,
    source: &str,
    source_path: Option<&Path>,
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

    // Prop names join the signal set for the namespace mangling so
    // `data-text="$placeholder"` (a prop reference) gets the same
    // per-instance scoping as `data-text="$count"` (a signal reference).
    // The wrapper auto-creates a signal for every prop at mount time.
    let mut scoped_names: Vec<String> = schema.signals.clone();
    scoped_names.extend(schema.props.iter().map(|p| p.name.clone()));
    let scoped_html = rewrite_signals(&rendered, tag, &scoped_names);

    // Style sourcing: in priority order, take the first non-empty:
    // - external file via <style src="./shared.css">
    // - inline <style>...</style> body
    // External + inline CAN coexist; external loads first, inline appends.
    // Both go through the scoper unless `global` is set.
    let style_body = resolve_style_body(&blocks, source_path)?.unwrap_or_default();
    let scoped_css = if style_body.is_empty() {
        String::new()
    } else if style_is_global(&blocks.style_attrs) {
        style_body
    } else {
        scope_css(&style_body, tag)
    };

    Ok(emit::emit(&emit::EmitInputs {
        tag,
        schema: &schema,
        user_script: blocks.script.as_deref().unwrap_or(""),
        rendered_html: &scoped_html,
        scoped_css: &scoped_css,
    }))
}

/// Resolve the style body for a component, combining `<style src="...">`
/// external content with the inline body. External content loads first,
/// inline appends. Returns `None` if neither source produces content.
fn resolve_style_body(
    blocks: &parser::SplitBlocks,
    source_path: Option<&Path>,
) -> Result<Option<String>, String> {
    let inline = blocks.style.as_deref().unwrap_or("");
    let src_attr = style_src(&blocks.style_attrs);

    let external = match (src_attr.as_deref(), source_path) {
        (Some(rel), Some(src_path)) => {
            // Resolve relative to the .ship file's directory.
            let base = src_path.parent().ok_or_else(|| {
                format!("source path has no parent: {}", src_path.display())
            })?;
            let target = base.join(rel);
            // Reject path traversal beyond the project's .ship dir is
            // out of scope here; the file is read by the daemon under
            // the trusted repo, not from user input.
            std::fs::read_to_string(&target)
                .map_err(|e| format!("style src `{}` ({}): {e}", rel, target.display()))?
        }
        (Some(rel), None) => {
            return Err(format!(
                "<style src=\"{rel}\"> requires a source-file path; pass via compile_component_with_path"
            ));
        }
        (None, _) => String::new(),
    };

    if external.is_empty() && inline.is_empty() {
        return Ok(None);
    }
    let mut combined = String::with_capacity(external.len() + inline.len() + 1);
    combined.push_str(&external);
    if !external.is_empty() && !inline.is_empty() {
        combined.push('\n');
    }
    combined.push_str(inline);
    Ok(Some(combined))
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
    fn compile_with_external_style_src_loads_file() {
        // Write a tiny shared CSS file to a temp dir, point a .ship file
        // at it via <style src=...>, confirm the daemon reads + scopes it.
        let dir = tempfile::tempdir().expect("tempdir");
        let css_path = dir.path().join("shared.css");
        std::fs::write(&css_path, ".tablist { display: flex; }").unwrap();

        let ship_path = dir.path().join("my-comp.ship");
        let src = r#"<style src="./shared.css"></style>
<div class="tablist"><slot></slot></div>"#;
        std::fs::write(&ship_path, src).unwrap();

        let out = compile_component_with_path(
            "my-comp",
            "components/my-comp.ship",
            src,
            Some(&ship_path),
        )
        .unwrap();
        // Loaded + scoped.
        assert!(out.contains("[data-component=\"my-comp\"] .tablist"));
        assert!(out.contains("display: flex;"));
    }

    #[test]
    fn compile_with_external_style_concatenates_with_inline() {
        let dir = tempfile::tempdir().expect("tempdir");
        let css_path = dir.path().join("base.css");
        std::fs::write(&css_path, ".base { color: red; }").unwrap();

        let ship_path = dir.path().join("my-comp.ship");
        let src = r#"<style src="./base.css">
.local { color: blue; }
</style>
<div></div>"#;
        std::fs::write(&ship_path, src).unwrap();

        let out = compile_component_with_path(
            "my-comp",
            "components/my-comp.ship",
            src,
            Some(&ship_path),
        )
        .unwrap();
        assert!(out.contains("[data-component=\"my-comp\"] .base"));
        assert!(out.contains("[data-component=\"my-comp\"] .local"));
    }

    #[test]
    fn compile_with_external_style_src_no_path_errors() {
        // If `src=` is set but no source_path is provided, fail loudly
        // — the import can't be resolved.
        let src = r#"<style src="./shared.css"></style><div></div>"#;
        let err = compile_component("my-comp", "components/my-comp.ship", src);
        assert!(err.is_err(), "should error without source path");
        assert!(err.unwrap_err().contains("source-file path"));
    }

    #[test]
    fn jinja_comments_are_stripped_from_template() {
        // {# ... #} should not survive into the served TEMPLATE constant
        // — minijinja strips them at render time.
        let src = "<script></script>\n{# private comment #}\n<div>visible</div>";
        let out = compile_component("my-x", "components/my-x.ship", src).unwrap();
        assert!(
            !out.contains("private comment"),
            "Jinja comment leaked into served JS: {out}"
        );
        assert!(out.contains("<div>visible</div>"));
    }

    #[test]
    fn props_become_scoped_signals_in_template() {
        // `$placeholder` in the template should rewrite to the scoped
        // namespace even though no `signal('placeholder')` call exists in
        // the script — declaring a prop is enough.
        let src = r#"<script>const { placeholder = 'Search' } = props</script>
<input data-attr:placeholder="$placeholder" />"#;
        let out = compile_component("my-input", "components/my-input.ship", src).unwrap();
        assert!(
            out.contains("$_my_input___SHIP_INST___placeholder"),
            "prop name should land in the scoped signal namespace"
        );
        // The wrapper should also pre-create the signal for this prop
        // and route attributeChangedCallback through it.
        assert!(
            out.contains("PROP_NAMES_CAMEL = [\"placeholder\"]"),
            "wrapper should know the camelCase prop name"
        );
        assert!(
            out.contains("host.__makeSignal(name, props[name])"),
            "wrapper should auto-create per-prop signal at bootstrap"
        );
        assert!(
            out.contains("attributeChangedCallback"),
            "wrapper should sync prop signals on attribute change"
        );
    }

    #[test]
    fn compile_global_style_skips_scoping() {
        let src = "<style global>li.active { color: red }</style>\n<div></div>";
        let out = compile_component("my-x", "components/my-x.ship", src).unwrap();
        assert!(out.contains("li.active { color: red }"));
        assert!(!out.contains("[data-component=\"my-x\"] li.active"));
    }
}
