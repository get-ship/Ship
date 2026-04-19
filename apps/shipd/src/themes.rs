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

// ── Built-in themes (compile-time embedded) ──────────────────────────────────

const GRUVBOX_CSS: &str = include_str!("../themes/gruvbox.css");
const NORD_CSS: &str = include_str!("../themes/nord.css");
const CATPPUCCIN_CSS: &str = include_str!("../themes/catppuccin.css");
const TOKYO_NIGHT_CSS: &str = include_str!("../themes/tokyo-night.css");

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

const BUILTIN_THEMES: &[BuiltinTheme] = &[
    BuiltinTheme { name: "gruvbox", css: GRUVBOX_CSS },
    BuiltinTheme { name: "nord", css: NORD_CSS },
    BuiltinTheme { name: "catppuccin", css: CATPPUCCIN_CSS },
    BuiltinTheme { name: "tokyo-night", css: TOKYO_NIGHT_CSS },
];

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

    // Global themes.
    if let Ok(global_dir) = runtime::project::get_global_dir() {
        collect_dir_themes(&global_dir.join("themes"), ThemeSource::Global, &mut themes);
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
        let path = global_dir.join("themes").join(format!("{name}.css"));
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
    fn builtin_themes_are_listed() {
        let themes = list_themes();
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"gruvbox"), "missing gruvbox");
        assert!(names.contains(&"nord"), "missing nord");
        assert!(names.contains(&"catppuccin"), "missing catppuccin");
        assert!(names.contains(&"tokyo-night"), "missing tokyo-night");
    }

    #[test]
    fn gruvbox_has_both_modes() {
        let themes = list_themes();
        let gruvbox = themes.iter().find(|t| t.name == "gruvbox").unwrap();
        assert!(gruvbox.has_light);
        assert!(gruvbox.has_dark);
    }

    #[test]
    fn tokyo_night_dark_only() {
        let themes = list_themes();
        let tn = themes.iter().find(|t| t.name == "tokyo-night").unwrap();
        // tokyo-night has no :root block, only .dark
        assert!(!tn.has_light);
        assert!(tn.has_dark);
    }

    #[test]
    fn default_theme_returns_empty_css() {
        // Default is "ship" which returns empty.
        let css = resolve_theme_css();
        assert!(css.is_empty(), "default theme should return empty CSS");
    }

    #[test]
    fn resolve_builtin_by_name() {
        let css = resolve_css_by_name("gruvbox");
        assert!(css.contains("--background"));
        assert!(css.contains("--primary"));
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
