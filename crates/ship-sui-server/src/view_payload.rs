use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use serde::Serialize;
use sui_compiler::{bindings, islands, parse, render};
use ship_sui_host::view_payload::ViewPayload;

use crate::resolver::resolve;

pub async fn serve_view_payload(Path(name): Path<String>) -> Response {
    serve(&name, None)
}

pub async fn serve_view_payload_subpath(
    Path((name, sub_path)): Path<(String, String)>,
) -> Response {
    serve(&name, Some(sub_path.as_str()))
}

fn serve(name: &str, sub_path: Option<&str>) -> Response {
    if !is_valid_view_name(name) {
        return json_error(
            StatusCode::NOT_FOUND,
            PayloadErrorBody::InvalidName { name: name.to_string() },
        );
    }
    let resolved = match resolve(name, sub_path) {
        Ok(r) => r,
        Err(_) => {
            return json_error(
                StatusCode::NOT_FOUND,
                PayloadErrorBody::ViewNotFound { name: name.to_string() },
            );
        }
    };
    let program = match parse::parse(&resolved.source) {
        Ok(p) => p,
        Err(err) => return compile_error_response(name, &resolved.source, err),
    };
    let registry = bindings::build(&program);
    let opts = render::RenderOpts { view_name: name.to_string() };
    let split = match render::render_split(&program, &registry, &opts) {
        Ok(r) => r,
        Err(err) => return compile_error_response(name, &resolved.source, err),
    };
    let css = collect_view_css(name, resolved.source_path.as_deref());
    let module_url = match sub_path {
        Some(s) => format!("/_sui/views/{name}/view.js?sub={s}"),
        None => format!("/_sui/views/{name}/view.js"),
    };
    let island_bundles: Vec<String> = islands::collect_islands(&program.template)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let payload = ViewPayload {
        name: name.to_string(),
        sub: sub_path.map(str::to_string),
        html: split.html,
        css,
        scope: split.init,
        module_url,
        islands: island_bundles,
    };
    json_ok(&payload)
}

fn collect_view_css(name: &str, source_path: Option<&std::path::Path>) -> String {
    if let Some(baked) = baked_css(name) {
        return baked.to_string();
    }
    let Some(p) = source_path else { return String::new() };
    let mut buf = String::new();
    if let Some(top) = top_level_view_dir(p, name) {
        push_css_from(&mut buf, &top);
        if let Some(parent) = p.parent()
            && parent != top
        {
            push_css_from(&mut buf, parent);
        }
    }
    buf
}

const AGENTS_VIEW_CSS: &str = include_str!("../../../views/agents/style.css");
const GIT_VIEW_CSS: &str = include_str!("../../../views/git/style.css");
const COUNTER_VIEW_CSS: &str = include_str!("../../../views/counter/style.css");

fn baked_css(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_CSS),
        "git" => Some(GIT_VIEW_CSS),
        "counter" => Some(COUNTER_VIEW_CSS),
        _ => None,
    }
}

fn push_css_from(buf: &mut String, dir: &std::path::Path) {
    if let Ok(s) = std::fs::read_to_string(dir.join("style.css"))
        .or_else(|_| std::fs::read_to_string(dir.join("index.css")))
        .or_else(|_| std::fs::read_to_string(dir.join("view.css")))
    {
        if !buf.is_empty() && !buf.ends_with('\n') {
            buf.push('\n');
        }
        buf.push_str(&s);
    }
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

fn compile_error_response(
    name: &str,
    source: &str,
    err: sui_compiler::diagnostics::CompileError,
) -> Response {
    let (line, col) = position_from(&err, source);
    let message = err.render(source);
    json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        PayloadErrorBody::CompileError {
            name: name.to_string(),
            line,
            col,
            message,
        },
    )
}

fn position_from(err: &sui_compiler::diagnostics::CompileError, source: &str) -> (usize, usize) {
    let Some(span) = err.span else { return (0, 0) };
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in source.char_indices() {
        if i >= span.start {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn json_ok<T: Serialize>(body: &T) -> Response {
    let body = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

#[derive(Debug, Serialize)]
struct PayloadError {
    error: PayloadErrorBody,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum PayloadErrorBody {
    ViewNotFound { name: String },
    CompileError { name: String, line: usize, col: usize, message: String },
    InvalidName { name: String },
}

fn json_error(status: StatusCode, body: PayloadErrorBody) -> Response {
    let body = serde_json::to_string(&PayloadError { error: body })
        .unwrap_or_else(|_| "{\"error\":{\"code\":\"serialize_failed\"}}".into());
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    const VAR: &str = "SHIP_DEV_PROJECT_DIR";

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        prev: Option<std::ffi::OsString>,
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => unsafe { std::env::set_var(VAR, v) },
                None => unsafe { std::env::remove_var(VAR) },
            }
        }
    }
    fn lock_with_dir(p: &std::path::Path) -> EnvGuard {
        let lock = crate::test_lock::lock();
        let prev = std::env::var_os(VAR);
        unsafe { std::env::set_var(VAR, p); }
        EnvGuard { _lock: lock, prev }
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

    async fn body_str(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn top_level_view_payload_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let view_dir = tmp.path().join(".ship/views/demo");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(view_dir.join("index.sui"),
            r#"<div><p>{count()}</p></div>"#).unwrap();
        fs::write(view_dir.join("index.controller.ts"),
            "import { controller, signal } from '@ship/sui-runtime'\nexport default controller({ scope: () => ({ count: signal(0) }) })\n").unwrap();
        fs::write(view_dir.join("index.css"), "p { color: red; }").unwrap();

        let resp = serve_view_payload(Path("demo".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        assert_eq!(v["name"], "demo");
        assert!(v["sub"].is_null());
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("<p>"), "html should contain rendered template, got {html}");
        assert!(!html.contains("__set_init__"),
            "html must NOT include the script island — scope is a separate field");
        assert_eq!(v["css"], "p { color: red; }");
        assert!(v["scope"].is_object(), "scope should be a JSON object, got {}", v["scope"]);
        assert_eq!(v["moduleUrl"], "/_sui/views/demo/view.js");
        assert!(v["islands"].as_array().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sub_path_payload_concatenates_css() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let view_dir = tmp.path().join(".ship/views/demo");
        let sub_dir = view_dir.join("[name]");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(view_dir.join("index.sui"), r#"<div></div>"#).unwrap();
        fs::write(view_dir.join("index.css"), ".parent {}").unwrap();
        fs::write(sub_dir.join("index.sui"), r#"<span></span>"#).unwrap();
        fs::write(sub_dir.join("index.css"), ".child {}").unwrap();

        let resp = serve_view_payload_subpath(
            Path(("demo".to_string(), "anything".to_string()))
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        assert_eq!(v["sub"], "anything");
        let css = v["css"].as_str().unwrap();
        assert!(css.contains(".parent"), "css missing parent: {css}");
        assert!(css.contains(".child"), "css missing child: {css}");
        assert!(
            css.find(".parent").unwrap() < css.find(".child").unwrap(),
            "parent must precede child for cascade order"
        );
        assert_eq!(v["moduleUrl"], "/_sui/views/demo/view.js?sub=anything");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_returns_404_json() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let resp = serve_view_payload(Path("nope".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        assert_eq!(v["error"]["view_not_found"]["name"], "nope");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_returns_404_json() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let resp = serve_view_payload(Path("BadName".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        assert!(v["error"]["invalid_name"].is_object(), "got: {v}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn compile_error_returns_500_with_position() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let view_dir = tmp.path().join(".ship/views/broken");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(view_dir.join("index.sui"), "<div><span></div>").unwrap();

        let resp = serve_view_payload(Path("broken".to_string())).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        let body = &v["error"]["compile_error"];
        assert_eq!(body["name"], "broken");
        assert!(body["line"].as_u64().unwrap() >= 1);
        assert!(!body["message"].as_str().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_tags_listed() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = lock_with_dir(tmp.path());
        let view_dir = tmp.path().join(".ship/views/term-view");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(view_dir.join("index.sui"),
            r#"<div><sui-terminal rows={24} cols={80}></sui-terminal></div>"#).unwrap();

        let resp = serve_view_payload(Path("term-view".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        let islands = v["islands"].as_array().unwrap();
        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0], "sui-terminal");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn first_party_view_payload_works() {
        let root = repo_root();
        let _g = lock_with_dir(root);
        let resp = serve_view_payload(Path("counter".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
        assert_eq!(v["name"], "counter");
        assert!(v["html"].as_str().unwrap().contains("button") || v["html"].as_str().unwrap().contains("Counter"));
        assert_eq!(v["moduleUrl"], "/_sui/views/counter/view.js");
    }
}
