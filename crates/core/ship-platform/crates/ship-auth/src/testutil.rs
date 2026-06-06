//! Test helpers for ship-auth integration tests.
//!
//! Registers the auth + grants projections onto the substrate event bus so that
//! `SqliteEventStore::new_for_scope` sees the full auth read-model. Called at
//! the start of any test that appends events and then reads projections.

use std::sync::Once;

static AUTH_INSTALLED: Once = Once::new();

/// Register auth projections (AuthSessions, DaemonOwner, ProvisionedGrants) on
/// the substrate projection factory. Idempotent — safe to call from multiple
/// tests. Returns the `.ship/` directory inside `base_dir`.
pub fn init_auth(base_dir: impl AsRef<std::path::Path>) -> anyhow::Result<std::path::PathBuf> {
    AUTH_INSTALLED.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(crate::projection::AuthSessionsProjection)
                    as Box<dyn ship_substrate::projections::Projection>,
                Box::new(crate::projection::DaemonOwnerProjection),
                Box::new(ship_substrate::projections::ProvisionedGrantsProjection),
            ]
        }));
    });

    let base_dir = base_dir.as_ref();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(base_dir);
    std::fs::create_dir_all(&ship_dir)?;
    let id = ship_substrate::gen_nanoid();
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        format!("{{\"id\": \"{id}\"}}\n"),
    )?;
    Ok(ship_dir)
}
