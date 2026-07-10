//! ConnectRPC wire layer for `ArtifactsService`.
//!
//! `ArtifactsWire<F>` wraps the service with a per-request context factory.
//! It implements the generated `ship_ops_rpc::ArtifactsService` trait so
//! shipd can mount it via `ArtifactsServiceExt::register`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;
use ship_substrate::{KernelLog, SpaceContext};

use ship_ops_rpc::{
    ArtifactRecord as ProtoRecord, ArtifactsListResponse, ArtifactsService as ArtifactsServiceTrait,
    Deletion, WatchEvent,
};

use crate::error::ArtifactError;
use crate::error::MAX_PAYLOAD_BYTES;
use crate::patch::{patch_frame_to_proto, snapshot_for_artifacts};
use crate::service::ArtifactService;
use crate::types::ArtifactRecord;

/// Wraps ArtifactService with a per-request context factory.
pub struct ArtifactsWire<F> {
    make_ctx: F,
    global_dir: PathBuf,
    reactive: Arc<ReactiveRegistry>,
}

impl<F> ArtifactsWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    pub fn new(kernel: Arc<KernelLog>, global_dir: PathBuf, make_ctx: F) -> Self {
        Self {
            make_ctx,
            global_dir,
            reactive: Arc::new(ReactiveRegistry::new(kernel)),
        }
    }

    fn make_svc(&self, rctx: &RequestContext) -> ArtifactService {
        ArtifactService::new((self.make_ctx)(rctx), self.global_dir.clone())
    }
}

fn record_to_proto(r: ArtifactRecord) -> ProtoRecord {
    ProtoRecord {
        id: r.id,
        name: r.name,
        mime: r.mime,
        size: r.size,
        path: r.path,
        scope: r.scope,
        space_id: r.space_id,
        run_id: r.run_id,
        content_hash: r.content_hash,
        expires_at: r.expires_at,
        owner_actor: r.owner_actor,
        collection_id: r.collection_id,
        item_id: r.item_id,
        created_at: r.created_at,
        ..Default::default()
    }
}

#[allow(refining_impl_trait)]
impl<F> ArtifactsServiceTrait for ArtifactsWire<F>
where
    F: Fn(&RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
{
    async fn list(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedListArtifactsRequestView,
    ) -> ServiceResult<ArtifactsListResponse> {
        let svc = self.make_svc(&ctx);
        let mut filter = BTreeMap::new();
        if !request.scope.is_empty() {
            filter.insert("scope".into(), sv(request.scope.to_string()));
        }
        if !request.mime.is_empty() {
            filter.insert("mime".into(), sv(request.mime.to_string()));
        }
        if !request.collection_id.is_empty() {
            filter.insert("collection_id".into(), sv(request.collection_id.to_string()));
        }
        if !request.item_id.is_empty() {
            filter.insert("item_id".into(), sv(request.item_id.to_string()));
        }
        if !request.owner_actor.is_empty() {
            filter.insert("owner_actor".into(), sv(request.owner_actor.to_string()));
        }
        if !request.content_hash.is_empty() {
            filter.insert("content_hash".into(), sv(request.content_hash.to_string()));
        }
        let limit = if request.limit == 0 { 100 } else { request.limit as u32 };
        let items = svc.list(filter, limit).await.map_err(ConnectError::from)?;
        let count = items.len() as u64;
        let scope = request.scope.to_string();
        Ok(Response::new(ArtifactsListResponse {
            items: items.into_iter().map(record_to_proto).collect(),
            count,
            scope,
            ..Default::default()
        }))
    }

    async fn watch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedArtifactsWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        let space_id = self.make_svc(&ctx).ctx.space_id().to_string();
        let scope_filter = request.scope.to_string();
        let mime_filter = request.mime.to_string();
        let scope_key = format!("artifacts:scope={scope_filter}:mime={mime_filter}");

        let query_hash: u64 = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            scope_key.hash(&mut h);
            h.finish()
        };

        let key = WatchKey { space_id, scope: scope_key.clone(), query_hash };

        let ctx_arc = self.make_svc(&ctx).ctx.clone();
        let scope_f = scope_filter.clone();
        let mime_f = mime_filter.clone();

        let spec = ResolverSpec {
            prefilter: Box::new(move |env| env.event_type.starts_with("artifact.")),
            snapshot: Box::new(move || {
                let ctx = ctx_arc.clone();
                let s = scope_f.clone();
                let m = mime_f.clone();
                Box::pin(async move { snapshot_for_artifacts(ctx, &s, &m).await })
            }),
        };

        let filter = EventFilter::default();
        let mut rx = self.reactive.watch(key, filter, spec);

        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(frame) => yield Ok(patch_frame_to_proto(frame)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // SHORTCOMING (artifacts authz pass): patch/promote/delete are C1 writes gated only by the
    // blast tier — they do NOT consult the caller's artifact Scopes. The role model grants agents
    // run-scope write but withholds Write/DeleteArtifactsProject (ship-auth-types scopes::roles::
    // pipeline_agent_artifact_scopes); nothing here checks it, so an agent can mutate/promote
    // PROJECT-scoped artifacts. Fix: thread caller scopes in (RequestContext → scopes) + a per-scope
    // check vs the target scope. Spec: #[ignore]'d test in ship-auth-types scopes/tests.rs.
    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchArtifactRequestView,
    ) -> ServiceResult<ProtoRecord> {
        let svc = self.make_svc(&ctx);
        let name = ne(request.name);
        let owner = ne(request.owner_actor);
        let exp = ne(request.expires_at);
        let record = svc
            .patch(request.id.as_ref(), name, owner, exp)
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(record_to_proto(record)))
    }

    async fn promote(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPromoteArtifactRequestView,
    ) -> ServiceResult<ProtoRecord> {
        let svc = self.make_svc(&ctx);
        let target = parse_target_scope(
            request.target_scope,
            ne(request.run_id),
            svc.ctx.space_id().as_str(),
        )
        .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
        let record = svc
            .promote(request.id.as_ref(), target, true)
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(record_to_proto(record)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteArtifactRequestView,
    ) -> ServiceResult<Deletion> {
        let svc = self.make_svc(&ctx);
        let deleted = svc.delete(request.id.as_ref()).await.map_err(ConnectError::from)?;
        if !deleted {
            return Err(ConnectError::not_found(format!("artifact '{}' not found", request.id)));
        }
        Ok(Response::new(Deletion { id: request.id.to_string(), ..Default::default() }))
    }

    /// Store a new artifact. Thin wire layer over `ArtifactService::store`:
    /// enforces the byte cap, parses the scope, and converts the typed error.
    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateArtifactRequestView,
    ) -> ServiceResult<ProtoRecord> {
        if request.data.len() > MAX_PAYLOAD_BYTES {
            return Err(ConnectError::invalid_argument(format!(
                "payload {} bytes exceeds cap of {} bytes",
                request.data.len(),
                MAX_PAYLOAD_BYTES
            )));
        }
        let svc = self.make_svc(&ctx);
        let scope = parse_target_scope(
            request.scope,
            ne(request.run_id),
            svc.ctx.space_id().as_str(),
        )
        .map_err(|e| ConnectError::invalid_argument(e.to_string()))?;
        let record = svc
            .store(
                request.name,
                request.mime,
                request.data,
                &scope,
                ne(request.collection_id),
                ne(request.item_id),
                ne(request.owner_actor),
                ne(request.expires_at),
            )
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(record_to_proto(record)))
    }

    /// Read an artifact's record + bytes. Thin wire layer over
    /// `ArtifactService::get_blob`: a missing id surfaces as `ConnectError::not_found`.
    async fn get_blob(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetArtifactBlobRequestView,
    ) -> ServiceResult<ship_ops_rpc::ArtifactBlobResponse> {
        let svc = self.make_svc(&ctx);
        let (record, bytes) = svc
            .get_blob(request.id.as_ref())
            .await
            .map_err(ConnectError::from)?;
        Ok(Response::new(ship_ops_rpc::ArtifactBlobResponse {
            record: ::buffa::MessageField::some(record_to_proto(record)),
            data: bytes,
            ..Default::default()
        }))
    }
}
fn sv(s: String) -> serde_json::Value {
    serde_json::Value::String(s)
}

/// Return `Some(s)` if non-empty, else `None`.
fn ne(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

fn parse_target_scope(
    target: &str,
    run_id: Option<&str>,
    space_id: &str,
) -> Result<crate::types::ArtifactScope, ArtifactError> {
    use crate::types::ArtifactScope;
    match target {
        "run" => {
            let run_id = run_id
                .ok_or_else(|| ArtifactError::Validation("run_id required for target_scope=run".into()))?;
            Ok(ArtifactScope::Run { space_id: space_id.to_string(), run_id: run_id.to_string() })
        }
        "project" => Ok(ArtifactScope::Project { space_id: space_id.to_string() }),
        other => Err(ArtifactError::Validation(format!(
            "target_scope must be run|project, got `{other}`"
        ))),
    }
}
