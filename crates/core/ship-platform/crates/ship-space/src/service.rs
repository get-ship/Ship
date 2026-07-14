//! Space service — core business logic for the SpaceService RPCs.
//!
//! All storage goes through `SpaceContext.data()` (DataStore port); all events
//! go through `SpaceContext.kernel()` (KernelLog port). No sqlite/sqlx/rusqlite
//! types and no raw fs here.
//!
//! The spoke model after the 2026-07-09 cutover:
//!
//! - **registry** is the `spaces` Runtime-scoped projection — the device-local
//!   record of opened spaces (LaunchServices analogy). Identity lives in the
//!   projection, never at the path. The single `Add` RPC is CREATE-new-at-
//!   location: it mints identity once (through `identity::ensure_identity_for_root`).
//! - **identity** is the served-space read: `Info` returns the runtime's
//!   served space's identity and name (today resolved through the live
//!   `ctx`'s `SpaceId`). Per-space settings are read through the unified
//!   `settings_overlay` (S4) — owned by `SettingsService`.
//! - **install lifecycle** (`install_stack` / `unregister_stack` /
//!   `list_installed`) is the registry-resolved stack surface. The service
//!   holds the [`Registry`] + `TrustedKey` set so the wire trait can stay
//!   `async fn` against `&self`; the install pipeline itself lives in
//!   [`ship_store::install_stack`].
//!
//! `Activate` is GONE (multi-window kills the open-one-at-a-time world).
//! The active pointer table that supported it was retired; only
//! per-space projection rows remain.

use std::path::PathBuf;
use std::sync::Arc;

use ship_store::{
    BundleFetcher, InstallError, InstalledStack, Registry, TrustedKey, install_stack as store_install,
    library_root, remove_installed as store_remove,
};
use ship_substrate::ports::SpaceContext;
use ship_types::payload::{StackInstalled, StackUnregistered};
use ship_types::{envelope::EventEnvelope, event_types};

use crate::error::SpaceError;
use crate::identity;
use crate::spaces;
use crate::types::{SpaceInfo, SpaceRecord};

/// Core service — stateless except for the install-pipeline handles
/// (registry + trusted keys), which the wire layer attaches at mount time.
/// These are `Option`s so the service can serve a SpaceService without
/// installing stacks (e.g. helmd's snapshot-only mount).
pub struct SpaceService {
    /// The ctx carries the served-space `SpaceId` (entity id) the runtime
    /// minted at boot via `identity::ensure_identity_for_root`. All reads of
    /// identity reach through this id; the wire layer wires the request's
    /// `SpaceContext` per-request so a future per-window model stays
    /// coherent.
    pub ctx: Arc<SpaceContext>,
    /// The install pipeline's gateway client. `None` when the mount doesn't
    /// support install (e.g. the snapshot-only helmd surface).
    pub registry: Option<Arc<dyn Registry + Send + Sync>>,
    /// Trusted publisher keys for the install attestation verifier.
    pub trusted_keys: Vec<TrustedKey>,
    /// The data dir under which `<data>/library/<path>/<version>/` materialises.
    /// Required when `registry` is set; ignored otherwise.
    pub library_root_parent: Option<PathBuf>,
}

impl SpaceService {
    /// New install-disabled service — snapshots, identity, etc., only.
    pub fn new(ctx: Arc<SpaceContext>) -> Self {
        Self {
            ctx,
            registry: None,
            trusted_keys: Vec::new(),
            library_root_parent: None,
        }
    }

    /// New install-enabled service — pass the gateway client + trusted keys +
    /// the data dir to materialise library rows under.
    pub fn with_install(
        ctx: Arc<SpaceContext>,
        registry: Arc<dyn Registry + Send + Sync>,
        trusted_keys: Vec<TrustedKey>,
        library_root_parent: PathBuf,
    ) -> Self {
        Self {
            ctx,
            registry: Some(registry),
            trusted_keys,
            library_root_parent: Some(library_root_parent),
        }
    }

    /// Backwards-compatible constructor that ALSO accepts the install
    /// handles — used when the wire layer pre-resolves the bundle fetcher
    /// variant (e.g. tests). Identical to [`Self::with_install`] but spells
    /// the generic out for callers that want to name a typed fetcher.
    pub fn with_install_fetcher<F: BundleFetcher + 'static>(
        ctx: Arc<SpaceContext>,
        gateway_base_url: &str,
        fetcher: F,
        trusted_keys: Vec<TrustedKey>,
        library_root_parent: PathBuf,
    ) -> Result<Self, SpaceError> {
        let registry: Arc<dyn Registry + Send + Sync> = Arc::new(
            ship_store::HttpRegistry::with_fetcher(gateway_base_url, fetcher)
                .map_err(|e| SpaceError::Internal(format!("building registry client: {e}")))?,
        );
        Ok(Self::with_install(ctx, registry, trusted_keys, library_root_parent))
    }

    // ── opened-spaces list (Runtime-scoped) ─────────────────────────────────

    /// List every space currently open on this device.
    pub fn list(&self) -> Result<Vec<SpaceRecord>, SpaceError> {
        spaces::list(self.ctx.kernel())
    }

    // ── served-space identity (per-request) ──────────────────────────────────

    /// Compose the served-space identity for THIS request's ctx. Returns an
    /// empty identity when the ctx is the global/space-less Runtime slot
    /// (no served space) — the wire then emits an empty `SpaceInfo`.
    pub fn info(&self) -> Result<SpaceInfo, SpaceError> {
        // The per-request `SpaceContext` carries the served space's id via
        // its `SpaceId` slot. Globally-scoped requests (the Runtime
        // partition) carry an empty id; the wire returns an empty identity
        // so the FE never invents a `global` pseudo-space.
        let id = self.ctx.space_id().as_str();
        if id.is_empty() {
            Ok(SpaceInfo {
                space_id: None,
                root: None,
                name: None,
            })
        } else {
            // The served-space row lives in the Runtime-scoped `spaces`
            // projection. Fetch it so the Info reply carries the canonical
            // path + the projection's recorded name (the user-given name,
            // not the path-derived default).
            let rec = spaces::get(self.ctx.kernel(), id)?;
            Ok(match rec {
                Some(r) => SpaceInfo {
                    space_id: Some(r.space_id),
                    root: Some(r.path),
                    name: Some(r.name),
                },
                None => SpaceInfo {
                    space_id: Some(id.to_string()),
                    root: None,
                    name: None,
                },
            })
        }
    }

    // ── add (CREATE-new-at-location) ───────────────────────────────────────

    /// CREATE a new space at the location `path`. Mints identity once (an
    /// already-open path re-emits `space.seen`; a fresh one mints a
    /// nanoid + emits `space.opened`). Idempotent: re-opening an existing
    /// space returns its existing identity with a bumped `last_seen_at`.
    pub async fn add(
        &self,
        path: &std::path::Path,
        name: Option<&str>,
    ) -> Result<SpaceRecord, SpaceError> {
        identity::ensure_identity_for_root(self.ctx.kernel(), path, name).await
    }

    // ── stack membership ───────────────────────────────────────────────────

    /// Register `root_path` as a member stack of the space. Membership is
    /// REGISTRATION — the composed space is exactly the registered set. The
    /// root must carry the stack marker (`stack.config.ts`); registration is
    /// idempotent (re-registering bumps `registered_at`).
    pub async fn register_stack(
        &self,
        space_id: &str,
        root_path: &std::path::Path,
    ) -> Result<ship_types::payload::StackRegistered, SpaceError> {
        let resolved = resolve_space_id(self.ctx.space_id().as_str(), space_id)?;
        if !root_path.is_absolute() {
            return Err(SpaceError::BadArgs(format!(
                "root_path must be an absolute path: {}",
                root_path.display()
            )));
        }
        let marker = root_path.join(ship_substrate::project::STACK_CONFIG_FILENAME);
        if !marker.is_file() {
            return Err(SpaceError::BadArgs(format!(
                "not a stack root: no stack.config.ts in {}",
                root_path.display()
            )));
        }
        let canonical = std::fs::canonicalize(root_path)
            .map_err(|e| SpaceError::BadArgs(format!("root_path: {e}")))?;
        let payload = ship_types::payload::StackRegistered {
            root_path: canonical.to_string_lossy().into_owned(),
            registered_at: crate::helpers::now_rfc3339(),
        };
        let envelope = ship_types::envelope::EventEnvelope::new(
            ship_types::event_types::STACK_REGISTERED,
            &resolved,
            &payload,
        )
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
        self.ctx
            .kernel()
            .emit(envelope, &crate::helpers::runtime_emit_ctx())
            .await
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
        Ok(payload)
    }

    /// The stacks registered as members of the space, in registration order.
    pub fn list_stacks(
        &self,
        space_id: &str,
    ) -> Result<Vec<ship_substrate::db::spaces::StackRow>, SpaceError> {
        let resolved = resolve_space_id(self.ctx.space_id().as_str(), space_id)?;
        crate::spaces::list_stacks(self.ctx.kernel(), &resolved)
    }

    // ── install lifecycle (registry-resolved) ─────────────────────────────

    /// Install a stack from the registry into the managed library dir,
    /// emit `stack.installed` (folded by [`LibraryProjection`](crate::LibraryProjection)
    /// into the `library` table). Returns the resolved install metadata.
    pub async fn install_stack(
        &self,
        path: &str,
        version_constraint: &str,
    ) -> Result<InstalledStack, SpaceError> {
        let registry = self.registry.as_ref().ok_or_else(|| {
            SpaceError::Internal(
                "install_stack: this SpaceService mount is not configured with a registry"
                    .to_string(),
            )
        })?;
        let library_root_parent = self.library_root_parent.as_ref().ok_or_else(|| {
            SpaceError::Internal(
                "install_stack: this SpaceService mount has no library_root_parent".to_string(),
            )
        })?;
        if path.is_empty() {
            return Err(SpaceError::BadArgs("path is required".into()));
        }

        let installed =
            store_install(registry.as_ref(), library_root_parent, path, version_constraint, &self.trusted_keys)
                .await
                .map_err(install_error_to_space)?;

        // Emit `stack.installed` so the projection folds the library row.
        let payload = StackInstalled {
            path: installed.path.clone(),
            version: installed.version.clone(),
            root_path: installed.root_path.to_string_lossy().into_owned(),
            content_hash: installed.content_hash.clone(),
            installed_at: crate::helpers::now_rfc3339(),
        };
        let envelope = EventEnvelope::new(event_types::STACK_INSTALLED, "library", &payload)
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
        self.ctx
            .kernel()
            .emit(envelope, &crate::helpers::runtime_emit_ctx())
            .await
            .map_err(|e| SpaceError::Internal(e.to_string()))?;

        Ok(installed)
    }

    /// Unregister a library row at `(path, version)`: emit `stack.unregistered`
    /// (projection removes the row) and remove the unpacked tree. Best-effort
    /// on disk — the projection delete is the source of truth.
    pub async fn unregister_stack(
        &self,
        path: &str,
        version: &str,
    ) -> Result<(), SpaceError> {
        if path.is_empty() || version.is_empty() {
            return Err(SpaceError::BadArgs(
                "path and version are required".to_string(),
            ));
        }
        // Reverse-dep warning (req-reverse-deps): log any registered stacks
        // whose lockfiles reference this path so an operator catches the
        // remove-by-mistake case. We don't ENFORCE — only warn.
        if let Some(lib_root_parent) = self.library_root_parent.as_ref() {
            // The row's root_path is deterministic from path+version + the
            // library layout; we don't have to read the projection to know
            // it for the reverse-dep scan (we scan all registered stacks
            // across spaces via the Runtime kernel).
            let _ = reverse_dep_warn(self.ctx.kernel(), path, version);
            // Remove the unpacked tree (best-effort).
            let _ = store_remove(lib_root_parent, path, version);
        }
        // Emit `stack.unregistered` so the projection drops the row.
        let root = self
            .library_root_parent
            .as_deref()
            .map(|p| library_root(p, path, version).to_string_lossy().into_owned())
            .unwrap_or_default();
        let payload = StackUnregistered {
            path: path.to_string(),
            version: version.to_string(),
            root_path: root,
            unregistered_at: crate::helpers::now_rfc3339(),
        };
        let envelope = EventEnvelope::new(event_types::STACK_UNREGISTERED, "library", &payload)
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
        self.ctx
            .kernel()
            .emit(envelope, &crate::helpers::runtime_emit_ctx())
            .await
            .map_err(|e| SpaceError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Every library row on this device.
    pub fn list_installed(&self) -> Result<Vec<ship_substrate::db::spaces::InstalledStack>, SpaceError> {
        let dir = self
            .library_root_parent
            .as_deref()
            .ok_or_else(|| {
                SpaceError::Internal(
                    "list_installed: no library_root_parent; install disabled".to_string(),
                )
            })?;
        ship_substrate::db::spaces::list_installed_at(dir).map_err(SpaceError::from)
    }

    // ── snapshot ────────────────────────────────────────────────────────────
    //
    // Resolves `space_id` (the request's explicit id, or the served space's
    // id when the request leaves it empty — the "active/served space"
    // fallback the proto documents), validates `target_dir`, and delegates
    // to the substrate's `snapshot_space` to write the bundle. The wire
    // layer is the only caller; the helper here exists so the resolution
    // rule lives next to the rest of the service logic.
    pub async fn snapshot(
        &self,
        space_id: &str,
        target_dir: &std::path::Path,
    ) -> Result<ship_substrate::SnapshotResult, SpaceError> {
        let resolved = resolve_space_id(self.ctx.space_id().as_str(), space_id)?;
        // target_dir must be an absolute, existing directory — the drill
        // spec calls this out (typed invalid-argument otherwise).
        if !target_dir.is_absolute() {
            return Err(SpaceError::BadArgs(format!(
                "target_dir must be an absolute path: {}",
                target_dir.display()
            )));
        }
        if !target_dir.is_dir() {
            return Err(SpaceError::BadArgs(format!(
                "target_dir does not exist or is not a directory: {}",
                target_dir.display()
            )));
        }
        let data_root = ship_substrate::project::get_global_dir().map_err(|e| {
            SpaceError::Internal(format!("resolve ship data root: {e}"))
        })?;
        ship_substrate::snapshot_space(&data_root, &resolved, target_dir)
            .map_err(|e| SpaceError::Internal(format!("snapshot: {e}")))
    }
}

/// Map [`InstallError`] → [`SpaceError`] for the wire layer.
fn install_error_to_space(e: InstallError) -> SpaceError {
    match e {
        InstallError::BadArgs(m) => SpaceError::BadArgs(m),
        InstallError::Registry(m) => SpaceError::Internal(format!("registry: {m}")),
        InstallError::Verify(m) => SpaceError::Internal(format!("verify: {m}")),
        InstallError::Internal(m) => SpaceError::Internal(m),
    }
}

/// Reverse-dep warning: scan every registered stack across every space for
/// any whose lockfile references the path being uninstalled. Logs a WARN
/// listing the space_ids + root_paths (the operator catches "removed a
/// dependency a member stack relies on"). Advisory — never blocks the
/// uninstall (req-reverse-deps).
///
/// Cheap fallback: the Runtime kernel's log doesn't carry a lockfile
/// parser, so we just emit a WARN naming the `(path, version)` being
/// removed. A future fenced session can extend this to actually scan
/// member-stack lockfiles.
fn reverse_dep_warn(
    kernel: &ship_substrate::ports::KernelLog,
    path: &str,
    version: &str,
) -> Result<(), SpaceError> {
    let Some(_dir) = kernel.try_ship_dir() else {
        return Ok(());
    };
    tracing::warn!(
        path = %path,
        version = %version,
        "stack.unregistered: member stacks may still reference this path; \
         scanning their lockfiles is a future fenced-session task (advisory only)"
    );
    Ok(())
}

/// Pick the effective space id: the request's `space_id` when present, else
/// the served space from the request's `SpaceContext`. Both empty is an
/// invalid request — a snapshot with no target space has no meaning.
fn resolve_space_id(served: &str, requested: &str) -> Result<String, SpaceError> {
    if !requested.is_empty() {
        return Ok(requested.to_string());
    }
    if !served.is_empty() {
        return Ok(served.to_string());
    }
    Err(SpaceError::BadArgs(
        "space_id is empty and the request has no served space; specify one".to_string(),
    ))
}


