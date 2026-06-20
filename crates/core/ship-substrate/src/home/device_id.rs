//! Per-machine device identity — on-disk read + mint.
//!
//! The device id lives at `<ShipHome::device_id_path()>` (i.e.
//! `~/.ship/machine/device-id`), outside `library/` so it does not
//! sync across machines and outside `machine/daemons/<hash>/` so it
//! survives per-worktree daemon churn. Mint-once: the first call
//! installs a fresh ULID; every subsequent call reads the same value
//! back. The daemon boot path in `apps/shipd` calls
//! `set_local_device_id` on the result so `ship_types::local_device_id`
//! returns this id for every scope binding.
//!
//! Mirrors the posture of `apps/shipd/src/auth/token.rs::read_or_generate_token`
//! (Unix-only chmod, parent-dir create, propagate IO errors) — the device id
//! is a per-machine secret, not per-daemon, so cleanup paths must NOT delete
//! it.

use std::io;

use ship_types::DeviceId;

use super::ShipHome;

/// Read the on-disk device id, or mint + persist a fresh one when the
/// file is missing or empty.
///
/// The minted id is a monotonic ULID prefixed with `dev_` so it stays
/// human-recognizable in logs and `~/.ship/machine/`. The parent
/// machine directory is created (mode 0700 on Unix) and the id file is
/// written with mode 0600 on Unix; non-Unix targets get the file
/// un-chmodded but the rest of the path is identical.
pub fn read_or_generate_device_id(home: &ShipHome) -> io::Result<DeviceId> {
    let path = home.device_id_path();
    match read_existing(&path) {
        Ok(Some(id)) => Ok(id),
        Ok(None) => {
            let id = mint();
            write_atomic(home, &path, id.as_str())?;
            Ok(id)
        }
        Err(e) => Err(e),
    }
}

fn read_existing(path: &std::path::Path) -> io::Result<Option<DeviceId>> {
    match std::fs::read_to_string(path) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(DeviceId::new(trimmed)))
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn mint() -> DeviceId {
    DeviceId::new(format!("dev_{}", crate::gen_ulid()))
}

fn write_atomic(home: &ShipHome, path: &std::path::Path, id: &str) -> io::Result<()> {
    let machine_dir = home.machine_dir();
    std::fs::create_dir_all(&machine_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&machine_dir, std::fs::Permissions::from_mode(0o700))?;
    }
    std::fs::write(path, id)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn mints_then_reads_same_id() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());

        let first = read_or_generate_device_id(&home).expect("first call mints");
        let second = read_or_generate_device_id(&home).expect("second call reads");
        let third = read_or_generate_device_id(&home).expect("third call reads");

        assert_eq!(first, second);
        assert_eq!(second, third);
        assert!(
            first.as_str().starts_with("dev_"),
            "minted id must carry the dev_ prefix: {}",
            first.as_str()
        );

        let on_disk = std::fs::read_to_string(home.device_id_path()).unwrap();
        assert_eq!(on_disk.trim(), first.as_str());
    }

    #[test]
    fn empty_file_is_treated_as_missing() {
        let tmp = tempdir().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let path = home.device_id_path();
        std::fs::create_dir_all(home.machine_dir()).unwrap();
        std::fs::write(&path, "").unwrap();

        let id = read_or_generate_device_id(&home).expect("empty file → mint");
        assert!(id.as_str().starts_with("dev_"));
    }
}
