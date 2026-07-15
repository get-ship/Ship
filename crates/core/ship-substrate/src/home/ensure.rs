//! Ensure the canonical XDG-split directory layout exists.
//!
//! On daemon start we just `mkdir -p` the four XDG roots plus the daemon
//! subdirs they own. There is no installed user base; nothing to
//! migrate. Per the home-layout ADR, no other code path may join these
//! paths directly — go through the typed `ShipHome` accessors.

use std::fs;
use std::io;

use crate::ShipHome;

/// Idempotent. Creates the four XDG roots and their daemon-owned
/// subdirs. Layout:
///
/// - `data/spaces/`
/// - `state/logs/`
/// - `state/crashes/`
/// - `state/daemon/`
/// - `cache/trash/`
/// - `runtime/daemon/`
///
/// Per XDG slice B, db files (`runtime.db` / `account.db` / `kv.db` /
/// `spaces/<id>/space.db`) live flat at the data root — no `data/db/`
/// subdir. The db openers provision their files on first write; this
/// helper does not manufacture base-shaped subdirs inside data.
pub fn ensure_layout(home: &ShipHome) -> io::Result<()> {
    // Data root.
    fs::create_dir_all(home.data_dir().join("spaces"))?;
    // State root.
    fs::create_dir_all(home.state_dir().join("logs"))?;
    fs::create_dir_all(home.state_dir().join("crashes"))?;
    fs::create_dir_all(home.state_dir().join("daemon"))?;
    // Cache root.
    fs::create_dir_all(home.cache_dir().join("trash"))?;
    // Runtime root (collapses to state when XDG_RUNTIME_DIR is unset).
    fs::create_dir_all(home.daemon_dir())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_canonical_layout_from_empty() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        ensure_layout(&home).unwrap();

        // Data root: spaces (db files live flat at the data root, NOT
        // under data/db/ — the openers provision on first write; the
        // layout helper does not manufacture base-shaped subdirs inside
        // data).
        assert!(home.data_dir().join("spaces").is_dir());
        assert!(home.state_dir().join("logs").is_dir());
        assert!(home.state_dir().join("crashes").is_dir());
        assert!(home.state_dir().join("daemon").is_dir());
        // Cache root: trash.
        assert!(home.cache_dir().join("trash").is_dir());
        // Runtime root: daemon dir (runtime collapses to state under
        // at() / for_test()).
        assert!(home.daemon_dir().is_dir());
    }

    #[test]
    fn is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        ensure_layout(&home).unwrap();
        ensure_layout(&home).unwrap();
        ensure_layout(&home).unwrap();
        assert!(home.state_dir().join("daemon").is_dir());
    }

    #[test]
    fn preserves_existing_content() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        ensure_layout(&home).unwrap();

        let token = home.daemon_file(super::super::daemon::DaemonFile::Token);
        std::fs::write(&token, "preserved").unwrap();

        ensure_layout(&home).unwrap();

        assert_eq!(std::fs::read_to_string(&token).unwrap(), "preserved");
    }
}
