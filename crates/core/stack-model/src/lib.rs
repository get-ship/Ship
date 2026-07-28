//! `stack-model` — the stack's data model and composition law (L0 pure
//! logic, no ship runtime deps).
//!
//! Owns the manifest, lockfile, skill parsing, library types, and the
//! compose law (`compose_space`). Standard-tier: shared by floki and
//! ship-code alike. Harness codegen lives in `harness-config-compiler` (the
//! third-party-harness driver box), which depends on this crate — never
//! the reverse.
//!
//! Key modules: `types` (all config types), `resolve` (library
//! resolution), `compose` (the space compose law), `loader`, `manifest`,
//! `lockfile`, `skill_parser`.

pub mod archive_header;
pub mod compose;
pub mod domain_dirs;
pub mod events;
pub mod loader;
pub mod lockfile;
pub mod manifest;
pub mod resolve;
pub mod route_tree;
pub mod skill_parser;
pub mod types;

// ─── Top-level re-exports ─────────────────────────────────────────────────────
pub use archive_header::ArchiveHeader;
pub use domain_dirs::{DOMAIN_DIRS, is_domain_dir};
pub use loader::load_library;

pub use compose::{ComposedLibrary, NounKind, ShadowRecord, compose_space};
pub use resolve::{
    ExtSource, ExtSourceKind, ExtensionDescriptor, ExtensionLibrary, ModuleDescriptor,
    PrecedenceRow, RollupInputs, WorkspaceOverrides, resolve, resolve_library,
};
pub use route_tree::{RouteNode, RouteTree, compose_route_tree};
pub use types::{
    AgentBundle, AgentLayerConfig, ArtifactType, CatalogCategory, CatalogEntry, CatalogKind,
    CollectionDef, CollectionFieldDef, CollectionFieldType, ListAgentsResponse, NamespaceConfig,
    PipelineDef, PipelineGate, PipelineStep, PipelineStepSpec, PullAgent, PullMcpServer,
    PullProfile, PullResponse, PullRule, PullSkill, SkillBundle, SpaceManifest, TransferBundle,
    TriggerAction, TriggerDef, TriggerEventMatch, TriggerSourceDef, list_catalog,
    list_catalog_by_kind, search_catalog,
};

/// Generate a nanoid using Ship's 56-character alphabet (no ambiguous chars).
pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
        'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}
