use super::*;

fn minimal_jsonc() -> &'static str {
    r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "1.0.0"
        }
    }"#
}

#[test]
fn parse_minimal_manifest() {
    let m = ShipManifest::from_jsonc_str(minimal_jsonc()).unwrap();
    assert_eq!(m.module.as_ref().unwrap().name, "github.com/owner/repo");
    assert_eq!(m.module.as_ref().unwrap().version, "1.0.0");
    assert!(m.module.as_ref().unwrap().description.is_none());
    assert!(m.module.as_ref().unwrap().license.is_none());
    assert!(m.dependencies.is_empty());
    assert!(m.exports.skills.is_empty());
    assert!(m.exports.agents.is_empty());
}

#[test]
fn parse_full_manifest() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/mylib",
            "version": "2.3.1",
            "description": "A great library",
            "license": "MIT"
        },
        "dependencies": {
            "github.com/a/b": "^1.0.0",
            "github.com/c/d": { "version": "~2.1.0", "grant": ["Bash", "Read"] }
        },
        "exports": {
            "skills": ["agents/skills/my-skill"],
            "agents": ["agents/profiles/my-agent.jsonc"],
            "views": ["views/dashboard"],
            "pipelines": ["pipelines/ship-feature.jsonc"],
            "triggers": ["triggers/on-push.jsonc"],
            "collections": ["collections/notes.jsonc"]
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().description.as_deref(), Some("A great library"));
    assert_eq!(m.module.as_ref().unwrap().license.as_deref(), Some("MIT"));
    assert_eq!(m.dependencies.len(), 2);

    let (_, dep_b) = m.resolved_deps().next().unwrap();
    assert_eq!(dep_b.version, "^1.0.0");
    assert!(dep_b.grant.is_empty());

    let dep_d = m
        .dependencies
        .get("github.com/c/d")
        .unwrap()
        .clone()
        .into_dep();
    assert_eq!(dep_d.version, "~2.1.0");
    assert_eq!(dep_d.grant, vec!["Bash", "Read"]);

    assert_eq!(m.exports.skills, vec!["agents/skills/my-skill"]);
    assert_eq!(m.exports.agents, vec!["agents/profiles/my-agent.jsonc"]);
    // The full traveling set — a package bundles any combination (#122 / store).
    assert_eq!(m.exports.views, vec!["views/dashboard"]);
    assert_eq!(m.exports.pipelines, vec!["pipelines/ship-feature.jsonc"]);
    assert_eq!(m.exports.triggers, vec!["triggers/on-push.jsonc"]);
    assert_eq!(m.exports.collections, vec!["collections/notes.jsonc"]);
}

#[test]
fn missing_name_is_error() {
    let jsonc = r#"{
        "module": { "version": "1.0.0" }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("[module].name"), "{err}");
}

#[test]
fn missing_version_is_error() {
    let jsonc = r#"{
        "module": { "name": "github.com/owner/repo" }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("[module].version"), "{err}");
}

#[test]
fn missing_module_section_is_valid() {
    let jsonc = r#"{
        "dependencies": { "github.com/a/b": "^1.0.0" }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert!(m.module.is_none());
    assert_eq!(m.dependencies.len(), 1);
}

#[test]
fn require_module_errors_when_absent() {
    let jsonc = r#"{
        "dependencies": { "github.com/a/b": "^1.0.0" }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    let err = m.require_module().unwrap_err();
    assert!(err.to_string().contains("[module]"), "{err}");
}

#[test]
fn invalid_semver_is_error() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "not-semver"
        }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("not valid semver"), "{err}");
}

#[test]
fn version_with_v_prefix_is_valid() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "v1.2.3"
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().version, "v1.2.3");
}

#[test]
fn empty_dependencies_is_valid() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "0.1.0"
        },
        "dependencies": {}
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert!(m.dependencies.is_empty());
}

#[test]
fn empty_version_constraint_is_error() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "0.1.0"
        },
        "dependencies": { "github.com/a/b": "" }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(
        err.to_string().contains("empty version constraint"),
        "{err}"
    );
}

#[test]
fn resolved_deps_normalises_shorthand() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "0.1.0"
        },
        "dependencies": { "github.com/a/b": "main" }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    let deps: Vec<_> = m.resolved_deps().collect();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0, "github.com/a/b");
    assert_eq!(deps[0].1.version, "main");
    assert!(deps[0].1.grant.is_empty());
}

#[test]
fn omitting_exports_is_valid() {
    let m = ShipManifest::from_jsonc_str(minimal_jsonc()).unwrap();
    assert!(m.exports.skills.is_empty());
    assert!(m.exports.agents.is_empty());
}

#[test]
fn name_rejects_uppercase() {
    let jsonc = r#"{
        "module": {
            "name": "GitHub.com/owner/repo",
            "version": "1.0.0"
        }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("invalid characters"), "{err}");
}

#[test]
fn name_rejects_spaces() {
    let jsonc = r#"{
        "module": {
            "name": "my package",
            "version": "1.0.0"
        }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("invalid characters"), "{err}");
}

#[test]
fn name_accepts_scoped() {
    let jsonc = r#"{
        "module": {
            "name": "@myorg/my-agent",
            "version": "1.0.0"
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().name, "@myorg/my-agent");
}

#[test]
fn name_accepts_dotted_host() {
    let jsonc = r#"{
        "module": {
            "name": "gitlab.com/owner/repo",
            "version": "1.0.0"
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().name, "gitlab.com/owner/repo");
}

#[test]
fn license_validates_spdx() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "1.0.0",
            "license": "MIT"
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().license.as_deref(), Some("MIT"));
}

#[test]
fn license_accepts_compound_expression() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "1.0.0",
            "license": "MIT OR Apache-2.0"
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().license.as_deref(), Some("MIT OR Apache-2.0"));
}

#[test]
fn license_rejects_unknown() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "1.0.0",
            "license": "MADE-UP-LICENSE"
        }
    }"#;
    let err = ShipManifest::from_jsonc_str(jsonc).unwrap_err();
    assert!(err.to_string().contains("not a valid SPDX"), "{err}");
}

#[test]
fn authors_field_parses() {
    let jsonc = r#"{
        "module": {
            "name": "github.com/owner/repo",
            "version": "1.0.0",
            "authors": ["Alice <alice@example.com>", "Bob"]
        }
    }"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().authors.len(), 2);
    assert_eq!(m.module.as_ref().unwrap().authors[0], "Alice <alice@example.com>");
}

#[test]
fn authors_defaults_to_empty() {
    let m = ShipManifest::from_jsonc_str(minimal_jsonc()).unwrap();
    assert!(m.module.as_ref().unwrap().authors.is_empty());
}

#[test]
fn sha_constraint_is_valid() {
    let sha = "abc1234567890abcdef1234567890abcdef1234";
    let jsonc = format!(
        r#"{{
            "module": {{
                "name": "github.com/owner/repo",
                "version": "0.1.0"
            }},
            "dependencies": {{ "github.com/a/b": "{sha}" }}
        }}"#
    );
    let m = ShipManifest::from_jsonc_str(&jsonc).unwrap();
    assert_eq!(
        m.dependencies
            .get("github.com/a/b")
            .unwrap()
            .clone()
            .into_dep()
            .version,
        sha
    );
}

#[test]
fn parse_jsonc_manifest() {
    let jsonc = r#"{
  // Ship manifest in JSONC format
  "module": {
    "name": "github.com/owner/repo",
    "version": "1.0.0",
    "description": "A library", // inline comment
  },
  "dependencies": {
    "github.com/a/b": "^1.0.0",
  },
}"#;
    let m = ShipManifest::from_jsonc_str(jsonc).unwrap();
    assert_eq!(m.module.as_ref().unwrap().name, "github.com/owner/repo");
    assert_eq!(m.module.as_ref().unwrap().version, "1.0.0");
    assert_eq!(m.module.as_ref().unwrap().description.as_deref(), Some("A library"));
    assert_eq!(m.dependencies.len(), 1);
}

#[test]
fn from_file_reads_jsonc() {
    let dir = tempfile::tempdir().unwrap();
    let jsonc_path = dir.path().join("ship.jsonc");
    std::fs::write(
        &jsonc_path,
        r#"{
  "module": { "name": "github.com/test/repo", "version": "0.1.0" }
}"#,
    )
    .unwrap();
    let m = ShipManifest::from_file(&jsonc_path).unwrap();
    assert_eq!(m.module.as_ref().unwrap().name, "github.com/test/repo");
}
