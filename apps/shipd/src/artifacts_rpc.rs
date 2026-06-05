//! `ArtifactsService` handler — proto-as-SSOT migration of `artifacts.*` (#89),
//! the typed-return variant of the recipe: writes return `ArtifactRecord` /
//! `Deletion`, not `ScopedMutation`. Mounted additively beside the generated
//! ReadOps/CommandOps until the GUI flip (#72).
//!
//! The C0 read calls the platform `ArtifactsService` directly — no more
//! `services::registry::call` string dispatch. The C1 writes gate inline via
//! `OpsImpl::gate` (proto policy + `gate::decide`) and call the platform
//! service directly — no more `services::command::call`. The per-artifact
//! scope ACL (`can_write_artifacts` / `can_delete_artifacts`) still runs
//! inside the handler against the resolved record scope; the gate is the
//! one caller×blast check, this is the one caller×artifact-scope check.
//! Byte-moving store/get are NOT here — blobs stay on serving routes.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use std::collections::BTreeMap;

use ship_artifacts::error::ArtifactError;
use ship_artifacts::types::ArtifactScope;
use ship_ops_rpc::{ArtifactRecord, ArtifactsListResponse, ArtifactsService, Deletion, WatchEvent};

use crate::auth::scopes::{can_delete_artifacts, can_write_artifacts};
use crate::auth::AuthContext;
use crate::ops_rpc::OpsImpl;

/// Borrow the request's `AuthContext` for the post-gate per-artifact ACL
/// (`can_write_artifacts` / `can_delete_artifacts`). Distinct from the gate's
/// internal auth read — the ACL checks caller × artifact-scope, not the
/// gate's caller × blast.
fn auth_for(ctx: &RequestContext) -> Result<&AuthContext, ConnectError> {
    ctx.extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))
}

#[allow(refining_impl_trait)]
impl ArtifactsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListArtifactsRequestView,
    ) -> ServiceResult<ArtifactsListResponse> {
        let mut filter: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let scope = if request.scope.is_empty() { "project".to_string() } else { request.scope.to_string() };
        filter.insert("scope".into(), serde_json::Value::String(scope.clone()));
        for key in ["mime", "collection_id", "item_id", "owner_actor", "content_hash"] {
            // Each `request.<key>` is `&str`; empty → filter irrelevant.
            let v = match key {
                "mime" => request.mime,
                "collection_id" => request.collection_id,
                "item_id" => request.item_id,
                "owner_actor" => request.owner_actor,
                "content_hash" => request.content_hash,
                _ => unreachable!(),
            };
            if !v.is_empty() {
                filter.insert(key.into(), serde_json::Value::String(v.to_string()));
            }
        }
        let limit = if request.limit == 0 { 50 } else { request.limit as usize };
        let svc = self
            .state()
            .artifacts()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        let items = svc
            .list(filter, limit as u32)
            .await
            .map_err(artifacts_err)?;
        let count = items.len();
        // Project each Rust `ArtifactRecord` into the proto `ArtifactRecord` (the
        // proto is a strict field-subset of the Rust struct; no `updated_at`).
        let items: Vec<ArtifactRecord> = items.into_iter().map(artifact_record_to_proto).collect();
        Ok(Response::new(ArtifactsListResponse {
            items,
            count: count as u64,
            scope,
            ..Default::default()
        }))
    }

    #[allow(refining_impl_trait)]
    async fn watch(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedArtifactsWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        Err(ConnectError::unimplemented(
            "artifacts Watch: use the ship-artifacts service mount (registered after this one)",
        ))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchArtifactRequestView,
    ) -> ServiceResult<ArtifactRecord> {
        self.gate(&ctx, "artifacts.patch")?;
        let svc = self
            .state()
            .artifacts()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        // Per-artifact scope ACL: fetch the record, then check write rights
        // against its scope before mutating.
        let auth = auth_for(&ctx)?;
        let scope = require_scope(&svc, request.id).await?;
        if !can_write_artifacts(auth, &scope) {
            return Err(ConnectError::permission_denied(
                "insufficient scope for artifact write",
            ));
        }
        let patched = svc
            .patch(
                request.id,
                opt_str(request.name),
                opt_str(request.owner_actor),
                opt_str(request.expires_at),
            )
            .await
            .map_err(artifacts_err)?;
        Ok(Response::new(artifact_record_to_proto(patched)))
    }

    async fn promote(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPromoteArtifactRequestView,
    ) -> ServiceResult<ArtifactRecord> {
        self.gate(&ctx, "artifacts.promote")?;
        let svc = self
            .state()
            .artifacts()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        let auth = auth_for(&ctx)?;
        let scope = require_scope(&svc, request.id).await?;
        if !can_write_artifacts(auth, &scope) {
            return Err(ConnectError::permission_denied(
                "insufficient scope for artifact write",
            ));
        }
        let target = parse_target(
            request.target_scope,
            opt_str(request.run_id),
            self.state().project_slug().unwrap_or_default().as_str(),
        )
        .map_err(ConnectError::invalid_argument)?;
        // `retained: true` mirrors the prior wrapper, which always retained.
        let promoted = svc
            .promote(request.id, target, true)
            .await
            .map_err(artifacts_err)?;
        Ok(Response::new(artifact_record_to_proto(promoted)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteArtifactRequestView,
    ) -> ServiceResult<Deletion> {
        self.gate(&ctx, "artifacts.delete")?;
        let svc = self
            .state()
            .artifacts()
            .map_err(|e| ConnectError::failed_precondition(e.to_string()))?;
        let auth = auth_for(&ctx)?;
        let scope = require_scope(&svc, request.id).await?;
        if !can_delete_artifacts(auth, &scope) {
            return Err(ConnectError::permission_denied(
                "insufficient scope for artifact delete",
            ));
        }
        let deleted = svc
            .delete(request.id)
            .await
            .map_err(artifacts_err)?;
        Ok(Response::new(Deletion {
            id: request.id.to_string(),
            deleted,
            ..Default::default()
        }))
    }
}

/// Optional string arg → `Option<&str>` (absent / empty both → None).
fn opt_str(raw: &str) -> Option<&str> {
    if raw.is_empty() { None } else { Some(raw) }
}

/// Fetch the artifact's scope string ("run" | "project") or 404.
async fn require_scope(
    svc: &ship_artifacts::service::ArtifactService,
    id: &str,
) -> Result<String, ConnectError> {
    match svc
        .get_record(id)
        .await
        .map_err(artifacts_err)?
    {
        Some(r) => Ok(r.scope),
        None => Err(ConnectError::not_found(format!("artifact '{id}' not found"))),
    }
}

/// Resolve `target_scope` ("run" | "project") + its required id into an
/// `ArtifactScope`, mirroring the prior route's `parse_scope`.
fn parse_target(
    target: &str,
    run_id: Option<&str>,
    project_slug: &str,
) -> Result<ArtifactScope, String> {
    match target {
        "run" => {
            let run_id = run_id
                .ok_or_else(|| "run_id required for target_scope=run".to_string())?;
            Ok(ArtifactScope::Run { space_id: project_slug.to_string(), run_id: run_id.to_string() })
        }
        "project" => Ok(ArtifactScope::Project { space_id: project_slug.to_string() }),
        other => Err(format!("target_scope must be run|project, got `{other}`")),
    }
}

/// `ArtifactError` → `ConnectError` (not-found vs bad-args vs internal).
fn artifacts_err(e: ArtifactError) -> ConnectError {
    match e {
        ArtifactError::NotFound(m) => ConnectError::not_found(m),
        ArtifactError::Validation(m) => ConnectError::invalid_argument(m),
        other => ConnectError::internal(other.to_string()),
    }
}

/// Project the Rust `ArtifactRecord` (ship-artifacts) onto the proto
/// `ArtifactRecord` (ship-ops-rpc). The proto is a strict field-subset; the
/// Rust struct carries an extra `updated_at` we drop here.
fn artifact_record_to_proto(
    r: ship_artifacts::types::ArtifactRecord,
) -> ArtifactRecord {
    ArtifactRecord {
        id: r.id,
        name: r.name,
        mime: r.mime,
        size: r.size,
        path: r.path,
        scope: r.scope,
        space_id: r.space_id,
        run_id: r.run_id,
        user_id: r.user_id,
        content_hash: r.content_hash,
        expires_at: r.expires_at,
        owner_actor: r.owner_actor,
        collection_id: r.collection_id,
        item_id: r.item_id,
        created_at: r.created_at,
        ..Default::default()
    }
}

