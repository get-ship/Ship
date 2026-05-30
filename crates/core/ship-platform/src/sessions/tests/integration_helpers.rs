/// Integration tests — sessions helpers: datetime parsing, text normalization,
/// branch-key validation, and space-id derivation.
///
/// All helpers are `pub(crate)` so they are visible here via the crate path.
/// Tests are hermetic — no shared state; each uses its own tempdir where needed.
use anyhow::Result;
use tempfile::tempdir;

use crate::sessions::helpers::{
    ensure_branch_key, normalize_agent_ref, normalize_harness_ref, normalize_optional_text,
    parse_datetime, parse_datetime_opt, space_id_for_ship_dir,
};
use crate::testutil::init_project;

// ── parse_datetime ────────────────────────────────────────────────────────────

#[test]
fn sessions_helpers_parse_datetime_valid_rfc3339() {
    let ts = "2024-01-15T10:30:00Z";
    let dt = parse_datetime(ts);
    // Year round-trips correctly.
    assert_eq!(dt.format("%Y").to_string(), "2024");
}

#[test]
fn sessions_helpers_parse_datetime_falls_back_to_now_on_invalid() {
    use chrono::Utc;
    let before = Utc::now();
    let dt = parse_datetime("not-a-timestamp");
    let after = Utc::now();
    // Fallback must be a plausible "now" — within a few seconds.
    assert!(dt >= before, "fallback must not be before call");
    assert!(dt <= after, "fallback must not be after call");
}

// ── parse_datetime_opt ────────────────────────────────────────────────────────

#[test]
fn sessions_helpers_parse_datetime_opt_none_returns_none() {
    assert!(parse_datetime_opt(None).is_none());
}

#[test]
fn sessions_helpers_parse_datetime_opt_invalid_string_returns_none() {
    assert!(parse_datetime_opt(Some("garbage".to_string())).is_none());
}

#[test]
fn sessions_helpers_parse_datetime_opt_valid_rfc3339_returns_some() {
    let result = parse_datetime_opt(Some("2024-06-01T00:00:00Z".to_string()));
    assert!(result.is_some());
    assert_eq!(result.unwrap().format("%Y-%m").to_string(), "2024-06");
}

// ── normalize_optional_text ───────────────────────────────────────────────────

#[test]
fn sessions_helpers_normalize_optional_text_none_returns_none() {
    assert!(normalize_optional_text(None).is_none());
}

#[test]
fn sessions_helpers_normalize_optional_text_whitespace_only_returns_none() {
    assert!(normalize_optional_text(Some("   ".to_string())).is_none());
    assert!(normalize_optional_text(Some("\t\n".to_string())).is_none());
}

#[test]
fn sessions_helpers_normalize_optional_text_trims_surrounding_whitespace() {
    let result = normalize_optional_text(Some("  hello world  ".to_string()));
    assert_eq!(result.as_deref(), Some("hello world"));
}

#[test]
fn sessions_helpers_normalize_optional_text_preserves_inner_whitespace() {
    let result = normalize_optional_text(Some("a  b".to_string()));
    assert_eq!(result.as_deref(), Some("a  b"));
}

// ── normalize_agent_ref ───────────────────────────────────────────────────────

#[test]
fn sessions_helpers_normalize_agent_ref_empty_returns_none() {
    assert!(normalize_agent_ref("").is_none());
    assert!(normalize_agent_ref("   ").is_none());
}

#[test]
fn sessions_helpers_normalize_agent_ref_valid_trims_and_returns_some() {
    assert_eq!(
        normalize_agent_ref("  my-agent  ").as_deref(),
        Some("my-agent")
    );
}

// ── normalize_harness_ref ─────────────────────────────────────────────────────

#[test]
fn sessions_helpers_normalize_harness_ref_empty_returns_none() {
    assert!(normalize_harness_ref("").is_none());
}

#[test]
fn sessions_helpers_normalize_harness_ref_lowercases_and_trims() {
    assert_eq!(
        normalize_harness_ref("  CLAUDE  ").as_deref(),
        Some("claude")
    );
}

// ── ensure_branch_key ────────────────────────────────────────────────────────

#[test]
fn sessions_helpers_ensure_branch_key_empty_returns_error() {
    let err = ensure_branch_key("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn sessions_helpers_ensure_branch_key_whitespace_only_returns_error() {
    let err = ensure_branch_key("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn sessions_helpers_ensure_branch_key_valid_returns_trimmed() -> Result<()> {
    let key = ensure_branch_key("feature/my-branch")?;
    assert_eq!(key, "feature/my-branch");
    Ok(())
}

// ── space_id_for_ship_dir ─────────────────────────────────────────────────────

#[test]
fn sessions_helpers_space_id_for_ship_dir_is_deterministic() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let id1 = space_id_for_ship_dir(&ship_dir);
    let id2 = space_id_for_ship_dir(&ship_dir);
    assert_eq!(id1, id2, "space_id must be deterministic for same path");
    Ok(())
}

#[test]
fn sessions_helpers_space_id_for_ship_dir_differs_across_projects() -> Result<()> {
    let tmp1 = tempdir()?;
    let tmp2 = tempdir()?;
    let ship_dir1 = init_project(tmp1.path())?;
    let ship_dir2 = init_project(tmp2.path())?;
    let id1 = space_id_for_ship_dir(&ship_dir1);
    let id2 = space_id_for_ship_dir(&ship_dir2);
    assert_ne!(id1, id2, "space_id must differ for different project paths");
    Ok(())
}

#[test]
fn sessions_helpers_space_id_is_non_empty() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let id = space_id_for_ship_dir(&ship_dir);
    assert!(!id.is_empty(), "space_id must not be empty");
    Ok(())
}
