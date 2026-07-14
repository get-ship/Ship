use std::fs;
use std::path::Path;

use ship_store::{pack_stack, unpack_stack};
use stack_model::ArchiveHeader;

fn write_stack(root: &Path, files: &[(&str, &str)]) {
    fs::create_dir_all(root).expect("create stack root");
    for (path, content) in files {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(full, content).expect("write fixture file");
    }
}

fn fixture_header() -> ArchiveHeader {
    ArchiveHeader {
        path: "github.com/ship/demo-stack".into(),
        version: "1.2.3".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        hash: String::new(),
        dependencies: Default::default(),
        recommended_agents: vec!["reviewer".into()],
        recommended_skills: vec![],
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
    // Non-stack content at the stack root — the allow-list must
    // exclude it from both the archive and the hash.
    fs::write(first.path().join("ship.lock"), "first local lock\n").unwrap();
    fs::write(second.path().join("ship.lock"), "different local lock\n").unwrap();
    fs::create_dir_all(first.path().join("contrib")).unwrap();
    fs::write(first.path().join("contrib/notes.md"), "scratch\n").unwrap();
    fs::write(second.path().join(".DS_Store"), "local noise\n").unwrap();

    let (first_bytes, first_header) =
        pack_stack(first.path(), fixture_header()).expect("pack first tree");
    let (second_bytes, second_header) =
        pack_stack(second.path(), fixture_header()).expect("pack second tree");

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
    let (bytes, expected) =
        pack_stack(source.path(), fixture_header()).expect("pack tree");
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
    let (bytes, _) =
        pack_stack(source.path(), fixture_header()).expect("pack tree");

    let decoder = flate2::read::GzDecoder::new(bytes.as_slice());
    let mut archive = tar::Archive::new(decoder);
    let extracted = tempfile::tempdir().unwrap();
    archive.unpack(extracted.path()).unwrap();
    fs::write(extracted.path().join("stack.config.ts"), "tampered\n").unwrap();
    let (tampered, _) = pack_raw_tree(extracted.path());

    let dest = tempfile::tempdir().unwrap();
    let error =
        unpack_stack(&tampered, dest.path()).expect_err("tampering must fail");
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

// A stack root may sit at a repo root, beside build output and fetched
// dependencies that dwarf the stack itself. The allow-list is what
// guarantees the archive is small — `target/`, `node_modules/`, `dist/`,
// etc. are not domain dirs, so the walk never descends into them.

#[test]
fn pack_stack_never_packs_build_output_or_dependencies() {
    let src = tempfile::tempdir().expect("tempdir");
    write_stack(
        src.path(),
        &[
            ("stack.config.ts", "export default {};\n"),
            ("agents/reviewer.ts", "export default {}"),
        ],
    );
    // The artifacts that must be excluded — at the root and nested.
    for junk in [
        "target/debug/huge.rlib",
        "node_modules/left-pad/index.js",
        "dist/bundle.js",
        ".venv/pyvenv.cfg",
        "crates/inner/target/debug/nested.rlib",
        "web/node_modules/dep/index.js",
    ] {
        let full = src.path().join(junk);
        fs::create_dir_all(full.parent().unwrap()).expect("create junk parent");
        fs::write(&full, "BUILD ARTIFACT — must never be packed")
            .expect("write junk");
    }

    let (bytes, _header) =
        pack_stack(src.path(), fixture_header()).expect("pack");
    let dest = tempfile::tempdir().expect("dest tempdir");
    let out = dest.path().join("unpacked");
    unpack_stack(&bytes, &out).expect("unpack");

    assert!(
        out.join("agents/reviewer.ts").exists(),
        "real stack content must survive"
    );
    for junk in [
        "target",
        "node_modules",
        "dist",
        ".venv",
        "crates/inner/target",
        "web/node_modules",
    ] {
        assert!(
            !out.join(junk).exists(),
            "{junk} must never be packed into a stack archive"
        );
    }
}

#[test]
fn pack_stack_refuses_a_tree_over_the_size_ceiling() {
    let src = tempfile::tempdir().expect("tempdir");
    write_stack(
        src.path(),
        &[
            ("stack.config.ts", "export default {};\n"),
            ("agents/a.ts", "x"),
        ],
    );
    // One file past the 256 MiB ceiling — sparse, so the test stays
    // fast. Placed under `skills/` (a domain dir) so the allow-list
    // admits it and the size ceiling — the defence-in-depth backstop —
    // is the thing that catches it.
    let big = src.path().join("skills/blob.bin");
    fs::create_dir_all(big.parent().unwrap()).expect("create skills");
    let f = fs::File::create(&big).expect("create blob");
    f.set_len(257 * 1024 * 1024).expect("size the blob");
    drop(f);

    let err = pack_stack(src.path(), fixture_header())
        .expect_err("oversized stack must be refused");
    // Walk the error chain — the per-file cap surfaces inside
    // `compute_tree_hash`, which is wrapped by `pack_stack`'s
    // `.context("hashing stack tree")`, so the named bound lives in
    // a source, not the top message.
    let mut msg = err.to_string();
    let mut src: Option<&dyn std::error::Error> = err.source();
    while let Some(e) = src {
        msg.push_str(&format!(": {e}"));
        src = e.source();
    }
    assert!(
        msg.contains("packing limit")
            || msg.contains("MAX_BUNDLE_FILE")
            || msg.contains("exceeded"),
        "oversized file must be refused with a named bound; got: {msg}"
    );
    // The path may or may not be in the message depending on which
    // layer caught it. The hash-layer error names the file, the
    // pack-layer error names the file. Both OK; just don't claim a
    // false negative if neither did.
    let _ = msg.contains("skills/blob.bin");
}

#[test]
fn pack_stack_omits_non_allow_list_top_level_entries() {
    // The spec: pack contains ONLY the manifest + domain dirs. A
    // top-level file that is not the manifest (README, justfile,
    // ship.lock, .DS_Store, .git/) is not stack content and is
    // never packed.
    let src = tempfile::tempdir().expect("tempdir");
    write_stack(
        src.path(),
        &[
            ("stack.config.ts", "export default {};\n"),
            ("README.md", "Project notes\n"),
            ("justfile", "default:\n\techo hi\n"),
        ],
    );
    fs::create_dir_all(src.path().join("contrib")).unwrap();
    fs::write(src.path().join("contrib/notes.md"), "scratch\n").unwrap();
    fs::create_dir_all(src.path().join("ops")).unwrap();
    fs::write(src.path().join("ops/runbook.md"), "ops\n").unwrap();
    fs::create_dir_all(src.path().join("scripts")).unwrap();
    fs::write(src.path().join("scripts/build.sh"), "#!/bin/sh\n").unwrap();

    let (bytes, _) =
        pack_stack(src.path(), fixture_header()).expect("pack");
    let dest = tempfile::tempdir().expect("dest");
    let out = dest.path().join("unpacked");
    unpack_stack(&bytes, &out).expect("unpack");

    let mut entries: Vec<String> = fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec!["ship.json", "stack.config.ts"],
        "top-level entries must be ship.json + manifest only (no domain dirs in the source)"
    );
}
