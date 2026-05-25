//! `GET /_sui/source/{tag}` — render a simple HTML page showing the
//! `.sui` template and `.controller.ts` for a SUI component.
//!
//! Powers the "View source" deep-link in ship-ui-studio. The whole
//! `packages/components/ui-components/src/` tree is baked into the daemon binary
//! at compile time via `include_dir!`, so the studio works even when
//! the daemon ships standalone (no workspace on disk). Look-ups go
//! through `is_valid_tag()` — only `sui-[a-z0-9-]+` matches; anything
//! else (path traversal, uppercase, foreign prefixes) returns 400.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use include_dir::{Dir, include_dir};

/// Embedded source tree for SUI components. The studio reads from this
/// directly rather than touching the filesystem, so shipped daemons
/// expose component source even when the monorepo isn't present.
static UI_COMPONENTS_SRC: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../packages/components/ui-components/src");

/// Tag whitelist — lowercase ASCII, digits, and hyphens after `sui-`.
/// Rejects path-traversal (`..`, `/`, `\\`) and any non-component name.
pub(crate) fn is_valid_tag(tag: &str) -> bool {
    if !tag.starts_with("sui-") {
        return false;
    }
    if tag.len() < 5 || tag.len() > 64 {
        return false;
    }
    tag.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// GET /_sui/source/{tag}
pub async fn serve_sui_source(Path(tag): Path<String>) -> Response {
    if !is_valid_tag(&tag) {
        return (StatusCode::BAD_REQUEST, "invalid tag").into_response();
    }

    let sui_rel = format!("{tag}/{tag}.sui");
    let ctrl_rel = format!("{tag}/{tag}.controller.ts");

    let Some(sui_src) = UI_COMPONENTS_SRC
        .get_file(sui_rel.as_str())
        .and_then(|f| f.contents_utf8())
    else {
        return (
            StatusCode::NOT_FOUND,
            format!("no source for <{tag}>"),
        )
            .into_response();
    };

    let ctrl_src = UI_COMPONENTS_SRC
        .get_file(ctrl_rel.as_str())
        .and_then(|f| f.contents_utf8());

    let body = render_html(&tag, sui_src, ctrl_src);
    let mut resp = Html(body).into_response();
    resp.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-cache"),
    );
    resp
}

fn render_html(tag: &str, sui_src: &str, ctrl_src: Option<&str>) -> String {
    let sui_html = html_escape(sui_src);
    let title = format!("Source · {tag}");
    let header = format!("&lt;{tag}&gt;");
    let ctrl_block = match ctrl_src {
        Some(c) => format!(
            "<h2>Controller (.controller.ts)</h2><pre>{}</pre>",
            html_escape(c),
        ),
        None => String::from(
            "<h2>Controller (.controller.ts)</h2>\
             <p class=\"meta\">No controller — declaration-only component.</p>",
        ),
    };
    format!(
        "<!doctype html>\n\
<html lang=\"en\"><head><meta charset=\"utf-8\">\n\
<title>{title}</title>\n\
<link rel=\"stylesheet\" href=\"/api/assets/view-tokens.css\">\n\
<style>\n\
  body {{ margin: 0; padding: 24px 32px; font-family: var(--font-body, system-ui, sans-serif); \
          color: var(--foreground, #111); background: var(--background, #fff); }}\n\
  h1 {{ font-family: var(--font-mono, ui-monospace, monospace); font-size: 1.25rem; margin: 0 0 4px; }}\n\
  h2 {{ font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.06em; \
        color: var(--muted-foreground, #555); margin: 24px 0 8px; font-weight: 600; }}\n\
  .meta {{ font-size: 0.85rem; color: var(--muted-foreground, #666); margin: 0 0 16px; \
           font-family: var(--font-mono, ui-monospace, monospace); }}\n\
  pre {{ background: #1e1e1e; color: #d4d4d4; padding: 16px; border-radius: 8px; \
         overflow: auto; font-family: var(--font-mono, ui-monospace, monospace); \
         font-size: 13px; line-height: 1.55; white-space: pre; tab-size: 2; }}\n\
</style>\n\
</head><body>\n\
<h1>{header}</h1>\n\
<p class=\"meta\">packages/components/ui-components/src/{tag}/</p>\n\
<h2>Template (.sui)</h2><pre>{sui_html}</pre>\n\
{ctrl_block}\n\
</body></html>"
    )
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        assert!(!is_valid_tag(".."));
        assert!(!is_valid_tag("../etc/passwd"));
        assert!(!is_valid_tag("sui-x/../y"));
        assert!(!is_valid_tag("sui-button/../"));
    }

    #[test]
    fn rejects_non_sui_tags() {
        assert!(!is_valid_tag(""));
        assert!(!is_valid_tag("button"));
        assert!(!is_valid_tag("SUI-button"));
        assert!(!is_valid_tag("ship-button"));
        assert!(!is_valid_tag("sui-"));
    }

    #[test]
    fn accepts_valid_tags() {
        assert!(is_valid_tag("sui-button"));
        assert!(is_valid_tag("sui-rail-item"));
        assert!(is_valid_tag("sui-area-chart"));
    }

    #[test]
    fn known_component_has_source() {
        // Smoke test that the embedded tree actually contains the bake-time
        // canary component. If this fails, `include_dir!` is mis-pathed.
        let f = UI_COMPONENTS_SRC.get_file("sui-pipeline-step/sui-pipeline-step.sui");
        assert!(
            f.is_some(),
            "sui-pipeline-step.sui missing from embedded tree",
        );
    }

    #[test]
    fn html_escapes_angle_brackets() {
        let out = html_escape("<a href=\"x\">&amp;</a>");
        assert!(out.contains("&lt;a"));
        assert!(out.contains("&quot;x&quot;"));
        assert!(out.contains("&amp;amp;"));
    }
}
