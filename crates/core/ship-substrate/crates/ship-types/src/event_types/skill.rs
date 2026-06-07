pub const CHANGED: &str = "skill.changed";
pub const STARTED: &str = "skill.started";
pub const COMPLETED: &str = "skill.completed";
pub const FAILED: &str = "skill.failed";
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
    CREATED,
    UPDATED,
    DELETED,
];
