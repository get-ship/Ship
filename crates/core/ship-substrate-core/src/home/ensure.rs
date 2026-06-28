//! Ensure the canonical `~/.ship/` directory layout exists.
//!
//! Pre-launch we ship a single layout (`library/` + `machine/<bucket>/`).
//! There is no installed user base; nothing to migrate. On daemon start
//! we just `mkdir -p` the canonical dirs and move on.

use std::fs;
use std::io;

use crate::ShipHome;

/// Idempotent. Creates the canonical dir tree under the given home root.
pub fn ensure_layout(home: &ShipHome) -> io::Result<()> {
    let root = home.root();
    fs::create_dir_all(home.library_dir())?;
    let machine = home.machine_dir();
    fs::create_dir_all(&machine)?;
    for sub in [
        "db",
        "daemon",
        "logs",
        "cache",
        "crashes",
    ] {
        fs::create_dir_all(machine.join(sub))?;
    }
    let _ = root; // silence unused warning if root() is ever inlined away.
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

        assert!(home.library_dir().is_dir());
        assert!(home.machine_dir().is_dir());
        for sub in ["db", "daemon", "logs", "cache", "crashes"] {
            assert!(
                home.machine_dir().join(sub).is_dir(),
                "machine/{sub} must exist"
            );
        }
    }

    #[test]
    fn is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        ensure_layout(&home).unwrap();
        ensure_layout(&home).unwrap();
        ensure_layout(&home).unwrap();
        assert!(home.machine_dir().join("daemon").is_dir());
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
