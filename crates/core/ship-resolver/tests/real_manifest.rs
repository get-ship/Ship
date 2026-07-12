//! The repo's OWN `ship.config.ts` must roll up clean — the standing gate
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
    // crates/core/ship-resolver -> repo root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("ship-resolver sits three levels under the repo root")
        .to_path_buf()
}

#[tokio::test]
async fn repo_manifest_rolls_up_with_all_nouns_present() {
    let root = repo_root();
    assert!(
        root.join("ship.config.ts").is_file(),
        "repo root manifest missing at {root:?} — did the layout move?"
    );

    let composed = ship_resolver::roll_up(&root, None, None)
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
         (check helmd/shipd logs for `ts_resolver: eval failed`)"
    );
    assert!(
        !lib.skills.is_empty(),
        "no skills composed from the real manifest"
    );

    // The dispatch lane the operational loop depends on.
    assert!(
        lib.agent_profiles.iter().any(|p| p.agent.id == "rust-lane"),
        "rust-lane missing from the composed agents — dispatch validation \
         will refuse every job"
    );
}
