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
    Builtin,
    Project,
    Global,
}

struct BuiltinTheme {
    name: &'static str,
    css: &'static str,
}

const BUILTIN_THEMES: &[BuiltinTheme] = &[];

/// Return metadata for all discovered themes across all sources.
pub fn list_themes() -> Vec<ThemeMeta> {
    let mut themes = Vec::new();

    // Built-in themes.
    for bt in BUILTIN_THEMES {
        themes.push(ThemeMeta {
            name: bt.name.to_string(),
            source: ThemeSource::Builtin,
            has_light: css_has_selector(bt.css, ":root"),
            has_dark: css_has_selector(bt.css, ".dark"),
        });
    }

    // Project themes.
    if let Ok(ship_dir) = runtime::project::get_project_dir(None) {
        collect_dir_themes(&ship_dir.join("themes"), ThemeSource::Project, &mut themes);
    }

    // Global themes — user-scope live under library/.
    if let Ok(global_dir) = runtime::project::get_global_dir() {
        let dir = runtime::ShipHome::at(global_dir)
            .vector_dir(runtime::Scope::User, runtime::Vector::Themes, None);
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
    runtime::get_setting("theme.name")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "ship".to_string())
}

fn resolve_css_by_name(name: &str) -> String {
    // Global > project > builtin (last-write-wins for overrides).
    if let Ok(global_dir) = runtime::project::get_global_dir() {
        let dir = runtime::ShipHome::at(global_dir)
            .vector_dir(runtime::Scope::User, runtime::Vector::Themes, None);
        let path = dir.join(format!("{name}.css"));
        if let Ok(css) = std::fs::read_to_string(&path) {
            return css;
        }
    }
    if let Ok(ship_dir) = runtime::project::get_project_dir(None) {
        let path = ship_dir.join("themes").join(format!("{name}.css"));
        if let Ok(css) = std::fs::read_to_string(&path) {
            return css;
        }
    }
    for bt in BUILTIN_THEMES {
        if bt.name == name {
            return bt.css.to_string();
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
    fn builtin_themes_are_empty_post_ship_set_migration() {
        // The canonical theme catalog moved to packages/ship-set/src/themes.css
        // (ship-light/-dark, paper, halo, console, contrast). The daemon-side
        // registry is now an override hook for project + global CSS files only;
        // no compile-time built-ins overlay the shell catalog.
        assert!(BUILTIN_THEMES.is_empty());
    }

    #[test]
    fn collect_dir_themes_picks_up_css_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("alpha.css"), ":root {}").unwrap();
        std::fs::write(tmp.path().join("beta.css"), ":root {}").unwrap();
        std::fs::write(tmp.path().join("notes.txt"), "ignored").unwrap();
        let mut out = Vec::new();
        collect_dir_themes(tmp.path(), ThemeSource::Project, &mut out);
        let mut names: Vec<&str> = out.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);
    }

    #[test]
    fn collect_dir_themes_detects_both_light_and_dark() {
        let tmp = tempfile::tempdir().unwrap();
        let css = ":root { --background: white; }\n.dark { --background: black; }";
        std::fs::write(tmp.path().join("dual.css"), css).unwrap();
        let mut out = Vec::new();
        collect_dir_themes(tmp.path(), ThemeSource::Project, &mut out);
        let dual = out.iter().find(|t| t.name == "dual").unwrap();
        assert!(dual.has_light);
        assert!(dual.has_dark);
    }

    #[test]
    fn collect_dir_themes_detects_dark_only() {
        let tmp = tempfile::tempdir().unwrap();
        let css = ".dark { --background: black; }";
        std::fs::write(tmp.path().join("midnight.css"), css).unwrap();
        let mut out = Vec::new();
        collect_dir_themes(tmp.path(), ThemeSource::Project, &mut out);
        let midnight = out.iter().find(|t| t.name == "midnight").unwrap();
        assert!(!midnight.has_light);
        assert!(midnight.has_dark);
    }

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
