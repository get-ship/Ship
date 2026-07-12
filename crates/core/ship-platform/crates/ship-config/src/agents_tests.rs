//! Unit tests for THE agent-def reader ([`super`]). Kept in a sibling file so
//! the reader source stays under the file cap. The broader parity + summary
//! proofs live in `tests/agent_reader.rs`; these pin the confinement edges.

use super::*;
use ship_substrate::ports::{FsBroker as _, LocalFsBroker};
use tempfile::tempdir;

/// Returns `ship_dir` for direct broker writes in tests. Canonicalizes the
/// tempdir root: on WSL/macOS `/tmp` is a symlink, and the broker's
/// symlink-escape guard canonicalizes the deepest existing ancestor before
/// comparing it to the cap root. Production ship dirs are already canonical.
fn ship_dir(tmp: &tempfile::TempDir) -> PathBuf {
    // Marker cutover: the reader anchors at the STACK ROOT.
    std::fs::canonicalize(tmp.path()).unwrap()
}

/// Author a canonical-shape file through the SAME broker door (no std::fs).
fn write(fs: &LocalFsBroker, dir: &Path, id: &str, doc: serde_json::Value) {
    let (cap, agents_dir) = agents_cap(dir, FsMode::ReadWrite);
    fs.create_dir_all(&cap, &agents_dir).unwrap();
    fs.write_atomic(
        &cap,
        &agent_rel(&agents_dir, id),
        serde_json::to_vec_pretty(&doc).unwrap().as_slice(),
    )
    .unwrap();
}

#[test]
fn read_agent_profiles_reads_canonical_via_fs_broker() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write(
        &fs,
        &dir,
        "alpha",
        serde_json::json!({
            "agent": { "id": "alpha", "name": "Alpha", "role": "worker" },
            "skills": { "refs": ["rust"] },
            "mcp": { "servers": ["fs"] }
        }),
    );

    let profiles = read_agent_profiles(&fs, &dir).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].agent.id, "alpha");
    assert_eq!(profiles[0].agent.role.as_deref(), Some("worker"));
    assert_eq!(profiles[0].skills.refs, vec!["rust".to_string()]);
    assert_eq!(profiles[0].mcp.servers, vec!["fs".to_string()]);
}

#[test]
fn read_agent_docs_in_keeps_rules_refs_the_nested_view_drops() {
    // The daemon view-state needs `rules.refs` — a field the nested
    // `compiler::AgentProfile` view does NOT carry. The raw-doc reader must
    // surface it so the cutover does not lose the summary's rule count.
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write(
        &fs,
        &dir,
        "alpha",
        serde_json::json!({
            "agent": { "id": "alpha", "name": "Alpha" },
            "rules": { "refs": ["discipline", "rust"] }
        }),
    );
    let agents_dir = dir.join("agents");
    let docs = read_agent_docs_in(&fs, &agents_dir).unwrap();
    assert_eq!(docs.len(), 1);
    let (stem, value) = &docs[0];
    assert_eq!(stem, "alpha");
    let refs = value["rules"]["refs"].as_array().unwrap();
    assert_eq!(refs.len(), 2, "rules.refs survive the raw-doc reader");
}

#[test]
fn read_agent_doc_in_single_by_id() {
    let tmp = tempdir().unwrap();
    let dir = ship_dir(&tmp);
    let fs = LocalFsBroker::new();
    write(&fs, &dir, "alpha", serde_json::json!({ "agent": { "id": "alpha", "name": "Alpha" } }));
    let agents_dir = dir.join("agents");

    let found = read_agent_doc_in(&fs, &agents_dir, "alpha").unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().0, "alpha");
    assert!(read_agent_doc_in(&fs, &agents_dir, "nope").unwrap().is_none());
}
