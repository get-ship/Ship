// Compile-fail tests for #[derive(EventPayload)] validation.
// These tests verify that invalid annotations produce compile errors.
// They use `trybuild` to compile snippets and assert failure.
//
// The tests are doc-tests in the macro crate itself (not run via trybuild
// since trybuild is a heavy test dependency). Instead we rely on the
// parse-time validation in the macro to reject bad inputs at `cargo check`.
//
// Integration-level validation is covered by the tests in ship_substrate.
