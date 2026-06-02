//! Typed config-document accessor — substrate owns the disk for `.ship/` config
//! files.
//!
//! The platform config service holds product logic (precedence, effective
//! merges); the bytes on disk belong to substrate. This module is the single
//! seam through which a typed JSONC document is read, parsed (comment-stripped),
//! and written atomically at a `ShipHome` path. L1 must not call `std::fs`
//! directly for config files — it routes through here.
//!
//! Documents are `serde`-serializable values persisted as pretty JSON with
//! JSONC comment tolerance on read. Hand-written comments survive only while a
//! document is not re-serialized; structural rewrites drop them (the format is
//! JSONC-on-read, JSON-on-write).

use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Read and parse a JSONC config document at `path`.
///
/// Returns `Ok(None)` when the file does not exist — callers decide whether a
/// missing document means "default". Returns `Err` only on I/O failures other
/// than not-found, or on parse failure of an existing document.
pub fn read_doc<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    Ok(Some(parse_doc(&raw)?))
}

/// Read a JSONC config document at `path`, falling back to `T::default()` when
/// the file is missing or unparseable.
///
/// This mirrors the historical lenient load behavior of `DiskBackedConfig`:
/// a corrupt or absent file yields defaults rather than an error, so a single
/// bad hand-edit never bricks the daemon.
pub fn read_doc_or_default<T: DeserializeOwned + Default>(path: &Path) -> T {
    match std::fs::read_to_string(path) {
        Ok(raw) => parse_doc(&raw).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// Parse a JSONC string into `T` by stripping comments first.
pub fn parse_doc<T: DeserializeOwned>(raw: &str) -> Result<T> {
    let stripped = crate::fs_util::strip_jsonc_comments(raw);
    Ok(serde_json::from_str(&stripped)?)
}

/// Atomically write `value` as pretty JSON to `path`, creating parent dirs.
///
/// Uses `fs_util::write_atomic` so a crash mid-write never leaves a partial
/// file. A trailing newline is appended to match editor conventions.
pub fn write_doc<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let body = serde_json::to_string_pretty(value)?;
    crate::fs_util::write_atomic(path, format!("{body}\n"))
}

/// Like [`write_doc`], then restrict the file to owner-only (`chmod 600`) on
/// Unix. For machine-local secrets (credentials). No-op permission step on
/// non-Unix platforms.
pub fn write_secret_doc<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    write_doc(path, value)?;
    restrict_to_owner(path);
    Ok(())
}

/// Best-effort `chmod 600` on Unix; no-op elsewhere.
fn restrict_to_owner(_path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(_path, std::fs::Permissions::from_mode(0o600));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Doc {
        name: String,
        count: u32,
    }

    #[test]
    fn read_missing_returns_none() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("nested/missing.jsonc");
        let got: Option<Doc> = read_doc(&path).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn write_then_read_round_trips() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("nested/doc.jsonc");
        let doc = Doc { name: "ship".into(), count: 7 };
        write_doc(&path, &doc).unwrap();

        let back: Doc = read_doc(&path).unwrap().unwrap();
        assert_eq!(back, doc);
    }

    #[test]
    fn write_creates_parent_dirs() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a/b/c/doc.jsonc");
        write_doc(&path, &Doc::default()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn read_strips_jsonc_comments() {
        let raw = r#"
        // a doc
        {
          "name": "ship", // inline
          /* block */ "count": 3,
        }
        "#;
        let doc: Doc = parse_doc(raw).unwrap();
        assert_eq!(doc, Doc { name: "ship".into(), count: 3 });
    }

    #[test]
    fn read_or_default_on_missing() {
        let tmp = tempdir().unwrap();
        let doc: Doc = read_doc_or_default(&tmp.path().join("nope.jsonc"));
        assert_eq!(doc, Doc::default());
    }

    #[test]
    fn read_or_default_on_corrupt() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("corrupt.jsonc");
        crate::fs_util::write_atomic(&path, "{ this is not json").unwrap();
        let doc: Doc = read_doc_or_default(&path);
        assert_eq!(doc, Doc::default());
    }

    #[test]
    fn read_doc_errors_on_corrupt_existing() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("corrupt.jsonc");
        crate::fs_util::write_atomic(&path, "{ nope").unwrap();
        let got: Result<Option<Doc>> = read_doc(&path);
        assert!(got.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn secret_doc_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("secret.jsonc");
        write_secret_doc(&path, &Doc::default()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
