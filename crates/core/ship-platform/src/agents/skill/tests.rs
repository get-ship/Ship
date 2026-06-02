use super::*;
use super::install::{
    find_skill_source_dir, looks_like_skills_cli_command, parse_skills_install_source,
    validate_skill_install_request,
};
use super::storage::skill_dir;
use ship_substrate::fs_util::write_atomic;
#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_file;
use tempfile::tempdir;

    #[test]
    fn create_and_get_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::testutil::init_project(tmp.path())?;
        let s = create_skill(
            &project_dir,
            "review",
            "Code Review",
            "Review this: $ARGUMENTS",
        )?;
        assert_eq!(s.id, "review");
        assert_eq!(s.source, SkillSource::Custom);
        let got = get_skill(&project_dir, "review")?;
        assert_eq!(got.content, "Review this: $ARGUMENTS");
        assert!(skill_dir(&project_dir, "review").is_dir());
        assert!(
            !skill_dir(&project_dir, "review")
                .join("skill.toml")
                .exists()
        );
        assert!(skill_dir(&project_dir, "review").join("SKILL.md").is_file());
        Ok(())
    }

    #[test]
    fn rejects_invalid_skill_dir_without_skill_md() -> Result<()> {
        let tmp = tempdir()?;
        let invalid_dir = skill_dir(tmp.path(), "broken-skill");
        fs::create_dir_all(&invalid_dir)?;
        write_atomic(
            &invalid_dir.join("skill.toml"),
            "id = \"broken-skill\"\nname = \"Broken Skill\"\n",
        )?;
        write_atomic(&invalid_dir.join("index.md"), "broken body")?;
        let err = get_skill(tmp.path(), "broken-skill").expect_err("expected parse failure");
        assert!(err.to_string().contains("Missing SKILL.md"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn find_skill_source_dir_accepts_symlinked_skill_md() -> Result<()> {
        let tmp = tempdir()?;
        let source_file = tmp.path().join("source-file.md");
        write_atomic(
            &source_file,
            r#"---
name: linked-skill
description: Linked skill
---
"#,
        )?;

        let symlink_parent = tmp.path().join("workspace/.claude/skills/linked-skill");
        fs::create_dir_all(&symlink_parent)?;
        symlink_file(&source_file, symlink_parent.join("SKILL.md"))?;

        let discovered = find_skill_source_dir(tmp.path(), "linked-skill")?;
        assert_eq!(discovered, symlink_parent);
        Ok(())
    }

    #[test]
    fn parses_agentskills_spec_format() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "rust-runtime");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: rust-runtime
stable-id: rust-runtime
description: Rust runtime and service layer implementation guidance.
tags: [runtime]
authors: [ship]
metadata:
  display_name: Rust Runtime
  source: custom
---

Use this skill when changing runtime internals.
"#,
        )?;

        let skill = get_skill(tmp.path(), "rust-runtime")?;
        assert_eq!(skill.id, "rust-runtime");
        assert_eq!(skill.name, "Rust Runtime");
        assert_eq!(
            skill.description.as_deref(),
            Some("Rust runtime and service layer implementation guidance.")
        );
        assert!(skill.content.contains("runtime internals"));
        Ok(())
    }

    #[test]
    fn rejects_agentskills_name_directory_mismatch() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "expected-id");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: wrong-id
stable-id: wrong-id
description: Should fail because skill id does not match folder.
tags: [test]
authors: [test]
---

Bad skill.
"#,
        )?;

        let err = get_skill(tmp.path(), "expected-id").expect_err("expected parse failure");
        assert!(err.to_string().contains("must match directory id"));
        Ok(())
    }

    #[test]
    fn rejects_agentskills_invalid_name_format() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "Bad_Name");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: Bad_Name
stable-id: bad-name
description: Invalid name format.
tags: [test]
authors: [test]
---

Bad.
"#,
        )?;

        let err = get_skill(tmp.path(), "Bad_Name").expect_err("expected parse failure");
        let msg = err.to_string();
        assert!(
            msg.contains("/name") && msg.contains("Invalid SKILL.md frontmatter"),
            "expected schema validation failure naming /name, got: {msg}"
        );
        Ok(())
    }

    #[test]
    fn rejects_agentskills_empty_description() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "empty-description");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: empty-description
stable-id: empty-description
description: "   "
tags: [test]
authors: [test]
---

Body.
"#,
        )?;

        let err = get_skill(tmp.path(), "empty-description").expect_err("expected parse failure");
        assert!(
            err.to_string()
                .contains("description must be 1..=1024 chars")
        );
        Ok(())
    }

    #[test]
    fn list_returns_all_skills() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::testutil::init_project(tmp.path())?;
        create_skill(&project_dir, "a", "A", "content a")?;
        create_skill(&project_dir, "b", "B", "content b")?;
        let skills = list_skills(&project_dir)?;
        assert!(skills.iter().any(|s| s.id == "a"));
        assert!(skills.iter().any(|s| s.id == "b"));
        Ok(())
    }

    #[test]
    fn list_empty_dir_returns_empty() -> Result<()> {
        let tmp = tempdir()?;
        assert!(list_skills(tmp.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn update_skill_persists() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::testutil::init_project(tmp.path())?;
        create_skill(&project_dir, "s", "Old", "old")?;
        update_skill(&project_dir, "s", Some("New"), Some("new $ARGUMENTS"))?;
        let reloaded = get_skill(&project_dir, "s")?;
        assert_eq!(reloaded.name, "New");
        assert_eq!(reloaded.content, "new $ARGUMENTS");
        Ok(())
    }

    #[test]
    fn delete_removes_skill() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::testutil::init_project(tmp.path())?;
        create_skill(&project_dir, "gone", "Gone", "x")?;
        delete_skill(&project_dir, "gone")?;
        assert!(get_skill(&project_dir, "gone").is_err());
        Ok(())
    }

    #[test]
    fn duplicate_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::testutil::init_project(tmp.path())?;
        create_skill(&project_dir, "dup", "Dup", "x")?;
        assert!(create_skill(&project_dir, "dup", "Dup2", "y").is_err());
        Ok(())
    }

    #[test]
    fn install_skill_accepts_skills_command_source() -> Result<()> {
        validate_skill_install_request(
            "npx -y skills add vercel-labs/agent-skills@vercel-react-best-practices",
            "vercel-react-best-practices",
        )?;
        Ok(())
    }

    #[test]
    fn install_skill_accepts_skills_command_without_explicit_id() -> Result<()> {
        validate_skill_install_request(
            "npx -y skills add vercel-labs/agent-skills@vercel-react-best-practices",
            "",
        )?;
        Ok(())
    }

    #[test]
    fn install_skill_accepts_skill_id_source() -> Result<()> {
        validate_skill_install_request(
            "vercel-labs/agent-skills@vercel-react-best-practices",
            "vercel-react-best-practices",
        )?;
        Ok(())
    }

    #[test]
    fn parse_skills_install_source_extracts_package_and_skill_from_command() -> Result<()> {
        let (package, skill) = parse_skills_install_source(
            "npx -y skills add vercel-labs/agent-skills --skill web-design-guidelines --yes",
        )?;
        assert_eq!(package, "vercel-labs/agent-skills");
        assert_eq!(skill.as_deref(), Some("web-design-guidelines"));
        Ok(())
    }

    #[test]
    fn parse_skills_install_source_extracts_shorthand_skill() -> Result<()> {
        let (package, skill) =
            parse_skills_install_source("vercel-labs/agent-skills@web-design-guidelines")?;
        assert_eq!(package, "vercel-labs/agent-skills");
        assert_eq!(skill.as_deref(), Some("web-design-guidelines"));
        Ok(())
    }

    #[test]
    fn parse_skills_install_source_keeps_plain_source() -> Result<()> {
        let (package, skill) = parse_skills_install_source("vercel-labs/agent-skills")?;
        assert_eq!(package, "vercel-labs/agent-skills");
        assert_eq!(skill, None);
        Ok(())
    }

    #[test]
    fn install_skill_rejects_non_skills_command_source() {
        let err =
            validate_skill_install_request("curl https://example.com/skill.sh", "skill-creator")
                .expect_err("non-skills command should be rejected");
        assert!(err.to_string().contains("Unsupported source"));
    }

    #[test]
    fn install_skill_rejects_shorthand_source_without_id() {
        let err = validate_skill_install_request("vercel-react-best-practices", "")
            .expect_err("shorthand source without id should be rejected");
        assert!(err.to_string().contains("Skill ID is required"));
    }

    #[test]
    fn install_skill_rejects_git_url_source() {
        let err = validate_skill_install_request(
            "https://github.com/example/skills.git",
            "skill-creator",
        )
        .expect_err("git URL sources should be rejected");
        assert!(
            err.to_string()
                .contains("Git URL sources are not supported")
        );
    }

    #[test]
    fn skills_command_detector_requires_add_and_skills_tokens() {
        assert!(looks_like_skills_cli_command(
            "npx -y skills add vercel-labs/agent-skills@vercel-react-best-practices"
        ));
        assert!(!looks_like_skills_cli_command("npx -y skills list"));
        assert!(!looks_like_skills_cli_command("echo skills add"));
    }

    // ── canonical SKILL.md schema enforcement ───────────────────────────────

    /// A SKILL.md that violates the canonical schema must be rejected with a
    /// clear error that names both the skill directory and the offending field
    /// path. Acceptance criterion #3 of the SKILL.md canonicalization job.
    #[test]
    fn rejects_malformed_skill_md_with_named_field() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "broken-fields");
        fs::create_dir_all(&dir)?;
        // Missing required: stable-id, tags, authors. Carries forbidden `packs`.
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: broken-fields
description: Missing required fields and includes a forbidden `packs` key.
packs: ["unsupported"]
---

body
"#,
        )?;
        let err = get_skill(tmp.path(), "broken-fields").expect_err("expected schema failure");
        let msg = err.to_string();
        assert!(
            msg.contains("broken-fields"),
            "error must reference the skill directory: {msg}"
        );
        assert!(
            msg.contains("Invalid SKILL.md frontmatter"),
            "error must signal schema failure: {msg}"
        );
        // jsonschema reports missing-required at the root path.
        assert!(
            msg.contains("stable-id") || msg.contains("required"),
            "error must name a missing required field: {msg}"
        );
        // additionalProperties violation surfaces the forbidden field.
        assert!(
            msg.contains("packs") || msg.contains("Additional"),
            "error must mention the forbidden `packs` field: {msg}"
        );
        Ok(())
    }

    /// Every committed `docs/ship-*/SKILL.md` must round-trip through the
    /// runtime loader (and therefore through the canonical schema).
    /// Acceptance criterion #2.
    #[test]
    fn every_committed_docs_skill_validates() {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists() && p.join("docs").is_dir())
            .expect("workspace root not found");
        let docs = workspace_root.join("docs");
        let mut failures = Vec::new();
        let mut checked = 0usize;
        for entry in fs::read_dir(&docs).expect("read docs/") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.starts_with("ship-") {
                continue;
            }
            if !path.join("SKILL.md").is_file() {
                continue;
            }
            checked += 1;
            if let Err(err) = super::storage::parse_skill_spec(&path) {
                failures.push(format!("{name}: {err}"));
            }
        }
        // Post doc-skill cull: only the conceptual docs-skills keep a SKILL.md
        // (concepts/architecture/contributing/agents/skills/adrs); the
        // code-mirroring ones were retired (their reference pages remain under
        // references/docs/). Floor guards against the corpus vanishing entirely.
        assert!(
            checked >= 5,
            "sanity: expected the conceptual ship-* SKILL.md corpus, found {checked}"
        );
        assert!(
            failures.is_empty(),
            "{} committed docs/ship-*/SKILL.md files failed schema validation:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// `packs:` is reserved — schema must forbid it.
    #[test]
    fn schema_forbids_packs_field() -> Result<()> {
        let tmp = tempdir()?;
        let dir = skill_dir(tmp.path(), "uses-packs");
        fs::create_dir_all(&dir)?;
        write_atomic(
            &dir.join("SKILL.md"),
            r#"---
name: uses-packs
stable-id: uses-packs
description: Demonstrates the reserved-`packs` rule.
tags: [test]
authors: [test]
packs: [some-pack]
---

body
"#,
        )?;
        let err = get_skill(tmp.path(), "uses-packs").expect_err("packs must be rejected");
        assert!(
            err.to_string().contains("Invalid SKILL.md frontmatter"),
            "got: {err}"
        );
        Ok(())
    }
