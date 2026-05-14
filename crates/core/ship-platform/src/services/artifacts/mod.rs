//! Artifacts domain — service trait + impl (DB-backed via DataService).
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/artifacts/mod.rs`.

pub mod service;
