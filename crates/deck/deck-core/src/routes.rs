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

/// URL the daemon serves for form-mutation POST submission.
/// Matches the daemon's `POST /_sui/mutation/{name}` route exactly.
pub fn mutation_action(name: &str, form_key: &str) -> String {
    format!("/_sui/mutation/{name}?form={form_key}")
}

/// URL the daemon serves for live-data SSE streaming.
/// Matches the daemon's `GET /_sui/views/{name}/sse` route exactly.
pub fn subscribe_url(name: &str) -> String {
    format!("/_sui/views/{name}/sse")
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

    #[test]
    fn mutation_action_builds_correct_url() {
        assert_eq!(
            mutation_action("todos", "b0"),
            "/_sui/mutation/todos?form=b0",
        );
        assert_eq!(
            mutation_action("my-view", "b3"),
            "/_sui/mutation/my-view?form=b3",
        );
    }

    #[test]
    fn subscribe_url_builds_correct_url() {
        assert_eq!(subscribe_url("clock"), "/_sui/views/clock/sse");
        assert_eq!(subscribe_url("studio/agents"), "/_sui/views/studio/agents/sse");
    }
}
