//! ui.* mount — crate-owned `UiWire` (ship-views).
//!
//! Replaces the previous `view_wire` + `sui` + `view` modules: the unified
//! `UiService` trait (12 RPCs) is implemented by `ship_views::wire::UiWire`,
//! and shipd supplies the per-daemon context (auth, edit-mode grants, tool
//! dispatch, scaffold ship_dir, and the `sources/` tree the bundler emits).
//!
//! Sources root: `<ship_dir>/dist/_deck/sources/`. The deck-bundler emits the
//! `sources/` tree at build time; the runtime reads it. When the path
//! doesn't exist yet (no bundle built) the `Packs`/`Read` RPCs return empty
//! results — the rest of the trait still serves its writes.

use std::sync::Arc;

use connectrpc::ConnectError;

use crate::ship_runtime::ApiState;

pub(super) fn mount(router: connectrpc::Router, state: &ApiState) -> connectrpc::Router {
    // Session writes (Patch/Append/Signal): the agent-canvas ViewSessionRegistry
    // was deleted with the UI purge — no surface can create (`view.open` is gone)
    // or watch (the SSE subscribe route is gone) a session, so the registry was
    // permanently empty and every call already answered "not found". These
    // closures preserve that exact observable behavior; the three ops now live
    // under `ship.ui.write` / C1.
    let view_patch_fn: ship_views::wire::PatchFn =
        Arc::new(move |id, _agent, _selector, _html| {
            Box::pin(async move { Err(no_view_session(&id)) })
        });
    let view_append_fn: ship_views::wire::AppendFn =
        Arc::new(move |id, _agent, _selector, _html| {
            Box::pin(async move { Err(no_view_session(&id)) })
        });
    let view_signal_fn: ship_views::wire::SignalFn =
        Arc::new(move |id, _agent, _name, _value| {
            Box::pin(async move { Err(no_view_session(&id)) })
        });
    // `UiService.Tool` — inlined from the now-cut
    // `routes::view_edit_tool_api::dispatch_tool_core`. The handler body
    // was a 3-line wrapper around
    // `ship_views::view_authoring::tool_dispatch::dispatch(&adapter, …)`;
    // with the REST shell gone, the Connect closure calls `dispatch`
    // directly and maps `DispatchResult` to the Connect surface.
    use ship_views::view_authoring::tool_dispatch::DispatchResult as DR;
    let view_state_for_tool = state.clone();
    let view_tool_fn: ship_views::wire::ToolFn =
        Arc::new(move |view, session_id, op, args| {
            let st = view_state_for_tool.clone();
            Box::pin(async move {
                let adapter = crate::ports::view::ViewToolAdapter::new(st);
                let result =
                    ship_views::view_authoring::tool_dispatch::dispatch(
                        &adapter, view, session_id, op, args,
                    )
                    .await;
                match result {
                    DR::Ok(v) => Ok(v.to_string()),
                    DR::BadRequest { code, message }
                    | DR::NotFound { code, message }
                    | DR::Conflict { code, message } => {
                        Err(ConnectError::invalid_argument(format!(
                            "{code}: {message}"
                        )))
                    }
                    DR::Forbidden { code, message } => {
                        Err(ConnectError::permission_denied(format!(
                            "{code}: {message}"
                        )))
                    }
                    DR::Internal { code, message } => Err(ConnectError::internal(format!(
                        "{code}: {message}"
                    ))),
                }
            })
        });
    let view_agent_fn: ship_views::wire::AgentFn =
        Arc::new(|ctx: &connectrpc::RequestContext| {
            ctx.extensions
                .get::<crate::auth::AuthContext>()
                .and_then(|a| a.agent_id.clone())
                .ok_or_else(|| ConnectError::permission_denied("agent identity required"))
        });
    // scaffold_fn: moves scaffold_view route-core into the UiWire.
    // Runs synchronous fs I/O in spawn_blocking so the future is Send.
    let scaffold_ship_dir = state.ship_dir().to_path_buf();
    let view_scaffold_fn: ship_views::wire::ScaffoldViewFn =
        Arc::new(move |name: String, shape: String, purpose: Option<String>| {
            let ship_dir = scaffold_ship_dir.clone();
            Box::pin(async move {
                tokio::task::spawn_blocking(move || {
                    scaffold_view_sync(&ship_dir, &name, &shape, purpose.as_deref())
                })
                .await
                .map_err(|e| ConnectError::internal(format!("scaffold task: {e}")))?
            })
        });
    // Sources root — the bundle's emitted `sources/` tree. Convention:
    // `<ship_dir>/dist/_deck/sources/`. May not exist yet (no bundle run);
    // the wire handles missing dirs with empty Packs / NotFound Read.
    let sources_root = state.ship_dir().join("dist").join("_deck").join("sources");
    let ui_wire = Arc::new(ship_views::wire::UiWire::new(
        sources_root,
        view_agent_fn,
        view_patch_fn,
        view_append_fn,
        view_signal_fn,
        view_tool_fn,
        view_scaffold_fn,
    ));
    ship_ops_rpc::UiServiceExt::register(ui_wire, router)
}

/// The error every session write answers with — identical to what the
/// always-empty registry's `SessionError::NotFound` mapped to before the
/// registry was deleted.
fn no_view_session(id: &str) -> ConnectError {
    ConnectError::internal(format!("view session '{id}' not found"))
}

// ── scaffold_view_sync ────────────────────────────────────────────────────────
// Synchronous scaffold logic for UiService::Scaffold. Moved from
// routes/create_api.rs::scaffold_view; runs in spawn_blocking from the closure.
fn scaffold_view_sync(
    ship_dir: &std::path::Path,
    name: &str,
    shape: &str,
    _purpose: Option<&str>,
) -> Result<(String, String), ConnectError> {
    const KEBAB: &str = "abcdefghijklmnopqrstuvwxyz0123456789-";
    let valid = !name.is_empty()
        && name.len() <= 48
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.chars().all(|c| KEBAB.contains(c));
    if !valid {
        return Err(ConnectError::invalid_argument("name must be kebab-case (a-z0-9-, max 48 chars)"));
    }

    let scaffold_rel = std::path::PathBuf::from("skills")
        .join("ship-views")
        .join("scaffolds")
        .join(shape);
    let mut scaffold_src: Option<std::path::PathBuf> = None;
    if let Ok(d) = ship_substrate::project::get_project_dir(None) {
        let p = d.join(&scaffold_rel);
        if p.is_dir() { scaffold_src = Some(p); }
    }
    if scaffold_src.is_none() {
        let p = ship_dir.join(&scaffold_rel);
        if p.is_dir() { scaffold_src = Some(p); }
    }
    let src = scaffold_src.ok_or_else(|| {
        ConnectError::invalid_argument(format!("unknown shape '{shape}'"))
    })?;

    let views_dir = ship_dir.join("views");
    std::fs::create_dir_all(&views_dir)
        .map_err(|e| ConnectError::internal(format!("create views dir: {e}")))?;
    let target = views_dir.join(name);
    if target.exists() {
        return Err(ConnectError::already_exists(format!("view '{name}' already exists")));
    }

    let title = title_case_view(name);
    copy_scaffold_view(&src, &target, &title, name)
        .inspect_err(|_e| { let _ = std::fs::remove_dir_all(&target); })?;

    let route = format!("{}/view/{name}", deck_core::routes::DECK_PREFIX);
    let path = target.display().to_string();
    Ok((route, path))
}

fn title_case_view(name: &str) -> String {
    name.split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn copy_scaffold_view(
    src: &std::path::Path,
    dst: &std::path::Path,
    title: &str,
    name: &str,
) -> Result<(), ConnectError> {
    std::fs::create_dir_all(dst)
        .map_err(|e| ConnectError::internal(format!("mkdir: {e}")))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| ConnectError::internal(format!("read src dir: {e}")))?
    {
        let entry = entry.map_err(|e| ConnectError::internal(format!("dir entry: {e}")))?;
        let ty = entry.file_type().map_err(|e| ConnectError::internal(e.to_string()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_scaffold_view(&from, &to, title, name)?;
        } else if ty.is_file() {
            let body = std::fs::read_to_string(&from).unwrap_or_default();
            let substituted = body.replace("{{TITLE}}", title).replace("{{NAME}}", name);
            std::fs::write(&to, substituted)
                .map_err(|e| ConnectError::internal(format!("write file: {e}")))?;
        }
    }
    Ok(())
}
