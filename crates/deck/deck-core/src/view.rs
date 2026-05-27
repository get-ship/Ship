use sui_compiler::{CompileOpts, compile};
use deck_host::resolver::ResolvedView;

use crate::{DaemonMode, DeckResponse};
use crate::assets::resolve_view_js;
use crate::resolver::resolve;

#[derive(Debug, serde::Deserialize)]
pub struct ViewSetupQuery {
    pub sub: Option<String>,
}

pub fn serve_sui_view_template(mode: DaemonMode, name: &str) -> DeckResponse {
    if !is_valid_view_name(name) {
        return DeckResponse::not_found("invalid view name");
    }
    let body = match resolve(name, None) {
        Ok(r) => r.source,
        Err(e) => return DeckResponse::not_found(e.message),
    };
    DeckResponse::text_view(body, mode)
}

pub fn serve_sui_view_setup(mode: DaemonMode, name: &str, sub: Option<&str>) -> DeckResponse {
    if !is_valid_view_name(name) {
        return DeckResponse::not_found("invalid view name");
    }
    if sub.is_none() {
        if let Some(js) = resolve_view_js(name) {
            return DeckResponse::js_view(js, mode);
        }
    }
    match resolve(name, sub) {
        Ok(r) => compile_view_js(name, &r, mode),
        Err(e) => DeckResponse::not_found(e.message),
    }
}

fn compile_view_js(name: &str, resolved: &ResolvedView, mode: DaemonMode) -> DeckResponse {
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
        Err(err) => return DeckResponse::render_error(err.render(&resolved.source), mode),
    };

    // A view with a paired controller must be served as a bundle (the compiled
    // module imports `./<controller>`). Deck bundles that in-process via
    // Rolldown to `.deck/views/<name>/view.js` (#167); the deck-axum handler
    // builds it on a cache miss before this path is reached, so reaching here
    // with a controller means the build failed. Surface that — never shell out.
    let final_js = match (&ts_path, resolved.source_path.as_deref()) {
        (Some(_ts), Some(_sui)) => {
            return DeckResponse::render_error(
                format!(
                    "view `{name}` has a paired controller but no bundle in .deck — \
                     the in-process bundler must build views/{name}/view"
                ),
                mode,
            );
        }
        _ => out.js,
    };

    DeckResponse::js_view(final_js, mode).with_header(
        "X-Sui-View",
        name.chars().filter(|c| c.is_ascii()).collect::<String>(),
    )
}

pub fn serve_sui_view_css(mode: DaemonMode, name: &str, sub: Option<&str>) -> DeckResponse {
    if !is_valid_view_name(name) {
        return DeckResponse::not_found("invalid view name");
    }
    let body: String = match resolve(name, sub) {
        Ok(r) => {
            let mut buf = String::new();
            if let Some(parent_dir) = r
                .source_path
                .as_ref()
                .and_then(|p| top_level_view_dir(p, name))
                && let Ok(s) = std::fs::read_to_string(parent_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("index.css")))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("view.css")))
            {
                buf.push_str(&s);
            }
            if sub.is_some()
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
        Err(e) => return DeckResponse::not_found(e.message),
    };
    DeckResponse::css_view(body, mode)
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

    #[test]
    fn view_template_served_verbatim() {
        let _lock = test_lock();
        let prev = std::env::var_os("SHIP_DEV_PROJECT_DIR");
        unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", repo_root()); }
        let resp = serve_sui_view_template(DaemonMode::Development, "git");
        match prev {
            Some(v) => unsafe { std::env::set_var("SHIP_DEV_PROJECT_DIR", v) },
            None => unsafe { std::env::remove_var("SHIP_DEV_PROJECT_DIR") },
        }
        assert_eq!(resp.status, 200);
    }
}
