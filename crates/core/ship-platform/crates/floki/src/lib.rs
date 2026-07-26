//! Compatibility re-export: **Yoke** is the native harness (Bridge monorepo).
//!
//! This package name remains `floki` only so existing Ship `use floki::…`
//! sites keep compiling during the cutover. Prefer depending on `yoke`
//! directly; remove this crate when shipd is RPC-only to Bridge.

pub use yoke::*;

/// Historical name for [`yoke::YokeProgram`].
pub use yoke::YokeProgram as FlokiProgram;
