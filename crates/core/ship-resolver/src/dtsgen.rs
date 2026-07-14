//! Space-scoped `.d.ts` emission for editor autocomplete inside a Ship space.
//!
//! The AgentCatalogs RPC powers STUDIO dropdowns (a UI calling the daemon
//! to populate a picker). It does NOT power EDITOR autocomplete inside
//! `stack.config.ts` / `agents/*.ts` — the TypeScript language server reads
//! `.d.ts` declarations from disk, it cannot call an RPC. To make
//! `defineAgent({ skills: [...] })` autocomplete the space's available
//! skills, the resolver EMITS a space-scoped `.d.ts` of noun unions that
//! the space's tsconfig picks up.
//!
//! ## Pattern parity with floki
//!
//! Floki already generates `.d.ts` for a different surface — its
//! `crates/core/ship-platform/crates/floki/src/dtsgen.rs` emits the
//! agent's RUNTIME capability API (`collections.read`, `lsp.hover`, gated
//! by grants), sourced from the `sdkgen::surface(grants)` registry. The
//! authoring `.d.ts` is a parallel emitter over the resolver's composed
//! catalog instead of the grant set: same pattern (sorted unions,
//! deterministic output, `never` for empty noun), different consumer
//! surface (authoring autocomplete vs runtime call typing).
//!
//! ## What gets emitted
//!
//! One union per composed noun the authoring surface consumes — the six
//! that `defineAgent({ skills, mcp, rules, … })` and the trigger /
//! pipeline / collection declare-forms reference:
//!
//! - `SpaceSkill`     — composed `library.skills` (key: `Skill.id`)
//! - `SpaceMcp`       — composed `library.mcp_servers` (key: `McpServerConfig.id`)
//! - `SpaceAgent`     — composed `library.agents` (key: `ProfileMeta.id`)
//! - `SpacePipeline`  — composed `library.pipelines` (key: `PipelineDef.name`)
//! - `SpaceCollection`— composed `library.collections` (key: `CollectionDef.id`)
//! - `SpaceTrigger`   — composed `library.triggers` (key: `TriggerDef.id`)
//!
//! Rules are intentionally NOT a union: post rules-format-cut, every rule
//! body lives inline on the agent composition as `rules: { inline: "..." }`.
//! The authoring surface for rules is a string literal, not an id picker.
//! Views are also excluded — they key on `(prefix, route)`, not bare id,
//! and are not consumed by `defineAgent`.
//!
//! ## Empty noun → `never`
//!
//! A noun with zero composed entries emits `type SpaceFoo = never;` so
//! downstream generic constraints (`defineAgent<S extends SpaceSkill>`)
//! remain well-typed. An agent in a space with no skills cannot
//! `defineAgent({ skills: [...] })` and have the LSP accept any id.
//!
//! ## Determinism
//!
//! Every union is sorted ascending by id. Same composed library → same
//! `.d.ts`. The descriptor cache key is a content hash, but the in-memory
//! emit is sorted so diff tests stay stable and so the LSP can rely on
//! stable positioning across re-emits.
use stack_model::{ComposedLibrary, types::AgentProfile};
/// Emit the space-scoped authoring `.d.ts` for a composed library.
///
/// Pure over the composed library: no filesystem access, no clock, no
/// process id. Same `ComposedLibrary` → same string. The integration
/// point (compile pipeline / dry-run viewer) is responsible for laying
/// the returned string down at `<space>/.ship/types/space.d.ts`.
pub fn emit_space_dts(composed: &ComposedLibrary) -> String {
    render_dts(&collect_noun_ids(composed))
}

/// The sorted, deduplicated id set for each composed noun the authoring
/// surface exposes. The renderer is pure over this struct, so tests can
/// construct it directly without spinning up a real composed library.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpaceNounIds {
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub agents: Vec<String>,
    pub pipelines: Vec<String>,
    pub collections: Vec<String>,
    pub triggers: Vec<String>,
}

impl SpaceNounIds {
    /// True when every noun has zero entries — useful for tests that
    /// assert the "empty space" case.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
            && self.mcp_servers.is_empty()
            && self.agents.is_empty()
            && self.pipelines.is_empty()
            && self.collections.is_empty()
            && self.triggers.is_empty()
    }
}

/// Walk the composed library's noun collections, dedupe, and sort each.
///
/// The composition law (see `stack_model::compose::compose_space`) has
/// already resolved id collisions: the resulting lists contain the
/// winning id once. We still dedupe defensively in case a future compose
/// change re-introduces a duplicate — duplicates would otherwise leak
/// into the union as `"foo" | "foo"` and break the type.
fn collect_noun_ids(composed: &ComposedLibrary) -> SpaceNounIds {
    let lib = &composed.library;
    SpaceNounIds {
        skills: extract(&lib.skills, |s| s.id.as_str()),
        mcp_servers: extract(&lib.mcp_servers, |m| m.id.as_str()),
        agents: extract(&lib.agents, agent_id),
        pipelines: extract(&lib.pipelines, |p| p.name.as_str()),
        collections: extract(&lib.collections, |c| c.id.as_str()),
        triggers: extract(&lib.triggers, |t| t.id.as_str()),
    }
}

fn agent_id(a: &AgentProfile) -> &str {
    a.agent.id.as_str()
}

/// Dedup (first wins) + sort ascending by `&str`. Stable, allocation-
/// bounded, no `HashSet` allocation required for a test-driven
/// six-element struct.
fn extract<T>(items: &[T], key: impl Fn(&T) -> &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        let k = key(item);
        if !seen.iter().any(|s| s == k) {
            seen.push(k.to_string());
        }
    }
    seen.sort();
    seen
}

/// Render the `.d.ts` from an explicit noun-id set. The renderer is pure
/// over `SpaceNounIds` — fed by `emit_space_dts` in production, fed by
/// tests with hand-built sets that exercise edge cases (empty noun,
/// cross-member composition, special characters in ids).
pub fn render_dts(ids: &SpaceNounIds) -> String {
    let mut out = String::new();
    out.push_str("// AUTO-GENERATED space-scoped authoring typings.\n");
    out.push_str("// Source: ship-resolver compose_space. Do not edit by hand.\n");
    out.push_str("//\n");
    out.push_str(
        "// Sorted, deduplicated, content-hash-stable. Empty noun → `never` so downstream\n",
    );
    out.push_str(
        "// generic constraints (`defineAgent<S extends SpaceSkill>`) remain well-typed.\n",
    );
    out.push_str("declare namespace Ship {\n");
    push_union(&mut out, "SpaceSkill", &ids.skills);
    push_union(&mut out, "SpaceMcp", &ids.mcp_servers);
    push_union(&mut out, "SpaceAgent", &ids.agents);
    push_union(&mut out, "SpacePipeline", &ids.pipelines);
    push_union(&mut out, "SpaceCollection", &ids.collections);
    push_union(&mut out, "SpaceTrigger", &ids.triggers);
    out.push_str("}\n");
    out
}

fn push_union(out: &mut String, name: &str, ids: &[String]) {
    out.push_str(&format!("  export type {name} = "));
    if ids.is_empty() {
        out.push_str("never;\n");
    } else {
        let quoted: Vec<String> = ids.iter().map(|s| format!("\"{s}\"")).collect();
        out.push_str(&quoted.join(" | "));
        out.push_str(";\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_model::compose::compose_space;
    use stack_model::types::{
        CollectionDef, CollectionFieldDef, CollectionFieldType, McpServerConfig, McpServerType,
        PipelineDef, ProfileMeta, Skill, TriggerDef, TriggerEventMatch,
    };
    use stack_model::{ComposedLibrary, ExtSource, ExtensionDescriptor, RouteTree, RollupInputs};

    fn empty_space() -> SpaceNounIds {
        SpaceNounIds::default()
    }

    fn single_each_space() -> SpaceNounIds {
        // Hand-built noun-id set mirroring what a one-skill / one-mcp
        // / one-agent composed space would yield. Bypasses the
        // compose_space law for these specific tests — composition-
        // level tests live further down.
        SpaceNounIds {
            skills: vec!["tdd".to_string()],
            mcp_servers: vec!["github".to_string()],
            agents: vec!["code-reviewer".to_string()],
            pipelines: vec!["nightly".to_string()],
            collections: vec!["bugs".to_string()],
            triggers: vec!["on-pr".to_string()],
        }
    }


    fn make_skill(id: &str) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: Default::default(),
            content: String::new(),
            source: Default::default(),
            vars: Default::default(),
            artifacts: vec![],
            grants: vec![],
            requires: vec![],
        }
    }

    fn make_mcp(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "noop".to_string(),
            args: vec![],
            env: Default::default(),
            scope: "global".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            cursor_env_file: None,
        }
    }

    fn make_collection(id: &str) -> CollectionDef {
        CollectionDef {
            id: id.to_string(),
            name: None,
            fields: vec![CollectionFieldDef {
                name: "title".to_string(),
                field_type: CollectionFieldType::String,
                required: false,
                values: vec![],
                default: None,
                filterable: None,
                sortable: None,
            }],
        }
    }

    fn make_trigger(id: &str) -> TriggerDef {
        TriggerDef {
            id: id.to_string(),
            trigger: TriggerEventMatch {
                event: "noop".to_string(),
            },
            source: None,
            actions: vec![],
        }
    }

    fn make_agent(id: &str) -> AgentProfile {
        AgentProfile {
            agent: ProfileMeta {
                id: id.to_string(),
                name: id.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// `ComposedLibrary` has no `Default` impl (its `library` field is
    /// `ExtensionLibrary::default()`-friendly but the struct itself
    /// doesn't carry one). Tests that need an empty composed library
    /// — most often to assert `emit_space_dts` over a hand-built
    /// `library: ExtensionLibrary` — go through this helper.
    fn empty_composed() -> ComposedLibrary {
        ComposedLibrary {
            library: stack_model::ExtensionLibrary::default(),
            shadows: vec![],
            route_tree: RouteTree::default(),
        }
    }

    // ── render_dts: structure ─────────────────────────────────────────────

    /// Empty noun-id set emits six `never` unions inside a
    /// `declare namespace Ship { … }` block. The renderer must NOT
    /// emit `"never" | "never"` for empty nouns (the "empty union
    /// becomes `never`" rule from the job spec).
    #[test]
    fn empty_space_renders_six_never_unions() {
        let dts = render_dts(&empty_space());
        assert!(dts.starts_with("// AUTO-GENERATED"), "header comment: {dts}");
        assert!(dts.contains("declare namespace Ship {"), "namespace open: {dts}");
        assert!(dts.contains("export type SpaceSkill = never;"), "SpaceSkill never: {dts}");
        assert!(dts.contains("export type SpaceMcp = never;"), "SpaceMcp never: {dts}");
        assert!(dts.contains("export type SpaceAgent = never;"), "SpaceAgent never: {dts}");
        assert!(dts.contains("export type SpacePipeline = never;"), "SpacePipeline never: {dts}");
        assert!(dts.contains("export type SpaceCollection = never;"), "SpaceCollection never: {dts}");
        assert!(dts.contains("export type SpaceTrigger = never;"), "SpaceTrigger never: {dts}");
        // Belt-and-braces: no union literal with an empty string slipped in.
        assert!(
            !dts.contains("\"\" |"),
            "no empty-string union literal: {dts}"
        );
    }

    /// A composed space with one entry per noun emits six single-element
    /// unions. The `never` branch must NOT trigger.
    #[test]
    fn populated_space_renders_one_element_unions() {
        let dts = render_dts(&single_each_space());
        assert!(dts.contains("export type SpaceSkill = \"tdd\";"));
        assert!(dts.contains("export type SpaceMcp = \"github\";"));
        assert!(dts.contains("export type SpaceAgent = \"code-reviewer\";"));
        assert!(dts.contains("export type SpacePipeline = \"nightly\";"));
        assert!(dts.contains("export type SpaceCollection = \"bugs\";"));
        assert!(dts.contains("export type SpaceTrigger = \"on-pr\";"));
        // None of the unions should have fallen through to `never`.
        assert!(!dts.contains("= never;"), "no `never` in populated unions: {dts}");
    }

    /// Multi-entry unions preserve the input order. The renderer is a
    /// pure formatter — sorting is `collect_noun_ids`'s job (driven by
    /// `extract`'s `seen.sort()`). The test passes a pre-sorted set
    /// and verifies the renderer joins in order; the
    /// `collect_noun_ids_sorts_via_emit` test below covers the
    /// sort-via-emit path through `emit_space_dts` end to end.
    #[test]
    fn renderer_preserves_input_order() {
        let sorted = SpaceNounIds {
            skills: vec!["alpha".to_string(), "mike".to_string(), "zzz".to_string()],
            mcp_servers: vec!["alpha".to_string(), "zoo".to_string()],
            agents: vec!["alpha".to_string(), "worker".to_string()],
            pipelines: vec!["a-daily".to_string(), "z-nightly".to_string()],
            collections: vec!["alpha".to_string(), "zeta".to_string()],
            triggers: vec!["a-trigger".to_string(), "z-trigger".to_string()],
        };
        let dts = render_dts(&sorted);
        for (union, expected) in [
            ("SpaceSkill = ", vec!["\"alpha\"", "\"mike\"", "\"zzz\""]),
            ("SpaceMcp = ", vec!["\"alpha\"", "\"zoo\""]),
            ("SpaceAgent = ", vec!["\"alpha\"", "\"worker\""]),
            ("SpacePipeline = ", vec!["\"a-daily\"", "\"z-nightly\""]),
            ("SpaceCollection = ", vec!["\"alpha\"", "\"zeta\""]),
            ("SpaceTrigger = ", vec!["\"a-trigger\"", "\"z-trigger\""]),
        ] {
            let body = dts.split(union).nth(1).expect("union present");
            let union_end = body.find(';').expect("union terminates");
            let body = &body[..union_end];
            for window in expected.windows(2) {
                let a = body.find(&window[0]).expect("first literal present");
                let b = body.find(&window[1]).expect("second literal present");
                assert!(
                    a < b,
                    "{union}must list `{}` before `{}`; got: {body}",
                    window[0],
                    window[1]
                );
            }
        }
    }

    /// End-to-end: a hand-built `ExtensionLibrary` with scrambled skill
    /// ids, fed through `emit_space_dts`, comes out sorted. This is
    /// the acceptance that "deterministic (sorted unions)" from the
    /// job spec — the sort happens inside `collect_noun_ids`/`extract`
    /// even when the source lists are not pre-sorted.
    #[test]
    fn collect_noun_ids_sorts_via_emit() {
        let composed = ComposedLibrary {
            library: stack_model::ExtensionLibrary {
                skills: vec![make_skill("zzz"), make_skill("alpha"), make_skill("mike")],
                mcp_servers: vec![make_mcp("zoo"), make_mcp("alpha")],
                agents: vec![make_agent("worker"), make_agent("alpha")],
                ..Default::default()
            },
            shadows: vec![],
            route_tree: RouteTree::default(),
        };
        let dts = emit_space_dts(&composed);
        // All three skill ids present, in alphabetical order.
        let skill_line = dts
            .lines()
            .find(|l| l.contains("SpaceSkill ="))
            .expect("SpaceSkill union present");
        let a = skill_line.find("\"alpha\"").expect("alpha present");
        let m = skill_line.find("\"mike\"").expect("mike present");
        let z = skill_line.find("\"zzz\"").expect("zzz present");
        assert!(a < m && m < z, "expected alpha < mike < zzz; got: {skill_line}");
    }
    // ── emit_space_dts: composition-level ────────────────────────────────

    /// `emit_space_dts` over an empty `ComposedLibrary` is the
    /// structurally identical six-`never` output — but the
    /// `ComposedLibrary` shape (not the `SpaceNounIds` struct) drives
    /// the emit. Confirms the public fn wires the composition-level
    /// shape into the renderer.
    #[test]
    fn emit_over_empty_composed_library_renders_never() {
        let composed = empty_composed();
        let dts = emit_space_dts(&composed);
        assert!(dts.contains("export type SpaceSkill = never;"));
        assert!(dts.contains("export type SpaceTrigger = never;"));
    }

    /// Cross-member composition: a skill defined in a UserGlobal
    /// (operator-enabled library) extension AND one in the Space tier
    /// both surface in the emitted union. The compose law resolves
    /// the multi-tier descriptor into a single composed library; the
    /// dtsgen reads that composed set, not the per-tier inputs. This
    /// is the "cross-member composition (a skill from another
    /// registered stack appears)" acceptance from the job spec.
    ///
    /// Note: `compose_space` only composes the noun set the format
    /// law covers (agents / agent_profiles / skills / rules /
    /// mcp_servers / views) — pipelines / collections / triggers are
    /// passed through the space tier only. The cross-member test
    /// therefore exercises skills, agents, mcp_servers; the
    /// remaining-noun unions (SpaceCollection, SpaceTrigger,
    /// SpacePipeline) get their own direct-composed-library tests
    /// below where the space-tier descriptor carries the entries.
    #[test]
    fn cross_member_composition_surfaces_skills_agents_mcp() {
        // Tier 1: Space defines one skill, one mcp, one agent.
        let space = ExtensionDescriptor {
            source: ExtSource::Space,
            space_root: "/space".into(),
            descriptor: stack_model::ExtensionLibrary {
                skills: vec![make_skill("space-skill")],
                mcp_servers: vec![make_mcp("space-mcp")],
                agents: vec![make_agent("space-agent")],
                ..Default::default()
            },
            content_hash: "space-hash".to_string(),
        };
        // Tier 2: UserGlobal defines one skill.
        let library_tier = ExtensionDescriptor {
            source: ExtSource::UserGlobal,
            space_root: "/library".into(),
            descriptor: stack_model::ExtensionLibrary {
                skills: vec![make_skill("library-skill")],
                ..Default::default()
            },
            content_hash: "library-hash".to_string(),
        };
        // Tier 3: Cache defines one skill, one mcp.
        let cache_tier = ExtensionDescriptor {
            source: ExtSource::Cache,
            space_root: "/cache".into(),
            descriptor: stack_model::ExtensionLibrary {
                skills: vec![make_skill("cache-skill")],
                mcp_servers: vec![make_mcp("cache-mcp")],
                ..Default::default()
            },
            content_hash: "cache-hash".to_string(),
        };
        let inputs = RollupInputs {
            extensions: vec![space, library_tier, cache_tier],
            precedence: RollupInputs::default_precedence(),
        };
        let composed = compose_space(&inputs).expect("compose ok");
        let dts = emit_space_dts(&composed);

        // Skills: one from each of Space, UserGlobal, Cache. All surface.
        assert!(
            dts.contains(
                "export type SpaceSkill = \"cache-skill\" | \"library-skill\" | \"space-skill\";",
            ),
            "got:\n{dts}"
        );
        // MCPs: one from Space, one from Cache.
        assert!(dts.contains("export type SpaceMcp = \"cache-mcp\" | \"space-mcp\";"));
        // Agent: only the Space tier defined one.
        assert!(dts.contains("export type SpaceAgent = \"space-agent\";"));
        // Nouns the law does not compose: `never`.
        assert!(dts.contains("export type SpaceCollection = never;"));
        assert!(dts.contains("export type SpaceTrigger = never;"));
        assert!(dts.contains("export type SpacePipeline = never;"));
    }

    /// Pipelines, collections, and triggers live on the space tier
    /// (the compose law does not merge them cross-tier — see the
    /// cross-member test's note). When the space tier defines one,
    /// the dtsgen surfaces it. This is the direct-equivalent
    /// acceptance for the three nouns `compose_space` does NOT
    /// cross-compose.
    #[test]
    fn space_tier_pipelines_collections_triggers_surface() {
        let space = ExtensionDescriptor {
            source: ExtSource::Space,
            space_root: "/space".into(),
            descriptor: stack_model::ExtensionLibrary {
                pipelines: vec![PipelineDef {
                    name: "nightly".to_string(),
                    steps: vec![],
                    auto_merge: false,
                }],
                collections: vec![make_collection("bugs")],
                triggers: vec![make_trigger("on-pr")],
                ..Default::default()
            },
            content_hash: "space-hash".to_string(),
        };
        let inputs = RollupInputs {
            extensions: vec![space],
            precedence: RollupInputs::default_precedence(),
        };
        let composed = compose_space(&inputs).expect("compose ok");
        let dts = emit_space_dts(&composed);
        assert!(dts.contains("export type SpacePipeline = \"nightly\";"));
        assert!(dts.contains("export type SpaceCollection = \"bugs\";"));
        assert!(dts.contains("export type SpaceTrigger = \"on-pr\";"));
    }

    /// Id-collision resolution: the higher-rank tier wins. A Space
    /// skill and a Cache skill with the same id resolve to the Space
    /// id; the dtsgen output contains the id exactly once (no
    /// `"x" | "x"` duplicate). This is the dedup acceptance the
    /// renderer doc claims it provides.
    #[test]
    fn id_collision_dedupes_in_emitted_union() {
        // Two tiers, same skill id, different ranks. Space (1000) wins
        // over Cache (0) — but the result is a single union element
        // either way; the assertion is "exactly one occurrence".
        let space = ExtensionDescriptor {
            source: ExtSource::Space,
            space_root: "/space".into(),
            descriptor: stack_model::ExtensionLibrary {
                skills: vec![make_skill("shared")],
                ..Default::default()
            },
            content_hash: "space".to_string(),
        };
        let cache = ExtensionDescriptor {
            source: ExtSource::Cache,
            space_root: "/cache".into(),
            descriptor: stack_model::ExtensionLibrary {
                skills: vec![make_skill("shared")],
                ..Default::default()
            },
            content_hash: "cache".to_string(),
        };
        let inputs = RollupInputs {
            extensions: vec![cache, space],
            precedence: RollupInputs::default_precedence(),
        };
        let composed = compose_space(&inputs).expect("compose ok");
        let dts = emit_space_dts(&composed);

        // Exactly one occurrence of "shared" in the SpaceSkill line.
        let skill_line = dts
            .lines()
            .find(|l| l.contains("SpaceSkill ="))
            .expect("SpaceSkill union present");
        let count = skill_line.matches("\"shared\"").count();
        assert_eq!(count, 1, "id collision must dedupe; got: {skill_line}");
    }

    /// Pipeline uses `name` (not `id`) as the key per its struct
    /// shape (`PipelineDef.name`); the emitter must use the right
    /// field. A pipeline with `name: "nightly-build"` emits
    /// `"nightly-build"`.
    #[test]
    fn pipeline_uses_name_field_not_a_nonexistent_id() {
        let composed = ComposedLibrary {
            library: stack_model::ExtensionLibrary {
                pipelines: vec![PipelineDef {
                    name: "nightly-build".to_string(),
                    steps: vec![],
                    auto_merge: false,
                }],
                ..Default::default()
            },
            shadows: empty_composed().shadows,
            route_tree: empty_composed().route_tree,
        };
        let dts = emit_space_dts(&composed);
        assert!(
            dts.contains("export type SpacePipeline = \"nightly-build\";"),
            "pipeline union must use name field: {dts}"
        );
    }

    /// Agent ids come from `ProfileMeta.id` (nested), not a top-level
    /// `id` field. Confirms the nested-field read.
    #[test]
    fn agent_union_reads_profile_meta_id() {
        let composed = ComposedLibrary {
            library: stack_model::ExtensionLibrary {
                agents: vec![make_agent("code-reviewer")],
                ..Default::default()
            },
            shadows: vec![],
            route_tree: RouteTree::default(),
        };
        let dts = emit_space_dts(&composed);
        assert!(
            dts.contains("export type SpaceAgent = \"code-reviewer\";"),
            "agent union must read ProfileMeta.id: {dts}"
        );
    }
}
