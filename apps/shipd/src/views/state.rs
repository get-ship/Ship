//! Centralized read helpers for daemon-rendered views.
//!
//! View handlers should never re-walk the filesystem or issue scattered
//! runtime calls. Everything they need arrives through this module, so
//! caching, error handling, and filter semantics live in one place.
//!
//! Pipeline domain state (PipelineDef, Filter, RunCategory, list_pipeline_*,
//! aggregate_pipeline_stats, partition_runs, status_token, status_tone) has
//! been relocated to `ship_pipeline_runtime::view::state` (L2 → L1).
//! This module re-exports them to preserve existing call sites.

// Agents + Skills state lives in sibling modules to respect the 300-line
// cap on this file. Re-exports let callers use `views::state::list_agents`
// without knowing about the split.
pub use ship_agents::services::agents::view_state::{
    AgentCatalogs, AgentDetail, AgentSummary, CatalogEntry, Permissions,
    ToolPatternEntry, list_agent_catalogs, list_agents, load_agent,
};
pub use ship_agents::services::skills::view_state::{
    SkillDetail, SkillKind, SkillSummary, list_skills, load_skill,
    load_skill_file,
};

// Pipeline domain — relocated to L1.
pub use ship_pipeline_runtime::view::state::{
    Filter, PipelineDef, RunCategory,
    filter_and_sort, list_pipeline_defs, list_pipeline_runs,
    aggregate_pipeline_stats, partition_runs, status_token, status_tone,
    pipeline_name_for_job_public,
};

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
