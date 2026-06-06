//! Skills domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/skills/mod.rs`.

pub mod service;
pub mod state;
pub mod view_state;
pub mod wire;

pub use wire::SkillsWire;
