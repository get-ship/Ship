use super::*;
use super::install::{
    find_skill_source_dir, looks_like_skills_cli_command, parse_skills_install_source,
    validate_skill_install_request,
};
use super::storage::skill_dir;
use crate::fs_util::write_atomic;
#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_file;
use tempfile::tempdir;

    #[test]
    fn create_and_get_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;
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
description: Rust runtime and service layer implementation guidance.
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
description: Should fail because skill id does not match folder.
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
description: Invalid name format.
---

Bad.
"#,
        )?;

        let err = get_skill(tmp.path(), "Bad_Name").expect_err("expected parse failure");
        assert!(
            err.to_string()
                .contains("must match ^[a-z0-9]+(-[a-z0-9]+)*$")
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
description: "   "
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
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;
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
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;
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
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        create_skill(&project_dir, "gone", "Gone", "x")?;
        delete_skill(&project_dir, "gone")?;
        assert!(get_skill(&project_dir, "gone").is_err());
        Ok(())
    }

    #[test]
    fn duplicate_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        create_skill(&project_dir, "dup", "Dup", "x")?;
        assert!(create_skill(&project_dir, "dup", "Dup2", "y").is_err());
        Ok(())
    }

    #[test]
    fn migrates_legacy_global_project_skills_into_repo_local_storage() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let legacy_skill_dir =
            crate::project::legacy_project_skills_dir(&project_dir).join("legacy-skill");
        fs::create_dir_all(&legacy_skill_dir)?;
        write_atomic(
            &legacy_skill_dir.join("SKILL.md"),
            r#"---
name: legacy-skill
description: Legacy skill migrated into repo-local project storage.
metadata:
  display_name: Legacy Skill
  source: imported
---

Legacy body.
"#,
        )?;

        let skills = list_skills(&project_dir)?;
        assert!(skills.iter().any(|skill| skill.id == "legacy-skill"));
        assert!(
            crate::project::skills_dir(&project_dir)
                .join("legacy-skill")
                .join("SKILL.md")
                .is_file()
        );
        assert!(
            !crate::project::legacy_project_skills_dir(&project_dir)
                .join("legacy-skill")
                .exists()
        );
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
