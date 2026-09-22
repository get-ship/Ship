use std::fs;

use ship_store::validate_publishable_stack;

fn fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }
    dir
}

fn assert_refused(files: &[(&str, &str)], expected: &str) {
    let dir = fixture(files);
    let error = validate_publishable_stack(dir.path()).expect_err("fixture must be refused");
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error:#}"
    );
}

#[test]
fn clean_stack_passes_the_publish_valve() {
    let dir = fixture(&[
        ("stack.config.ts", "export default defineConfig({});\n"),
        (
            "agents/reviewer.ts",
            "defineAgent({ rules: { inline: 'Review carefully.' } });\n",
        ),
        ("agents/reviewer/hooks/stop.sh", "exit 0\n"),
        ("skills/review/SKILL.md", "# Review\n"),
    ]);

    validate_publishable_stack(dir.path()).expect("clean stack passes");
}

#[test]
fn missing_stack_marker_is_refused() {
    assert_refused(
        &[("skills/review/SKILL.md", "# Review\n")],
        "no stack.config.ts found",
    );
}

#[test]
fn space_scoped_artifact_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("space.config.ts", "export default {};\n"),
        ],
        "space-scoped",
    );
}

#[test]
fn hooks_outside_an_agent_are_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("hooks/stop.sh", "exit 0\n"),
        ],
        "agent-scoped",
    );
}

#[test]
fn triggers_are_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("triggers/on-push.ts", "defineTrigger({});\n"),
        ],
        "triggers are not publishable",
    );
}

#[test]
fn custom_event_vocabulary_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            (
                "pipelines/on-review.jsonc",
                r#"{"trigger":{"event":"vendor.reviewed"}}"#,
            ),
        ],
        "custom event 'vendor.reviewed'",
    );
}

#[test]
fn stack_level_rules_directory_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("rules/style.md", "Be concise.\n"),
        ],
        "rules/ is not publishable",
    );
}

#[test]
fn nested_rules_directory_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("agents/reviewer/rules/style.md", "Be concise.\n"),
        ],
        "rules/ is not publishable",
    );
}

#[test]
fn empty_rules_directory_is_refused() {
    let dir = fixture(&[("stack.config.ts", "export default {};\n")]);
    fs::create_dir_all(dir.path().join("rules")).unwrap();

    let error = validate_publishable_stack(dir.path()).expect_err("rules dir must be refused");
    assert!(
        error.to_string().contains("rules/ is not publishable"),
        "{error:#}"
    );
}

#[test]
fn registered_platform_event_is_allowed() {
    let dir = fixture(&[
        ("stack.config.ts", "export default {};\n"),
        (
            "pipelines/on-setting.jsonc",
            r#"{"trigger":{"event":"settings.changed"}}"#,
        ),
    ]);

    validate_publishable_stack(dir.path()).expect("platform vocabulary passes");
}

#[test]
fn moved_define_trigger_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("misc/automation.ts", "defineTrigger({ id: 'hidden' });\n"),
        ],
        "defineTrigger",
    );
}

#[test]
fn legacy_space_policy_files_are_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            (".ship/permissions.jsonc", "{}\n"),
        ],
        "space-scoped",
    );
}

#[test]
fn root_hooks_marker_is_refused_even_when_empty() {
    let dir = fixture(&[("stack.config.ts", "export default {};\n")]);
    fs::create_dir_all(dir.path().join("hooks")).unwrap();

    let error = validate_publishable_stack(dir.path()).expect_err("hooks dir must be refused");
    assert!(error.to_string().contains("agent-scoped"), "{error:#}");
}

#[test]
fn root_triggers_marker_is_refused_even_when_empty() {
    let dir = fixture(&[("stack.config.ts", "export default {};\n")]);
    fs::create_dir_all(dir.path().join("triggers")).unwrap();

    let error = validate_publishable_stack(dir.path()).expect_err("triggers dir must be refused");
    assert!(
        error.to_string().contains("triggers are not publishable"),
        "{error:#}"
    );
}

#[test]
fn ship_state_is_refused() {
    assert_refused(
        &[
            ("stack.config.ts", "export default {};\n"),
            ("ship.state", "{}\n"),
        ],
        "space-scoped",
    );
}
