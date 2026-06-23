// build.rs for ship-views
//
// Seeds stub SDK manifest JSON files under packages/sdk/src/generated/
// when they are absent. The sdk_manifest module embeds these with include_str!
// at compile time; in un-bootstrapped environments (fresh worktrees, CI that
// has not run `just gen-sdk`) the directory doesn't exist because it is
// gitignored.
//
// The stubs are minimal valid JSON (`{"entries":{}}`) that:
//  - satisfy the compile-time include_str! requirement, and
//  - make sdk_manifest::tests::parses_embedded_manifest pass.
//
// `just gen-sdk` overwrites the stubs with real content; the
// cargo:rerun-if-changed directives below ensure ship-views is recompiled when
// that happens.

use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    // Navigate up from:
    //   crates/core/ship-platform/crates/ship-views/ (0)
    //   → crates/core/ship-platform/crates/          (1)
    //   → crates/core/ship-platform/                 (2)
    //   → crates/core/                               (3)
    //   → crates/                                    (4)
    //   → <workspace-root>                           (5)
    let workspace_root = manifest_dir
        .ancestors()
        .nth(5)
        .expect("ship-views build.rs: could not locate workspace root (expected ≥5 ancestor dirs)")
        .to_path_buf();
    let generated = workspace_root.join("packages/sdk/src/generated");

    std::fs::create_dir_all(&generated)
        .expect("ship-views build.rs: failed to create packages/sdk/src/generated/");

    let stubs = [
        "sdk-urls.json",
        "sdk-view-contexts.json",
        "sdk-views.json",
        "sdk-collections.json",
        "sdk-triggers.json",
    ];

    for name in stubs {
        let path = generated.join(name);
        if !path.exists() {
            // {"entries":{}} is the minimal shape that satisfies both the
            // include_str! requirement and the entries(&p.views).is_some()
            // assertion in sdk_manifest::tests::parses_embedded_manifest.
            std::fs::write(&path, r#"{"entries":{}}"#)
                .unwrap_or_else(|e| panic!("ship-views build.rs: failed to write stub {name}: {e}"));
        }
        // Re-run if gen-sdk overwrites the file with real content.
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
