//! Slice 4 — SPA build output (deck's `next export` analog).
//!
//! `deck build` (→ `deck_bundler::app::bundle_app`) takes an app root with an
//! `index.html` + `src/main.ts` + `src/chrome/layout.css` and emits a static
//! `dist/`: a hashed entry chunk, a hashed stylesheet, and a rewritten
//! `index.html` — with the framework left EXTERNAL via the importmap. This
//! asserts that output for the example's `spa/` entry.
//!
//! The `--bundle_framework` (fully self-contained, daemon-less) variant is NOT
//! exercised here: it currently requires a built island dir (`bail`s without
//! one) and rebuilds islands from the repo root, so it's coupled to the
//! monorepo and can't yet produce a clean self-contained build for an external,
//! island-free app. Tracked as a finding (README); the static *serving* path is
//! proven separately by `e2e/static.sh`.
//!
//! Run: `cargo test -p deck-demo --test build`

use std::path::{Path, PathBuf};

use deck::bundler::app::bundle_app;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("examples/deck-demo → repo root")
        .to_path_buf()
}

#[tokio::test(flavor = "multi_thread")]
async fn spa_build_emits_hashed_bundle_and_rewrites_html() {
    let spa = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spa");
    let repo = repo_root();
    let out = tempfile::tempdir().expect("tempdir");
    let framework_assets = repo.join("crates/deck/deck-core/assets");
    let deck_out = repo.join(".deck");

    // bundle_framework = false: the SPA mode where a host/daemon serves /_sui/*.
    let build = bundle_app(
        &spa, // repo_root (cwd for module resolution; framework is external)
        &spa, // shell_root (index.html + src/)
        out.path(),
        false, // minify
        false, // bundle_framework
        &framework_assets,
        &deck_out,
    )
    .await
    .expect("spa build");

    // 1. Entry chunk is hashed, and index.html points at it (not the raw /src ref).
    assert!(
        build.js_href.starts_with("/assets/main-") && build.js_href.ends_with(".js"),
        "expected a hashed entry chunk, got {}",
        build.js_href
    );
    let html = std::fs::read_to_string(out.path().join("index.html")).expect("dist index.html");
    assert!(html.contains(&build.js_href), "html must reference the hashed bundle");
    assert!(!html.contains("/src/main.ts"), "raw entry ref must be rewritten");
    assert!(html.contains("type=\"importmap\""), "importmap must be preserved");

    // 2. The chrome stylesheet is copied under a hashed name.
    assert!(
        build.css_href.starts_with("/assets/layout-") && build.css_href.ends_with(".css"),
        "expected a hashed stylesheet, got {}",
        build.css_href
    );

    // 3. The runtime is left EXTERNAL (single instance), never inlined.
    let entry = out.path().join(build.js_href.trim_start_matches('/'));
    let js = std::fs::read_to_string(&entry).expect("entry chunk");
    assert!(
        js.contains("@ship/sui-runtime"),
        "runtime must stay external (bare import), resolved at runtime via the importmap"
    );
}

/// Fix B (was a Slice-4 finding): `--bundle_framework` must produce a clean
/// self-contained deploy for an island-free app — ship the four framework
/// singletons, skip the ship-static island targets whose entries don't exist
/// under the app root, and NOT bail on a missing island dir.
#[tokio::test(flavor = "multi_thread")]
async fn self_contained_build_ships_singletons_for_island_free_app() {
    let spa = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spa");
    let repo = repo_root();
    let out = tempfile::tempdir().expect("tempdir");
    let framework_assets = repo.join("crates/deck/deck-core/assets");
    let deck_out = repo.join(".deck");

    // repo_root = the SPA dir (an island-free app root) + bundle_framework=true.
    let build = bundle_app(
        &spa,
        &spa,
        out.path(),
        false, // minify
        true,  // bundle_framework — the self-contained / static path
        &framework_assets,
        &deck_out,
    )
    .await
    .expect("self-contained build must succeed for an island-free app");

    for f in ["runtime.js", "dom.js", "sdk-client.js", "components.js"] {
        assert!(
            out.path().join("_sui").join(f).exists(),
            "self-contained build must materialize /_sui/{f}"
        );
    }
    assert!(
        build.framework_assets >= 4,
        "expected the four framework singletons, got {}",
        build.framework_assets
    );
}
