//! Middleware that makes in-shell view navigation clean.
//!
//! When the shell does `@get('/api/runtime/view/skills')` from within an
//! already-mounted view, Datastar morphs the response into the current
//! DOM. Two problems if we don't intervene:
//!
//! 1. Chrome loss — Datastar morphs at `<body>` level. Response body has
//!    one child (the view wrapper), the DOM has three (`<header>`,
//!    `<main>`, `<aside>`), so Datastar removes `<header>`/`<aside>`.
//!
//! 2. `moveBefore` errors — even when we set datastar-selector, idiomorph
//!    chokes on trying to atomically move a `<div id="ship-view-root">`
//!    that's still wrapped in the response's `<html><body>`.
//!
//! Fix: for view routes that return `text/html`, when the request is from
//! Datastar (`Datastar-Request: true` header), strip the outer
//! `<!DOCTYPE html>...<body>` wrapping and return ONLY the inner view
//! content. Also set `datastar-selector: #ship-view-root` and
//! `datastar-mode: outer` so Datastar patches only that element.
//! Non-Datastar callers (plain browser, curl) get the full page unchanged.

use axum::{
    body::{self, Body},
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Wrap responses from view routes for in-shell navigation compatibility.
pub async fn view_morph_target_middleware(req: Request<Body>, next: Next) -> Response {
    let is_view_route = req.uri().path().starts_with("/api/runtime/view/");
    let is_datastar = req
        .headers()
        .get("datastar-request")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let response = next.run(req).await;

    if !is_view_route {
        return response;
    }

    let is_html = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/html"))
        .unwrap_or(false);

    if !is_html {
        return response;
    }

    // For non-Datastar callers, just tag headers. Browsers and curl ignore
    // them; the full page renders as usual.
    if !is_datastar {
        let mut response = response;
        let headers = response.headers_mut();
        headers
            .entry("datastar-selector")
            .or_insert(HeaderValue::from_static("#ship-view-root"));
        headers
            .entry("datastar-mode")
            .or_insert(HeaderValue::from_static("outer"));
        return response;
    }

    // Datastar caller: the view handler already renders a clean fragment
    // (no DOCTYPE/html/head/body) when it sees the Datastar-Request header.
    // We just need to set the selector/mode headers and, as a safety net,
    // fall back to extraction if the response still contains a full page.
    let (mut parts, body) = response.into_parts();
    let bytes = match body::to_bytes(body, 8 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };
    let text = String::from_utf8_lossy(&bytes);

    // If the response is already a clean fragment (no DOCTYPE), pass it
    // through. Otherwise fall back to the old extraction logic for any
    // templates that haven't been updated yet.
    let output = if text.trim_start().starts_with("<!DOCTYPE") || text.trim_start().starts_with("<html") {
        extract_fragment(&text)
    } else {
        text.into_owned()
    };

    parts
        .headers
        .entry("datastar-selector")
        .or_insert(HeaderValue::from_static("#ship-view-root"));
    parts
        .headers
        .entry("datastar-mode")
        .or_insert(HeaderValue::from_static("outer"));
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);

    Response::from_parts(parts, Body::from(output))
}

/// Extract the fragment rooted at `<* id="ship-view-root">` (and its
/// closing tag) from a full HTML document. Also prepends any `<style>`
/// blocks from before the root element so view-specific CSS isn't lost
/// when the `<head>` is stripped.
fn extract_fragment(html: &str) -> String {
    let needle = "id=\"ship-view-root\"";
    let Some(id_pos) = html.find(needle) else {
        return html.to_string();
    };
    // Walk back to the start of the tag that owns this attribute.
    let start = html[..id_pos].rfind('<').unwrap_or(0);

    // Collect all <style>...</style> blocks from before the root element.
    // These live in <head> and would be lost when we strip the page wrapper.
    let mut styles = String::new();
    let before_root = &html[..start];
    let mut search_from = 0;
    while let Some(style_start) = before_root[search_from..].find("<style") {
        let abs_start = search_from + style_start;
        if let Some(style_end) = before_root[abs_start..].find("</style>") {
            let abs_end = abs_start + style_end + "</style>".len();
            styles.push_str(&before_root[abs_start..abs_end]);
            styles.push('\n');
            search_from = abs_end;
        } else {
            break;
        }
    }
    // From the start tag, walk forward balancing open/close tags so nested
    // elements with the same tag name don't prematurely end the fragment.
    // Simpler approach: find the matching close tag by counting opens/closes
    // of the SAME element name (the one that carries the id).
    let tag_name = parse_tag_name(&html[start..]);
    let Some(tag_name) = tag_name else {
        return html[start..].to_string();
    };
    let open_pat = format!("<{tag_name}");
    let close_pat = format!("</{tag_name}>");
    let mut depth = 0usize;
    let mut cursor = start;
    let bytes = html.as_bytes();
    while cursor < bytes.len() {
        let rest = &html[cursor..];
        let next_open = rest.find(&open_pat).map(|i| i + cursor);
        let next_close = rest.find(&close_pat).map(|i| i + cursor);
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                cursor = o + open_pat.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    let end = c + close_pat.len();
                    let root = &html[start..end];
                    return if styles.is_empty() {
                        root.to_string()
                    } else {
                        format!("{styles}{root}")
                    };
                }
                cursor = c + close_pat.len();
            }
            _ => break,
        }
    }
    let rest = &html[start..];
    if styles.is_empty() {
        rest.to_string()
    } else {
        format!("{styles}{rest}")
    }
}

fn parse_tag_name(s: &str) -> Option<String> {
    let s = s.strip_prefix('<')?;
    let end = s.find(|c: char| c.is_whitespace() || c == '>')?;
    Some(s[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_from_full_page() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>x</title></head>
<body>
<div id="ship-view-root" class="ship-view">hello <span>world</span></div>
</body>
</html>"#;
        let frag = extract_fragment(html);
        assert!(frag.starts_with(r#"<div id="ship-view-root""#));
        assert!(frag.ends_with("</div>"));
        assert!(frag.contains("hello"));
        assert!(!frag.contains("DOCTYPE"));
        assert!(!frag.contains("<body>"));
    }

    #[test]
    fn handles_nested_divs() {
        let html = r#"<body>
<div id="ship-view-root"><div>inner</div><div>inner 2</div></div>
</body>"#;
        let frag = extract_fragment(html);
        assert!(frag.contains("inner 2"));
        assert!(frag.ends_with("</div>"));
        assert_eq!(frag.matches("</div>").count(), 3);
    }

    #[test]
    fn works_on_main_element() {
        let html = r#"<body><main id="ship-view-root">content</main></body>"#;
        let frag = extract_fragment(html);
        assert_eq!(frag, r#"<main id="ship-view-root">content</main>"#);
    }

    #[test]
    fn falls_back_when_marker_missing() {
        let html = "<body>no wrapper here</body>";
        assert_eq!(extract_fragment(html), html);
    }

    #[test]
    fn preserves_style_blocks_from_head() {
        let html = r#"<!DOCTYPE html>
<html>
<head>
<style>.foo { color: red; }</style>
<style>.bar { padding: 10px; }</style>
</head>
<body>
<div id="ship-view-root">content</div>
</body>
</html>"#;
        let frag = extract_fragment(html);
        assert!(frag.contains(".foo { color: red; }"), "first style block preserved");
        assert!(frag.contains(".bar { padding: 10px; }"), "second style block preserved");
        assert!(frag.contains("id=\"ship-view-root\""), "root element present");
        assert!(!frag.contains("DOCTYPE"), "no DOCTYPE");
    }
}
