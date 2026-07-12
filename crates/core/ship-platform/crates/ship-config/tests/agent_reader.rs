//! THE reader proofs (S2 — agents arc, cutover).
//!
//! ONE agent-def file-access reader serves BOTH granularities off a single
//! byte-read:
//!   - FULL `stack_model::AgentProfile` (nested) — what the compiler's
//!     `load_agent_profiles` consumes; NO field loss.
//!   - SUMMARY `AgentDefSource` — what the read-only index needs.
//!
//! These proofs pin the parity contract: the reader returns the SAME full
//! profile the loader's `stack_model::jsonc::from_jsonc_str` would parse, and the
//! summary is a faithful derivation of the same bytes (incl. `agent.harnesses`,
//! which the nested `stack_model::AgentProfile` view drops). Defs are authored
//! through the SAME `FsBroker` door the reader uses — zero raw fs of content.

use std::path::{Path, PathBuf};

use ship_config::agents::{
    read_agent_def, read_agent_defs, read_agent_profile, read_agent_profile_at,
    read_agent_profiles, read_agent_profiles_in,
};
use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker, SpaceId};

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// Canonical project `.ship` dir under a fresh tempdir. `/tmp` is symlinked on
/// WSL/macOS and the FsBroker's symlink-escape guard canonicalizes the deepest
/// existing ancestor, so the cap root must be canonical — production dirs are.
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    // Marker cutover: the reader anchors at the STACK ROOT.
    std::fs::canonicalize(tmp.path()).unwrap()
}

/// Author a def file through the broker (no std::fs writes of content). Roots
/// the cap at `<ship_dir>/agents` directly — the same dir the reader confines.
fn write_def(ship_dir: &Path, id: &str, body: &str) {
    let dir = ship_dir.join("agents");
    std::fs::create_dir_all(&dir).unwrap();
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(SpaceId::new("config")),
        FsMode::ReadWrite,
    );
    let fs = LocalFsBroker::new();
    fs.write_atomic(&cap, &format!("{id}.jsonc"), body.as_bytes())
        .unwrap();
}

const FULL_DEF: &str = r#"{
  "agent": {
    "id": "code-reviewer",
    "name": "Code Reviewer",
    "version": "1.2.0",
    "description": "Reviews pull requests.",
    "role": "worker",
    "tags": ["review", "testing"],
    "tools_preset": "worker",
    "active_tools": ["Read", "Grep"],
    "icon": "test-tube",
    "harnesses": ["claude", "codex"]
  },
  "skills": { "refs": ["review-pr", "lint-fix"] },
  "mcp": { "servers": ["github", "linear"] },
  "plugins": { "install": ["eslint-plugin"], "scope": "workspace" },
  "permissions": {
    "preset": "ship-standard",
    "tools_allow": ["Read", "Grep", "Bash"],
    "tools_ask": ["Edit"],
    "tools_deny": ["WebFetch"],
    "default_mode": "acceptEdits"
  },
  "rules": { "inline": "You are a senior code reviewer." },
  "hooks": [
    { "id": "session-end", "trigger": "Stop", "command": "ship session end" },
    { "id": "sub-end", "trigger": "SubagentStop", "command": "ship sub end" }
  ],
  "apps": { "studio": true },
  "harness_settings": { "claude": { "model": "opus", "temperature": 0.2 } }
}"#;

// ── Parity: the full profile loses NO fields ───────────────────────────────────

#[test]
fn read_agent_profiles_returns_full_nested_profile_no_field_loss() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "code-reviewer", FULL_DEF);

    let fs = LocalFsBroker::new();
    let profiles = read_agent_profiles(&fs, &dir).unwrap();
    assert_eq!(profiles.len(), 1);
    let p = &profiles[0];

    // meta
    assert_eq!(p.agent.id, "code-reviewer");
    assert_eq!(p.agent.name, "Code Reviewer");
    assert_eq!(p.agent.version.as_deref(), Some("1.2.0"));
    assert_eq!(p.agent.description.as_deref(), Some("Reviews pull requests."));
    assert_eq!(p.agent.role.as_deref(), Some("worker"));
    assert_eq!(p.agent.tags, vec!["review", "testing"]);
    assert_eq!(p.agent.tools_preset.as_deref(), Some("worker"));
    assert_eq!(p.agent.active_tools, vec!["Read", "Grep"]);
    assert_eq!(p.agent.icon.as_deref(), Some("test-tube"));
    // nested collections — the field loss the spec forbids
    assert_eq!(p.skills.refs, vec!["review-pr", "lint-fix"]);
    assert_eq!(p.mcp.servers, vec!["github", "linear"]);
    assert_eq!(p.plugins.install, vec!["eslint-plugin"]);
    assert_eq!(p.plugins.scope.as_deref(), Some("workspace"));
    assert_eq!(p.permissions.preset.as_deref(), Some("ship-standard"));
    assert_eq!(p.permissions.tools_allow, vec!["Read", "Grep", "Bash"]);
    assert_eq!(p.permissions.tools_ask, vec!["Edit"]);
    assert_eq!(p.permissions.tools_deny, vec!["WebFetch"]);
    assert_eq!(p.permissions.default_mode.as_deref(), Some("acceptEdits"));
    assert_eq!(p.rules.inline.as_deref(), Some("You are a senior code reviewer."));
    assert_eq!(p.hooks.len(), 2);
    assert_eq!(p.hooks[0].command, "ship session end");
    assert_eq!(p.hooks[1].command, "ship sub end");
    assert!(p.apps.studio);
    let claude = p.harness_settings.get("claude").expect("claude settings");
    assert_eq!(claude.get("model").and_then(|v| v.as_str()), Some("opus"));
}

#[test]
fn reader_parse_matches_loader_jsonc_parse_byte_for_byte() {
    // The compiler's loader parses defs via `stack_model::jsonc::from_jsonc_str`.
    // The reader must yield an identical `stack_model::AgentProfile` so the cutover
    // is behavior-preserving.
    let loader_parse: stack_model::AgentProfile =
        stack_model::jsonc::from_jsonc_str(FULL_DEF).expect("loader parse");

    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "code-reviewer", FULL_DEF);
    let fs = LocalFsBroker::new();
    let reader_parse = read_agent_profiles(&fs, &dir).unwrap().remove(0);

    // Compare via canonical JSON — the structural identity the loader relied on.
    assert_eq!(
        serde_json::to_value(&loader_parse).unwrap(),
        serde_json::to_value(&reader_parse).unwrap(),
        "reader profile must equal the loader's jsonc parse",
    );
}

#[test]
fn read_agent_profiles_sorted_by_id_and_skips_malformed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "zeta", r#"{ "agent": { "id": "zeta", "name": "Zeta" } }"#);
    write_def(&dir, "alpha", r#"{ "agent": { "id": "alpha", "name": "Alpha" } }"#);
    // Malformed (no `agent` wrapper) — the loader skips, not fails.
    write_def(&dir, "broken", r#"{ "name": "Broken" }"#);

    let fs = LocalFsBroker::new();
    let profiles = read_agent_profiles(&fs, &dir).unwrap();
    let ids: Vec<&str> = profiles.iter().map(|p| p.agent.id.as_str()).collect();
    assert_eq!(ids, vec!["alpha", "zeta"], "sorted by id, malformed skipped");
}

#[test]
fn read_agent_profiles_missing_dir_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let fs = LocalFsBroker::new();
    let profiles = read_agent_profiles(&fs, &ship_dir(&tmp)).unwrap();
    assert!(profiles.is_empty());
}

#[test]
fn read_agent_profile_single_by_stem() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "code-reviewer", FULL_DEF);
    let fs = LocalFsBroker::new();

    let found = read_agent_profile(&fs, &dir, "code-reviewer").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().agent.id, "code-reviewer");
    assert!(read_agent_profile(&fs, &dir, "nope").unwrap().is_none());
}

// ── view_state seam: read directly from an explicit agents dir ─────────────────

#[test]
fn read_agent_profiles_in_explicit_agents_dir() {
    // view_state scans `<ship>/library/agents` and `<project>/.ship/agents`
    // directly — the reader must serve a raw agents-dir path, not only a
    // `ship_dir + /agents` derivation.
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "alpha", r#"{ "agent": { "id": "alpha", "name": "Alpha" } }"#);
    let agents_dir = dir.join("agents");

    let fs = LocalFsBroker::new();
    let profiles = read_agent_profiles_in(&fs, &agents_dir).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].agent.id, "alpha");

    // Missing dir → empty.
    let missing = dir.join("nope");
    assert!(read_agent_profiles_in(&fs, &missing).unwrap().is_empty());
}

// ── CLI seam: read a single explicit file path ─────────────────────────────────

#[test]
fn read_agent_profile_at_explicit_path() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "code-reviewer", FULL_DEF);
    let path = dir.join("agents").join("code-reviewer.jsonc");

    let fs = LocalFsBroker::new();
    let profile = read_agent_profile_at(&fs, &path).unwrap();
    assert!(profile.is_some());
    assert_eq!(profile.unwrap().agent.id, "code-reviewer");

    let missing = dir.join("agents").join("nope.jsonc");
    assert!(read_agent_profile_at(&fs, &missing).unwrap().is_none());
}

// ── Summary: one byte-read, two parsers — harnesses survives ───────────────────

#[test]
fn read_agent_defs_summary_keeps_harnesses_off_the_same_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = ship_dir(&tmp);
    write_def(&dir, "code-reviewer", FULL_DEF);

    let fs = LocalFsBroker::new();
    let defs = read_agent_defs(&fs, &dir).unwrap();
    assert_eq!(defs.len(), 1);
    let d = &defs[0];
    assert_eq!(d.id, "code-reviewer");
    assert_eq!(d.name, "Code Reviewer");
    assert_eq!(d.role.as_deref(), Some("worker"));
    assert_eq!(d.tags, vec!["review", "testing"]);
    // `harnesses` is dropped by the nested `stack_model::AgentProfile` view — the
    // summary must still carry it (read off the same bytes).
    assert_eq!(d.harness, vec!["claude", "codex"]);
    assert_eq!(d.path, "code-reviewer.jsonc");
    assert!(!d.content_hash.is_empty());

    let single = read_agent_def(&fs, &dir, "code-reviewer").unwrap().unwrap();
    assert_eq!(single.harness, vec!["claude", "codex"]);
    assert_eq!(single.content_hash, d.content_hash, "single + walk agree");
}
