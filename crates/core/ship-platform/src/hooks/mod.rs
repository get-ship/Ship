//! Hook engine — re-exported from `ship-agents::hooks` (C8 keystone).
pub use ship_agents::hooks::{
    extract_skill_id,
    handle_before_tool,
    handle_after_tool,
    handle_session_start,
    handle_session_end,
    tool_gate,
};
