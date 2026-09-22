//! trybuild gate: unstamped append does not compile.
//!
//! Requires the trybuild harness; skipped under miri / when the feature set
//! changes stderr formatting.

#[test]
fn unstamped_append_is_compile_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/unstamped_append.rs");
}
