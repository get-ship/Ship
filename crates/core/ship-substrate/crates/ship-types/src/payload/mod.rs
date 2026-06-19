// Canonical event payload structs for ship-types. The 90+ payloads were a
// single flat module to keep one shared re-export surface and one registry
// identity in `ship_substrate::events::payload_registry`; they are now grouped
// by domain into sub-modules to satisfy the 400-line file cap (architecture.md).
//
// Every public symbol is re-exported from this module root so the public
// `ship_types::payload::*` surface (used by every consumer) is byte-identical
// to the previous flat module. Consumers must not be forced to import from
// sub-paths.

mod account;
mod actor;
// artifact payload structs moved to ship-artifacts/src/events.rs
// (keystone-A de-bucket, 2026-06-10): the ARTIFACT domain owns its events
// at L1. No re-export shim — consumers import from ship_artifacts::events.
mod auth;
// collection payload structs moved to ship-collection/src/events.rs
// (keystone-A de-bucket, 2026-06-10): the COLLECTION domain owns its events
// at L1. No re-export shim — consumers import from ship_collection::events.
mod compile;
mod config;
mod crud;
mod daemon;
mod kernel;
mod mesh;
mod process;
mod project;
mod skill;

mod spec6;
// studio payload structs moved to ship-agents/src/studio_events.rs
// (keystone-A de-bucket, 2026-06-10): the STUDIO domain owns its events at L1.
// No re-export shim — consumers import from ship_agents::studio_events.
mod sync;
mod watcher;

pub use account::{AccountKeyRefreshed, AccountLoginCompleted, AccountLoginStarted, AccountLogout};
pub use actor::{
    ActorCrashed, ActorCreated, ActorMailboxFull, ActorSlept, ActorStopped, ActorWoke,
};
pub use auth::{AgentGrantsProvisioned, AuthSessionCreated, AuthSessionExpired, AuthSessionRevoked};
// CollectionCreated / CollectionItem* / CollectionSchemaChanged moved to
// ship-collection/src/events.rs — no re-export here.
pub use compile::CompileCompleted;
pub use config::{ConfigChanged, ConfigFileChanged, SettingChanged, SpaceSettingChanged};
pub use crud::{
    PermissionsUpdated, RuleCreated, RuleDeleted, RuleUpdated, SkillCreated, SkillDeleted,
    SkillUpdated,
};
pub use daemon::{
    DaemonCrashed, DaemonOwnerBound, DaemonOwnerUnbound, DaemonPidfileAcquired,
    DaemonPidfileStaleTaken, DaemonPortBindFailed, DaemonShutdownCompleted, DaemonStartupCompleted,
    SupervisorShuttingDown,
};
pub use kernel::{KernelActorRestored, KernelActorSnapshot, KernelActorSuspended};
pub use mesh::{MeshDeregister, MeshMessage, MeshRegister};
pub use skill::{SkillCompleted, SkillFailed, SkillStarted};
pub use spec6::{AgentTaskComplete, HumanResponse, PipelineStepRetry};
// StudioAnnotationCreated / StudioChatTurn / StudioFeedbackCreated / StudioReply
// moved to ship-agents/src/studio_events.rs — no re-export here.
pub use sync::{SyncPushCompleted, SyncPushFailed, SyncStarted, SyncStopped};
pub use watcher::{
    AgentChanged, AgentDefUpdated, PipelineChanged, RuleChanged, SkillChanged, ViewChanged,
};
pub use project::{ProjectArchived, ProjectRegistered, ProjectSeen, ProjectSwitched};
pub use process::ProcessOrphaned;


