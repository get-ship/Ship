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
mod process;
mod space;
mod skill;

mod spec6;
mod sync;
mod watcher;

pub use account::{AccountKeyRefreshed, AccountLoginCompleted, AccountLoginStarted, AccountLogout};
pub use actor::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};
pub use auth::{AgentGrantsProvisioned, AuthSessionCreated, AuthSessionExpired, AuthSessionRevoked};
// ship-collection/src/events.rs — no re-export here.
pub use compile::CompileCompleted;
pub use config::{ConfigChanged, ConfigFileChanged, SettingChanged};
pub use crud::{
    PermissionsUpdated, RuleCreated, RuleDeleted, RuleUpdated, SkillCreated, SkillDeleted,
    SkillUpdated,
};
pub use daemon::{DaemonOwnerBound, DaemonOwnerUnbound, DaemonPortBindFailed, SupervisorShuttingDown};
// KernelActor* retired (B2, 2026-06-20).
pub use skill::SkillFailed;
pub use spec6::{AgentTaskComplete, HumanResponse, PipelineStepRetry};
pub use sync::{SyncPushCompleted, SyncPushFailed, SyncStarted, SyncStopped};
pub use watcher::{AgentDefUpdated, SkillChanged};
pub use space::{SpaceOpened, SpaceSeen, StackRegistered};
pub use process::ProcessOrphaned;


