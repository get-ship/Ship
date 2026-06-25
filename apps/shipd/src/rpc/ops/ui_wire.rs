//! ui.* mount — transport glue for the unified `UiService` (ship-ui).
//!
//! The actual trait impl lives in `ship_ui::wire::UiWire`. shipd
//! only constructs the wire and registers it via
//! `UiServiceExt::register`; the platform owns the protocol. The
//! chain resolver (`ship_ui::registry::ChainResolver`) is wired with
//! the host's configured roots: workspace components (default =
//! `ship_dir`) + the current space's `.ship/views/`. Future roots
//! (shop registry, CDN) drop in without wire changes.

use std::sync::Arc;

use connectrpc::ConnectError;

use crate::ship_runtime::ApiState;

pub(super) fn mount(router: connectrpc::Router, state: &ApiState) -> connectrpc::Router {
    // B12: the 3 session-write RPCs (Patch/Append/Signal) were removed — the
    // ViewSessionRegistry was deleted with the UI purge and these closures
    // were always answering "not found". `UiWire` no longer takes
    // patch/append/signal closures; only the agent/tool/scaffold closures
    // remain.
    // `UiService.Tool` — inlined from the now-cut
    // `routes::view_edit_tool_api::dispatch_tool_core`. The handler body
    // was a 3-line wrapper around
    // `ship_ui::view_authoring::tool_dispatch::dispatch(&adapter, …)`;
    // with the REST shell gone, the Connect closure calls `dispatch`
    // directly and maps `DispatchResult` to the Connect surface.
    use ship_ui::view_authoring::tool_dispatch::DispatchResult as DR;
    let view_state_for_tool = state.clone();
    let view_tool_fn: ship_ui::wire::ToolFn =
        Arc::new(move |view, session_id, op, args| {
            let st = view_state_for_tool.clone();
            Box::pin(async move {
                let adapter = crate::ports::view::ViewToolAdapter::new(st);
                let result =
                    ship_ui::view_authoring::tool_dispatch::dispatch(
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
    // scaffold_fn: moves scaffold_view route-core into the UiWire.
    // Runs synchronous fs I/O in spawn_blocking so the future is Send.
    let scaffold_ship_dir = state.ship_dir().to_path_buf();
    let view_scaffold_fn: ship_ui::wire::ScaffoldViewFn =
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
    // Resolver chain — the wire is generic; shipd configures the
    // roots. Default to: workspace components (ship_dir) + the
    // current space's .ship/views/ dir. The host (ship.jsonc or
    // `ship ui set-roots`) can override. Multiple roots compose;
    // first match wins.
    let ship_dir = state.ship_dir().to_path_buf();
    let views_root = ship_dir.join("views");
    let resolver: Arc<dyn ship_ui::registry::ComponentResolver> =
        Arc::new(ship_ui::registry::ChainResolver::new(vec![
            ship_ui::registry::Root::Workspace { dir: ship_dir },
            ship_ui::registry::Root::ViewsDir { dir: views_root },
        ]));
    let ui_wire = Arc::new(ship_ui::wire::UiWire::new(
        resolver,

        view_tool_fn,
        view_scaffold_fn,
    ));
    ship_ops_rpc::UiServiceExt::register(ui_wire, router)
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
        .join("ship-ui")
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
