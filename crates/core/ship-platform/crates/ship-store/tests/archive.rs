use std::fs;
use std::path::Path;

use ship_store::{pack_stack, unpack_stack};
use stack_model::{ArchiveHeader, manifest::ManifestExports};

fn write_stack(root: &Path, files: &[(&str, &str)]) {
    let header = ArchiveHeader {
        path: "github.com/ship/demo-stack".into(),
        version: "1.2.3".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        hash: String::new(),
        exports: ManifestExports::default(),
        dependencies: Default::default(),
        recommended_agents: vec!["reviewer".into()],
        recommended_skills: vec![],
    };
    fs::write(
        root.join("ship.json"),
        serde_json::to_vec_pretty(&header).expect("serialize source header"),
    )
    .expect("write source header");
    for (path, content) in files {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(full, content).expect("write fixture file");
    }
}

#[test]
fn pack_stack_is_byte_deterministic_for_the_same_tree() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    write_stack(
        first.path(),
        &[
            ("stack.config.ts", "export default {};\n"),
            ("skills/review/SKILL.md", "# Review\n"),
        ],
    );
    write_stack(
        second.path(),
        &[
            ("skills/review/SKILL.md", "# Review\n"),
            ("stack.config.ts", "export default {};\n"),
        ],
    );
    fs::write(first.path().join("ship.lock"), "first local lock\n").unwrap();
    fs::write(second.path().join("ship.lock"), "different local lock\n").unwrap();
    fs::create_dir_all(first.path().join("state")).unwrap();
    fs::write(first.path().join("state/local.json"), "local state\n").unwrap();
    fs::write(second.path().join(".DS_Store"), "local noise\n").unwrap();

    let (first_bytes, first_header) = pack_stack(first.path()).expect("pack first tree");
    let (second_bytes, second_header) = pack_stack(second.path()).expect("pack second tree");

    assert_eq!(first_header, second_header);
    assert_eq!(
        first_bytes, second_bytes,
        "creation order must not affect archive bytes"
    );
    assert!(first_header.hash.starts_with("sha256:"));
}

#[test]
fn unpack_stack_round_trips_content_and_header() {
    let source = tempfile::tempdir().unwrap();
    write_stack(
        source.path(),
        &[
            ("stack.config.ts", "export default {};\n"),
            ("agents/reviewer.ts", "agent\n"),
        ],
    );
    let (bytes, expected) = pack_stack(source.path()).expect("pack tree");
    let dest = tempfile::tempdir().unwrap();

    let actual = unpack_stack(&bytes, dest.path()).expect("unpack tree");

    assert_eq!(actual, expected);
    assert_eq!(
        fs::read_to_string(dest.path().join("agents/reviewer.ts")).unwrap(),
        "agent\n"
    );
    let archived_header: ArchiveHeader = serde_json::from_slice(
        &fs::read(dest.path().join("ship.json")).expect("read archived header"),
    )
    .expect("parse archived header");
    assert_eq!(archived_header, expected);
}

#[test]
fn unpack_stack_rejects_content_tampering() {
    let source = tempfile::tempdir().unwrap();
    write_stack(
        source.path(),
        &[("stack.config.ts", "export default {};\n")],
    );
    let (bytes, _) = pack_stack(source.path()).expect("pack tree");

    let decoder = flate2::read::GzDecoder::new(bytes.as_slice());
    let mut archive = tar::Archive::new(decoder);
    let extracted = tempfile::tempdir().unwrap();
    archive.unpack(extracted.path()).unwrap();
    fs::write(extracted.path().join("stack.config.ts"), "tampered\n").unwrap();
    let (tampered, _) = pack_raw_tree(extracted.path());

    let dest = tempfile::tempdir().unwrap();
    let error = unpack_stack(&tampered, dest.path()).expect_err("tampering must fail");
    assert!(
        error.to_string().contains("tree hash mismatch"),
        "{error:#}"
    );
}

fn pack_raw_tree(root: &Path) -> (Vec<u8>, ()) {
    use std::io::Write;
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        builder.append_dir_all(".", root).unwrap();
        builder.finish().unwrap();
    }
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    (encoder.finish().unwrap(), ())
}
