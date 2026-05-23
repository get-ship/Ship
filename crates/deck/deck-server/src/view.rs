use axum::{
    Extension,
    body::Body,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::Response,
};
use sui_compiler::{CompileOpts, compile};
use deck_host::resolver::ResolvedView;

use crate::{DaemonMode, compute_etag, mode_from_ext, view_cache};
use crate::assets::resolve_view_js;
use crate::resolver::resolve;

pub async fn serve_sui_view_template(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
) -> Response {
    let mode = mode_from_ext(mode);
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match resolve(&name, None) {
        Ok(r) => r.source,
        Err(e) => return not_found(&e.message),
    };
    let cache = view_cache(mode);
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, cache);
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_view_setup(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    let mode = mode_from_ext(mode);
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    if q.sub.is_none() {
        if let Some(js) = resolve_view_js(&name) {
            return js_response_mode(js, mode);
        }
    }
    match resolve(&name, q.sub.as_deref()) {
        Ok(r) => compile_view_js(&name, &r, mode),
        Err(e) => not_found(&e.message),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ViewSetupQuery {
    pub sub: Option<String>,
}

fn compile_view_js(name: &str, resolved: &ResolvedView, mode: DaemonMode) -> Response {
    let ts_filename = match resolved
        .source_path
        .as_ref()
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
    {
        Some("page") => "controller.ts",
        _ => "index.controller.ts",
    };
    let ts_path = resolved
        .source_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|d| d.join(ts_filename))
        .filter(|p| p.is_file());

    // emit_dts is held off here on purpose: this path serves JS per
    // HTTP request and never persists outputs. dts lives next to the
    // source and is the build-step's job (sui-build runner + FFI).
    let opts = CompileOpts {
        source_path: resolved.source_path.clone(),
        emit_dts: false,
        setup_url: ts_path.as_ref().map(|_| format!("./{ts_filename}")),
        ..CompileOpts::default()
    };
    let out = match compile(&resolved.source, opts) {
        Ok(o) => o,
        Err(err) => return render_error(err.render(&resolved.source), mode),
    };

    let final_js = match (ts_path, resolved.source_path.as_deref()) {
        (Some(ts), Some(sui)) => match deck_host::bundle::bundle_view(name, &out.js, sui, &ts) {
            Ok(b) => b,
            Err(e) => return render_error(e, mode),
        },
        _ => out.js,
    };

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, view_cache(mode))
        .header(
            "X-Sui-View",
            name.chars().filter(|c| c.is_ascii()).collect::<String>(),
        );
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(final_js.as_bytes()));
    }
    builder.body(Body::from(final_js)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_view_css(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    let mode = mode_from_ext(mode);
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body: String = match resolve(&name, q.sub.as_deref()) {
        Ok(r) => {
            let mut buf = String::new();
            if let Some(parent_dir) = r
                .source_path
                .as_ref()
                .and_then(|p| top_level_view_dir(p, &name))
                && let Ok(s) = std::fs::read_to_string(parent_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("index.css")))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("view.css")))
            {
                buf.push_str(&s);
            }
            if q.sub.is_some()
                && let Some(sub_dir) = r.source_path.as_ref().and_then(|p| p.parent())
                && let Ok(s) = std::fs::read_to_string(sub_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(sub_dir.join("index.css")))
            {
                if !buf.is_empty() && !buf.ends_with('\n') {
                    buf.push('\n');
                }
                buf.push_str(&s);
            }
            buf
        }
        Err(e) => return not_found(&e.message),
    };
    css_response_mode(body, mode)
}

fn css_response_mode(body: String, mode: DaemonMode) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, view_cache(mode));
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response_mode(body: String, mode: DaemonMode) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, view_cache(mode));
    if mode == DaemonMode::Production {
        builder = builder.header(header::ETAG, compute_etag(body.as_bytes()));
    }
    builder.body(Body::from(body)).unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn top_level_view_dir(p: &std::path::Path, view_name: &str) -> Option<std::path::PathBuf> {
    let mut cur = p.parent()?;
    loop {
        if cur.file_name().and_then(|s| s.to_str()) == Some(view_name) {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

pub(crate) fn is_valid_view_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let segments: Vec<&str> = s.split('/').collect();
    if segments.len() > 8 {
        return false;
    }
    for seg in segments {
        if seg.is_empty() || seg.len() > 64 {
            return false;
        }
        if seg == ".." || seg.starts_with('.') || seg.starts_with('-') {
            return false;
        }
        if !seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return false;
        }
    }
    true
}

fn render_error(message: String, mode: DaemonMode) -> Response {
    let body = if mode == DaemonMode::Production {
        "render error".to_string()
    } else {
        message
    };
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("compile error")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::test_lock::lock()
    }

    fn repo_root() -> &'static std::path::Path {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            let mut cur = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            loop {
                let cargo = cur.join("Cargo.toml");
                if cargo.is_file()
                    && std::fs::read_to_string(&cargo)
                        .map(|s| s.contains("[workspace]"))
                        .unwrap_or(false)
                {
                    return cur;
                }
                if !cur.pop() {
                    panic!("could not find repo root from {}", env!("CARGO_MANIFEST_DIR"));
                }
            }
        });
        ROOT.get().unwrap()
    }

    #[test]
    fn view_name_accepts_digits_upper_under_dash() {
        assert!(is_valid_view_name("v2"));
        assert!(is_valid_view_name("MyView"));
        assert!(is_valid_view_name("my_view"));
        assert!(is_valid_view_name("my-view"));
        assert!(is_valid_view_name("v2-dashboard"));
        assert!(is_valid_view_name("agents"));
    }

    #[test]
    fn view_name_rejects_traversal_and_dotted_prefixes() {
        assert!(!is_valid_view_name("../etc"));
        assert!(!is_valid_view_name(".."));
        assert!(!is_valid_view_name(".hidden"));
        assert!(!is_valid_view_name("-leading-dash"));
    }

    #[test]
    fn view_name_rejects_empty_and_oversize() {
        assert!(!is_valid_view_name(""));
        let too_long: String = "a".repeat(65);
        assert!(!is_valid_view_name(&too_long));
        let max_ok: String = "a".repeat(64);
        assert!(is_valid_view_name(&max_ok));
    }

    #[test]
    fn view_name_rejects_invalid_chars() {
        assert!(!is_valid_view_name("with space"));
        assert!(!is_valid_view_name("with.dot"));
        assert!(!is_valid_view_name("with$dollar"));
    }

    #[test]
    fn view_name_segment_limits() {
        assert!(is_valid_view_name("a/b/c"));
        let too_deep = "a/b/c/d/e/f/g/h/i";
        assert!(!is_valid_view_name(too_deep));
        assert!(!is_valid_view_name("foo//bar"));
        assert!(!is_valid_view_name("foo/../bar"));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(
        clippy::await_holding_lock,
        reason = "test serializes env-var mutation across the await — the whole point of this lock"
    )]
    async fn view_template_served_verbatim() {
        let _lock = test_lock();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
        let resp = serve_sui_view_template(None, Path("git".to_string())).await;
        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
