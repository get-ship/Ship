/**
 * Event envelope — must match the Rust EventEnvelope exactly.
 * Field names, types, everything.
 */
export interface EventEnvelope {
  id: string;                // ULID
  event_type: string;        // e.g. "workspace.created"
  entity_id: string;
  actor: string;             // always "ship" for now
  payload_json: string;      // JSON string
  version: number;           // always 1
  correlation_id?: string;
  causation_id?: string;
  workspace_id?: string;
  session_id?: string;
  actor_id?: string;
  parent_actor_id?: string;
  elevated: boolean;
  created_at: string;        // ISO 8601
}

/** Platform-level event types (workspace lifecycle, jobs, config) */
export const PLATFORM_EVENT_TYPES = {
  // Workspace lifecycle
  WORKSPACE_CREATED: 'workspace.created',
  WORKSPACE_ACTIVATED: 'workspace.activated',
  WORKSPACE_ARCHIVED: 'workspace.archived',
  WORKSPACE_DELETED: 'workspace.deleted',
  WORKSPACE_COMPILED: 'workspace.compiled',

  // Jobs
  JOB_QUEUED: 'job.queued',
  JOB_STARTED: 'job.started',
  JOB_COMPLETED: 'job.completed',
  JOB_FAILED: 'job.failed',

  // Config
  CONFIG_UPDATED: 'config.updated',
} as const;

/** Workspace-level event types (actors, sessions, skills) */
export const WORKSPACE_EVENT_TYPES = {
  // Actors
  ACTOR_CREATED: 'actor.created',
  ACTOR_UPDATED: 'actor.updated',
  ACTOR_DELETED: 'actor.deleted',

  // Sessions
  SESSION_STARTED: 'session.started',
  SESSION_COMPLETED: 'session.completed',
  SESSION_FAILED: 'session.failed',

  // Skills
  SKILL_REGISTERED: 'skill.registered',
  SKILL_UPDATED: 'skill.updated',
  SKILL_REMOVED: 'skill.removed',
} as const;
