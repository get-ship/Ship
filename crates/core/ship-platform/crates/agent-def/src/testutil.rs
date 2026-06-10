//! Test helpers for agent-def — re-exported from `ship_platform::testutil`.
//!
//! `ship_platform::testutil` is the authoritative copy; agent-def forwards to
//! it via the `testutil` feature on the dev-dep.  The old inline copy that
//! duplicated the function body has been deleted (SD-5).
pub use ship_platform::testutil::init_project;
