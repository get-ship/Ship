//! Deck's own `/_sui/...` serve-scheme URL builders.
//!
//! Deck owns the URL contract for the assets and views it hosts. Anything that
//! needs a `/_sui/...` URL on the Rust side builds it here rather than hand-
//! writing the literal, so the scheme has one source of truth in the layer that
//! owns serving. (The TypeScript side derives the same scheme from the daemon's
//! generated route table; this is the deck-internal counterpart — see #157/#158
//! "deck owns the serve contract".)

/// URL of a view's compiled ESM module. `sub` is the optional sub-route segment
/// for nested/parameterized views (`/x/:id`), carried as a `?sub=` query so the
/// host resolves the right leaf.
pub fn view_module_url(name: &str, sub: Option<&str>) -> String {
    match sub.filter(|s| !s.is_empty()) {
        Some(s) => format!("/_sui/views/{name}/view.js?sub={s}"),
        None => format!("/_sui/views/{name}/view.js"),
    }
}

/// URL of a view's raw `.sui` template source.
pub fn view_template_url(name: &str) -> String {
    format!("/_sui/views/{name}/view.sui")
}

/// URL of a view's compiled CSS.
pub fn view_css_url(name: &str) -> String {
    format!("/_sui/views/{name}/view.css")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_module_url_with_and_without_sub() {
        assert_eq!(view_module_url("demo", None), "/_sui/views/demo/view.js");
        assert_eq!(
            view_module_url("demo", Some("anything")),
            "/_sui/views/demo/view.js?sub=anything",
        );
        // Empty sub is treated as no sub.
        assert_eq!(view_module_url("demo", Some("")), "/_sui/views/demo/view.js");
    }
}
