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
mod artifact;
mod auth;
mod collection;
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
mod studio;
mod sync;
mod watcher;
mod workspace;

pub use account::{AccountKeyRefreshed, AccountLoginCompleted, AccountLoginStarted, AccountLogout};
pub use actor::{
    ActorCrashed, ActorCreated, ActorMailboxFull, ActorSlept, ActorStopped, ActorWoke,
};
pub use artifact::{ArtifactCreated, ArtifactDeleted, ArtifactMoved, ArtifactUpdated};
pub use auth::{AgentGrantsProvisioned, AuthSessionCreated, AuthSessionExpired, AuthSessionRevoked};
pub use collection::{
    CollectionCreated, CollectionItemCreated, CollectionItemDeleted, CollectionItemUpdated,
    CollectionSchemaChanged,
};
pub use compile::CompileCompleted;
pub use config::{ConfigChanged, ConfigFileChanged, ProjectLog, SettingChanged, SpaceSettingChanged};
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
pub use mesh::{
    MeshBroadcast, MeshDeregister, MeshDeregistered, MeshDiscoverResponse, MeshMessage,
    MeshRegister, MeshRegistered, MeshSend, MeshSendFailed, MeshStatus,
};
pub use skill::{SkillCompleted, SkillFailed, SkillStarted};
pub use spec6::{AgentTaskComplete, HumanResponse, PipelineStepRetry};
pub use studio::{


    StudioAnnotationCreated, StudioChatTurn, StudioFeedbackCreated, StudioReply,
};
pub use sync::{SyncPushCompleted, SyncPushFailed, SyncStarted, SyncStopped};
pub use watcher::{
    AgentChanged, AgentDefUpdated, PipelineChanged, RuleChanged, SkillChanged, ViewChanged,
};
pub use project::{ProjectArchived, ProjectRegistered, ProjectSeen, ProjectSwitched};
pub use process::ProcessOrphaned;
pub use workspace::{WorkspaceActivated, WorkspaceArchived, WorkspaceCreated};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_created_payload_deserializes() {
        let original = WorkspaceCreated {
            workspace_id: "ws-1".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: WorkspaceCreated = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.workspace_id, "ws-1");
    }

}

