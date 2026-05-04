//! `ArtifactsService` — db-bound trait + impl wrapping the runtime
//! `ArtifactService` with shared kernel + data handles.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use runtime::artifacts::{ArtifactRecord, ArtifactScope, ArtifactService};
use runtime::collections::query::QueryRequest;
use runtime::data::DataService;
use runtime::events::KernelRouter;

use crate::ship_runtime::ensure_data_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("io: {0}")]
    Io(String),
    #[error("internal: {0}")]
    Internal(String),
}

pub type SharedKernel = Arc<Mutex<KernelRouter>>;
pub type SharedData = Arc<Mutex<Option<DataService>>>;

#[derive(Debug, Clone)]
pub struct StoreArgs {
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
    pub scope: ArtifactScope,
    pub collection_id: Option<String>,
    pub item_id: Option<String>,
    pub owner_actor: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PatchArgs {
    pub name: Option<String>,
    pub owner_actor: Option<String>,
    pub expires_at: Option<String>,
}

#[async_trait]
pub trait ArtifactsService: Send + Sync + 'static {
    async fn store(&self, args: StoreArgs) -> Result<ArtifactRecord, Error>;
    /// Returns `(record, bytes)`.
    async fn get(&self, id: &str) -> Result<(ArtifactRecord, Vec<u8>), Error>;
    async fn get_record(&self, id: &str) -> Result<ArtifactRecord, Error>;
    async fn patch(&self, id: &str, args: PatchArgs) -> Result<ArtifactRecord, Error>;
    async fn delete(&self, id: &str) -> Result<bool, Error>;
    async fn promote(&self, id: &str, target: ArtifactScope) -> Result<ArtifactRecord, Error>;
    async fn list(&self, req: QueryRequest) -> Result<Vec<ArtifactRecord>, Error>;
}

pub struct ArtifactsServiceImpl {
    ship_dir: PathBuf,
    kernel: SharedKernel,
    data: SharedData,
}

impl ArtifactsServiceImpl {
    pub fn new(ship_dir: PathBuf, kernel: SharedKernel, data: SharedData) -> Self {
        Self { ship_dir, kernel, data }
    }

    pub fn arc(
        ship_dir: PathBuf,
        kernel: SharedKernel,
        data: SharedData,
    ) -> Arc<dyn ArtifactsService> {
        Arc::new(Self::new(ship_dir, kernel, data))
    }

    fn runtime_svc(&self) -> ArtifactService {
        ArtifactService::new(&self.ship_dir)
    }
}

#[async_trait]
impl ArtifactsService for ArtifactsServiceImpl {
    async fn store(&self, args: StoreArgs) -> Result<ArtifactRecord, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        svc.store(
            data,
            &self.kernel,
            &args.name,
            &args.mime,
            &args.bytes,
            &args.scope,
            args.collection_id.as_deref(),
            args.item_id.as_deref(),
            args.owner_actor.as_deref(),
            args.expires_at.as_deref(),
        )
        .await
        .map_err(|e| classify(e.to_string()))
    }

    async fn get(&self, id: &str) -> Result<(ArtifactRecord, Vec<u8>), Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        match svc.get(data, id) {
            Ok(Some(pair)) => Ok(pair),
            Ok(None) => Err(Error::NotFound(format!("artifact '{id}' not found"))),
            Err(e) => Err(classify(e.to_string())),
        }
    }

    async fn get_record(&self, id: &str) -> Result<ArtifactRecord, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        match svc.get_record(data, id) {
            Ok(Some(rec)) => Ok(rec),
            Ok(None) => Err(Error::NotFound(format!("artifact '{id}' not found"))),
            Err(e) => Err(Error::Internal(e.to_string())),
        }
    }

    async fn patch(&self, id: &str, args: PatchArgs) -> Result<ArtifactRecord, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        match svc
            .patch(
                data,
                &self.kernel,
                id,
                args.name.as_deref(),
                args.owner_actor.as_deref(),
                args.expires_at.as_deref(),
            )
            .await
        {
            Ok(Some(rec)) => Ok(rec),
            Ok(None) => Err(Error::NotFound(format!("artifact '{id}' not found"))),
            Err(e) => Err(Error::Internal(e.to_string())),
        }
    }

    async fn delete(&self, id: &str) -> Result<bool, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        svc.delete(data, &self.kernel, id)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }

    async fn promote(&self, id: &str, target: ArtifactScope) -> Result<ArtifactRecord, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        svc.promote(data, &self.kernel, id, target)
            .await
            .map_err(|e| classify(e.to_string()))
    }

    async fn list(&self, req: QueryRequest) -> Result<Vec<ArtifactRecord>, Error> {
        let svc = self.runtime_svc();
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let data = guard.as_mut().expect("data service initialized");
        svc.list(data, req).map_err(|e| Error::Internal(e.to_string()))
    }
}

fn classify(msg: String) -> Error {
    let lower = msg.to_lowercase();
    if lower.contains("not found") || lower.contains("blob missing") {
        Error::NotFound(msg)
    } else if lower.contains("validation") {
        Error::Validation(msg)
    } else {
        Error::Internal(msg)
    }
}
