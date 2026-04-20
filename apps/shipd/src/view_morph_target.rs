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
//! `datastar-mode: inner` so Datastar replaces the mount point's
//! children without touching the shell's stable `<main id="ship-view-root"
//! class="ship-content">` element itself (keeping its layout class).
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
            .or_insert(HeaderValue::from_static("inner"));
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
        .or_insert(HeaderValue::from_static("inner"));
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);

    Response::from_parts(parts, Body::from(output))
}

/// Extract the body content of a full HTML document. Returns everything
/// between `<body...>` and `</body>`, prepended by any `<style>` blocks
/// from `<head>` (so per-view CSS isn't lost when the chrome is stripped).
/// Falls back to returning the input unchanged if no `<body>` is found.
fn extract_fragment(html: &str) -> String {
    // Collect <style> blocks from the <head> so per-view CSS survives.
    let mut styles = String::new();
    let head_end = html.find("</head>").unwrap_or(0);
    if head_end > 0 {
        let head_slice = &html[..head_end];
        let mut search_from = 0;
        while let Some(style_start) = head_slice[search_from..].find("<style") {
            let abs_start = search_from + style_start;
            if let Some(style_end) = head_slice[abs_start..].find("</style>") {
                let abs_end = abs_start + style_end + "</style>".len();
                styles.push_str(&head_slice[abs_start..abs_end]);
                styles.push('\n');
                search_from = abs_end;
            } else {
                break;
            }
        }
    }

    // Find <body ...> and </body> and return the content between.
    let Some(body_open_start) = html.find("<body") else {
        return html.to_string();
    };
    let Some(body_open_end_rel) = html[body_open_start..].find('>') else {
        return html.to_string();
    };
    let content_start = body_open_start + body_open_end_rel + 1;
    let Some(body_close_start) = html[content_start..].rfind("</body>") else {
        return html.to_string();
    };
    let content = html[content_start..content_start + body_close_start].trim();
    if styles.is_empty() {
        content.to_string()
    } else {
        format!("{styles}\n{content}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_body_content_from_full_page() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>x</title></head>
<body>
<div class="ship-view">hello <span>world</span></div>
</body>
</html>"#;
        let frag = extract_fragment(html);
        assert!(frag.starts_with(r#"<div class="ship-view""#));
        assert!(frag.ends_with("</div>"));
        assert!(frag.contains("hello"));
        assert!(!frag.contains("DOCTYPE"));
        assert!(!frag.contains("<head>"));
        assert!(!frag.contains("<body>"));
    }

    #[test]
    fn handles_nested_divs() {
        let html = r#"<!DOCTYPE html><html><head></head><body>
<div class="ship-view"><div>inner</div><div>inner 2</div></div>
</body></html>"#;
        let frag = extract_fragment(html);
        assert!(frag.contains("inner 2"));
        assert_eq!(frag.matches("</div>").count(), 3);
    }

    #[test]
    fn works_when_root_is_main_element() {
        let html = r#"<!DOCTYPE html><html><head></head><body><main class="ship-view">content</main></body></html>"#;
        let frag = extract_fragment(html);
        assert_eq!(frag, r#"<main class="ship-view">content</main>"#);
    }

    #[test]
    fn falls_back_when_no_body() {
        let html = "no doctype no body";
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
<div class="ship-view">content</div>
</body>
</html>"#;
        let frag = extract_fragment(html);
        assert!(frag.contains(".foo { color: red; }"), "first style block preserved");
        assert!(frag.contains(".bar { padding: 10px; }"), "second style block preserved");
        assert!(frag.contains("class=\"ship-view\""), "root element present");
        assert!(!frag.contains("DOCTYPE"), "no DOCTYPE");
        assert!(!frag.contains("<head>"), "no head tag");
    }
}
