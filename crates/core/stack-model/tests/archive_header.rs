use stack_model::archive_header::ArchiveHeader;

const GOLDEN: &str = include_str!("fixtures/archive_header.json");

#[test]
fn archive_header_round_trips_the_golden_fixture() {
    let header: ArchiveHeader = serde_json::from_str(GOLDEN).expect("golden header parses");

    assert_eq!(header.path, "github.com/ship/demo-stack");
    assert_eq!(header.version, "1.2.3");
    assert_eq!(header.commit, "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(
        header.hash,
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
    assert_eq!(header.exports.skills, ["skills/review"]);
    assert_eq!(header.exports.agents, ["agents/reviewer.ts"]);
    assert_eq!(
        header.dependencies["github.com/ship/base-stack"]
            .clone()
            .into_dep()
            .version,
        "^2.0.0"
    );
    assert_eq!(header.recommended_agents, ["reviewer"]);
    assert_eq!(header.recommended_skills, ["review"]);

    let serialized = serde_json::to_string_pretty(&header).expect("serialize header") + "\n";
    assert_eq!(
        serialized, GOLDEN,
        "golden ship.json bytes must stay stable"
    );
    let reparsed: ArchiveHeader = serde_json::from_str(&serialized).expect("reparse header");
    assert_eq!(reparsed, header);
}

#[test]
fn archive_header_defaults_recommendations_for_older_headers() {
    let header: ArchiveHeader = serde_json::from_str(
        r#"{
            "path": "github.com/ship/minimal",
            "version": "0.1.0",
            "commit": "abcdef",
            "hash": "sha256:deadbeef",
            "exports": {},
            "dependencies": {}
        }"#,
    )
    .expect("minimal header parses");

    assert!(header.recommended_agents.is_empty());
    assert!(header.recommended_skills.is_empty());
}
