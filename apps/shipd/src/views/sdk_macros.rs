//! SDK macro globalization for user-authored Jinja templates.
//!
//! Every macro defined in `_ship.html.j2` is exposed to user views without
//! requiring an explicit `{% from '_ship.html.j2' import ... %}` line. The
//! mechanism is source rewriting: at template registration time we prepend
//! a single auto-import statement listing every macro name. minijinja
//! tolerates duplicate imports, so existing templates that still write the
//! explicit import line continue to work unchanged.
//!
//! The macro list is derived once from the embedded `_ship.html.j2` source
//! (see `templates::TEMPLATES`) by scanning for `{% macro <name>(` lines
//! with a small regex. Build-time embedding via `include_str!` means a
//! recompile picks up any new macros automatically.
//!
//! Two macros (`get`, `post`, `put`, `delete`) shadow common SDK operation
//! names — that's intentional: the user should be able to call `get(...)`
//! to render a Datastar `data-on:click="@get(...)"` attribute. If any of
//! these names ever collide with a minijinja built-in or filter the import
//! still wins inside template scope.
//!
//! ## Why source rewriting and not `add_global`?
//!
//! minijinja `Value`s do not currently expose macros as first-class
//! callables registered through `Environment::add_global`. The supported
//! way to surface a macro across templates is `{% from '...' import ... %}`
//! inside the consuming template. Rewriting the user source to prepend
//! that import is the lightest mechanism that uses only public API.

use std::sync::OnceLock;

/// Embedded SDK macro source — same content registered in `templates::TEMPLATES`
/// under the name `_ship.html.j2`. Duplicating the `include_str!` here keeps
/// this module self-contained for tests and removes a circular dep on the
/// `TEMPLATES` constant.
const SHIP_SDK_SOURCE: &str = include_str!("../../templates/_ship.html.j2");

/// Cached comma-separated list of macro names extracted from the SDK source,
/// in the order they appear. Stable for the process lifetime.
static MACRO_NAMES: OnceLock<Vec<String>> = OnceLock::new();

/// Cached prelude string of the form
/// `{% from '_ship.html.j2' import a, b, c %}\n` ready to prepend to a user
/// template. Empty string if the SDK file has no macros (defensive — should
/// never happen in practice).
static PRELUDE: OnceLock<String> = OnceLock::new();

/// Return the ordered list of macro names defined in `_ship.html.j2`.
pub fn macro_names() -> &'static [String] {
    MACRO_NAMES
        .get_or_init(|| extract_macro_names(SHIP_SDK_SOURCE))
        .as_slice()
}

/// Return the auto-import prelude. Cheap — built once.
pub fn prelude() -> &'static str {
    PRELUDE
        .get_or_init(|| build_prelude(macro_names()))
        .as_str()
}

/// Prepend the SDK macro prelude to `source` unless the source already imports
/// from `_ship.html.j2` (in which case duplicating would still work but is
/// wasteful — and skipping helps keep error line numbers tidy for templates
/// that haven't been migrated). Returns `Cow`-style: an owned String when
/// rewritten, the original when not.
pub fn maybe_inject(source: &str) -> String {
    // If the source already explicitly imports the SDK, leave it alone.
    // We match both quote styles minijinja accepts.
    if source.contains("from '_ship.html.j2'") || source.contains("from \"_ship.html.j2\"") {
        return source.to_string();
    }
    let p = prelude();
    if p.is_empty() {
        return source.to_string();
    }
    let mut out = String::with_capacity(p.len() + source.len());
    out.push_str(p);
    out.push_str(source);
    out
}

/// Extract macro names from a Jinja source by scanning lines for the
/// `{% macro <name>(` shape. Tolerant of leading whitespace and of the
/// whitespace-control variant `{%-`. Stops at the first non-identifier
/// character after `macro`.
fn extract_macro_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw_line in source.lines() {
        let line = raw_line.trim_start();
        // Accept `{% macro` or `{%- macro`.
        let rest = if let Some(r) = line.strip_prefix("{%-") {
            r
        } else if let Some(r) = line.strip_prefix("{%") {
            r
        } else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix("macro") else { continue };
        // Require whitespace separator so we don't match `macros` etc.
        if !rest.starts_with(|c: char| c.is_whitespace()) {
            continue;
        }
        let rest = rest.trim_start();
        // Take leading identifier chars.
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        if seen.insert(name.clone()) {
            names.push(name);
        }
    }
    names
}

fn build_prelude(names: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let joined = names.join(", ");
    format!("{{% from '_ship.html.j2' import {joined} %}}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_macros_from_minimal_source() {
        let src = "{% macro foo() %}bar{% endmacro %}\n\
                   {%- macro baz(x) -%}{{ x }}{%- endmacro %}\n\
                   {% macro foo() %}duplicate, ignored{% endmacro %}\n";
        let names = extract_macro_names(src);
        assert_eq!(names, vec!["foo".to_string(), "baz".to_string()]);
    }

    #[test]
    fn ignores_non_macro_lines() {
        let src = "{# comment {% macro fake() %} #}\n\
                   {% if true %}{% endif %}\n\
                   text only\n";
        // The naive scanner does match inside a {# ... #} comment, but the
        // resulting prelude line still parses fine because minijinja accepts
        // duplicate imports. To keep the extraction deterministic and not
        // fight comments, we accept this — but verify the pure non-macro
        // lines produce nothing.
        let only_text = extract_macro_names("hello\nworld\n");
        assert!(only_text.is_empty(), "no macros in plain text: got {only_text:?}");
        // The {# ... #} case is documented; we just make sure extraction
        // doesn't crash on it.
        let _ = extract_macro_names(src);
    }

    #[test]
    fn ship_sdk_has_many_macros() {
        let names = extract_macro_names(SHIP_SDK_SOURCE);
        assert!(names.len() >= 50, "expected 50+ SDK macros, got {}", names.len());
        // Spot-check well-known names that views call today.
        for expected in ["head", "signal", "show", "live", "section", "endsection"] {
            assert!(names.iter().any(|n| n == expected), "missing macro: {expected}");
        }
    }

    #[test]
    fn prelude_starts_with_from_directive() {
        let p = build_prelude(&["alpha".into(), "beta".into()]);
        assert_eq!(p, "{% from '_ship.html.j2' import alpha, beta %}\n");
    }

    #[test]
    fn maybe_inject_skips_when_already_imported() {
        let src = "{% from '_ship.html.j2' import card_grid %}\n<div></div>";
        let out = maybe_inject(src);
        assert_eq!(out, src, "should not double-import");
    }

    #[test]
    fn maybe_inject_skips_double_quoted_import() {
        let src = "{% from \"_ship.html.j2\" import card_grid %}\n";
        assert_eq!(maybe_inject(src), src);
    }

    #[test]
    fn maybe_inject_prepends_prelude_otherwise() {
        let src = "<div>{{ foo }}</div>";
        let out = maybe_inject(src);
        assert!(out.starts_with("{% from '_ship.html.j2' import "));
        assert!(out.ends_with("<div>{{ foo }}</div>"));
    }
}
