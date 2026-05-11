use axum::{
    body::Body,
    extract::{Path, Query},
    http::{StatusCode, header},
    response::Response,
};
use sui_compiler::{CompileOpts, compile};
use ship_sui_host::resolver::ResolvedView;

use crate::assets::{resolve_view_js, resolve_view_js_baked, resolve_view_css_baked};
use crate::resolver::resolve;

pub async fn serve_sui_view_template(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match resolve(&name, None) {
        Ok(r) => r.source,
        Err(e) => return not_found(&e.message),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_view_setup(
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    if q.sub.is_none() {
        if let Some(js) = resolve_view_js(&name) {
            return js_response_owned(js);
        }
        if let Some(baked) = resolve_view_js_baked(&name) {
            return js_response(baked);
        }
    }
    match resolve(&name, q.sub.as_deref()) {
        Ok(r) => compile_view_js(&name, &r),
        Err(e) => not_found(&e.message),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ViewSetupQuery {
    pub sub: Option<String>,
}

fn compile_view_js(name: &str, resolved: &ResolvedView) -> Response {
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

    let opts = CompileOpts {
        source_path: resolved.source_path.clone(),
        emit_dts: false,
        setup_url: ts_path.as_ref().map(|_| format!("./{ts_filename}")),
        ..CompileOpts::default()
    };
    let out = match compile(&resolved.source, opts) {
        Ok(o) => o,
        Err(err) => return render_error(err.render(&resolved.source)),
    };

    let final_js = match (ts_path, resolved.source_path.as_deref()) {
        (Some(ts), Some(sui)) => match ship_sui_host::bundle::bundle_view(name, &out.js, sui, &ts) {
            Ok(b) => b,
            Err(e) => return render_error(e),
        },
        _ => out.js,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(
            "X-Sui-View",
            name.chars().filter(|c| c.is_ascii()).collect::<String>(),
        )
        .body(Body::from(final_js))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

pub async fn serve_sui_view_css(
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    if q.sub.is_none() {
        if let Some(css) = resolve_view_css_baked(&name) {
            return css_response(css.to_string());
        }
    }
    let body: String = match resolve(&name, q.sub.as_deref()) {
        Ok(r) => {
            let mut buf = String::new();
            if let Some(parent_dir) = r
                .source_path
                .as_ref()
                .and_then(|p| top_level_view_dir(p, &name))
            {
                if let Ok(s) = std::fs::read_to_string(parent_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("index.css")))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("view.css")))
                {
                    buf.push_str(&s);
                }
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
    css_response(body)
}

fn css_response(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
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

fn js_response(body: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response_owned(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

fn render_error(message: String) -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(message))
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

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_template_served_verbatim() {
        let _lock = test_lock();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
        let resp = serve_sui_view_template(Path("agents".to_string())).await;
        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
