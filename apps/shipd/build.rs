//! Build script for the shipd daemon.
//!
//! The previous build.rs was 724 lines of REST-surface codegen (parsed
//! `Route` enum variants, generated `ShipRouterExt::ship_route`, generated
//! the TS route catalog). The REST surface was deleted in the launch
//! unification (Connect is the only path), so the codegen is dead.
//!
//! This minimal build script only emits build identity (git SHA, timestamp,
//! rustc version, profile) for `build_info` — shared across shipd.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

fn main() {
    build_info::emit_build_env();
}
