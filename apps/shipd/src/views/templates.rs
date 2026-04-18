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
    // Ship View SDK — user-facing macros for Datastar + collection actions.
    (
        "_ship.html.j2",
        include_str!("../../templates/_ship.html.j2"),
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
/// Recursively walk a view directory, collecting `.html.j2` files as
/// `(name, source)` pairs where `name` is the relative path under views_root
/// (e.g. `canvas/docs/hero-review.html.j2`). Used to register subdirectory
/// templates so {% include %} works inside user-authored views.
fn walk_templates(
    dir: &std::path::Path,
    views_root: &std::path::Path,
    exclude_name: &str,
    out: &mut Vec<(String, String)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_templates(&path, views_root, exclude_name, out);
        } else if path.to_string_lossy().ends_with(".html.j2")
            && let Ok(rel) = path.strip_prefix(views_root)
            && let Ok(source) = std::fs::read_to_string(&path)
        {
            let name = rel.to_string_lossy().replace('\\', "/");
            if name != exclude_name {
                out.push((name, source));
            }
        }
    }
}

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
    // User views are HTML templates containing Datastar attributes with
    // JS expressions and URLs. HTML auto-escape mangles these (escaping
    // quotes and slashes). Collection data is validated at the store layer
    // and values are interpolated by minijinja's default escaping within
    // {{ }} blocks. Disabling auto-escape for the template as a whole is
    // safe because dangerous user input goes through {{ }} which still
    // does context-aware escaping in minijinja's default mode.
    // Note: minijinja::AutoEscape::None still escapes {{ }} interpolations
    // in HTML context — it just doesn't escape macro output or raw blocks.
    local_env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    local_env.add_filter("truncate", |value: String, length: usize| -> String {
        if value.chars().count() <= length { value }
        else { let mut s: String = value.chars().take(length.saturating_sub(1)).collect(); s.push('…'); s }
    });
    // Register SDK filters and functions for user views.
    super::view_filters::register_all(&mut local_env);
    // Copy all registered templates (macros, partials).
    for (tpl_name, tpl_src) in TEMPLATES {
        let _ = local_env.add_template(tpl_name, tpl_src);
    }
    // Register sibling templates from the view's directory tree so templates
    // can {% include %} partials. e.g. canvas/index.html.j2 includes
    // canvas/docs/hero-review.html.j2. Scans project .ship/views/ then global
    // ~/.ship/views/ (project wins on path collisions).
    let mut sibling_sources: Vec<(String, String)> = Vec::new();
    if let Some(view_name) = name.split('/').next() {
        let mut registered: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ship_dir in [
            runtime::project::get_project_dir(None).ok(),
            runtime::project::get_global_dir().ok(),
        ]
        .into_iter()
        .flatten()
        {
            let views_root = ship_dir.join("views");
            let view_dir = views_root.join(view_name);
            let mut found: Vec<(String, String)> = Vec::new();
            walk_templates(&view_dir, &views_root, name, &mut found);
            for (n, s) in found {
                if registered.insert(n.clone()) {
                    sibling_sources.push((n, s));
                }
            }
        }
    }
    for (n, s) in &sibling_sources {
        let _ = local_env.add_template(n, s);
    }
    // Add the view source as a named template.
    if let Err(e) = local_env.add_template(name, source) {
        let msg = format!("view '{name}' template parse error: {e}");
        tracing::warn!(view = %name, "{msg}");
        return Ok(super::error_overlay::render_error_overlay(name, source, &msg));
    }

    let tpl = local_env.get_template(name)
        .map_err(|e| format!("view '{name}' not found after add: {e}"))?;
    let value = Value::from_serialize(ctx);
    match tpl.render(value) {
        Ok(html) => Ok(html),
        Err(e) => {
            let msg = format!("view '{name}' render error: {e}");
            tracing::warn!(view = %name, "{msg}");
            Ok(super::error_overlay::render_error_overlay(name, source, &msg))
        }
    }
}

#[cfg(test)]
#[path = "templates_tests.rs"]
mod tests;
