//! Centralized read helpers for daemon-rendered views.
//!
//! View handlers should never re-walk the filesystem or issue scattered
//! runtime calls. Everything they need arrives through this module, so
//! caching, error handling, and filter semantics live in one place.
//!
//! Agents + Skills state lives in sibling modules to respect the 300-line
//! cap on this file. Re-exports let callers use `views::state::list_agents`
//! without knowing about the split.
pub use ship_agents::services::agents::view_state::{
    AgentCatalogs, AgentDetail, AgentSummary, CatalogEntry, Permissions,
    ToolPatternEntry, list_agent_catalogs, list_agents, load_agent,
};
pub use ship_agents::services::skills::view_state::{
    SkillDetail, SkillKind, SkillSummary, list_skills, load_skill,
    load_skill_file,
};