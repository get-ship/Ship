//! Provisioned-grants projection — re-exported from ship-substrate.
//!
//! The implementation lives in `ship_substrate::projections::grants` because
//! `ship-auth` depends on substrate (not platform), and we need a cycle-free
//! path for both the projection and the read functions.

pub use ship_substrate::projections::grants::{
    ProvisionedGrantsProjection, ProvisionedGrantsRow, provisioned_grants_for,
    provisioned_grants_for_at, provisioned_row,
};
