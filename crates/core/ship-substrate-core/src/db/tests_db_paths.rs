//! Fossil guard (invariant I-6): the sanctioned db set is closed. The only db
//! files the substrate can construct are `runtime.db`, `account.db`,
//! `spaces/<id>/space.db`, and the machine `kv.db`. Re-adding a dead store
//! (platform.db / work.db / data.db) fails HERE — a compile error via the
//! exhaustive `Db` match plus the exact-path assertions — not just in the
//! grep guard (`scripts/lint/check-dead-db-names.sh`).

use std::path::Path;

use crate::db::{account_db_path, kv_db_path, runtime_db_path, space_db_path};

/// Exhaustive match over the machine-db enum: adding a `Db` variant refuses to
/// compile until this guard names it, so the sanctioned set is re-decided
/// consciously instead of drifting back in.
fn sanctioned_machine_db_file(db: crate::Db) -> &'static str {
    match db {
        crate::Db::Kv => "kv.db",
    }
}

#[test]
fn machine_db_enum_is_kv_only() {
    assert_eq!(sanctioned_machine_db_file(crate::Db::Kv), "kv.db");
    assert_eq!(crate::Db::Kv.as_str(), "kv.db");
}

#[test]
fn db_path_constructors_expose_only_the_sanctioned_set() {
    let ship_dir = Path::new("/home/x/.ship");
    assert_eq!(
        runtime_db_path(ship_dir),
        Path::new("/home/x/.ship/runtime.db")
    );
    assert_eq!(
        account_db_path(ship_dir),
        Path::new("/home/x/.ship/account.db")
    );
    assert_eq!(
        space_db_path(ship_dir, "s1"),
        Path::new("/home/x/.ship/spaces/s1/space.db")
    );
    // kv.db routes through the canonical machine-db resolver. The exact-path
    // assertion only holds without the workflow-only SHIP_DEV_DB_DIR override
    // (never set in production; a parallel test could set it), so it is
    // guarded; the filename assertion is unconditional.
    let kv = kv_db_path(ship_dir);
    assert_eq!(kv.file_name().and_then(|n| n.to_str()), Some("kv.db"));
    if crate::env::read(crate::env::Var::DevDbDir).is_none() {
        assert_eq!(kv, Path::new("/home/x/.ship/machine/db/kv.db"));
    }
}
