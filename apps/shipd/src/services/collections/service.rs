//! `CollectionsService` — db-bound trait + impl backed by the runtime
//! `DataService`. Holds a clone of the same `Arc<Mutex<Option<DataService>>>`
//! the runtime owns so a single connection covers both call paths.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use ship_platform::collections::{builtin, query::QueryRequest, schema, service as col_svc, store};
use ship_substrate::data::DataService;
use ship_substrate::events::KernelRouter;

use crate::ship_runtime::ensure_data_service;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
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
pub struct CreateItemArgs {
    pub collection: String,
    pub id: String,
    pub data: serde_json::Value,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateItemArgs {
    pub collection: String,
    pub id: String,
    pub data: serde_json::Value,
}

#[async_trait]
pub trait CollectionsService: Send + Sync + 'static {
    /// User-visible schemas (built-in + project-defined). Excludes system
    /// collections like feedback / annotations.
    fn list_user_schemas(&self) -> Result<Vec<schema::CollectionSchema>, Error>;
    /// Schema by id, including system-internal ones.
    fn get_schema(&self, collection: &str) -> Result<schema::CollectionSchema, Error>;
    fn create_schema(&self, schema: schema::CollectionSchema) -> Result<(), Error>;
    fn update_schema(
        &self,
        collection: &str,
        schema: schema::CollectionSchema,
    ) -> Result<(), Error>;

    async fn create_item(&self, args: CreateItemArgs) -> Result<store::Item, Error>;
    async fn get_item(&self, collection: &str, id: &str) -> Result<store::Item, Error>;
    async fn update_item(&self, args: UpdateItemArgs) -> Result<store::Item, Error>;
    async fn delete_item(&self, collection: &str, id: &str) -> Result<bool, Error>;
    async fn query_items(
        &self,
        collection: &str,
        req: QueryRequest,
    ) -> Result<Vec<store::Item>, Error>;
}

pub struct CollectionsServiceImpl {
    ship_dir: PathBuf,
    kernel: SharedKernel,
    data: SharedData,
}

impl CollectionsServiceImpl {
    pub fn new(ship_dir: PathBuf, kernel: SharedKernel, data: SharedData) -> Self {
        Self { ship_dir, kernel, data }
    }

    pub fn arc(
        ship_dir: PathBuf,
        kernel: SharedKernel,
        data: SharedData,
    ) -> Arc<dyn CollectionsService> {
        Arc::new(Self::new(ship_dir, kernel, data))
    }

    fn load_any_schema(&self, collection: &str) -> Result<schema::CollectionSchema, Error> {
        let project_dir = runtime::project::get_project_dir(None).ok();
        let user_schemas = schema::load_schemas_merged(&self.ship_dir, project_dir.as_deref())
            .map_err(|e| Error::Io(e.to_string()))?;
        let mut schemas = builtin::all_schemas();
        schemas.extend(user_schemas);
        schemas
            .into_iter()
            .find(|s| s.id == collection)
            .ok_or_else(|| Error::NotFound(format!("collection '{collection}' not found")))
    }
}

#[async_trait]
impl CollectionsService for CollectionsServiceImpl {
    fn list_user_schemas(&self) -> Result<Vec<schema::CollectionSchema>, Error> {
        let project_dir = runtime::project::get_project_dir(None).ok();
        let user = schema::load_schemas_merged(&self.ship_dir, project_dir.as_deref())
            .map_err(|e| Error::Io(e.to_string()))?;
        let mut schemas = builtin::user_schemas();
        schemas.extend(user);
        Ok(schemas)
    }

    fn get_schema(&self, collection: &str) -> Result<schema::CollectionSchema, Error> {
        self.load_any_schema(collection)
    }

    fn create_schema(&self, schema_def: schema::CollectionSchema) -> Result<(), Error> {
        let known = [builtin::NOTES_ID, builtin::FEEDBACK_ID, builtin::ANNOTATIONS_ID];
        if known.contains(&schema_def.id.as_str()) {
            return Err(Error::Conflict(format!(
                "'{}' is a reserved built-in collection id",
                schema_def.id,
            )));
        }
        if schema_def.id.is_empty()
            || !schema_def.id.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(Error::Validation(format!(
                "collection id '{}' must be non-empty alphanumeric/underscore",
                schema_def.id,
            )));
        }
        let dir = self.ship_dir.join("collections");
        std::fs::create_dir_all(&dir).map_err(|e| Error::Io(format!(
            "cannot create {}: {e}", dir.display(),
        )))?;
        let path = dir.join(format!("{}.jsonc", schema_def.id));
        if path.exists() {
            return Err(Error::Conflict(format!(
                "collection '{}' already exists",
                schema_def.id,
            )));
        }
        let json = serde_json::to_string_pretty(&schema_def)
            .map_err(|e| Error::Validation(format!("serialize error: {e}")))?;
        let tmp = path.with_extension("jsonc.tmp");
        std::fs::write(&tmp, &json).map_err(|e| Error::Io(format!("write failed: {e}")))?;
        std::fs::rename(&tmp, &path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Error::Io(format!("rename failed: {e}"))
        })?;
        Ok(())
    }

    fn update_schema(
        &self,
        collection: &str,
        updated: schema::CollectionSchema,
    ) -> Result<(), Error> {
        let known = [builtin::NOTES_ID, builtin::FEEDBACK_ID, builtin::ANNOTATIONS_ID];
        if known.contains(&collection) {
            return Err(Error::Conflict(format!(
                "'{collection}' is a built-in collection; its schema is defined in the runtime and cannot be edited through the API",
            )));
        }
        if updated.id != collection {
            return Err(Error::Validation(format!(
                "path collection '{collection}' does not match body id '{}' — rename is unsupported",
                updated.id,
            )));
        }
        let dir = self.ship_dir.join("collections");
        std::fs::create_dir_all(&dir).map_err(|e| Error::Io(format!(
            "cannot create {}: {e}", dir.display(),
        )))?;
        let path = dir.join(format!("{collection}.jsonc"));
        let json = serde_json::to_string_pretty(&updated)
            .map_err(|e| Error::Validation(format!("could not serialize schema: {e}")))?;
        let tmp = path.with_extension("jsonc.tmp");
        std::fs::write(&tmp, &json).map_err(|e| Error::Io(format!("write failed: {e}")))?;
        std::fs::rename(&tmp, &path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Error::Io(format!("rename failed: {e}"))
        })?;
        Ok(())
    }

    async fn create_item(&self, args: CreateItemArgs) -> Result<store::Item, Error> {
        let schema = self.load_any_schema(&args.collection)?;
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialized");
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        col_svc::create_item(
            svc,
            &self.kernel,
            &schema,
            &args.id,
            &args.data,
            args.created_by.as_deref(),
        )
        .await
        .map_err(|e| classify(e.to_string()))
    }

    async fn get_item(&self, collection: &str, id: &str) -> Result<store::Item, Error> {
        let schema = self.load_any_schema(collection)?;
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialized");
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        match col_svc::get_item(svc, &schema, id) {
            Ok(Some(item)) => Ok(item),
            Ok(None) => Err(Error::NotFound(format!("item '{id}' not found"))),
            Err(e) => Err(Error::Internal(e.to_string())),
        }
    }

    async fn update_item(&self, args: UpdateItemArgs) -> Result<store::Item, Error> {
        let schema = self.load_any_schema(&args.collection)?;
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialized");
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        col_svc::update_item(svc, &self.kernel, &schema, &args.id, &args.data)
            .await
            .map_err(|e| classify(e.to_string()))
    }

    async fn delete_item(&self, collection: &str, id: &str) -> Result<bool, Error> {
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialized");
        col_svc::delete_item(svc, &self.kernel, collection, id)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }

    async fn query_items(
        &self,
        collection: &str,
        req: QueryRequest,
    ) -> Result<Vec<store::Item>, Error> {
        let schema = self.load_any_schema(collection)?;
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialized");
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        col_svc::query_items(svc, &schema, &req).map_err(|e| Error::Internal(e.to_string()))
    }
}

fn classify(msg: String) -> Error {
    let lower = msg.to_lowercase();
    if lower.contains("validation") {
        Error::Validation(msg)
    } else if lower.contains("not found") {
        Error::NotFound(msg)
    } else {
        Error::Internal(msg)
    }
}
