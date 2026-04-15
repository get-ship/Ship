//! Minijinja template environment for daemon-rendered views.
//!
//! Templates live at `apps/shipd/templates/**/*.html.j2` and are embedded
//! into the binary at compile time via `minijinja::path_loader` over an
//! `include_dir!`-style bundle. Because the templates must be available
//! without the source tree present at runtime, we embed each template via
//! `include_str!` and register it manually with `Environment::add_template`.
//!
//! Shared partials live at the top level of `templates/` and are prefixed
//! with an underscore (`_page_shell.html.j2`, `_loading.html.j2`, ...).
//! View-specific templates go under `templates/<view>/...`. A view template
//! typically `{% extends '_page_shell.html.j2' %}` and overrides blocks.
//!
//! ## Blocks exposed by `_page_shell.html.j2`
//!
//! - `title` — document `<title>` text (default: "Ship").
//! - `head` — extra `<link>` / `<style>` / `<meta>` tags. The shell already
//!   loads tokens CSS, datastar.js, and ship-elements.js.
//! - `body_class` — extra class names on the `.ship-view` wrapper.
//! - `body` — primary content. Required for every extending template.
//!
//! ## Context contract
//!
//! Every template receives a serializable context struct via
//! [`render`]. Context structs should be `#[derive(Serialize)]` and live
//! next to the handler that renders them.

use minijinja::{Environment, Value};
use serde::Serialize;
use std::sync::OnceLock;

/// All templates shipped with the daemon. `(name, source)` pairs.
///
/// New templates must be added here to be loadable. Names use forward
/// slashes regardless of host OS (minijinja convention). Keep this list
/// alphabetized for easier diffing.
const TEMPLATES: &[(&str, &str)] = &[
    (
        "_breadcrumb.html.j2",
        include_str!("../../templates/_breadcrumb.html.j2"),
    ),
    // Auto-generated URL macros. Written to disk by `apps/shipd/build.rs`
    // at compile time and included here like any other template.
    (
        "_urls.html.j2",
        include_str!("../../templates/_urls.html.j2"),
    ),
    (
        "_empty.html.j2",
        include_str!("../../templates/_empty.html.j2"),
    ),
    (
        "_error.html.j2",
        include_str!("../../templates/_error.html.j2"),
    ),
    (
        "_loading.html.j2",
        include_str!("../../templates/_loading.html.j2"),
    ),
    (
        "_page_shell.html.j2",
        include_str!("../../templates/_page_shell.html.j2"),
    ),
    (
        "_raw_shell.html.j2",
        include_str!("../../templates/_raw_shell.html.j2"),
    ),
    (
        "_toast.html.j2",
        include_str!("../../templates/_toast.html.j2"),
    ),
    (
        "_toolbar.html.j2",
        include_str!("../../templates/_toolbar.html.j2"),
    ),
    (
        "agents/shell.html.j2",
        include_str!("../../templates/agents/shell.html.j2"),
    ),
    (
        "agents/detail.html.j2",
        include_str!("../../templates/agents/detail.html.j2"),
    ),
    (
        "agents/partials/card.html.j2",
        include_str!("../../templates/agents/partials/card.html.j2"),
    ),
    (
        "canvas/blank.html.j2",
        include_str!("../../templates/canvas/blank.html.j2"),
    ),
    (
        "chat/shell.html.j2",
        include_str!("../../templates/chat/shell.html.j2"),
    ),
    (
        "chat/partials/composer.html.j2",
        include_str!("../../templates/chat/partials/composer.html.j2"),
    ),
    (
        "chat/partials/message.html.j2",
        include_str!("../../templates/chat/partials/message.html.j2"),
    ),
    (
        "chat/partials/messages.html.j2",
        include_str!("../../templates/chat/partials/messages.html.j2"),
    ),
    (
        "events/shell.html.j2",
        include_str!("../../templates/events/shell.html.j2"),
    ),
    (
        "events/content.html.j2",
        include_str!("../../templates/events/content.html.j2"),
    ),
    (
        "events/partials/event_row.html.j2",
        include_str!("../../templates/events/partials/event_row.html.j2"),
    ),
    (
        "events/partials/filters.html.j2",
        include_str!("../../templates/events/partials/filters.html.j2"),
    ),
    (
        "git/partials/diff_viewer.html.j2",
        include_str!("../../templates/git/partials/diff_viewer.html.j2"),
    ),
    (
        "git/partials/log_section.html.j2",
        include_str!("../../templates/git/partials/log_section.html.j2"),
    ),
    (
        "git/partials/review_banner.html.j2",
        include_str!("../../templates/git/partials/review_banner.html.j2"),
    ),
    (
        "git/partials/status_block.html.j2",
        include_str!("../../templates/git/partials/status_block.html.j2"),
    ),
    (
        "git/shell.html.j2",
        include_str!("../../templates/git/shell.html.j2"),
    ),
    (
        "pipelines/content.html.j2",
        include_str!("../../templates/pipelines/content.html.j2"),
    ),
    (
        "pipelines/partials/active.html.j2",
        include_str!("../../templates/pipelines/partials/active.html.j2"),
    ),
    (
        "pipelines/partials/def_card.html.j2",
        include_str!("../../templates/pipelines/partials/def_card.html.j2"),
    ),
    (
        "pipelines/partials/definitions.html.j2",
        include_str!("../../templates/pipelines/partials/definitions.html.j2"),
    ),
    (
        "pipelines/partials/history.html.j2",
        include_str!("../../templates/pipelines/partials/history.html.j2"),
    ),
    (
        "pipelines/partials/history_row.html.j2",
        include_str!("../../templates/pipelines/partials/history_row.html.j2"),
    ),
    (
        "pipelines/partials/run_card.html.j2",
        include_str!("../../templates/pipelines/partials/run_card.html.j2"),
    ),
    (
        "pipelines/partials/step_card.html.j2",
        include_str!("../../templates/pipelines/partials/step_card.html.j2"),
    ),
    (
        "pipelines/shell.html.j2",
        include_str!("../../templates/pipelines/shell.html.j2"),
    ),
    (
        "settings/partials/about.html.j2",
        include_str!("../../templates/settings/partials/about.html.j2"),
    ),
    (
        "settings/partials/advanced.html.j2",
        include_str!("../../templates/settings/partials/advanced.html.j2"),
    ),
    (
        "settings/partials/provider_row.html.j2",
        include_str!("../../templates/settings/partials/provider_row.html.j2"),
    ),
    (
        "settings/partials/providers.html.j2",
        include_str!("../../templates/settings/partials/providers.html.j2"),
    ),
    (
        "settings/partials/terminal.html.j2",
        include_str!("../../templates/settings/partials/terminal.html.j2"),
    ),
    (
        "settings/shell.html.j2",
        include_str!("../../templates/settings/shell.html.j2"),
    ),
    (
        "skills/shell.html.j2",
        include_str!("../../templates/skills/shell.html.j2"),
    ),
    (
        "skills/partials/rail.html.j2",
        include_str!("../../templates/skills/partials/rail.html.j2"),
    ),
    (
        "skills/partials/detail_pane.html.j2",
        include_str!("../../templates/skills/partials/detail_pane.html.j2"),
    ),
    (
        "specs/content.html.j2",
        include_str!("../../templates/specs/content.html.j2"),
    ),
    (
        "specs/partials/detail.html.j2",
        include_str!("../../templates/specs/partials/detail.html.j2"),
    ),
    (
        "specs/partials/launch_bar.html.j2",
        include_str!("../../templates/specs/partials/launch_bar.html.j2"),
    ),
    (
        "specs/partials/rail.html.j2",
        include_str!("../../templates/specs/partials/rail.html.j2"),
    ),
    (
        "specs/shell.html.j2",
        include_str!("../../templates/specs/shell.html.j2"),
    ),
    // (Data browser view lives in .ship/views/data/ as a client-side view,
    // not as daemon-rendered Jinja templates.)
];

static ENV: OnceLock<Environment<'static>> = OnceLock::new();

/// Lazily-initialized template environment. Safe to call from any thread;
/// construction runs exactly once.
pub fn env() -> &'static Environment<'static> {
    ENV.get_or_init(build_env)
}

fn build_env() -> Environment<'static> {
    let mut env = Environment::new();
    // Lenient: `{{ foo | default('x') }}` and `{% if foo is defined %}` work
    // naturally. Strict mode errors on any undefined lookup — too noisy for
    // partials whose context fields are documented as optional.
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    env.add_filter("truncate", |value: String, length: usize| -> String {
        if value.chars().count() <= length {
            value
        } else {
            let mut s: String = value.chars().take(length.saturating_sub(1)).collect();
            s.push('…');
            s
        }
    });
    for (name, src) in TEMPLATES {
        // The registered `TEMPLATES` list is a compile-time constant of
        // valid minijinja sources — a parse failure here is a programmer
        // error, not a runtime condition, so unwrap is appropriate.
        env.add_template(name, src)
            .expect("template source must parse");
    }
    env
}

/// Render a registered template with a serializable context.
///
/// Errors propagate as `String` for easy composition inside axum handlers
/// that already convert to 500 responses. Template-not-found and render
/// errors both surface here with a useful message.
pub fn render<C: Serialize>(template: &str, ctx: &C) -> Result<String, String> {
    let tpl = env()
        .get_template(template)
        .map_err(|e| format!("template '{template}' not found: {e}"))?;
    let value = Value::from_serialize(ctx);
    tpl.render(value)
        .map_err(|e| format!("template '{template}' render error: {e}"))
}

/// Render an ad-hoc template string with access to all registered macros.
///
/// Used for `.ship/views/` HTML files that contain Jinja syntax. The
/// template inherits the same environment as daemon-internal views —
/// URL macros, filters, lenient undefined behavior — so views in
/// `.ship/views/` can use `{% from '_urls.html.j2' import ... %}`.
pub fn render_view_source<C: Serialize>(name: &str, source: &str, ctx: &C) -> Result<String, String> {
    // If the source contains no Jinja syntax, return it as-is (fast path).
    if !source.contains("{%") && !source.contains("{{") {
        return Ok(source.to_string());
    }

    // Create a child environment that inherits all registered templates
    // (macros, partials) but can also compile an ad-hoc source string.
    // minijinja doesn't support child envs, so we add the source as a
    // temporary template to the shared env — but the env is immutable
    // (OnceLock). Instead, create a fresh env per call with the same
    // templates plus the ad-hoc one.
    let mut local_env = Environment::new();
    local_env.set_undefined_behavior(minijinja::UndefinedBehavior::Lenient);
    // Auto-escape HTML in user views to prevent XSS from collection data.
    local_env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    local_env.add_filter("truncate", |value: String, length: usize| -> String {
        if value.chars().count() <= length { value }
        else { let mut s: String = value.chars().take(length.saturating_sub(1)).collect(); s.push('…'); s }
    });
    // Copy all registered templates (macros, partials).
    for (tpl_name, tpl_src) in TEMPLATES {
        let _ = local_env.add_template(tpl_name, tpl_src);
    }
    // Add the view source as a named template.
    local_env.add_template(name, source)
        .map_err(|e| format!("view '{name}' template parse error: {e}"))?;

    let tpl = local_env.get_template(name)
        .map_err(|e| format!("view '{name}' not found after add: {e}"))?;
    let value = Value::from_serialize(ctx);
    tpl.render(value)
        .map_err(|e| format!("view '{name}' render error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct EmptyCtx {
        icon: &'static str,
        title: &'static str,
        body: &'static str,
    }

    #[test]
    fn env_initializes_and_registers_every_template() {
        let e = env();
        for (name, _) in TEMPLATES {
            assert!(
                e.get_template(name).is_ok(),
                "template '{name}' must be registered"
            );
        }
    }

    #[test]
    fn render_empty_partial_produces_ship_empty_state() {
        let html = render(
            "_empty.html.j2",
            &EmptyCtx {
                icon: "inbox",
                title: "Nothing here",
                body: "Create one to get started.",
            },
        )
        .expect("render must succeed");
        assert!(html.contains("ship-empty-state"));
        assert!(html.contains("Nothing here"));
    }

    #[test]
    fn missing_template_yields_error() {
        #[derive(Serialize)]
        struct Empty {}
        let err = render("does-not-exist.html.j2", &Empty {}).unwrap_err();
        assert!(err.contains("not found"));
    }
}
