//! Declarative-tier event authorship — the config SERVICE (not the OS watcher)
//! names the domain `event_type`, sets `subject`/`entity_id`, and mints the
//! scope binding.
//!
//! THE EVICTION: the pre-carve `config/watcher.rs` content-hashed a changed file
//! with raw `std::fs::read`. Here the hash read goes through the [`FsBroker`]
//! gated door (empty hash on read failure = deleted marker, behavior preserved).
//!
//! The inotify primitive itself (the substrate `FsWatch` port that wraps
//! `notify` and feeds a debounced delta stream) is a collaborator OUT OF SCOPE
//! for this crate — config subscribes domains against it, it does not absorb it.
//! This module owns only the authorship the watch drives, which is what the
//! keystone test exercises directly with a synthetic signal (no inotify, not
//! flaky).
//!
//! The spawner ([`spawn_config_watcher`]) and its RAII handle ([`ConfigWatcher`])
//! live in this module too: they open the six declarative `.ship/<domain>/`
//! watches (agents, skills, views, pipelines, rules, collections) for both
//! user-scope and project-scope, drive the `author_*` helpers above from each
//! domain's delta stream, and tear the whole thing down on drop.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::Mutex;

use ship_substrate::ShipHome;
use ship_substrate::events::EmitContext;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::ports::{
    FsBroker, FsCap, FsDelta, FsMode, FsRoot, FsWatch, KernelLog, LocalFsBroker, LocalFsWatch,
    WatchOpts,
};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{ConfigChanged, ConfigFileChanged};
/// Author the coarse `<domain>.changed` config event with the SAME `event_type`
/// / `subject` / `entity_id` the watcher used to emit, so config reactivity
/// (compile invalidation, Studio) is unbroken.
///
/// `<domain>.changed` events are Space-scoped (registry); when `space` is
/// `Some`, the kernel sink routes the event to the bound space's db.
pub async fn author_change(
    kernel: &Arc<KernelLog>,
    event_type: &str,
    subject: &str,
    space: Option<SpaceId>,
) {
    let payload = ConfigChanged { subject: subject.to_string(), detail: None };
    let envelope = match EventEnvelope::new(event_type, subject, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build {event_type} envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit {event_type}: {e}");
    }
}

/// Author the fine-grained `config.file_changed` for one changed file: the path
/// relative to the watched domain dir + its content hash (empty when removed).
///
/// `config.file_changed` is Runtime-scoped → it lands in runtime.db. The content
/// hash is read through `fs` confined by a cap rooted at `dir`; a read failure
/// yields an empty hash = a valid "deleted" marker (behavior preserved from the
/// raw-`std::fs::read` predecessor).
pub async fn author_file_changed(
    kernel: &Arc<KernelLog>,
    fs: &dyn FsBroker,
    domain: &str,
    dir: &Path,
    path: &Path,
    space: Option<SpaceId>,
) {
    let rel = path.strip_prefix(dir).unwrap_or(path).to_string_lossy().to_string();
    let cap = FsCap::new_unchecked(
        dir.to_path_buf(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    // Empty hash signals a removed file (read fails) — a valid "deleted" marker.
    let content_hash = fs
        .read(&cap, &rel)
        .map(|bytes| ship_types::sha256_hex(&bytes))
        .unwrap_or_default();
    let payload = ConfigFileChanged {
        domain: domain.to_string(),
        path: rel.clone(),
        content_hash,
    };
    let envelope = match EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, &rel, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build config.file_changed envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit config.file_changed: {e}");
    }
}

// ─── Spawner ──────────────────────────────────────────────────────────────────

/// RAII handle holding every config-domain consumer task. Dropping it aborts
/// the tasks, dropping each [`FsDelta`] receiver and tearing down the substrate
/// watch behind it.
pub struct ConfigWatcher {
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl ConfigWatcher {
    fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    fn push(&mut self, task: tokio::task::JoinHandle<()>) {
        self.tasks.push(task);
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

struct DomainCfg {
    dir_name: &'static str,
    event_type: &'static str,
    recursive: bool,
}

const DOMAINS: &[DomainCfg] = &[
    DomainCfg { dir_name: "agents", event_type: event_types::AGENT_CHANGED, recursive: false },
    DomainCfg { dir_name: "skills", event_type: event_types::SKILL_CHANGED, recursive: true },
    DomainCfg { dir_name: "views", event_type: event_types::VIEW_CHANGED, recursive: true },
    DomainCfg { dir_name: "pipelines", event_type: event_types::PIPELINE_CHANGED, recursive: false },
    DomainCfg { dir_name: "rules", event_type: event_types::RULE_CHANGED, recursive: false },
    DomainCfg {
        dir_name: "collections",
        event_type: event_types::COLLECTION_CHANGED,
        recursive: false,
    },
];

/// Spawn watchers for the six declarative config domains, both at user-scope
/// and (when present) project-scope.
pub fn spawn_config_watcher(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    workspace_dir: Option<PathBuf>,
) -> Result<ConfigWatcher> {
    use crate::settings::resolve_watcher_debounce;

    let library_dir = ShipHome::at(ship_dir).library_dir();

    let project_space = workspace_dir
        .as_deref()
        .map(|root| SpaceId::new(ship_substrate::home::path_hash(root)));
    let project_ship_dir = workspace_dir.map(ShipHome::project_ship_dir);

    let mut watcher = ConfigWatcher::new();
    let debounce = resolve_watcher_debounce();

    for domain in DOMAINS {
        let user_dir = library_dir.join(domain.dir_name);
        if !user_dir.exists() {
            let _ = std::fs::create_dir_all(&user_dir);
        }
        spawn_domain_watch(&mut watcher, kernel.clone(), user_dir, domain, None, debounce);

        if let Some(ref pd) = project_ship_dir {
            let proj_dir = pd.join(domain.dir_name);
            spawn_domain_watch(
                &mut watcher,
                kernel.clone(),
                proj_dir,
                domain,
                project_space.clone(),
                debounce,
            );
        }
    }

    Ok(watcher)
}

fn spawn_domain_watch(
    watcher: &mut ConfigWatcher,
    kernel: Arc<Mutex<KernelRouter>>,
    dir: PathBuf,
    domain: &'static DomainCfg,
    space: Option<SpaceId>,
    debounce: Duration,
) {
    if !dir.exists() {
        return;
    }
    let cap = FsCap::new_unchecked(
        dir.clone(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    let opts = if domain.recursive {
        WatchOpts::recursive(Some(debounce))
    } else {
        WatchOpts::non_recursive(Some(debounce))
    };
    let rx = match LocalFsWatch::new().watch(&cap, ".", opts) {
        Ok(rx) => rx,
        Err(e) => {
            tracing::warn!("config watcher: could not watch {}: {e}", dir.display());
            return;
        }
    };
    watcher.push(spawn_domain_consumer(kernel, rx, domain, dir, space));
}

fn spawn_domain_consumer(
    kernel: Arc<Mutex<KernelRouter>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<FsDelta>,
    domain: &'static DomainCfg,
    dir: PathBuf,
    space: Option<SpaceId>,
) -> tokio::task::JoinHandle<()> {
    let event_type = domain.event_type;
    let subject = domain.dir_name.to_string();
    let fs = LocalFsBroker::new();
    tokio::spawn(async move {
        while let Some(delta) = rx.recv().await {
            author_change_router(&kernel, event_type, &subject, space.clone()).await;
            for rel in &delta.rel {
                author_file_changed_router(&kernel, &fs, &subject, &dir, rel, space.clone()).await;
            }
        }
    })
}

/// KernelRouter-based variant of [`author_change`] used by the spawner.
async fn author_change_router(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    subject: &str,
    space: Option<SpaceId>,
) {
    let payload = ConfigChanged { subject: subject.to_string(), detail: None };
    let envelope = match EventEnvelope::new(event_type, subject, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build {event_type} envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit {event_type}: {e}");
    }
}

/// KernelRouter-based per-file authorship used by the spawner.
///
/// Always emits the generic, **transient** `config.file_changed` signal
/// — even for the agents domain. The watcher no longer names
/// `agent.def.updated`; that durable event is owned by the `agent-def`
/// domain, which subscribes to this signal and emits the durable row
/// (job/transient-events spike, `.artifacts/transient-events-design.md`).
/// The watcher therefore carries ZERO domain / agent knowledge — only
/// the watched directory's name and the file's rel path + content hash.
async fn author_file_changed_router(
    kernel: &Arc<Mutex<KernelRouter>>,
    fs: &dyn FsBroker,
    domain: &str,
    dir: &Path,
    rel: &str,
    space: Option<SpaceId>,
) {
    let cap = FsCap::new_unchecked(
        dir.to_path_buf(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    let content_hash =
        fs.read(&cap, rel).map(|b| ship_types::sha256_hex(&b)).unwrap_or_default();

    let payload = ConfigFileChanged {
        domain: domain.to_string(),
        path: rel.to_string(),
        content_hash,
    };
    let envelope = match serde_json::to_value(&payload)
        .map_err(|e| e.to_string())
        .and_then(|p| {
            EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, rel, &p)
                .map_err(|e| e.to_string())
        }) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config: failed to build config.file_changed envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext::daemon_internal(space);
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit config.file_changed: {e}");
    }
}
