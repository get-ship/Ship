//! `apps.jsonc` reader — the Ship monorepo's app list. Translated into
//! `Vec<deck::AppServeConfig>` for shipd's router.
//!
//! `apps.jsonc` is the host (shipd) config; deck (L1) knows nothing
//! about the file or its shape. The reader is shipd-only.
//!
//! ## Schema
//!
//! ```jsonc
//! {
//!   "apps": [
//!     {
//!       "name": "hub",                          // required, the URL namespace
//!       "root": "apps/hub",                     // required, repo-relative
//!       "source": "apps/hub/.deck",             // required, the bundler output
//!       "route": "/",                           // required, the URL prefix
//!       "injectGlobals": { "KEY": "value" }     // optional, default {}
//!     }
//!   ]
//! }
//! ```
//!
//! `name` is the app's URL namespace; the framework serves the app's
//! assets under `/<name>/<file>`. `root` is the dev's authored source
//! tree (the bundler reads it). `source` is the bundler's output
//! (`<root>/.deck/`). The two paths are the same as `apps/hub` and
//! `apps/hub/.deck` for the per-app bundler model; we keep both
//! fields explicit so an external deck user (a marketing site, docs,
//! etc.) can declare apps whose source and output differ.
use std::path::Path;

use anyhow::{Context, Result};
use deck::AppServeConfig;
use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppsConfigFile {
    #[serde(default)]
    apps: Vec<AppEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppEntry {
    name: String,
    /// The dev's authored source tree. Read by the Justfile / build
    /// orchestrator (shipd doesn't read this directly — the bundler
    /// does, when the host invokes `deck build --project <root>`).
    /// The bundler's output. Read by shipd when serving the app.
    /// Repo-relative; resolved under the project root at serve time.
    source: String,
    /// URL route prefix for the app. e.g. `/` (hub) or `/space/{id}`
    /// (spaces — the `{id}` is a path capture, accessible via
    /// `inject_globals` substitution).
    route: String,
    #[serde(default)]
    inject_globals: std::collections::BTreeMap<String, String>,
}

/// Load `<path>` and translate it into deck's per-app config shape.
///
/// Missing file: returns an empty list (the daemon serves no app — a
/// clean 404 with a clear message, NOT a panic). A present-but-
/// malformed file: returns an error so the dev sees the parse failure
/// at startup, not at first request.
pub fn load_apps_config(path: &Path) -> Result<Vec<AppServeConfig>> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "read {}: {e}",
                path.display()
            ));
        }
    };
    // Strip JSONC (the same algorithm deck-config uses — kept in lockstep
    // because the per-app file is JSONC and hosts shouldn't have to
    // depend on deck's internals to read it).
    let stripped = strip_jsonc_comments(&raw);
    let parsed: AppsConfigFile = serde_json::from_str(&stripped)
        .with_context(|| format!("parse {}", path.display()))?;
    let apps = parsed
        .apps
        .into_iter()
        .map(|e| AppServeConfig {
            name: e.name,
            route: e.route,
            source: e.source,
            inject_globals: e.inject_globals,
        })
        .collect();
    Ok(apps)
}

/// Strip JSONC comments from a string. Mirrors the algorithm in
/// `deck-config` — kept in lockstep (both files are JSONC; the
/// alternative is a `deck-config` dependency, but shipd is L2 and
/// shouldn't reach into L1's crate graph for a 50-line utility).
///
/// Respects string literals so URLs like `"http://"` are not mangled,
/// and removes `//` line comments + `/* */` block comments + trailing
/// commas before `]` or `}`.
fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                // Line comment — consume to EOL.
                chars.next();
                while let Some(&nc) = chars.peek() {
                    if nc == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                // Block comment — consume to `*/`.
                chars.next();
                while let Some(nc) = chars.next() {
                    if nc == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    // Strip trailing commas before `]` or `}`.
    let mut out = out.replace(",\n]", "\n]").replace(",\n}", "\n}");
    // Catch inline trailing commas: `]}` and `},` patterns.
    let mut result = String::with_capacity(out.len());
    let mut chars = out.chars().peekable();
    while let Some(c) = chars.next() {
        result.push(c);
        if c == ',' {
            // Skip whitespace; if next non-ws is `]` or `}`, drop the comma.
            while let Some(&nc) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            if matches!(chars.peek(), Some(']' | '}')) {
                // Drop the comma we just pushed.
                result.pop();
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_returns_empty_apps() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("apps.jsonc");
        std::fs::write(&p, "{}").unwrap();
        let apps = load_apps_config(&p).unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn missing_file_returns_empty_apps() {
        let dir = tempfile::tempdir().unwrap();
        let apps = load_apps_config(&dir.path().join("nope.jsonc")).unwrap();
        assert!(apps.is_empty());
    }

    #[test]
    fn single_app_parses() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("apps.jsonc");
        std::fs::write(
            &p,
            r#"{
                "apps": [
                    { "name": "hub", "root": "apps/hub", "source": "apps/hub/.deck", "route": "/" }
                ]
            }"#,
        )
        .unwrap();
        let apps = load_apps_config(&p).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "hub");
        assert_eq!(apps[0].route, "/");
        assert_eq!(apps[0].source, "apps/hub/.deck");
    }

    #[test]
    fn spaces_app_with_inject_globals() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("apps.jsonc");
        std::fs::write(
            &p,
            r#"{
                "apps": [
                    {
                        "name": "spaces", "root": "apps/spaces",
                        "source": "apps/spaces/.deck", "route": "/space/{id}",
                        "injectGlobals": { "DECK_APP_ID": "$id" }
                    }
                ]
            }"#,
        )
        .unwrap();
        let apps = load_apps_config(&p).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "spaces");
        assert_eq!(apps[0].route, "/space/{id}");
        assert_eq!(
            apps[0].inject_globals.get("DECK_APP_ID").map(String::as_str),
            Some("$id")
        );
    }

    #[test]
    fn jsonc_comments_stripped() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("apps.jsonc");
        std::fs::write(
            &p,
            r#"// header
            /* block */
            {
                "apps": [
                    { "name": "hub", "root": "apps/hub", "source": "apps/hub/.deck", "route": "/", },
                ],
            }
            "#,
        )
        .unwrap();
        let apps = load_apps_config(&p).unwrap();
        assert_eq!(apps.len(), 1);
    }
}
