//! Agent profile authoring commands — CRUD over the platform `AgentsService`
//! (`ctx.state.agents()`). Note the prefix split: `agents.*` (plural) authors
//! profiles (this module); `agent.*` (singular) is runtime control
//! (dispatch/stop/steer — see command::agent). Authoring a profile defines an
//! agent's capability, so the gate classifies `agents.*` restricted
//! (ship-authoring), the same grant as skills. Reads are agents.list/get.
//!
//! Scope-resolving → `ScopedMutation`. The agent doc / RFC-6902 patch ride as
//! JSON-string args (`doc_json` / `patch_json`).

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

use ship_platform::services::agents::service::{CreateArgs, DuplicateArgs, PutArgs};
use ship_platform::services::agents::state;
use ship_platform::services::files::Scope;
use ship_substrate::ports::{KernelLog, LocalFsBroker};
use ship_substrate::ShipHome;
use ship_types::SpaceId;

use super::{req_str, CommandCtx, CommandError};

fn parse_scope(args: &Value, key: &str) -> Result<Option<Scope>, CommandError> {
    match args.get(key).and_then(|v| v.as_str()) {
        None | Some("") => Ok(None),
        Some("project") => Ok(Some(Scope::Project)),
        Some("user") => Ok(Some(Scope::User)),
        Some(other) => Err(CommandError::bad_args(format!(
            "`{key}` must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn req_scope(args: &Value, key: &str) -> Result<Scope, CommandError> {
    parse_scope(args, key)?
        .ok_or_else(|| CommandError::bad_args(format!("`{key}` is required (project|user)")))
}

fn req_json(args: &Value, key: &str) -> Result<Value, CommandError> {
    let raw = req_str(args, key)?;
    serde_json::from_str(&raw)
        .map_err(|e| CommandError::bad_args(format!("`{key}` is not valid JSON: {e}")))
}

fn map_err(e: state::Error) -> CommandError {
    match e {
        state::Error::NotFound(m) => CommandError::not_found(m),
        state::Error::AlreadyExists(m) => CommandError::bad_args(m),
        state::Error::BadId(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

/// `{ ok, id, scope }` — decodes into the typed `ScopedMutation`.
fn ok(id: &str, scope: Scope) -> Value {
    serde_json::json!({ "ok": true, "id": id, "scope": state::ScopeTag::from(scope) })
}

/// The `(ship_dir, space)` the resolved write scope re-indexes under. Project
/// scope keys by the workspace path-hash identity (the SAME key the config
/// watcher mints); user scope is keyless daemon-internal pending P3 account
/// scoping. `None` when project scope has no attached workspace (nothing to
/// notify — the write could not have targeted a project dir).
fn scope_target(ctx: &CommandCtx<'_>, scope: Scope) -> Option<(PathBuf, Option<SpaceId>)> {
    match scope {
        Scope::Project => {
            let ws = ctx.state.workspace_dir()?;
            let space = SpaceId::new(ship_substrate::home::path_hash(ws));
            Some((ShipHome::project_ship_dir(ws), Some(space)))
        }
        Scope::User => Some((ctx.state.ship_dir().to_path_buf(), None)),
    }
}

/// Post-write NOTIFY (S5): after a successful gated def write, emit
/// `agent.def.updated` for `<id>.jsonc` in `scope` so the read-only index
/// re-reads and stays live — deterministically, not via the inotify watcher.
/// Reuses the one write-door notify (`ship_platform::config::notify_agent_def_saved`);
/// the emit is idempotent at the index, so a watcher double-fire is harmless.
async fn notify_saved(ctx: &CommandCtx<'_>, id: &str, scope: Scope) {
    let Some((ship_dir, space)) = scope_target(ctx, scope) else {
        return;
    };
    // Ride the SHARED `KernelLog` the live index consumer subscribes to (each
    // `KernelLog::new` owns its own broadcast — a fresh one would not reach the
    // consumer). When the index isn't mounted (space-less boot), fall back to a
    // fresh log over the shared router so the event is still durably persisted;
    // the next boot rebuild reconciles the index.
    let kernel = match ctx.state.agent_def_kernel() {
        Some(shared) => shared.clone(),
        None => Arc::new(KernelLog::new(ctx.state.kernel().clone())),
    };
    let fs = LocalFsBroker::new();
    ship_platform::config::notify_agent_def_saved(&kernel, &fs, &ship_dir, id, space).await;
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let agents = ctx.state.agents();
    match name {
        "agents.create" => {
            let id = req_str(args, "id")?;
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .create(CreateArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            notify_saved(ctx, &id, resolved).await;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.put" => {
            let id = req_str(args, "id")?;
            let doc = req_json(args, "doc_json")?;
            let resolved = agents
                .put(PutArgs { id: id.clone(), doc, scope: parse_scope(args, "scope")? })
                .map_err(map_err)?;
            notify_saved(ctx, &id, resolved).await;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.patch" => {
            let id = req_str(args, "id")?;
            let patch: json_patch::Patch = serde_json::from_value(req_json(args, "patch_json")?)
                .map_err(|e| CommandError::bad_args(format!("bad patch shape: {e}")))?;
            let resolved =
                agents.patch(&id, parse_scope(args, "scope")?, &patch).map_err(map_err)?;
            notify_saved(ctx, &id, resolved).await;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.delete" => {
            let id = req_str(args, "id")?;
            let resolved = agents.delete(&id, parse_scope(args, "scope")?).map_err(map_err)?;
            // The file is gone → the notify carries the empty-hash deleted marker
            // → the index drops the row.
            notify_saved(ctx, &id, resolved).await;
            Ok(Some(ok(&id, resolved)))
        }
        "agents.rename" => {
            let id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved =
                agents.rename(&id, &new_id, parse_scope(args, "scope")?).map_err(map_err)?;
            // Rename touches BOTH files in the resolved scope: the old id is now
            // absent (drop) and the new id is present (upsert).
            notify_saved(ctx, &id, resolved).await;
            notify_saved(ctx, &new_id, resolved).await;
            Ok(Some(ok(&new_id, resolved)))
        }
        "agents.duplicate" => {
            let source_id = req_str(args, "id")?;
            let new_id = req_str(args, "new_id")?;
            let resolved = agents
                .duplicate(DuplicateArgs {
                    source_id,
                    source_scope: parse_scope(args, "scope")?,
                    new_id: new_id.clone(),
                    to_scope: parse_scope(args, "to")?,
                })
                .map_err(map_err)?;
            notify_saved(ctx, &new_id, resolved).await;
            Ok(Some(ok(&new_id, resolved)))
        }
        "agents.move_scope" => {
            let id = req_str(args, "id")?;
            let from = req_scope(args, "from")?;
            let to = req_scope(args, "to")?;
            agents.move_scope(&id, from, to).map_err(map_err)?;
            // Move touches both scopes: the def left `from` (drop) and arrived in
            // `to` (upsert).
            notify_saved(ctx, &id, from).await;
            notify_saved(ctx, &id, to).await;
            Ok(Some(ok(&id, to)))
        }
        _ => Ok(None),
    }
}
