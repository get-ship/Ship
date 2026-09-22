//! PROBE: pack `/home/madvib/dev/Ship/ship-meta` (a real stack root) and
//! report the packed byte size + the exact top-level entry list. It must
//! contain ONLY the manifest + domain dirs — no `contrib/`, `ops/`,
//! `scripts/`, `justfile`, `README.md`. Expect kilobytes.

use std::path::Path;

use ship_store::{pack_stack, unpack_stack};
use stack_model::ArchiveHeader;

#[test]
fn proof_pack_ship_meta() {
    let root = Path::new("/home/madvib/dev/Ship/ship-meta");
    let header = ArchiveHeader {
        path: "github.com/ship/ship-meta".into(),
        version: "0.4.0".into(),
        commit: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
        hash: String::new(),
        dependencies: Default::default(),
        recommended_agents: vec![],
        recommended_skills: vec![],
    };
    let (bytes, _header) = pack_stack(root, header).expect("pack ship-meta");
    println!("\n== ship-meta pack proof ==");
    println!("packed bytes: {}", bytes.len());

    let dest = tempfile::tempdir().expect("dest");
    unpack_stack(&bytes, dest.path()).expect("unpack");

    let mut entries: Vec<String> = std::fs::read_dir(dest.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();
    println!("top-level entries ({}):", entries.len());
    for e in &entries {
        println!("  {}", e);
    }
    // ship-meta ships only `agents/` and `skills/` (no hooks/views/etc.).
    // The transport header (ship.json) and the manifest (stack.config.ts)
    // are the only top-level source files. The allow-list rule guarantees
    // nothing else.
    assert_eq!(
        entries,
        vec!["agents", "ship.json", "skills", "stack.config.ts"],
        "top-level entries must be ship.json + manifest + the domain dirs the source actually has"
    );
    for forbidden in ["contrib", "ops", "scripts", "justfile", "README.md"] {
        assert!(
            !entries.iter().any(|e| e == forbidden),
            "{forbidden} must not appear in the packed archive"
        );
    }
    // Defence-in-depth — confirm the size is on the order the spec
    // promises (kilobytes, not megabytes — build output would blow
    // this up many times over).
    assert!(
        bytes.len() < 256 * 1024,
        "ship-meta pack should be on the order of kilobytes; got {} bytes",
        bytes.len()
    );
}
