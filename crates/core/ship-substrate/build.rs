//! Build script: force a recompile when the embedded migrations change.
//!
//! `sqlx::migrate!("./migrations")` bakes the SQL into the binary at *compile*
//! time. Cargo's default fingerprint does not reliably treat a newly-added
//! migration file as a reason to recompile this crate, so a fresh migration can
//! silently fail to embed — at runtime the table is simply missing
//! (`no such table`). That footgun bit P0.1.
//!
//! Emitting `rerun-if-changed` for the migration directories (and every file in
//! them) ties this crate's recompile to migration changes: touching, adding, or
//! removing a migration reruns the build script, which invalidates the crate
//! fingerprint, which re-expands `sqlx::migrate!` with the current set.
//!
//! Covers both stores: `migrations/` (the main SQLite schema) and
//! `migrations_kv/` (the KV store, embedded via `sqlx::migrate!("./migrations_kv")`).

use std::path::Path;

fn emit_dir(dir: &Path) {
    // Watch the directory itself (so adds/removes are seen) ...
    println!("cargo:rerun-if-changed={}", dir.display());
    // ... and every file within (so content edits are seen).
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            emit_dir(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn main() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed=build.rs");
    for sub in ["migrations", "migrations_kv"] {
        let dir = manifest_dir.join(sub);
        if dir.is_dir() {
            emit_dir(&dir);
        }
    }
}
