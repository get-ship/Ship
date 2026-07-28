pub mod catalog;
pub mod collection;
pub mod config;
pub mod pipeline;
pub mod transfer;
pub mod trigger;
pub mod view;

pub use catalog::{
    CatalogCategory, CatalogEntry, CatalogKind, list_catalog, list_catalog_by_kind, search_catalog,
};
pub use collection::{CollectionDef, CollectionFieldDef, CollectionFieldType};
pub use config::{AgentLayerConfig, NamespaceConfig, SpaceManifest};
pub use pipeline::{PipelineDef, PipelineGate, PipelineStep, PipelineStepSpec};
pub use transfer::{
    AgentBundle, ArtifactType, ListAgentsResponse, PullAgent, PullMcpServer, PullProfile,
    PullResponse, PullRule, PullSkill, SkillBundle, TransferBundle,
};
pub use trigger::{TriggerAction, TriggerDef, TriggerEventMatch, TriggerSourceDef};
pub use view::ViewDescriptor;
