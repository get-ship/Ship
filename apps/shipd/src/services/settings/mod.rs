//! Settings domain service. View-shaping only — the actual read/write
//! ops go through `runtime::config::{get_setting, set_setting}` +
//! the HTTP endpoints under `/api/settings/*`. Registry exposes
//! `settings.state` → view ctx.

pub mod view_state;
