//! Theme discovery and resolution.
//!
//! Themes override design tokens served by `view-tokens.css`. Three sources
//! are searched (later sources win on name collision):
//!
//! 1. **Built-in** — compiled into the binary from `apps/shipd/themes/`.
//! 2. **Project** — `.ship/themes/*.css` in the current project.
//! 3. **Global** — `~/.ship/themes/*.css` in the user's home directory.
//!
//! The active theme is determined by the `theme.name` setting (default "ship").
//! "ship" means no override — the base tokens are used as-is.

use serde::Serialize;
use std::path::Path;

// ── Built-in themes ──────────────────────────────────────────────────
//
// The canonical theme catalog lives in `packages/ship-set/src/themes.css`
// (`ship-light`, `ship-dark`, `paper`, `halo`, `console`, `contrast`) and
// ships to the browser via `view-tokens.css`. The daemon-side theme.css
// endpoint is now a power-user override hook — project/global CSS files
// merge with the ship-set catalog. No compile-time built-ins overlay the
// shell catalog anymore.

/// Metadata for a discovered theme.
#[derive(Debug, Clone, Serialize)]
pub struct ThemeMeta {
    pub name: String,
    pub source: ThemeSource,
    pub has_light: bool,
    pub has_dark: bool,
}

/// Where a theme was loaded from.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSource {
    Project,
    Global,
}

/// Return metadata for all discovered themes across all sources.
pub fn list_themes() -> Vec<ThemeMeta> {
    let mut themes = Vec::new();

    // Project themes.
    if let Ok(ship_dir) = ship_substrate::project::get_project_dir(None) {
        collect_dir_themes(&ship_dir.join("themes"), ThemeSource::Project, &mut themes);
    }

    // Global themes — user-scope live under library/.
    if let Ok(global_dir) = ship_substrate::project::get_global_dir() {
        let dir = ship_substrate::ShipHome::at(global_dir)
            .vector_dir(ship_types::Scope::User, ship_types::Vector::Themes, None);
        collect_dir_themes(&dir, ThemeSource::Global, &mut themes);
    }

    themes
}

/// Resolve the CSS for the currently active theme.
///
/// Returns an empty string when the active theme is "ship" (the default)
/// or when the requested theme cannot be found.
pub fn resolve_theme_css() -> String {
    let name = active_theme_name();
    if name == "ship" {
        return String::new();
    }
    resolve_css_by_name(&name)
}

// ── Internals ────────────────────────────────────────────────────────────────

fn active_theme_name() -> String {
    ship_platform::config::get_setting("theme.name")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "ship".to_string())
}

fn resolve_css_by_name(name: &str) -> String {
    // Global > project (last-write-wins for overrides).
    if let Ok(global_dir) = ship_substrate::project::get_global_dir() {
        let dir = ship_substrate::ShipHome::at(global_dir)
            .vector_dir(ship_types::Scope::User, ship_types::Vector::Themes, None);
        let path = dir.join(format!("{name}.css"));
        if let Ok(css) = std::fs::read_to_string(&path) {
            return css;
        }
    }
    if let Ok(ship_dir) = ship_substrate::project::get_project_dir(None) {
        let path = ship_dir.join("themes").join(format!("{name}.css"));
        if let Ok(css) = std::fs::read_to_string(&path) {
            return css;
        }
    }
    String::new()
}

fn collect_dir_themes(dir: &Path, source: ThemeSource, out: &mut Vec<ThemeMeta>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("css") {
            continue;
        }
        let name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let css = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        out.push(ThemeMeta {
            name,
            source: source.clone(),
            has_light: css_has_selector(&css, ":root"),
            has_dark: css_has_selector(&css, ".dark"),
        });
    }
}

/// Quick heuristic: does the CSS contain a given selector block?
fn css_has_selector(css: &str, selector: &str) -> bool {
    css.contains(selector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_returns_empty_css() {
        // Default is "ship" which returns empty.
        let css = resolve_theme_css();
        assert!(css.is_empty(), "default theme should return empty CSS");
    }

    #[test]
    fn resolve_unknown_returns_empty() {
        let css = resolve_css_by_name("nonexistent-theme");
        assert!(css.is_empty());
    }

    #[test]
    fn css_has_selector_works() {
        assert!(css_has_selector(":root { color: red; }", ":root"));
        assert!(!css_has_selector(":root { color: red; }", ".dark"));
    }
}
