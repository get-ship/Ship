pub const CHANGED: &str = "skill.changed";
pub const STARTED: &str = "skill.started";
pub const COMPLETED: &str = "skill.completed";
pub const FAILED: &str = "skill.failed";
/// Emitted after `skill.started` for library-mode invokes. Records the
/// session_id (existing chat session or the synthetic invoke run id)
/// that the skill's reference content is attached to. `entity_id` is the
/// skill id. Payload: `SkillAttachedToSession`.
pub const ATTACHED_TO_SESSION: &str = "skill.attached_to_session";
/// Skill CRUD — a skill SKILL.md (project- or user-scoped) was written
/// for the first time. `entity_id` is the skill id.
pub const CREATED: &str = "skill.created";
/// Skill CRUD — content or front-matter of an existing skill changed.
/// `entity_id` is the skill id.
pub const UPDATED: &str = "skill.updated";
/// Skill CRUD — a skill was removed from disk. `entity_id` is the
/// skill id.
pub const DELETED: &str = "skill.deleted";

pub const ALL_SKILL: &[&str] = &[
    CHANGED,
    STARTED,
    COMPLETED,
    FAILED,
    ATTACHED_TO_SESSION,
    CREATED,
    UPDATED,
    DELETED,
];
