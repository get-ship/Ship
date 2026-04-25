//! One-shot home-layout migrator.
//!
//! Per ADR `home-layout-and-scope-convention`, `~/.ship/` splits into two
//! buckets: `library/` (portable user content) and `machine/` (host-local
//! runtime state). This module migrates pre-split installs into the canonical
//! shape on daemon start, idempotently.
//!
//! Strategy: stage all moves in a sibling temp directory under the same
//! filesystem (so `rename` is atomic), then commit by renaming entries into
//! place. On any failure mid-stage the temp dir is destroyed and the original
//! layout is left untouched.
//!
//! Detection: the marker file `<root>/machine/layout-version` is the single
//! source of truth. Present → no-op. Absent and `<root>/library/` exists →
//! partial-legacy state (hard error). Absent and pre-split content present →
//! migrate.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use super::ship_home::ShipHome;

/// Current layout-version marker schema. Bumped only when the canonical
/// shape changes again — never for silent fixes.
pub const LAYOUT_VERSION: u32 = 1;

const LAYOUT_VERSION_FILE: &str = "layout-version";

/// Directories migrated from root → `library/<name>/`.
const LIBRARY_DIRS: &[&str] = &[
    "agents",
    "skills",
    "views",
    "pipelines",
    "triggers",
    "rules",
    "permissions",
    "themes",
];

/// Runtime-state directories migrated to `machine/<name>/`.
const MACHINE_DIRS: &[&str] = &[
    "cache",
    "crashes",
    "logs",
    "sessions",
    "actors",
    "projects",
];

/// SQLite triplets (file + WAL + SHM) migrated to `machine/db/`.
const DB_FILES: &[&str] = &["platform.db", "data.db"];

/// Marker file written after a successful migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayoutMarker {
    version: u32,
    migrated_at: String,
}

/// Run the migrator once. Idempotent. Returns `Ok(())` on no-op as well as
/// successful migration.
pub fn migrate_home(home: &ShipHome) -> Result<()> {
    let root = home.root().to_path_buf();

    // No `~/.ship/` at all: caller will create the canonical layout fresh.
    if !root.exists() {
        fs::create_dir_all(&root)
            .with_context(|| format!("create ship home {}", root.display()))?;
        write_marker(&root)?;
        return Ok(());
    }

    if marker_path(&root).exists() {
        return Ok(()); // already migrated
    }

    let plan = inspect_layout(&root)?;
    if plan.is_empty() {
        // Fresh empty `~/.ship/` — write marker and exit.
        write_marker(&root)?;
        return Ok(());
    }

    if plan.partial_legacy {
        bail!(
            "ship home at {} is in a partial-legacy state. \
             Found `library/` or `machine/` but no `machine/layout-version` marker. \
             Inspect the directory by hand and either remove the partial split \
             or run a clean install. Refusing to migrate to avoid data loss.",
            root.display()
        );
    }

    let staging = staging_dir(&root)?;
    let result: Result<()> = (|| {
        stage_moves(&root, &staging, &plan)?;
        commit_moves(&root, &staging, &plan)?;
        write_marker(&root)?;
        Ok(())
    })();

    if let Err(e) = result {
        rollback(&root, &staging, &plan)?;
        return Err(e);
    }

    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

fn marker_path(root: &Path) -> PathBuf {
    root.join("machine").join(LAYOUT_VERSION_FILE)
}

fn write_marker(root: &Path) -> Result<()> {
    let machine = root.join("machine");
    fs::create_dir_all(&machine)
        .with_context(|| format!("create {}", machine.display()))?;
    fs::create_dir_all(root.join("library"))
        .with_context(|| format!("create {}", root.join("library").display()))?;
    let marker = LayoutMarker {
        version: LAYOUT_VERSION,
        migrated_at: chrono::Utc::now().to_rfc3339(),
    };
    let body = serde_json::to_string_pretty(&marker)?;
    fs::write(machine.join(LAYOUT_VERSION_FILE), body)
        .with_context(|| format!("write {}", marker_path(root).display()))?;
    Ok(())
}

/// What needs to move and where.
#[derive(Debug, Default)]
struct Plan {
    /// `(rel_src, rel_dst)` pairs relative to the home root.
    moves: Vec<(PathBuf, PathBuf)>,
    /// True if `library/` or `machine/` already exists without the marker —
    /// a sign that a prior partial run left the home in a mixed state.
    partial_legacy: bool,
    /// Whether a TOML config file is present at `<root>/config.toml`. Drives
    /// Phase 4 conversion inside the migration pass.
    config_toml: bool,
}

impl Plan {
    fn is_empty(&self) -> bool {
        self.moves.is_empty() && !self.config_toml
    }
}

fn inspect_layout(root: &Path) -> Result<Plan> {
    let mut plan = Plan::default();
    let library = root.join("library");
    let machine = root.join("machine");
    plan.partial_legacy = library.exists() || machine.exists();

    for name in LIBRARY_DIRS {
        let src = root.join(name);
        if src.is_dir() {
            plan.moves
                .push((PathBuf::from(name), PathBuf::from("library").join(name)));
        }
    }

    // schemas/ → library/collections/
    if root.join("schemas").is_dir() {
        plan.moves.push((
            PathBuf::from("schemas"),
            PathBuf::from("library").join("collections"),
        ));
    }

    // mcp.jsonc → library/mcp/config.jsonc (single file becomes a directory).
    if root.join("mcp.jsonc").is_file() {
        plan.moves.push((
            PathBuf::from("mcp.jsonc"),
            PathBuf::from("library").join("mcp").join("config.jsonc"),
        ));
    }

    for name in MACHINE_DIRS {
        let src = root.join(name);
        if src.is_dir() {
            plan.moves
                .push((PathBuf::from(name), PathBuf::from("machine").join(name)));
        }
    }

    // kernel/ subtree is dead architecture (per ADR §7) — drop it on
    // migration rather than carry forward. Stage it but never commit.
    // Tracked as a destination-less move so rollback reasoning is simple.
    if root.join("kernel").is_dir() {
        plan.moves.push((
            PathBuf::from("kernel"),
            PathBuf::from("machine").join(".discard").join("kernel"),
        ));
    }

    // Database triplets → machine/db/.
    for db in DB_FILES {
        for suffix in ["", "-shm", "-wal"] {
            let name = format!("{db}{suffix}");
            if root.join(&name).is_file() {
                plan.moves.push((
                    PathBuf::from(&name),
                    PathBuf::from("machine").join("db").join(&name),
                ));
            }
        }
    }
    if root.join("archive").is_dir() {
        plan.moves.push((
            PathBuf::from("archive"),
            PathBuf::from("machine").join("db").join("archive"),
        ));
    }

    // Daemon handshake files → machine/daemon/.
    for (src, dst) in [
        ("hook.token", "machine/daemon/token"),
        ("network.pid", "machine/daemon/pid"),
        ("daemon.port", "machine/daemon/port"),
        ("daemon.pid", "machine/daemon/pid"),
        ("daemon.token", "machine/daemon/token"),
    ] {
        if root.join(src).is_file() {
            plan.moves
                .push((PathBuf::from(src), PathBuf::from(dst)));
        }
    }

    plan.config_toml = root.join("config.toml").is_file();

    // If the only "evidence" is the partial_legacy flag (i.e. the user already
    // has `library/` or `machine/` but no marker AND no legacy content to
    // move), keep the partial-legacy flag set so we hard-error.
    Ok(plan)
}

fn staging_dir(root: &Path) -> Result<PathBuf> {
    let dir = root.join(".migrate-tmp");
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .with_context(|| format!("clear stale staging {}", dir.display()))?;
    }
    fs::create_dir_all(&dir)
        .with_context(|| format!("create staging {}", dir.display()))?;
    Ok(dir)
}

fn stage_moves(root: &Path, staging: &Path, plan: &Plan) -> Result<()> {
    // Track which destination tops we've created so we don't double-up.
    let mut made: BTreeSet<PathBuf> = BTreeSet::new();
    for (src_rel, dst_rel) in &plan.moves {
        let src = root.join(src_rel);
        let staged = staging.join(dst_rel);
        if let Some(parent) = staged.parent() {
            if made.insert(parent.to_path_buf()) {
                fs::create_dir_all(parent)
                    .with_context(|| format!("staging mkdir {}", parent.display()))?;
            }
        }
        rename_or_copy(&src, &staged)
            .with_context(|| format!("stage {} → {}", src.display(), staged.display()))?;
    }

    if plan.config_toml {
        let staged = staging.join("library").join("settings.jsonc");
        let local = staging.join("machine").join("local.jsonc");
        if let Some(p) = staged.parent() {
            fs::create_dir_all(p)?;
        }
        if let Some(p) = local.parent() {
            fs::create_dir_all(p)?;
        }
        let toml_text = fs::read_to_string(root.join("config.toml"))
            .context("read config.toml")?;
        let (settings, local_settings) = super::migrate_toml::split_user_config(&toml_text)
            .context("convert config.toml → settings.jsonc + local.jsonc")?;
        fs::write(&staged, settings)?;
        fs::write(&local, local_settings)?;
    }
    Ok(())
}

fn commit_moves(root: &Path, staging: &Path, plan: &Plan) -> Result<()> {
    // Ensure top-level buckets exist on the live tree.
    fs::create_dir_all(root.join("library"))?;
    fs::create_dir_all(root.join("machine"))?;

    // Walk staged children of library/ and machine/ and rename into place.
    for top in ["library", "machine"] {
        let staged_top = staging.join(top);
        if !staged_top.exists() {
            continue;
        }
        for entry in fs::read_dir(&staged_top)? {
            let entry = entry?;
            let name = entry.file_name();
            let dst = root.join(top).join(&name);
            if dst.exists() {
                bail!(
                    "destination already exists: {} (refusing to overwrite)",
                    dst.display()
                );
            }
            rename_or_copy(&entry.path(), &dst)
                .with_context(|| format!("commit {} → {}", entry.path().display(), dst.display()))?;
        }
    }

    // Remove the legacy sources that succeeded (the rename above moved them
    // into staging, so the originals are already gone — but `config.toml`
    // and the `.discard/` kernel subtree are special cases).
    if plan.config_toml {
        let _ = fs::remove_file(root.join("config.toml"));
    }
    let discard = root.join("machine").join(".discard");
    if discard.exists() {
        fs::remove_dir_all(&discard)
            .with_context(|| format!("remove discard {}", discard.display()))?;
    }
    Ok(())
}

fn rollback(root: &Path, staging: &Path, plan: &Plan) -> Result<()> {
    // Best-effort: move anything still in staging back to its original
    // location. The `config.toml` source is never deleted before commit, so
    // it's already in place on failure.
    for (src_rel, dst_rel) in &plan.moves {
        let staged = staging.join(dst_rel);
        let original = root.join(src_rel);
        if !staged.exists() || original.exists() {
            continue;
        }
        if let Some(parent) = original.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = rename_or_copy(&staged, &original);
    }
    let _ = fs::remove_dir_all(staging);
    Ok(())
}

/// Rename `src` to `dst`, falling back to a recursive copy + remove when the
/// rename crosses a filesystem boundary (`EXDEV`).
fn rename_or_copy(src: &Path, dst: &Path) -> io::Result<()> {
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc_exdev()) => {
            copy_recursive(src, dst)?;
            if src.is_dir() {
                fs::remove_dir_all(src)?;
            } else {
                fs::remove_file(src)?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn libc_exdev() -> i32 {
    // EXDEV is 18 on Linux, 18 on macOS. We hard-code rather than pull `libc`
    // because the migrator runs on every daemon start and the dependency
    // surface should stay tiny.
    18
}

fn copy_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// Read the layout-version marker if present. Surfaces invalid JSON as an
/// error so callers can refuse to start on a corrupted marker.
pub fn read_layout_version(home: &ShipHome) -> Result<Option<u32>> {
    let path = marker_path(home.root());
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let marker: LayoutMarker = serde_json::from_str(&body)
        .map_err(|e| anyhow!("parse {}: {e}", path.display()))?;
    Ok(Some(marker.version))
}

#[cfg(test)]
#[path = "migrate_tests.rs"]
mod migrate_tests;
