//! Per-project sandbox registry — pure bookkeeping over `Sandbox`
//! state machines. No backend calls, no async runtime concerns.
//!
//! The registry's job is to answer "which sandbox does this actor
//! belong to" and to track lifecycle transitions. Actual sandbox
//! preparation (Docker prepare, youki bundle, ...) is the
//! orchestrator's job and lands in Chunk C.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::types::{
    RuntimeEndpoint, Sandbox, SandboxError, SandboxId, SandboxName, SandboxState,
};
use ship_isolation::tier::IsolationProfile;

#[derive(Debug, Default)]
pub struct SandboxRegistry {
    sandboxes: HashMap<SandboxId, Sandbox>,
    /// Project + name → sandbox id index for fast lookup.
    by_name: HashMap<(String, SandboxName), SandboxId>,
    /// Actor → sandbox id index. Each actor lives in at most one
    /// sandbox at a time.
    actor_index: HashMap<String, SandboxId>,
}

impl SandboxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a fresh sandbox. Fails if the (project, name) pair
    /// already exists — duplicate names within a project are
    /// rejected at registration time.
    pub fn register(
        &mut self,
        project: impl Into<String>,
        name: SandboxName,
        isolation: IsolationProfile,
    ) -> Result<&Sandbox, SandboxError> {
        let project = project.into();
        let key = (project.clone(), name.clone());
        if self.by_name.contains_key(&key) {
            return Err(SandboxError::DuplicateName {
                project,
                name: name.0,
            });
        }
        let sandbox = Sandbox::new(project, name, isolation);
        let id = sandbox.id.clone();
        self.by_name.insert(key, id.clone());
        self.sandboxes.insert(id.clone(), sandbox);
        Ok(self.sandboxes.get(&id).unwrap())
    }

    pub fn get(&self, id: &SandboxId) -> Option<&Sandbox> {
        self.sandboxes.get(id)
    }

    pub fn get_mut(&mut self, id: &SandboxId) -> Option<&mut Sandbox> {
        self.sandboxes.get_mut(id)
    }

    pub fn by_project_name(
        &self,
        project: &str,
        name: &SandboxName,
    ) -> Option<&Sandbox> {
        let id = self.by_name.get(&(project.to_string(), name.clone()))?;
        self.sandboxes.get(id)
    }

    /// Return the sandbox an actor currently belongs to, if any.
    pub fn sandbox_for_actor(&self, actor_id: &str) -> Option<&Sandbox> {
        let id = self.actor_index.get(actor_id)?;
        self.sandboxes.get(id)
    }

    /// Iterate all sandboxes in a project.
    pub fn sandboxes_in_project<'a>(
        &'a self,
        project: &'a str,
    ) -> impl Iterator<Item = &'a Sandbox> + 'a {
        self.sandboxes.values().filter(move |s| s.project == project)
    }

    /// Transition Pending → Running, attaching the runtime endpoint.
    /// Idempotent — calling on an already-Running sandbox returns Ok
    /// without state change.
    pub fn mark_running(
        &mut self,
        id: &SandboxId,
        endpoint: RuntimeEndpoint,
    ) -> Result<(), SandboxError> {
        let sandbox = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| SandboxError::NotFound(id.0.clone()))?;
        match sandbox.state {
            SandboxState::Running => Ok(()),
            SandboxState::Pending | SandboxState::Idle => {
                sandbox.state = SandboxState::Running;
                sandbox.runtime_endpoint = Some(endpoint);
                Ok(())
            }
            other => Err(SandboxError::InvalidStateTransition {
                id: id.0.clone(),
                state: other,
                target: SandboxState::Running,
            }),
        }
    }

    /// Add an actor to a sandbox. Fails if the sandbox isn't ready
    /// to accept actors (must be Running or Idle).
    pub fn add_actor(
        &mut self,
        sandbox_id: &SandboxId,
        actor_id: impl Into<String>,
    ) -> Result<(), SandboxError> {
        let actor_id = actor_id.into();
        let sandbox = self
            .sandboxes
            .get_mut(sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.0.clone()))?;
        match sandbox.state {
            SandboxState::Running | SandboxState::Idle => {
                if !sandbox.actors.iter().any(|a| a == &actor_id) {
                    sandbox.actors.push(actor_id.clone());
                }
                // First actor cancels the idle timer
                if sandbox.state == SandboxState::Idle {
                    sandbox.state = SandboxState::Running;
                }
                sandbox.idle_since = None;
                self.actor_index.insert(actor_id, sandbox_id.clone());
                Ok(())
            }
            other => Err(SandboxError::InvalidStateForActor {
                id: sandbox_id.0.clone(),
                state: other,
                actor: actor_id,
            }),
        }
    }

    /// Remove an actor from its sandbox. If the sandbox has no more
    /// actors, transition to Idle and start the idle timer.
    pub fn remove_actor(
        &mut self,
        actor_id: &str,
        now: DateTime<Utc>,
    ) -> Result<(), SandboxError> {
        let sandbox_id = self
            .actor_index
            .remove(actor_id)
            .ok_or_else(|| SandboxError::NotFound(actor_id.to_string()))?;
        let sandbox = self
            .sandboxes
            .get_mut(&sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.0.clone()))?;
        sandbox.actors.retain(|a| a != actor_id);
        if sandbox.actors.is_empty() && sandbox.state == SandboxState::Running {
            sandbox.state = SandboxState::Idle;
            sandbox.idle_since = Some(now);
        }
        Ok(())
    }

    /// Transition Idle/Running → Reaping. The actual teardown (kill
    /// container, etc.) is the caller's job; this just locks out new
    /// actor spawns.
    pub fn mark_reaping(&mut self, id: &SandboxId) -> Result<(), SandboxError> {
        let sandbox = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| SandboxError::NotFound(id.0.clone()))?;
        match sandbox.state {
            SandboxState::Idle | SandboxState::Running => {
                sandbox.state = SandboxState::Reaping;
                Ok(())
            }
            other => Err(SandboxError::InvalidStateTransition {
                id: id.0.clone(),
                state: other,
                target: SandboxState::Reaping,
            }),
        }
    }

    /// Final state. Removes name + actor indexes; sandbox struct
    /// stays around for observability until explicitly dropped.
    pub fn mark_stopped(&mut self, id: &SandboxId) -> Result<(), SandboxError> {
        let sandbox = self
            .sandboxes
            .get_mut(id)
            .ok_or_else(|| SandboxError::NotFound(id.0.clone()))?;
        sandbox.state = SandboxState::Stopped;
        sandbox.runtime_endpoint = None;
        let key = (sandbox.project.clone(), sandbox.name.clone());
        let actors = std::mem::take(&mut sandbox.actors);
        self.by_name.remove(&key);
        for actor in actors {
            self.actor_index.remove(&actor);
        }
        Ok(())
    }

    /// Drop a Stopped sandbox from the registry entirely. No-op on
    /// other states.
    pub fn forget(&mut self, id: &SandboxId) {
        if let Some(sandbox) = self.sandboxes.get(id)
            && sandbox.state == SandboxState::Stopped
        {
            self.sandboxes.remove(id);
        }
    }

    /// Enumerate sandboxes the reaper should consider reaping right
    /// now. Caller drives the actual transition.
    pub fn reap_candidates(&self, now: DateTime<Utc>) -> Vec<SandboxId> {
        self.sandboxes
            .values()
            .filter(|s| s.is_ready_to_reap(now))
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.sandboxes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sandboxes.is_empty()
    }

    /// All unique project ids currently in the registry.
    pub fn all_projects(&self) -> Vec<String> {
        let mut seen = std::collections::BTreeSet::new();
        for s in self.sandboxes.values() {
            seen.insert(s.project.clone());
        }
        seen.into_iter().collect()
    }

    /// All sandboxes across all projects.
    pub fn all_sandboxes(&self) -> impl Iterator<Item = &Sandbox> {
        self.sandboxes.values()
    }
}
