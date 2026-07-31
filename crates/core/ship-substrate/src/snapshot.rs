//! Space snapshot + restore — the substrate-level backup primitive.
//!
//! A snapshot is a *consistent* point-in-time view of one space's
//! per-scope SQLite store (`spaces/<id>/space.db`) plus its content
//! blobs (`spaces/<id>/blobs/<h[:2]>/<h>`). The output is a bundle
//! directory:
//!
//! ```text
//! <target_dir>/space-<space_id>-<utc_stamp>/
//! ├── space.db              # consistent copy via `VACUUM INTO`
//! ├── blobs/<h[:2]>/<h>     # content-addressed blob bytes
//! └── manifest.json         # { space_id, created_at, db_bytes,
//!                            #   db_sha256, blob_count, blob_bytes,
//!                            #   tool_version }
//! ```
//!
//! ## Consistency: db first, then blobs
//!
//! The db is snapshotted via `VACUUM INTO` — SQLite's recommended
//! online backup. Unlike a raw `cp`/read, `VACUUM INTO` is
//! crash-consistent: it serialises the live db (including its WAL) into
//! a clean, standalone file even while writers are active. (Live
//! readers on the original db are unaffected — `VACUUM INTO` writes to
//! a separate file and commits at the end.)
//!
//! Blobs are copied **after** the db snapshot completes. Blobs are
//! content-addressed and immutable-once-written, so a copy that lands
//! after the db can never produce a dangling reference *from* the
//! snapshot: any blob the db references by hash is guaranteed to be the
//! one we copy (we just wrote its hash), and any blob the db does NOT
//! reference is harmless dead weight. The mirror is also safe in the
//! other direction: a blob could be deleted on the live system *after*
//! our db snapshot but *before* we copy the blob — we still copy what
//! we see on disk, which is a snapshot of the live state, not a
//! historical claim.
//!
//! ## Restore
//!
//! `restore_space` is the inverse: read the manifest, verify the db
//! sha256, place db + blobs at the canonical layout under the new
//! data root. Refuses to overwrite an existing space unless `force` is
//! set. Restoring is library + test only (no wire surface yet — the
//! restore wire verb is a later judged op).
//!
//! ## On-disk layout dependencies
//!
//! - space.db path: `data_root/spaces/<space_id>/space.db` (see
//!   `db::space_db_path`)
//! - blobs path: `data_root/spaces/<space_id>/blobs/<h[:2]>/<h>` (the
//!   `LocalFsBlobStore` layout for `BlobScope::Space`)
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ship_types::sha256_hex;

use crate::db::{block_on, open_space_db, space_db_path};

/// The on-disk tool version recorded in `manifest.json`. Bumped when
/// the bundle layout or manifest schema changes in a way a restore
/// would care about.
pub const TOOL_VERSION: &str = "ship-snapshot/1";

const BUNDLE_DB_FILE: &str = "space.db";
const BUNDLE_BLOBS_DIR: &str = "blobs";
const BUNDLE_MANIFEST: &str = "manifest.json";
const SPACE_BLOBS_DIR: &str = "blobs";
#[cfg(test)]
const BLOB_FANOUT: usize = 2;
/// Result of a successful snapshot. Mirrors the wire `SpaceSnapshotResult`
/// (see `ship.ops.v1.SpaceSnapshotResult` in space.proto).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotResult {
    pub space_id: String,
    /// Bundle directory written under `target_dir`.
    pub path: PathBuf,
    pub db_bytes: u64,
    pub blob_count: u32,
    pub blob_bytes: u64,
    pub db_sha256: String,
}

/// Manifest persisted at `<bundle>/manifest.json`. The `db_sha256` is
/// the integrity anchor for restore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotManifest {
    pub space_id: String,
    /// RFC3339 UTC.
    pub created_at: String,
    pub db_bytes: u64,
    pub db_sha256: String,
    pub blob_count: u32,
    pub blob_bytes: u64,
    pub tool_version: String,
}

/// Result of a successful restore. Returns what was placed (paths are
/// under the new data root).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreResult {
    pub space_id: String,
    pub data_root: PathBuf,
    pub db_path: PathBuf,
    pub blob_count: u32,
    pub blob_bytes: u64,
}

/// Snapshot a single space's storage to `target_dir`.
///
/// `data_root` is the XDG `data` dir (e.g. `$XDG_DATA_HOME/ship` in
/// production, `<base>/data` in `SHIP_HOME` mode). `target_dir` must
/// exist and be writable. The bundle is written into a fresh
/// `space-<id>-<utc_stamp>/` subdirectory so multiple snapshots of
/// the same space do not collide.
pub fn snapshot_space(
    data_root: &Path,
    space_id: &str,
    target_dir: &Path,
) -> Result<SnapshotResult> {
    if !target_dir.is_dir() {
        return Err(anyhow!(
            "snapshot target_dir does not exist or is not a directory: {}",
            target_dir.display()
        ));
    }

    let src_db = space_db_path(data_root, space_id);
    if !src_db.is_file() {
        return Err(anyhow!(
            "snapshot source space.db not found: {}",
            src_db.display()
        ));
    }
    let src_blobs = data_root
        .join(crate::db::SPACES_DIR)
        .join(space_id)
        .join(SPACE_BLOBS_DIR);

    // Fresh bundle dir per call. The UTC stamp is the snapshot instant
    // and is also written into the manifest.
    let stamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let bundle = target_dir.join(format!("space-{space_id}-{stamp}"));
    if bundle.exists() {
        return Err(anyhow!(
            "snapshot bundle already exists: {}",
            bundle.display()
        ));
    }
    std::fs::create_dir_all(&bundle)
        .with_context(|| format!("create_dir_all {}", bundle.display()))?;
    let bundle_blobs = bundle.join(BUNDLE_BLOBS_DIR);
    std::fs::create_dir_all(&bundle_blobs)
        .with_context(|| format!("create_dir_all {}", bundle_blobs.display()))?;

    // ── 1. db snapshot via VACUUM INTO ────────────────────────────────────
    // VACUUM INTO is the SQLite-recommended online backup: serialises
    // the live db (including WAL) into a clean, standalone file. Safe
    // to run while writers are active. Runs on a fresh connection so
    // the page cache + WAL state match the snapshot we will copy.
    let bundle_db = bundle.join(BUNDLE_DB_FILE);
    {
        let mut conn = open_space_db(data_root, space_id)
            .with_context(|| format!("open live space db for {space_id}"))?;
        let sql = format!(
            "VACUUM INTO '{}'",
            // Escape single quotes for the SQL literal — VACUUM INTO takes
            // a string literal, not a parameter binding. The path is one
            // we just created; single quotes in a path are exotic.
            bundle_db.display().to_string().replace('\'', "''")
        );
        block_on(async { sqlx::query(&sql).execute(&mut conn).await })
            .with_context(|| format!("VACUUM INTO {}", bundle_db.display()))?;
    }

    let db_bytes = std::fs::metadata(&bundle_db)
        .with_context(|| format!("stat {}", bundle_db.display()))?
        .len();
    let db_sha256 = {
        let bytes =
            std::fs::read(&bundle_db).with_context(|| format!("read {}", bundle_db.display()))?;
        sha256_hex(&bytes)
    };

    // ── 2. blobs: copy AFTER the db snapshot completes ─────────────────────
    // See module doc for the consistency argument. The live
    // `LocalFsBlobStore` may have a `<dir>/<space_id>/blobs/<h[:2]>/<h>`
    // layout (see `BlobScope::Space` + the two-char fan-out) — we walk
    // and copy. Missing source dir = no blobs (the space has no
    // content yet), which is fine.
    let mut blob_count: u32 = 0;
    let mut blob_bytes: u64 = 0;
    if src_blobs.is_dir() {
        copy_blobs(&src_blobs, &bundle_blobs, &mut blob_count, &mut blob_bytes)?;
    }

    // ── 3. manifest ────────────────────────────────────────────────────────
    let manifest = SnapshotManifest {
        space_id: space_id.to_string(),
        created_at: Utc::now().to_rfc3339(),
        db_bytes,
        db_sha256: db_sha256.clone(),
        blob_count,
        blob_bytes,
        tool_version: TOOL_VERSION.to_string(),
    };
    let manifest_path = bundle.join(BUNDLE_MANIFEST);
    let manifest_bytes =
        serde_json::to_vec_pretty(&manifest).context("serialize snapshot manifest")?;
    fs_util::write_atomic(&manifest_path, &manifest_bytes)
        .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok(SnapshotResult {
        space_id: space_id.to_string(),
        path: bundle,
        db_bytes,
        blob_count,
        blob_bytes,
        db_sha256,
    })
}

/// Recursive copy of `<src>/**/*` into `<dst>/**/*` (files only). Counts
/// files + bytes for the manifest. The source layout under `src` is
/// `blobs/<h[:2]>/<h>` — preserving the same relative tree under `dst`
/// keeps restore's copy direction trivial.
fn copy_blobs(src_root: &Path, dst_root: &Path, count: &mut u32, bytes: &mut u64) -> Result<()> {
    for entry in walkdir::WalkDir::new(src_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        // Skip the atomic-write temp files the LocalFsBlobStore uses —
        // these are zero-or-newer-than-flushed partial writes and must
        // not be promoted into a backup.
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(".tmp-"))
        {
            continue;
        }
        let rel = path
            .strip_prefix(src_root)
            .with_context(|| format!("relativize {}", path.display()))?;
        let dst = dst_root.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create_dir_all {}", parent.display()))?;
        }
        let src_bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        // Hash-check on copy: content-addressed store, so the filename
        // IS the hash — but verify anyway so a corrupt source never
        // reaches the backup. Mismatches are a data-integrity bug we
        // want to surface, not paper over.
        let actual_hash = sha256_hex(&src_bytes);
        let expected_hash = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("blob filename is not utf-8: {}", path.display()))?;
        if actual_hash != expected_hash {
            return Err(anyhow!(
                "blob content hash mismatch: {} (expected {expected_hash}, got {actual_hash})",
                path.display()
            ));
        }
        fs_util::write_atomic(&dst, &src_bytes)
            .with_context(|| format!("write {}", dst.display()))?;
        *count += 1;
        *bytes += src_bytes.len() as u64;
    }
    Ok(())
}

/// Restore a snapshot bundle to the canonical layout under `data_root`.
///
/// Reads `<bundle_dir>/manifest.json`, verifies the bundled
/// `space.db` sha256 against the manifest, then copies the db +
/// blobs to the same paths `LocalFsBlobStore` / `db::space_db_path`
/// produce. Refuses to overwrite an existing space at
/// `data_root/spaces/<space_id>` unless `force` is set.
///
/// The canonical placement is exactly what a fresh space would write:
///   - `data_root/spaces/<space_id>/space.db`
///   - `data_root/spaces/<space_id>/blobs/<h[:2]>/<h>`
pub fn restore_space(
    bundle_dir: &Path,
    data_root: &Path,
    space_id: &str,
    force: bool,
) -> Result<RestoreResult> {
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);
    let manifest_bytes = std::fs::read(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: SnapshotManifest = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    if manifest.space_id != space_id {
        return Err(anyhow!(
            "manifest space_id {} does not match requested {space_id}",
            manifest.space_id
        ));
    }
    if manifest.tool_version != TOOL_VERSION {
        return Err(anyhow!(
            "manifest tool_version {} does not match current {}",
            manifest.tool_version,
            TOOL_VERSION
        ));
    }

    let bundle_db = bundle_dir.join(BUNDLE_DB_FILE);
    let bundle_blobs = bundle_dir.join(BUNDLE_BLOBS_DIR);

    // ── integrity: verify db sha256 BEFORE we touch the destination ────────
    let db_bytes =
        std::fs::read(&bundle_db).with_context(|| format!("read {}", bundle_db.display()))?;
    let actual_db_hash = sha256_hex(&db_bytes);
    if actual_db_hash != manifest.db_sha256 {
        return Err(anyhow!(
            "snapshot db sha256 mismatch: expected {}, got {actual_db_hash}",
            manifest.db_sha256
        ));
    }
    if db_bytes.len() as u64 != manifest.db_bytes {
        return Err(anyhow!(
            "snapshot db size mismatch: expected {}, got {}",
            manifest.db_bytes,
            db_bytes.len()
        ));
    }

    let dest_dir = data_root.join(crate::db::SPACES_DIR).join(space_id);
    let dest_db = dest_dir.join(BUNDLE_DB_FILE);
    let dest_blobs = dest_dir.join(SPACE_BLOBS_DIR);

    if dest_db.exists() && !force {
        return Err(anyhow!(
            "space already exists at {} (refusing to overwrite; pass force to replace)",
            dest_db.display()
        ));
    }
    if dest_blobs.exists() && !force {
        return Err(anyhow!(
            "blobs dir already exists at {} (refusing to overwrite; pass force to replace)",
            dest_blobs.display()
        ));
    }

    // Lay down the canonical layout. db first (the integrity anchor),
    // then blobs — same ordering as the snapshot. The fan-out layout
    // (blobs/<h[:2]>/<h>) is preserved by the recursive copy.
    std::fs::create_dir_all(&dest_dir)
        .with_context(|| format!("create_dir_all {}", dest_dir.display()))?;
    fs_util::write_atomic(&dest_db, &db_bytes)
        .with_context(|| format!("write {}", dest_db.display()))?;
    let mut blob_count: u32 = 0;
    let mut blob_bytes_total: u64 = 0;
    if bundle_blobs.is_dir() {
        std::fs::create_dir_all(&dest_blobs)
            .with_context(|| format!("create_dir_all {}", dest_blobs.display()))?;
        copy_blobs(
            &bundle_blobs,
            &dest_blobs,
            &mut blob_count,
            &mut blob_bytes_total,
        )?;
    }

    Ok(RestoreResult {
        space_id: space_id.to_string(),
        data_root: data_root.to_path_buf(),
        db_path: dest_db,
        blob_count,
        blob_bytes: blob_bytes_total,
    })
}

/// On-disk blob path under a space's data root: `blobs/<h[:2]>/<h>`.
/// Mirrors `LocalFsBlobStore`'s `BlobScope::Space` layout — kept in one
/// place here so the two stay in lockstep.
#[cfg(test)]
pub(crate) fn blob_path_for(data_root: &Path, space_id: &str, hash: &str) -> PathBuf {
    let fan = &hash[..BLOB_FANOUT.min(hash.len())];
    data_root
        .join(crate::db::SPACES_DIR)
        .join(space_id)
        .join(SPACE_BLOBS_DIR)
        .join(fan)
        .join(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn ensure_space_db(data_root: &Path, space_id: &str) -> PathBuf {
        let p = space_db_path(data_root, space_id);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        // Open + write one row so the file materialises and is non-empty
        // (an empty db still snapshot's, but the bundled file would be
        // ~12 KiB of header).
        let mut conn = crate::db::open_space_db(data_root, space_id).unwrap();
        let _ = block_on(async {
            sqlx::query("CREATE TABLE IF NOT EXISTS ping (n INTEGER)")
                .execute(&mut conn)
                .await?;
            sqlx::query("INSERT INTO ping (n) VALUES (1)")
                .execute(&mut conn)
                .await
        })
        .unwrap();
        p
    }

    #[test]
    fn snapshot_creates_bundle_with_manifest_and_blobs() {
        let data_root = tempdir().unwrap();
        let target = tempdir().unwrap();
        let space_id = "snap-test";
        let _ = ensure_space_db(data_root.path(), space_id);

        // Plant two blobs.
        let blob1: &[u8] = b"hello world";
        let blob2: &[u8] = b"a much larger blob of bytes that should still match its content hash";
        let h1 = sha256_hex(blob1);
        let h2 = sha256_hex(blob2);
        for (h, bytes) in [(&h1, blob1), (&h2, blob2)] {
            let p = blob_path_for(data_root.path(), space_id, h);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs_util::write_atomic(&p, bytes).unwrap();
        }

        let res = snapshot_space(data_root.path(), space_id, target.path()).expect("snapshot ok");
        assert_eq!(res.space_id, space_id);
        assert!(res.path.is_dir());
        assert_eq!(res.blob_count, 2);
        assert_eq!(res.blob_bytes as usize, blob1.len() + blob2.len());
        assert!(res.db_bytes > 0);
        // db_sha256 should be a real hex digest.
        assert_eq!(res.db_sha256.len(), 64);

        // Manifest round-trips.
        let m_path = res.path.join(BUNDLE_MANIFEST);
        let m: SnapshotManifest = serde_json::from_slice(&std::fs::read(&m_path).unwrap()).unwrap();
        assert_eq!(m.space_id, space_id);
        assert_eq!(m.db_bytes, res.db_bytes);
        assert_eq!(m.db_sha256, res.db_sha256);
        assert_eq!(m.blob_count, 2);
        assert_eq!(m.tool_version, TOOL_VERSION);

        // Bundled blobs land at blobs/<h[:2]>/<h> and round-trip bytes-equal.
        let originals: &[(&[u8], &str)] = &[(blob1, &h1), (blob2, &h2)];
        for (bytes, h) in originals {
            let p = res.path.join(BUNDLE_BLOBS_DIR).join(&h[..2]).join(h);
            assert!(p.is_file(), "bundled blob missing: {}", p.display());
            assert_eq!(std::fs::read(&p).unwrap(), *bytes);
        }
    }

    #[test]
    fn restore_verifies_sha256_and_refuses_overwrite_without_force() {
        let data_root = tempdir().unwrap();
        let target = tempdir().unwrap();
        let space_id = "restore-test";
        let _ = ensure_space_db(data_root.path(), space_id);
        let bundle = snapshot_space(data_root.path(), space_id, target.path()).expect("snapshot");

        // First restore (no existing space) succeeds.
        let dest = tempdir().unwrap();
        let r1 = restore_space(&bundle.path, dest.path(), space_id, false).expect("restore");
        assert_eq!(r1.space_id, space_id);
        assert!(r1.db_path.is_file());
        assert_eq!(r1.blob_count, 0);

        // Second restore without force fails.
        let r2 = restore_space(&bundle.path, dest.path(), space_id, false);
        assert!(r2.is_err(), "second restore without force should fail");

        // With force, succeeds again.
        let r3 = restore_space(&bundle.path, dest.path(), space_id, true).expect("force restore");
        assert!(r3.db_path.is_file());
    }

    #[test]
    fn restore_rejects_tampered_db() {
        let data_root = tempdir().unwrap();
        let target = tempdir().unwrap();
        let space_id = "tamper";
        let _ = ensure_space_db(data_root.path(), space_id);
        let bundle = snapshot_space(data_root.path(), space_id, target.path()).expect("snapshot");

        // Corrupt the bundled db (append a byte past the existing content).
        let db = bundle.path.join(BUNDLE_DB_FILE);
        let mut bytes = std::fs::read(&db).unwrap();
        bytes.push(0xff);
        std::fs::write(&db, &bytes).unwrap();

        let dest = tempdir().unwrap();
        let r = restore_space(&bundle.path, dest.path(), space_id, false);
        assert!(r.is_err(), "tampered db must fail restore");
    }

    /// Live-writes test: spawn a writer task that hammers the live db
    /// while a snapshot is in flight. The bundled db must still pass
    /// `PRAGMA integrity_check` and contain a consistent prefix (the
    /// writer's later writes are simply not in the snapshot — that's
    /// the contract; this is *not* a checkpointing system).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn snapshot_under_live_writes_produces_consistent_db() {
        let data_root = tempdir().unwrap();
        let target = tempdir().unwrap();
        let space_id = "live-writes";
        let _ = ensure_space_db(data_root.path(), space_id);
        let db_path = space_db_path(data_root.path(), space_id);

        // Spawn a writer task that hammers the live db while we snapshot.
        let writer = {
            let db_path = db_path.clone();
            tokio::spawn(async move {
                for i in 0..50 {
                    if let Ok(mut conn) = crate::db::open_db_at(&db_path) {
                        let _ = sqlx::query(&format!("INSERT INTO ping (n) VALUES ({})", i + 1000))
                            .execute(&mut conn)
                            .await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
                }
            })
        };

        let res = snapshot_space(data_root.path(), space_id, target.path()).expect("snapshot");
        let _ = writer.await;

        // PRAGMA integrity_check on the bundled db.
        let mut conn = crate::db::open_db_at(&res.path.join(BUNDLE_DB_FILE)).unwrap();
        let rows: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
            .fetch_all(&mut conn)
            .await
            .unwrap();
        assert!(
            rows.len() == 1 && rows[0].0 == "ok",
            "bundled db must be integrity-clean; got {rows:?}"
        );
        // Bundled db must contain the table (a consistent prefix — even if
        // the writer raced, `VACUUM INTO` is crash-consistent).
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ping")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert!(count.0 >= 1, "bundled db must contain at least one row");
    }
}
