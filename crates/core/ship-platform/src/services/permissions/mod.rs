//! Permissions domain — method-oriented service trait + state.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/permissions/mod.rs`.

// `provisioned_grants` projection — co-located with the permissions
// domain so a future crate-carve takes it along.
pub mod projection;
pub mod service;
pub mod state;
pub mod wire_types;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

pub use projection::{
    ProvisionedGrantsProjection, ProvisionedGrantsRow, provisioned_grants_for, provisioned_row,
};
