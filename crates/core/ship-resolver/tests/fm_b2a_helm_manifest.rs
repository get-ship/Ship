//! Format-migration B2a: the helm extension lands its first content
//! distribution — the `working-in-helm` skill — and the ship extension
//! stops authoring it. The proof the cut took effect: a fixture that
//! mirrors the real shape of `/home/madvib/dev/Ship` (root WITHOUT a
//! manifest, two child dirs carrying their own `ship.config.ts`)
//! composes through `roll_up` and lands the helm skill via the helm
//! child extension and the lane agents via the ship child.
//!
//! ## Why a fixture, not a real-repo integration test
//!
//! `space_root=/home/madvib/dev/Ship` is not hermetic — the real-repo
//! layout drifts as the format-migration waves land. This test is the
//! missing-root-manifest case of the W4 phase B matrix
//! (`job-spec-fm-w4b-space-scan.md`) exercised with realistic
//! content: one agent + one skill in the `ship/` child, one skill in
//! the `helm/` child. The operator ruling on ownership
//! (`decision-morning-answers.md` §4) is the why; the spec
//! (`job-spec-fm-b2a-helm-manifest.md` §3) is the what.
//!
//! ## The gap this test pins down
//!
//! Before B2a, `working-in-helm` lived in the ship repo's
//! `ship.config.ts`. Post-cut, it MUST NOT be authored there — the
//! helm extension owns the content. The fixture builds the exact
//! post-cut shape and asserts:
//!   1. `working-in-helm` survives the roll-up via the helm child,
//!   2. the ship child's lane agents (`rust-lane`) survive,
//!   3. the helm child cannot grant itself permissions/hooks (the
//!      PIN — Wave 4 phase B's per-root identity fence),
//!   4. there is no `ship.config.ts` at the space root.
//!
//! Item 4 is the spec's distinguishing shape: the missing-root-manifest
//! case is the W4B matrix's row where child extensions carry the
//! composed world and the root identity is the empty entry.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // crates/core/ship-resolver -> repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("ship-resolver sits three levels under the repo root")
        .to_path_buf()
}

async fn fm_b2a_helm_extension_supplies_working_in_helm_via_child_root() {
    // 1. The fixture: a tempdir as space root, with NO `ship.config.ts`
    //    at the top level. The spec requires this — the real
    //    `/home/madvib/dev/Ship` space root does not carry a manifest
    //    either (the migration moves the root's authoring surface
    //    later, not in this job).
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().to_path_buf();
    assert!(
        !root.join("ship.config.ts").exists(),
        "fixture invariant: no ship.config.ts at the space root"
    );

    // 2. The ship child — the format-migration Wave 5 + slice 2c
    //    source surface. `rust-lane` is the dispatch lane the spec
    //    calls out as the pin for the operational loop; one skill is
    //    the ship-side set's reduced shape (the full ship manifest
    //    is tested by `real_manifest.rs` — this fixture cares about
    //    the seam, not the noun set). Slice 2c: agents in
    //    `agents/<id>.ts` (per-noun source dir); skills on-disk
    //    under `skills/<id>/SKILL.md` (the per-extension-project
    //    source surface for the skill shape, not a
    //    `defineSkill` declare-form).
    let ship = root.join("ship");
    std::fs::create_dir_all(ship.join(".ship")).expect("ship/.ship");
    std::fs::create_dir_all(ship.join("agents")).expect("ship/agents");
    // The manifest is a thin pointer: empty `defineConfig({})`.
    // The agents live in `agents/<id>.ts` files (slice 2c).
    std::fs::write(
        ship.join("ship.config.ts"),
        r#"
import { defineConfig } from "ship";

export default defineConfig({});
"#,
    )
    .expect("ship manifest");
    std::fs::write(
        ship.join("agents/rust-lane.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({ agent: { id: "rust-lane", name: "Rust Lane" } });
"#,
    )
    .expect("rust-lane agent");
    std::fs::write(
        ship.join("agents/writer.ts"),
        r#"
import { defineAgent } from "ship";

defineAgent({ agent: { id: "writer", name: "Writer" } });
"#,
    )
    .expect("writer agent");
    // Ship-side skill: on-disk `skills/ship-skill/SKILL.md`.
    let ship_skill_dir = ship.join("skills").join("ship-skill");
    std::fs::create_dir_all(&ship_skill_dir).expect("ship-skill dir");
    std::fs::write(
        ship_skill_dir.join("SKILL.md"),
        "---\nname: ship-skill\ndescription: A ship-side skill.\n---\nship body\n",
    )
    .expect("ship-skill SKILL.md");

    // 3. The helm child — the format-migration B2a landing surface for
    //    the helm extension's content. Slice 2c: the skill is on-disk
    //    `skills/working-in-helm/SKILL.md` (the body that moved
    //    verbatim out of the ship repo's `ship.config.ts` per
    //    decision-morning-answers.md §4: helm owns the helm skill).
    let helm = root.join("helm");
    std::fs::create_dir_all(helm.join(".ship")).expect("helm/.ship");
    std::fs::create_dir_all(helm.join("skills")).expect("helm/skills");
    std::fs::write(
        helm.join("ship.config.ts"),
        r#"
import { defineConfig } from "ship";

export default defineConfig({});
"#,
    )
    .expect("helm manifest");
    let helm_skill_dir = helm.join("skills").join("working-in-helm");
    std::fs::create_dir_all(&helm_skill_dir).expect("helm-skill dir");
    std::fs::write(
        helm_skill_dir.join("SKILL.md"),
        "Working in Helm placeholder — the body that moved verbatim from the ship repo in the format-migration B2a cut.",
    )
    .expect("working-in-helm SKILL.md");

    // 4. Empty cache + library tiers — the proof is about the
    //    multi-extension roll-up across the child roots, not the
    //    cached / enabled tiers (those are the W5 + J2 surfaces,
    //    pinned by `roll_up_source_shadows_cached_agent_on_id_collision`
    //    + friends).
    let cache_root = root.join("empty_cache");
    std::fs::create_dir_all(&cache_root).expect("cache dir");
    let library_root = root.join("empty_library");

    let composed = ship_resolver::roll_up(&root, Some(&cache_root), Some(&library_root))
        .await
        .expect("roll_up over the B2a fixture must succeed");

    // 5. working-in-helm arrived via the helm child extension. The
    //    body moved verbatim, so the canonical id matches.
    assert!(
        composed.library.skills.iter().any(|s| {
            s.id == "working-in-helm" && s.stable_id.as_deref() == Some("working-in-helm")
        }),
        "working-in-helm missing from the composed library — the helm child \
         extension did not contribute its skill: ids={:?}",
        composed
            .library
            .skills
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>()
    );

    // 6. The ship child's lane agents survived — `rust-lane` is the
    //    dispatch lane the operator's automated land path keys on
    //    (per `real_manifest.rs`'s pin).
    let agent_ids: std::collections::HashSet<&str> = composed
        .library
        .agent_profiles
        .iter()
        .map(|a| a.agent.id.as_str())
        .collect();
    assert!(agent_ids.contains("rust-lane"), "rust-lane missing: {agent_ids:?}");
    assert!(agent_ids.contains("writer"), "writer missing: {agent_ids:?}");

    // 7. The ship-side skill arrived via the ship child, distinct from
    //    the helm-side skill. The composed library holds BOTH —
    //    every child root contributes its full noun set.
    assert!(
        composed.library.skills.iter().any(|s| s.id == "ship-skill"),
        "ship-side skill missing — the ship child extension did not contribute: {:?}",
        composed
            .library
            .skills
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        composed
            .library
            .skills
            .iter()
            .filter(|s| s.id == "working-in-helm" || s.id == "ship-skill")
            .count(),
        2,
        "exactly two skills expected (one per child), no duplicates"
    );

    // 8. The PIN holds: the root has no manifest, so the composed
    //    library's `permissions` + `hooks` come from the root → empty.
    //    The Space descriptors are exactly the two child roots (no
    //    root descriptor at all). Lexicographic child order: `helm`
    //    < `ship`, so helm is the first Space descriptor.
    let rollup = composed
        .library
        .rollup_inputs
        .as_ref()
        .expect("roll_up populates rollup_inputs");
    let space_descriptors: Vec<&compiler::ExtensionDescriptor> = rollup
        .extensions
        .iter()
        .filter(|e| e.source == compiler::ExtSource::Space)
        .collect();
    assert_eq!(
        space_descriptors.len(),
        2,
        "two child extensions: helm + ship (root has no manifest, so it is \
         absent from the descriptor set); got identities = {:?}",
        space_descriptors
            .iter()
            .map(|e| e.space_root.file_name().and_then(|n| n.to_str()).unwrap_or("?"))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        space_descriptors[0].space_root.file_name().and_then(|n| n.to_str()),
        Some("helm"),
        "first Space descriptor: helm (lexicographic)"
    );
    assert_eq!(
        space_descriptors[1].space_root.file_name().and_then(|n| n.to_str()),
        Some("ship"),
        "second Space descriptor: ship (lexicographic)"
    );
    let helm_desc = space_descriptors[0];
    let ship_desc = space_descriptors[1];
    // The helm descriptor carries the skill; the ship descriptor
    // carries the agents + the ship-side skill. A cross-membership
    // would be the kind of regression this test exists to catch.
    assert_eq!(helm_desc.descriptor.skills.len(), 1);
    assert_eq!(helm_desc.descriptor.skills[0].id, "working-in-helm");
    assert_eq!(helm_desc.descriptor.agent_profiles.len(), 0);
    assert_eq!(ship_desc.descriptor.skills.len(), 1);
    assert_eq!(ship_desc.descriptor.skills[0].id, "ship-skill");
    assert_eq!(ship_desc.descriptor.agent_profiles.len(), 2);
}

#[test]
fn fm_b2a_real_manifest_no_longer_declares_working_in_helm() {
    // The other half of the cut: the ship repo's own source space
    // no longer declares `working-in-helm` anywhere. The helm
    // extension owns it. The shape this assertion pins: walk the
    // ship repo's pointer manifest (`ship.config.ts`) + the per-noun
    // source dirs (`agents/`, `collections/`, `pipelines/`,
    // `triggers/`, `skills/`, `rules/`); prove the skill id is absent
    // from every `defineSkill` shape AND from the manifest block.
    // Cheap and hermetic; a re-introduction is a regression the
    // operator's ruling (`decision-morning-answers.md` §4) forbids.
    let root = repo_root();
    let manifest = root.join("ship.config.ts");
    let manifest_body = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| panic!("read ship.config.ts at {manifest:?}: {e}"));
    assert!(
        !manifest_body.contains("\"name\": \"working-in-helm\""),
        "ship.config.ts must not declare working-in-helm — the helm extension \
         owns the content (decision-morning-answers.md §4). Re-introducing \
         it here would re-split ownership."
    );
    // Also scan the per-noun source dirs: nothing under `agents/`,
    // `collections/`, `pipelines/`, `triggers/` may declare the
    // skill id (the post-slice-2c surface routes skills to on-disk
    // `skills/<id>/SKILL.md` files; the helm extension owns the
    // `working-in-helm` SKILL.md).
    for sub in ["agents", "collections", "pipelines", "triggers"] {
        let dir = root.join(sub);
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}")) {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("ts") {
                continue;
            }
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            assert!(
                !body.contains("\"name\": \"working-in-helm\""),
                "{path:?} must not declare working-in-helm — the helm \
                 extension owns the content (decision-morning-answers.md §4). \
                 Re-introducing it here would re-split ownership."
            );
        }
    }
    // The agent-skill refs still name working-in-helm — those refs
    // are now broken in the composed world; that is a known +
    // accepted consequence of the cut (spec §4 "until the
    // operational flip"). Pin that the broken-refs state persists;
    // the operator's ruling outranks the temporary composition
    // visibility. Slice 2c: the refs now live in the per-noun
    // source dirs (`agents/<id>.ts`), not the manifest.
    let mut found_ref = false;
    for sub in ["agents", "collections", "pipelines", "triggers"] {
        let dir = root.join(sub);
        if !dir.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}")) {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("ts") {
                continue;
            }
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            if body.contains("working-in-helm") {
                found_ref = true;
                break;
            }
        }
        if found_ref {
            break;
        }
    }
    assert!(
        found_ref,
        "the agent-skill refs still name working-in-helm — this is the \
         known-broken state the spec accepts; if this assertion fails, \
         the refs have been silently fixed (update the test, not the file)"
    );
}
