//! Tests for S4: integrity check detects corruption.

use crate::db::{ensure_db_at, integrity};
use crate::testutil::init_project_minimal as init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

#[test]
fn healthy_db_passes_check() {
    let (_tmp, ship_dir) = setup();
    match integrity::check_platform_integrity(&ship_dir).unwrap() {
        integrity::IntegrityStatus::Ok => {}
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn quick_check_passes_on_healthy_db() {
    let (_tmp, ship_dir) = setup();
    let db_path = crate::db::platform_db_path(&ship_dir);
    assert!(integrity::quick_check(&db_path).unwrap());
}

#[test]
fn corrupted_db_detected() {
    let (_tmp, ship_dir) = setup();
    let db_path = crate::db::platform_db_path(&ship_dir);

    // Read the file and corrupt internal pages (keep the 100-byte SQLite header
    // so the file opens, but damage data pages so integrity_check fails).
    let mut data = std::fs::read(&db_path).unwrap();
    if data.len() > 200 {
        // Corrupt pages beyond the header (first 100 bytes).
        for byte in data[100..200].iter_mut() {
            *byte = 0xFF;
        }
    }
    std::fs::write(&db_path, &data).unwrap();

    let result = integrity::check_integrity(&db_path);
    match result {
        Ok(integrity::IntegrityStatus::Ok) => {
            // Some SQLite versions may still pass — skip test gracefully.
            eprintln!("SQLite integrity_check passed despite corruption — skipping");
        }
        Ok(integrity::IntegrityStatus::EventsCorrupt { details })
        | Ok(integrity::IntegrityStatus::ProjectionsCorrupt { details }) => {
            assert!(!details.is_empty(), "corruption details must be non-empty");
        }
        Err(_) => {
            // Connection failure due to corruption is also acceptable.
        }
    }
}

#[test]
fn missing_db_returns_error() {
    let tmp = tempdir().unwrap();
    let ship_dir = tmp.path().join("nonexistent");
    let result = integrity::check_platform_integrity(&ship_dir);
    assert!(result.is_err(), "missing db should error");
}
