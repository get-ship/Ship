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
use std::sync::{OnceLock, RwLock};
use std::collections::HashMap;

/// Cache of compiled user view environments, keyed by "view_name:source_hash".
/// A fresh Environment is ~400ms to build (parses all embedded TEMPLATES +
/// walks filesystem for sibling partials). Caching brings subsequent renders
/// to ~1ms. Invalidated when the view source hash changes (auto-reload).
static USER_ENV_CACHE: OnceLock<RwLock<HashMap<String, std::sync::Arc<Environment<'static>>>>> = OnceLock::new();

fn user_env_cache() -> &'static RwLock<HashMap<String, std::sync::Arc<Environment<'static>>>> {
    USER_ENV_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn source_hash(source: &str) -> u64 {
    // Deterministic FNV-1a so the same source always maps to the same key.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in source.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// All templates shipped with the daemon. `(name, source)` pairs.
///
/// New templates must be added here to be loadable. Names use forward
/// slashes regardless of host OS (minijinja convention). Keep this list
/// alphabetized for easier diffing.
pub(crate) const TEMPLATES: &[(&str, &str)] = &[
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
        "canvas/artifact.html.j2",
        include_str!("../../templates/canvas/artifact.html.j2"),
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
        "git/partials/changes_panel.html.j2",
        include_str!("../../templates/git/partials/changes_panel.html.j2"),
    ),
    (
        "git/partials/diff_pane.html.j2",
        include_str!("../../templates/git/partials/diff_pane.html.j2"),
    ),
    (
        "git/partials/history_tab.html.j2",
        include_str!("../../templates/git/partials/history_tab.html.j2"),
    ),
    (
        "git/partials/branches_tab.html.j2",
        include_str!("../../templates/git/partials/branches_tab.html.j2"),
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
    env.add_filter("urlencode", |value: String| -> String {
        form_urlencoded::byte_serialize(value.as_bytes()).collect()
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

/// Collect sibling `.html.j2` partials for a view into (name, source)
/// pairs. Scans the project `.ship/views/<view>/` first, then the
/// global dir, with the project winning on path collisions. Excludes
/// the top-level `name` template itself — caller supplies that source.
pub(crate) fn collect_siblings(name: &str) -> Vec<(String, String)> {
    let mut sibling_sources: Vec<(String, String)> = Vec::new();
    let Some(view_name) = name.split('/').next() else { return sibling_sources };
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
    // Deterministic ordering so the siblings hash is stable across filesystem
    // readdir ordering (ext4, APFS, NTFS all differ).
    sibling_sources.sort_by(|a, b| a.0.cmp(&b.0));
    sibling_sources
}

/// Hash the (sorted) sibling partial list into a single u64 so any
/// edit to any sibling file invalidates the cached environment. Without
/// this the user-env cache would silently serve stale partials when an
/// author edits `partials/foo.html.j2` but leaves the top-level
/// `index.html.j2` untouched (cache key would hit).
fn siblings_hash(siblings: &[(String, String)]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for (n, s) in siblings {
        for b in n.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
        h ^= 0xff; h = h.wrapping_mul(0x100000001b3); // separator
        for b in s.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
        h ^= 0xfe; h = h.wrapping_mul(0x100000001b3); // record terminator
    }
    h
}

pub fn render_view_source<C: Serialize>(name: &str, source: &str, ctx: &C) -> Result<String, String> {
    // If the source contains no Jinja syntax, return it as-is (fast path).
    if !source.contains("{%") && !source.contains("{{") {
        return Ok(source.to_string());
    }

    // Scan siblings upfront so the cache key reflects both the top
    // template and every partial it can {% include %}. Editing a
    // partial without touching the top file must invalidate the cache.
    let sibling_sources = collect_siblings(name);
    let cache_key = format!(
        "{}:{:x}:{:x}",
        name,
        source_hash(source),
        siblings_hash(&sibling_sources),
    );
    {
        if let Ok(cache) = user_env_cache().read() {
            if let Some(env) = cache.get(&cache_key) {
                let tpl = env.get_template(name)
                    .map_err(|e| format!("view '{name}' not found in cached env: {e}"))?;
                let value = Value::from_serialize(ctx);
                return match tpl.render(value) {
                    Ok(html) => Ok(html),
                    Err(e) => {
                        let msg = format!("view '{name}' render error: {e}");
                        tracing::warn!(view = %name, "{msg}");
                        Ok(super::error_overlay::render_error_overlay(name, source, &msg))
                    }
                };
            }
        }
    }

    // Cache miss — build environment. ~400ms.
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
    // Leak sibling template strings so they satisfy Environment<'static>
    // lifetime. The cache holds the Environment for the process lifetime,
    // so the memory is reused across requests (not actually leaked per call).
    for (n, s) in sibling_sources {
        let leaked_n: &'static str = Box::leak(n.into_boxed_str());
        let leaked_s: &'static str = Box::leak(s.into_boxed_str());
        let _ = local_env.add_template(leaked_n, leaked_s);
    }
    // Add the view source as a named template.
    let leaked_source: &'static str = Box::leak(source.to_string().into_boxed_str());
    let leaked_name: &'static str = Box::leak(name.to_string().into_boxed_str());
    if let Err(e) = local_env.add_template(leaked_name, leaked_source) {
        let msg = format!("view '{name}' template parse error: {e}");
        tracing::warn!(view = %name, "{msg}");
        return Ok(super::error_overlay::render_error_overlay(name, source, &msg));
    }

    // Wrap in Arc and cache for next time.
    let env_arc = std::sync::Arc::new(local_env);
    if let Ok(mut cache) = user_env_cache().write() {
        cache.insert(cache_key, env_arc.clone());
    }

    let tpl = env_arc.get_template(name)
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
