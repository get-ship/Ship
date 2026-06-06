//! AgentSpec loader — project the `.ship/agents/<id>.jsonc` superset down to
//! the native four-facet `AgentSpec`.
//!
//! The superset shape carries harness/compiler fields (permissions, plugins,
//! hooks, apps, harness_settings, mcp, …) that Ship Code consumes but the
//! native runtime does not. This loader's only job is the projection: pick
//! the core subset, leave the rest on the floor, never invent a grant.
//!
//! Identity       ← `agent` block
//! Disposition    ← `rules.inline` + best-effort model resolution
//! Capability     ← `capability.grants` + `skills.refs` (NOT from `permissions`)
//! Memory         ← `memory.enabled`
//!
//! Files may be JSONC (json5) — `//` and `/* */` comments and trailing commas
//! are tolerated. We parse with the `json5` crate, then deserialize into a
//! permissive private shim, then project.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::spec::AgentSpec;

/// One agent definition as it lives in the file. Every field is optional —
/// missing pieces fall back to the native default. Unknown top-level fields
/// are ignored, so a single loader handles every superset version.
///
/// NOTE: `permissions` / `plugins` / `hooks` / `apps` / `mcp` are intentionally
/// NOT modeled — they are harness-only and must never influence the native
/// runtime. They are still accepted in the file (serde ignores unknown fields
/// on a `#[derive(Deserialize)]` struct by default).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SupersetDoc {
    agent: Option<IdentityBlock>,
    model: Option<String>,
    rules: Option<RulesBlock>,
    capability: Option<CapabilityBlock>,
    skills: Option<SkillsBlock>,
    memory: Option<MemoryBlock>,
    harness_settings: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct IdentityBlock {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RulesBlock {
    inline: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CapabilityBlock {
    grants: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SkillsBlock {
    refs: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct MemoryBlock {
    enabled: Option<bool>,
}

/// Parse a single agent definition (JSON or JSONC) and project it to the
/// native four-facet `AgentSpec`. See module docs for the projection rules.
pub fn parse_agent_doc(doc: &str) -> anyhow::Result<AgentSpec> {
    let parsed: SupersetDoc = json5::from_str(doc)?;
    Ok(project(parsed))
}

/// Read `<agents_dir>/<id>.jsonc` (with `.json` fallback) and project to native.
pub fn load_agent_spec(agents_dir: &Path, id: &str) -> anyhow::Result<AgentSpec> {
    for ext in ["jsonc", "json"] {
        let path: PathBuf = agents_dir.join(format!("{id}.{ext}"));
        match std::fs::read_to_string(&path) {
            Ok(text) => return parse_agent_doc(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(anyhow::Error::new(e)
                    .context(format!("read {}", path.display())))
            }
        }
    }
    anyhow::bail!(
        "no agent file found for id={id} in {} (tried .jsonc, .json)",
        agents_dir.display()
    )
}

fn project(doc: SupersetDoc) -> AgentSpec {
    let identity = {
        let a = doc.agent.unwrap_or_default();
        let id = a.id.unwrap_or_default();
        let name = a.name.unwrap_or_else(|| id.clone());
        let version = a.version.unwrap_or_else(crate::spec::default_version);
        let description = a.description.unwrap_or_default();
        let tags = a.tags.unwrap_or_default();
        crate::spec::Identity {
            id,
            name,
            version,
            description,
            tags,
        }
    };

    let rules = doc
        .rules
        .and_then(|r| r.inline)
        .unwrap_or_default();

    let model = resolve_model(doc.model.as_deref(), doc.harness_settings.as_ref());

    let disposition = crate::spec::Disposition {
        rules,
        model,
        temperature: None,
        max_turns: None,
    };

    // Grants come ONLY from the native `capability.grants` block. `permissions`
    // (ToolPatterns) are harness-only and must never be reinterpreted as grants.
    let grants = doc
        .capability
        .and_then(|c| c.grants)
        .unwrap_or_default();

    let skills = doc.skills.and_then(|s| s.refs).unwrap_or_default();

    let capability = crate::spec::Capability { grants, skills };

    let memory = crate::spec::Memory {
        enabled: doc.memory.and_then(|m| m.enabled).unwrap_or(false),
    };

    AgentSpec {
        identity,
        disposition,
        capability,
        memory,
    }
}

/// Resolve the model: top-level `model` wins, else the first harness model
/// under `harness_settings` in (claude, codex, gemini, opencode) order, else
/// empty. Returns the model string only; vendor-specific fields under
/// `harness_settings` are otherwise ignored.
fn resolve_model(top: Option<&str>, harness: Option<&serde_json::Value>) -> String {
    if let Some(m) = top {
        if !m.is_empty() {
            return m.to_string();
        }
    }
    let Some(h) = harness else { return String::new() };
    let Some(obj) = h.as_object() else { return String::new() };
    for key in ["claude", "codex", "gemini", "opencode"] {
        if let Some(v) = obj.get(key) {
            if let Some(m) = v.get("model").and_then(|m| m.as_str()) {
                if !m.is_empty() {
                    return m.to_string();
                }
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- helpers ---------------------------------------------------------

    fn parse(doc: &str) -> AgentSpec {
        parse_agent_doc(doc).expect("parse")
    }

    // ----- identity & disposition (representative file) ---------------------

    #[test]
    fn representative_file_projects_identity_and_disposition() {
        // Resembles a real .ship/agents file: agent block, rules.inline,
        // harness_settings, permissions (must be ignored), plugins, hooks.
        let doc = r#"
        {
            "agent": {
                "id": "shipwright",
                "name": "Shipwright",
                "version": "1.4.0",
                "description": "Builds Ship features end-to-end.",
                "tags": ["builder", "ship"],
                "icon": "ship.svg",                  // ignored (Identity)
                "role": "engineer",                  // ignored (deleted from native)
                "tools_preset": "core",              // ignored
                "active_tools": ["Read", "Write"]    // ignored
            },
            "rules": { "inline": "be precise; never self-elevate." },
            "harness_settings": {
                "claude": { "model": "claude-opus-4-7", "theme": "dark" }
            },
            "permissions": {                         // ignored entirely
                "tools_allow": ["Read", "Bash(*)"],
                "tools_deny": ["WebFetch"]
            },
            "plugins": ["some-harness-plugin"],      // ignored
            "hooks": { "PreToolUse": [] },           // ignored
            "apps": { "vscode": { "enabled": true } } // ignored
        }
        "#;

        let s = parse(doc);
        assert_eq!(s.identity.id, "shipwright");
        assert_eq!(s.identity.name, "Shipwright");
        assert_eq!(s.identity.version, "1.4.0");
        assert_eq!(s.identity.description, "Builds Ship features end-to-end.");
        assert_eq!(s.identity.tags, vec!["builder", "ship"]);

        assert_eq!(s.disposition.rules, "be precise; never self-elevate.");
        assert_eq!(s.disposition.model, "claude-opus-4-7");
    }

    // ----- capability.grants: native is the only source --------------------

    #[test]
    fn capability_grants_native_array_is_kept() {
        let doc = r#"
        { "agent": { "id": "x" },
          "capability": { "grants": ["collections", "runtime"] } }
        "#;
        let s = parse(doc);
        assert_eq!(s.capability.grants, vec!["collections", "runtime"]);
    }

    #[test]
    fn permissions_toolpatterns_do_not_become_grants() {
        // permissions with tools_allow but NO native capability.grants →
        // empty grants. ToolPatterns are harness-only.
        let doc = r#"
        { "agent": { "id": "x" },
          "permissions": { "tools_allow": ["Read", "Bash(*)"],
                           "tools_deny": ["WebFetch"] } }
        "#;
        let s = parse(doc);
        assert!(
            s.capability.grants.is_empty(),
            "ToolPatterns must not become grants, got {:?}",
            s.capability.grants
        );
    }

    #[test]
    fn missing_capability_yields_empty_grants_cleanly() {
        let doc = r#"{ "agent": { "id": "bare" } }"#;
        let s = parse(doc);
        assert!(s.capability.grants.is_empty());
        assert!(s.capability.skills.is_empty());
        assert!(!s.memory.enabled);
    }

    // ----- skills ----------------------------------------------------------

    #[test]
    fn skills_refs_become_capability_skills() {
        let doc = r#"
        { "agent": { "id": "x" },
          "skills": { "refs": ["tdd", "web-qa", "ship-canvas"] } }
        "#;
        let s = parse(doc);
        assert_eq!(s.capability.skills, vec!["tdd", "web-qa", "ship-canvas"]);
    }

    // ----- model resolution -----------------------------------------------

    #[test]
    fn top_level_model_wins_over_harness() {
        let doc = r#"
        { "agent": { "id": "x" },
          "model": "gpt-5.2",
          "harness_settings": { "claude": { "model": "claude-opus-4-7" } } }
        "#;
        let s = parse(doc);
        assert_eq!(s.disposition.model, "gpt-5.2");
    }

    #[test]
    fn harness_model_picks_in_declared_order() {
        // opencode has a model; claude does not → opencode wins because it's
        // listed later in the order? No: order is (claude, codex, gemini,
        // opencode). claude missing → falls through to opencode.
        let doc = r#"
        { "agent": { "id": "x" },
          "harness_settings": {
              "opencode": { "model": "opencode-1" },
              "codex":    { "model": "codex-1" }
          } }
        "#;
        let s = parse(doc);
        assert_eq!(s.disposition.model, "codex-1");
    }

    #[test]
    fn missing_model_everywhere_yields_empty() {
        let doc = r#"
        { "agent": { "id": "x" },
          "harness_settings": { "claude": { "theme": "dark" } } }
        "#;
        let s = parse(doc);
        assert_eq!(s.disposition.model, "");
    }

    // ----- unknown / harness fields ignored -------------------------------

    #[test]
    fn unknown_top_level_fields_are_ignored() {
        let doc = r#"
        { "agent": { "id": "x" },
          "future_field": 42,
          "another": { "deep": [1,2,3] } }
        "#;
        let s = parse(doc);
        assert_eq!(s.identity.id, "x");
    }

    #[test]
    fn harness_settings_extras_are_ignored() {
        // theme, sandbox, anything under harness_settings besides `model` →
        // dropped, no error, no spillover.
        let doc = r#"
        { "agent": { "id": "x" },
          "harness_settings": {
              "claude": { "model": "claude-opus-4-7", "theme": "dark",
                          "sandbox": { "image": "firecracker" } }
          } }
        "#;
        let s = parse(doc);
        assert_eq!(s.disposition.model, "claude-opus-4-7");
        // identity untouched, no panic
        assert_eq!(s.identity.id, "x");
    }

    // ----- memory ----------------------------------------------------------

    #[test]
    fn memory_enabled_true_round_trips() {
        let doc = r#"
        { "agent": { "id": "x" },
          "memory": { "enabled": true } }
        "#;
        let s = parse(doc);
        assert!(s.memory.enabled);
    }

    // ----- jsonc tolerance ------------------------------------------------

    #[test]
    fn jsonc_with_comments_and_trailing_commas_parses() {
        let doc = r#"
        // line comment at top
        {
            /* block comment */
            "agent": {
                "id": "commented",  // trailing inline
                "name": "Commented",
                "tags": ["a", "b",],  // trailing comma in array
            },  // trailing comma in object
        }
        "#;
        let s = parse(doc);
        assert_eq!(s.identity.id, "commented");
        assert_eq!(s.identity.name, "Commented");
        assert_eq!(s.identity.tags, vec!["a", "b"]);
    }

    // ----- load_agent_spec: file lookup -----------------------------------

    #[test]
    fn load_agent_spec_reads_jsonc_from_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("foo.jsonc");
        std::fs::write(
            &path,
            r#"
            { "agent": { "id": "foo", "name": "Foo" },
              "rules": { "inline": "hi" },
              "capability": { "grants": ["collections"] } }
            "#,
        )
        .expect("write");
        let s = load_agent_spec(dir.path(), "foo").expect("load");
        assert_eq!(s.identity.id, "foo");
        assert_eq!(s.disposition.rules, "hi");
        assert_eq!(s.capability.grants, vec!["collections"]);
    }

    #[test]
    fn load_agent_spec_falls_back_to_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("bar.json"),
            r#"{ "agent": { "id": "bar" } }"#,
        )
        .expect("write");
        let s = load_agent_spec(dir.path(), "bar").expect("load");
        assert_eq!(s.identity.id, "bar");
    }

    #[test]
    fn load_agent_spec_errors_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = load_agent_spec(dir.path(), "nope").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nope"), "msg should mention id: {msg}");
    }
}
