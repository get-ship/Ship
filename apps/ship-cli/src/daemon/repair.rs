use anyhow::{Context, Result};

/// Read-only migration drift diagnostic. Reports how `platform.db`'s
/// `_sqlx_migrations` log relates to the migration set this binary embeds —
/// and changes nothing. The tolerant migration runner (`db::migrate`)
/// reconciles a DB that is merely *ahead* of the binary on its own, so the
/// old mutating escape hatches (`--mark-applied`, `--prune-orphans`,
/// checksum patching) are gone. What's left is the tool for understanding
/// the one drift the runner still rejects: an applied migration whose
/// checksum no longer matches its embedded source.
pub(super) fn daemon_repair_migrations(reset: bool) -> Result<()> {
    use ship_substrate::db::repair;

    let global_dir = ship_substrate::project::get_global_dir()
        .context("resolve global ship dir")?;
    let db_path = ship_substrate::db::platform_db_path(&global_dir);
    if !db_path.exists() {
        println!("no platform.db at {} — nothing to report", db_path.display());
        return Ok(());
    }
    let mut conn = ship_substrate::db::open_db_at(&db_path)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;

    let report = rt.block_on(async { repair::migration_drift(&mut conn).await })?;

    println!("migration drift: {}", db_path.display());
    println!("  in sync:   {} migrations", report.in_sync.len());
    if !report.pending.is_empty() {
        println!(
            "  pending (embedded but not yet applied — runs on next start): {:?}",
            report.pending
        );
    }
    if !report.db_ahead.is_empty() {
        println!(
            "  db ahead (in the log but not in this binary — tolerated, no action): {:?}",
            report.db_ahead
        );
    }
    if !report.checksum_mismatch.is_empty() {
        println!(
            "  checksum mismatch (applied migration was edited): {:?}",
            report.checksum_mismatch
        );
        if !reset {
            println!(
                "  → run `ship daemon repair-migrations --reset` to back up and \
                 recreate platform.db (pre-launch escape hatch)."
            );
        }
    }
    if report.is_clean() {
        println!("clean — the DB and this binary's migration set agree.");
        return Ok(());
    }

    if reset && !report.checksum_mismatch.is_empty() {
        // Close the connection before moving the file.
        drop(conn);
        let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        for suffix in ["", "-wal", "-shm"] {
            let src = db_path.with_extension(format!("db{suffix}"));
            if !src.exists() {
                continue;
            }
            let bak = src.with_extension(format!("db{suffix}.bak-{ts}"));
            std::fs::rename(&src, &bak)
                .with_context(|| format!("backup {} → {}", src.display(), bak.display()))?;
            println!("  backed up {} → {}", src.display(), bak.display());
        }
        println!(
            "reset complete — next `ship daemon start` re-applies migrations from scratch."
        );
    } else if reset {
        println!("--reset specified but no checksum mismatch — nothing to do.");
    }
    Ok(())
}
