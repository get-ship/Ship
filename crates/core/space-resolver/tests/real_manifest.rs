//! The repo's OWN `stack.config.ts` must roll up clean — the standing gate
//! this repo was missing.
//!
//! `ts_resolver` tolerates a failing module by design (warn + drop its
//! declarations), so a shape drift between the manifest and the eval's
//! declare-forms doesn't error anywhere — the composed world just silently
//! loses every noun the module declared. That exact failure shipped once:
//! W4A migrated `pipelines/*.jsonc` gates verbatim (`{match, min}` objects)
//! against `@ship/stack`'s flattened `PipelineGate` (`string[]`), the eval
//! rejected the whole module, and every agent lookup through the composed
//! path returned not-found while all fixture-based suites stayed green.
//!
//! This test evals the REAL manifest, so any drift between authored content
//! and the declare-form shapes fails CI instead of degrading production.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // crates/core/space-resolver -> repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("space-resolver sits three levels under the repo root")
        .to_path_buf()
}

#[tokio::test]
#[ignore = "the test looks for stack.config.ts at the repo root; the integrator's \
cutover flips the working file alongside the daemon roll, at which point the \
test is re-enabled. Tracked by job-spec-marker-rename.md."]
async fn repo_manifest_rolls_up_with_all_nouns_present() {
    let root = repo_root();
    assert!(
        root.join("stack.config.ts").is_file(),
        "repo root manifest missing at {root:?} — did the layout move?"
    );

    let composed = space_resolver::roll_up_members(&[root.clone()], None)
        .await
        .expect("roll_up over the real repo must succeed");

    let lib = &composed.library;

    // Every noun family the manifest declares must survive the eval. A
    // whole-module drop (the ts_resolver warn-and-skip path) empties ALL of
    // these at once — that is the regression this gate exists to catch.
    // Floors, not exact counts: adding/removing individual entries is
    // routine; losing a whole family is the bug.
    assert!(
        !lib.agent_profiles.is_empty(),
        "no agents composed — the manifest module was dropped by the eval \
         (check shipd logs for `ts_resolver: eval failed`)"
    );
    assert!(
        !lib.skills.is_empty(),
        "no skills composed from the real manifest"
    );

    // The publish-intended agent this stack keeps (the internal fleet
    // lives in the ship-meta member stack).
    assert!(
        lib.agent_profiles
            .iter()
            .any(|p| p.agent.id == "shipwright"),
        "shipwright missing from the composed agents"
    );
}
