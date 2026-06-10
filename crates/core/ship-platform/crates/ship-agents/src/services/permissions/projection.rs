//! Provisioned-grants projection — re-exported from ship-auth.
//!
//! The implementation moved from `ship_substrate::projections::grants` to
//! `ship_auth::grants_projection` (keystone-A Phase A bundle 2, 2026-06-10):
//! the AUTH domain owns its projection at L1.

pub use ship_auth::grants_projection::{
    ProvisionedGrantsProjection, ProvisionedGrantsRow, provisioned_grants_for,
    provisioned_grants_for_at, provisioned_row,
};
