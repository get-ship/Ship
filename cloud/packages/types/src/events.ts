// Canonical event envelope + event-type string constants live in ./generated,
// produced by `cargo xtask gen-types` from crates/core/ship-types. Import
// from './generated' for anything that must stay in lockstep with the Rust
// runtime's emitted events.
//
// The PLATFORM_/WORKSPACE_EVENT_TYPES groupings below are cloud-specific
// roll-ups that map Ship Cloud's Durable Object handlers to event names.
// DRIFT: several entries (noted inline) reference event names the Rust
// runtime does not emit today. Projection repos switching on those cases
// never match. Follow-up: decide whether to delete the dead branches or
// extend the Rust taxonomy.

export type { EventEnvelope } from './generated.js';

export const PLATFORM_EVENT_TYPES = {
  // Workspace lifecycle
  WORKSPACE_CREATED: 'workspace.created',
  WORKSPACE_ACTIVATED: 'workspace.activated',
  WORKSPACE_ARCHIVED: 'workspace.archived',
  WORKSPACE_DELETED: 'workspace.deleted',
  WORKSPACE_COMPILED: 'workspace.compiled',

  // Jobs — drift: Rust emits job.created / job.dispatched, not job.queued / job.started
  JOB_QUEUED: 'job.queued',
  JOB_STARTED: 'job.started',
  JOB_COMPLETED: 'job.completed',
  JOB_FAILED: 'job.failed',

  // Config — drift: Rust emits config.changed, not config.updated
  CONFIG_UPDATED: 'config.updated',
} as const;

export const WORKSPACE_EVENT_TYPES = {
  // Actors — drift: Rust emits actor.woke / actor.slept / actor.stopped / actor.crashed,
  // not actor.updated / actor.deleted
  ACTOR_CREATED: 'actor.created',
  ACTOR_UPDATED: 'actor.updated',
  ACTOR_DELETED: 'actor.deleted',

  // Sessions — drift: Rust emits session.ended, not session.completed / session.failed
  SESSION_STARTED: 'session.started',
  SESSION_COMPLETED: 'session.completed',
  SESSION_FAILED: 'session.failed',

  // Skills — drift: Rust emits skill.started / skill.completed / skill.failed,
  // not skill.registered / skill.updated / skill.removed
  SKILL_REGISTERED: 'skill.registered',
  SKILL_UPDATED: 'skill.updated',
  SKILL_REMOVED: 'skill.removed',
} as const;
