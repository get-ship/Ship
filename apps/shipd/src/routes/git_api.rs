//! GitService — HTTP surface at `/api/v1/git/*`.
//!
//! Write handlers and body types live in `ship_git::rest_handlers`; they take
//! `State<Arc<dyn GitService>>` and carry no shipd types. The router mounts
//! them via a sub-router scoped to `state.git_arc()`.
//!
//! The SSE change-stream handler stays here because it uses shipd's
//! `ViewSse`/`ViewStream` transport types (daemon-edge, not domain logic).

pub use ship_git::rest_handlers::{
    BranchCreateBody, BranchDeleteBody, CheckoutBody, CommitBody, DiffQuery, InitBody, LogQuery,
    PathsBody, PushBody, RemoteBody, Resp, WorkspaceQuery, branch_create_handler,
    branch_delete_handler, checkout_handler, commit_handler, err, fetch_handler, init_handler,
    map_err, pull_handler, push_handler, stage_handler, unstage_handler,
};

use std::time::Duration;

use axum::Extension;
use axum::extract::{Query, State};
use axum::response::Sse;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;
use crate::views::stream::{ViewSse, ViewStream};

/// GET /api/v1/git/sse — polls status at 5s cadence; patches `#git-service-data`
/// with a JSON blob inside a `<script>` tag so any view can subscribe to the
/// server-side truth without re-running its own libgit2 queries.
///
/// Stays in shipd: depends on `ViewSse`/`ViewStream` which are daemon-only
/// transport types.
pub async fn sse_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<WorkspaceQuery>,
) -> Result<ViewSse, Resp> {
    let (vs, sse) = ViewStream::new();
    let ship_dir = state.ship_dir().to_path_buf();
    let workspace_id = q.workspace_id.clone();

    tokio::spawn(async move {
        let _hb = vs.heartbeat_every(Duration::from_secs(15));
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            if vs.is_closed() {
                break;
            }
            let root = match ship_platform::services::git::state::resolve_root(
                &ship_dir,
                workspace_id.as_deref(),
            ) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let buckets = ship_platform::services::git::read::load_status_buckets(&root);
            let (ahead, behind) =
                ship_platform::services::git::read::load_ahead_behind(&root).unwrap_or((0, 0));
            let body = serde_json::json!({
                "branch": ship_platform::services::git::read::load_branch(&root),
                "ahead": ahead,
                "behind": behind,
                "clean": buckets.clean,
                "total": buckets.total,
            });
            let html = format!(
                r#"<script id="git-service-data" type="application/json">{}</script>"#,
                body,
            );
            if vs.patch("#git-service-data", html).await.is_err() {
                break;
            }
        }
    });

    Ok(sse)
}
