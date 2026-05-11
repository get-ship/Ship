//! Regression tests for inputs found by `cargo +nightly fuzz` runs.
//! These ensure previously-discovered crash inputs do not regress.

use sui_compiler::{CompileMode, CompileOpts, compile};

/// Issue: `){&&deg;\x129\n}` crashed the compiler.
/// Found by `compile_full` fuzz on 2026-05-11.
#[test]
fn fuzz_2026_05_11_amp_entity_in_interpolation() {
    let src_bytes = [41u8, 123, 38, 38, 100, 101, 103, 59, 18, 57, 10, 125];
    let src = std::str::from_utf8(&src_bytes).expect("utf8");
    // Compile must return Err(CompileError) gracefully, not panic.
    let _ = compile(
        src,
        CompileOpts {
            mode: CompileMode::View,
            ..Default::default()
        },
    );
}
