//! Inject daemon-mode meta tag into served HTML shells.
//!
//! shell's index.html ships a static <meta name="ship-mode"
//! content="development"> tag. When shipd serves the same HTML in
//! production mode it rewrites that tag via `inject_mode_meta` so the
//! shell's status bar can render the DEV badge appropriately.

use crate::DaemonMode;

/// Short string label for the daemon mode. Used as the `content`
/// attribute of `<meta name="ship-mode">`.
pub fn mode_str(mode: DaemonMode) -> &'static str {
    match mode {
        DaemonMode::Development => "development",
        DaemonMode::Production => "production",
    }
}

/// Rewrite (or insert) the `<meta name="ship-mode" content="…">` tag
/// in an HTML string to reflect the daemon's current mode.
///
/// If the tag exists with any `content`, replace it. Otherwise insert
/// a fresh tag right after `<head>` so it appears before scripts.
pub fn inject_mode_meta(html: &str, mode: DaemonMode) -> String {
    let value = mode_str(mode);
    let needle = "<meta name=\"ship-mode\"";
    if let Some(start) = html.find(needle) {
        // Find the closing `>` of this tag.
        if let Some(rel_end) = html[start..].find('>') {
            let end = start + rel_end + 1;
            let replacement =
                format!("<meta name=\"ship-mode\" content=\"{value}\" />");
            let mut out = String::with_capacity(html.len() + replacement.len());
            out.push_str(&html[..start]);
            out.push_str(&replacement);
            out.push_str(&html[end..]);
            return out;
        }
    }
    // Tag missing — insert after <head>.
    let head_open = "<head>";
    if let Some(idx) = html.find(head_open) {
        let insert_at = idx + head_open.len();
        let mut out = String::with_capacity(html.len() + 80);
        out.push_str(&html[..insert_at]);
        out.push_str(&format!(
            "\n  <meta name=\"ship-mode\" content=\"{value}\" />"
        ));
        out.push_str(&html[insert_at..]);
        return out;
    }
    // No <head> — return as-is.
    html.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_existing_tag() {
        let html =
            "<head>\n<meta name=\"ship-mode\" content=\"development\" />\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert!(out.contains("content=\"production\""));
        assert!(!out.contains("content=\"development\""));
    }

    #[test]
    fn inserts_when_missing() {
        let html = "<head>\n<title>X</title>\n</head>";
        let out = inject_mode_meta(html, DaemonMode::Development);
        assert!(out.contains("<meta name=\"ship-mode\" content=\"development\""));
    }

    #[test]
    fn no_head_returns_unchanged() {
        let html = "<html><body>x</body></html>";
        let out = inject_mode_meta(html, DaemonMode::Production);
        assert_eq!(out, html);
    }
}
