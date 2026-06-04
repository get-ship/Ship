use serde::Serialize;
use sui_compiler::{bindings, parse, render};
pub use deck_host::view_payload::ViewPayload;
use deck_host::resolver::read_view_artifact;

use crate::{CachePolicy, DaemonMode, DeckResponse};
use crate::resolver::resolve;

pub fn serve_view_payload(mode: DaemonMode, name: &str) -> DeckResponse {
    serve(name, None, mode)
}

pub fn serve_view_payload_subpath(mode: DaemonMode, name: &str, sub_path: &str) -> DeckResponse {
    serve(name, Some(sub_path), mode)
}

fn serve(name: &str, sub_path: Option<&str>, mode: DaemonMode) -> DeckResponse {
    if !crate::view::is_valid_view_name(name) {
        return json_error(404, PayloadErrorBody::InvalidName { name: name.to_string() });
    }
    let resolved = match resolve(name, sub_path) {
        Ok(r) => r,
        Err(_) => {
            return json_error(404, PayloadErrorBody::ViewNotFound { name: name.to_string() });
        }
    };
    let program = match parse::parse(&resolved.source) {
        Ok(p) => p,
        Err(err) => return compile_error_response(name, &resolved.source, err, mode),
    };
    let registry = bindings::build(&program);
    // For a nested route (e.g. /studio/agents) the live-data SSE channel lives
    // at /_sui/views/<name>/<sub>/sse, not /_sui/views/<name>/sse. Fold the sub
    // into the resolved view_name so render_split emits the right __subscribe__
    // URL (mirrors the module_url ?sub= handling just below).
    let view_name = match sub_path {
        Some(s) => format!("{name}/{s}"),
        None => name.to_string(),
    };
    let opts = render::RenderOpts { view_name };
    let split = match render::render_split(&program, &registry, &opts) {
        Ok(r) => r,
        Err(err) => return compile_error_response(name, &resolved.source, err, mode),
    };
    let css = collect_view_css(name, resolved.source_path.as_deref());
    let module_url = crate::routes::view_module_url(name, sub_path);
    let html = read_precompiled_ssr(resolved.source_path.as_deref())
        .unwrap_or(split.html);
    let payload = ViewPayload {
        name: name.to_string(),
        sub: sub_path.map(str::to_string),
        html,
        css,
        scope: split.init,
        module_url,
        params: resolved.params,
    };
    json_ok(&payload, mode)
}

fn read_precompiled_ssr(source_path: Option<&std::path::Path>) -> Option<String> {
    let p = source_path?;
    let stem = p.file_stem()?.to_str()?;
    let stem_clean = stem.trim_end_matches(".html");
    let mirror = sui_compiler::out_layout::resolve_out_dir(p);
    let ssr_path = mirror.join(format!("{stem_clean}.ssr.html"));
    std::fs::read_to_string(&ssr_path).ok()
}

fn collect_view_css(name: &str, source_path: Option<&std::path::Path>) -> String {
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

fn push_css_from(buf: &mut String, dir: &std::path::Path) {
    if let Some(s) = read_view_artifact(&dir.join("style.css"))
        .or_else(|| read_view_artifact(&dir.join("index.css")))
        .or_else(|| read_view_artifact(&dir.join("view.css")))
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
    mode: DaemonMode,
) -> DeckResponse {
    if mode == DaemonMode::Production {
        return json_error(500, PayloadErrorBody::RedactedError {});
    }
    let (line, col) = position_from(&err, source);
    let message = err.render(source);
    json_error(
        500,
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

fn json_ok<T: Serialize>(body: &T, mode: DaemonMode) -> DeckResponse {
    let body_str = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    DeckResponse::json_view(body_str, mode)
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
    RedactedError {},
}

fn json_error(status: u16, body: PayloadErrorBody) -> DeckResponse {
    let body = serde_json::to_string(&PayloadError { error: body })
        .unwrap_or_else(|_| "{\"error\":{\"code\":\"serialize_failed\"}}".into());
    // Errors are never cached and carry no ETag — CachePolicy::None handles
    // both regardless of mode.
    DeckResponse::new(status, "application/json; charset=utf-8", CachePolicy::None, DaemonMode::Development, body.into_bytes())
}
