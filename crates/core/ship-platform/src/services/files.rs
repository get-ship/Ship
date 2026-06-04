//! Shared filesystem helper for domain services that persist to disk.
//!
//! One source of truth for reading/writing under `.ship/` and `~/.ship/`.
//! Every file-backed service (agents, skills, rules, mcp, permissions,
//! pipelines, views, ...) delegates I/O here — path scoping, JSONC
//! round-trip, atomic writes, and trash-on-delete are uniform.
//!
//! No HTTP surface; this module is internal to the daemon.
//!
//! # Path scoping
//!
//! All paths are **relative to the scope root**. Absolute paths are
//! rejected. Symlinks are followed but the resolved target must stay
//! inside the scope root — attempts to escape (via `..` or crossing
//! symlinks) return `Error::PathEscape`.
//!
//! # JSONC
//!
//! Reads strip line/block comments + trailing commas before deserialising.
//! Writes emit plain pretty JSON; comments on the *unchanged* keys survive
//! because we never walk them, but comments on *edited* keys are lost
//! (best-effort; span-preserving round-trip is a follow-up).
//!
//! # Atomic writes
//!
//! Every write goes through `<path>.tmp` + `fsync` + `rename`. No partial
//! files on crash. The tmp sidecar shares the same directory so the
//! rename is atomic on POSIX filesystems.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Scope selector. Project is the enclosing repo's `.ship/` resolved via
/// `ship_substrate::project::get_project_dir`. User is `~/.ship/`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Project,
    User,
}

#[derive(Debug)]
pub enum Error {
    /// Path escaped the scope root (absolute, `..`, symlink crossing).
    PathEscape(PathBuf),
    /// Requested scope is not configured (no project root discovered).
    ScopeUnavailable(Scope),
    /// Target already exists and the operation refused to overwrite.
    AlreadyExists(PathBuf),
    /// Target does not exist.
    NotFound(PathBuf),
    /// JSON(C) parse / serialise error.
    Json(String),
    /// JSON Patch application failed.
    Patch(String),
    /// Underlying filesystem I/O error.
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathEscape(p) => write!(f, "path escapes scope root: {}", p.display()),
            Self::ScopeUnavailable(s) => write!(f, "scope {s:?} is not configured"),
            Self::AlreadyExists(p) => write!(f, "{} already exists", p.display()),
            Self::NotFound(p) => write!(f, "{} not found", p.display()),
            Self::Json(msg) => write!(f, "json: {msg}"),
            Self::Patch(msg) => write!(f, "patch: {msg}"),
            Self::Io(e) => write!(f, "io: {e}"),
        }
    }
}
impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self { Self::Io(value) }
}

pub struct FilesService {
    project_root: Option<PathBuf>,
    user_root: PathBuf,
}

impl FilesService {
    /// Construct from a resolved daemon ship_dir (the user root) and an
    /// optional project ship_dir (typically discovered via
    /// `ship_substrate::project::get_project_dir`).
    pub fn new(user_root: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self { project_root, user_root }
    }

    /// Discover roots from the current process environment. The user root
    /// is the daemon's global `.ship/`; the project root is the enclosing
    /// project's `.ship/` if present.
    pub fn resolve_roots(ship_dir: &Path) -> Self {
        let project_root = ship_substrate::project::get_project_dir(None)
            .ok()
            .filter(|p| p != ship_dir);
        Self {
            project_root,
            user_root: ship_dir.to_path_buf(),
        }
    }

    fn root(&self, scope: Scope) -> Result<&Path, Error> {
        match scope {
            Scope::Project => self
                .project_root
                .as_deref()
                .ok_or(Error::ScopeUnavailable(Scope::Project)),
            Scope::User => Ok(&self.user_root),
        }
    }

    /// The `.ship` scope-root dir for `scope` (project's `.ship` or the user
    /// `.ship`). [`Error::ScopeUnavailable`] when project scope has no
    /// discovered root. Domains that route disk through a substrate port (the
    /// agent-def write door) resolve the scope→dir here, then hand the dir to
    /// the gated broker.
    pub fn scope_root(&self, scope: Scope) -> Result<&Path, Error> {
        self.root(scope)
    }

    /// Resolve a relative path against a scope root, rejecting escapes.
    /// Accepts non-existent paths so writes can create new files; only
    /// the parent chain is path-canonicalised for validation.
    pub fn resolve(&self, scope: Scope, rel: &Path) -> Result<PathBuf, Error> {
        if rel.is_absolute() {
            return Err(Error::PathEscape(rel.to_path_buf()));
        }
        let root = self.root(scope)?;
        let joined = root.join(rel);

        // Walk existing ancestors to catch `..` or symlink escapes. We
        // canonicalise the longest existing prefix and verify it's still
        // inside the root; anything past that is a new-file suffix and
        // can't leak because we control the prefix.
        let mut probe = joined.as_path();
        let canonical_root = fs::canonicalize(root).map_err(Error::Io)?;
        loop {
            match fs::canonicalize(probe) {
                Ok(real) => {
                    if !real.starts_with(&canonical_root) {
                        return Err(Error::PathEscape(rel.to_path_buf()));
                    }
                    break;
                }
                Err(_) => {
                    // Climb to parent until we find something that exists.
                    match probe.parent() {
                        Some(p) if !p.as_os_str().is_empty() => probe = p,
                        _ => break,
                    }
                }
            }
        }
        Ok(joined)
    }

    // ── Read ──────────────────────────────────────────────────────────

    pub fn exists(&self, scope: Scope, rel: &Path) -> bool {
        self.resolve(scope, rel).map(|p| p.exists()).unwrap_or(false)
    }

    pub fn read(&self, scope: Scope, rel: &Path) -> Result<Vec<u8>, Error> {
        let path = self.resolve(scope, rel)?;
        fs::read(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound(path.clone())
            } else {
                Error::Io(e)
            }
        })
    }

    pub fn read_string(&self, scope: Scope, rel: &Path) -> Result<String, Error> {
        let bytes = self.read(scope, rel)?;
        String::from_utf8(bytes).map_err(|e| Error::Json(format!("utf-8: {e}")))
    }

    /// Parse a JSONC document into `T`. Strips comments + trailing commas
    /// before deserialising.
    pub fn read_jsonc<T: DeserializeOwned>(&self, scope: Scope, rel: &Path) -> Result<T, Error> {
        let raw = self.read_string(scope, rel)?;
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(&raw);
        serde_json::from_str(&stripped).map_err(|e| Error::Json(e.to_string()))
    }

    /// Direct access to the parsed JSON value — useful for services that
    /// patch and re-serialise without a typed round-trip.
    pub fn read_jsonc_value(&self, scope: Scope, rel: &Path) -> Result<serde_json::Value, Error> {
        self.read_jsonc(scope, rel)
    }

    /// List immediate children of a directory. Names only (file stems
    /// via `basename`). Non-existent directories return empty.
    pub fn list_dir(&self, scope: Scope, rel: &Path) -> Result<Vec<PathBuf>, Error> {
        let path = self.resolve(scope, rel)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&path)? {
            out.push(entry?.path());
        }
        out.sort();
        Ok(out)
    }

    // ── Write ─────────────────────────────────────────────────────────

    /// Atomic write: `<path>.tmp` → fsync → rename. Creates parent dirs.
    pub fn write(&self, scope: Scope, rel: &Path, bytes: &[u8]) -> Result<(), Error> {
        let path = self.resolve(scope, rel)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(Error::Io)?;
        }
        let tmp = path.with_extension(tmp_extension(&path));
        {
            let mut file = fs::File::create(&tmp).map_err(Error::Io)?;
            file.write_all(bytes).map_err(Error::Io)?;
            file.sync_all().map_err(Error::Io)?;
        }
        fs::rename(&tmp, &path).map_err(Error::Io)
    }

    /// Serialise `value` to pretty JSON and write atomically.
    pub fn write_jsonc<T: Serialize>(
        &self,
        scope: Scope,
        rel: &Path,
        value: &T,
    ) -> Result<(), Error> {
        let body =
            serde_json::to_string_pretty(value).map_err(|e| Error::Json(e.to_string()))?;
        self.write(scope, rel, body.as_bytes())
    }

    /// Read JSONC → apply RFC 6902 patch → write. Best-effort comment
    /// preservation (comments on untouched keys survive, touched keys
    /// reflow).
    pub fn patch_jsonc(
        &self,
        scope: Scope,
        rel: &Path,
        patch: &json_patch::Patch,
    ) -> Result<(), Error> {
        let mut doc: serde_json::Value = self.read_jsonc(scope, rel)?;
        json_patch::patch(&mut doc, patch).map_err(|e| Error::Patch(e.to_string()))?;
        self.write_jsonc(scope, rel, &doc)
    }

    /// Refuse-on-overwrite variant of `write` for create flows.
    pub fn create(&self, scope: Scope, rel: &Path, bytes: &[u8]) -> Result<(), Error> {
        let path = self.resolve(scope, rel)?;
        if path.exists() {
            return Err(Error::AlreadyExists(path));
        }
        self.write(scope, rel, bytes)
    }

    // ── Trash (soft delete) ───────────────────────────────────────────

    /// Move `rel` under `<scope_root>/.trash/<timestamp>-<basename>`.
    /// No hard delete; callers who want permanent deletion walk `.trash/`.
    pub fn trash(&self, scope: Scope, rel: &Path) -> Result<(), Error> {
        let path = self.resolve(scope, rel)?;
        if !path.exists() {
            return Err(Error::NotFound(path));
        }
        let root = self.root(scope)?;
        let trash = root.join(".trash");
        fs::create_dir_all(&trash).map_err(Error::Io)?;
        let basename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let dest = trash.join(format!("{ts}-{basename}"));
        fs::rename(&path, &dest).map_err(Error::Io)
    }
}

fn tmp_extension(path: &Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.is_empty() {
        "tmp".into()
    } else {
        format!("{ext}.tmp")
    }
}

#[cfg(test)]
#[path = "files_tests.rs"]
mod tests;
