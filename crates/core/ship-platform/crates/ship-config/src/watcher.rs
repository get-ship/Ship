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
//! live in this module too: they open the remaining declarative `<domain>/`
//! watches (`views` post Wave 5; the others moved into `stack.config.ts`
//! declare-forms) for both user-scope (the XDG `data/library/<domain>/` per
//! the home-layout ADR) and project-scope (under
//! `<workspace>/.ship/<domain>/`),
//! drive the `author_*` helpers above from each domain's delta stream, and
//! tear the whole thing down on drop. (Post format-migration Wave 4
//! phase A: `pipelines`, `rules`, and `collections` are gone — those
//! nouns move exclusively into `stack.config.ts` declare-forms and
//! the resolver's merge step, not the file watcher set.)
//!
//! ## Source-root coverage (Phase-3 J-event-channel)
//!
//! The one remaining declarative domain watch (`views`) doesn't reach
//! `stack.config.ts` at the space root. The retired ship-resolver private
//! watcher used to cover that surface off its own
//! `tokio::sync::watch` channel; per the operator ruling (2026-07-06,
//! `job-spec-p3-event-channel.md`) reactivity flows through the kernel log
//! only, so [`spawn_config_watcher`] extends its watch set to:
//! [`spawn_source_root_watch`] opens a watch on `stack.config.ts`, authoring
//! the SAME `config.file_changed` shape under one `source` domain label.
//! ONE observer, ONE authored signal, N subscribers.
//!
//! The pre-cutover `space.skill_paths` extras-watch loop died with the
//! dot-ship close-out. The `space.skill_paths` ship.jsonc field is gone;
//! the option moved to `defineConfig({ skills: { paths } })` in each
//! extension's `stack.config.ts` — the per-extension-project authoring
//! surface. The `stack.config.ts` watch above is the trigger: a
//! `stack.config.ts` change re-runs the resolver, which re-evaluates
//! the `defineConfig` paths and re-walks them. The default
//! `<space_root>/skills/` path is skill-marker-only (skills arrive via
//! `defineSkill` declare-forms in `stack.config.ts` post Wave 5) and
//! shares the same re-evaluation trigger; no separate watch is needed.
//!
//! (Watching on-disk skill files under the `defineConfig` paths
//! directly is a follow-up — out of scope for this slice. The
//! `stack.config.ts` re-eval is the current cache-invalidation path
//! for skill assets, and the descriptor cache's content hash captures
//! the union of declare-forms + on-disk skill content for the
//! descriptor-cache-shaped consumers.)

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use ship_substrate::ShipHome;
use ship_substrate::events::EmitContext;
use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot, KernelLog, LocalFsBroker};
use fs_watch::{FsDelta, FsWatch, LocalFsWatch, WatchOpts};
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

// Post format-migration Wave 5: only `views` remains. The `pipelines`,
// `rules`, and `collections` entries died in Wave 4 phase A (moved
// exclusively to `stack.config.ts` declare-forms). The `agents` and
// `skills` entries die with this wave — those nouns now arrive via
// `defineAgent` / `defineSkill` declare-forms in `stack.config.ts`,
// and the `stack.config.ts`-rooted source-root watch already covers
// their only authoring surface. Domain watches for non-existent
// directories are dead weight.
// (census-format-migration.md §4 row 3.)
const DOMAINS: &[DomainCfg] = &[
    DomainCfg { dir_name: "views", event_type: event_types::VIEW_CHANGED, recursive: true },
];

/// Spawn watchers for the one remaining declarative config domain
/// (`views` post Wave 5; `agents` and `skills` died with this wave,
/// `pipelines`/`rules`/`collections` in Wave 4), both at user-scope
/// and (when present) project-scope.
pub fn spawn_config_watcher(
    kernel: Arc<KernelLog>,
    workspace_dir: Option<PathBuf>,
) -> Result<ConfigWatcher> {
    use crate::settings::resolve_watcher_debounce;

    let library_dir = ShipHome::from_env().library_dir();

    // The space-id for the project-scope watcher is the CONTENT-ID from the
    // `projects` pointer (never a path-hash or tree file), so
    // config.file_changed events route to the same `spaces/<content-id>/space.db`
    // every other operational path uses. Unregistered workspace → no space id.
    let project_space = workspace_dir
        .as_deref()
        .and_then(ship_substrate::project::resolve_active_space)
        .map(SpaceId::new);
    // Marker cutover: content lives at the stack root — the watch root IS the workspace dir.
    let project_ship_dir = workspace_dir.clone();

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

    // Source-root coverage: `stack.config.ts` + `space.skill_paths` extras, the
    // surface the three domain dirs above don't reach. User-scope has no
    // notion of a space-root `stack.config.ts`, so this only runs when a project
    // is registered.
    if let (Some(ws_dir), Some(ref proj_ship_dir)) =
        (workspace_dir.as_deref(), project_ship_dir.as_deref())
    {
        spawn_source_root_watch(
            &mut watcher,
            kernel.clone(),
            ws_dir,
            proj_ship_dir,
            project_space.clone(),
            debounce,
        );
    }

    Ok(watcher)
}

/// The domain label the source-root watch authors `config.file_changed`
/// under. Distinct from the [`DOMAINS`] labels — `stack.config.ts` lives
/// at the space root, not inside a `.ship/<domain>/` dir.
const SOURCE_ROOT_DOMAIN: &str = "source";

/// Cover the space-shape surface the one remaining declarative domain
/// watch (`views`) doesn't reach: `stack.config.ts` at the space root.
///
/// Authors the SAME `config.file_changed` shape the domain watches use
/// ([`author_file_changed_router`]) under the one [`SOURCE_ROOT_DOMAIN`]
/// label. No coarse `<domain>.changed` ping is authored — no event type
/// exists for "source changed" and none is added here; `config.file_changed`
/// is the only signal a resolver-descriptor-cache-shaped consumer needs.
///
/// The pre-cutover `space.skill_paths` extras watch died with the
/// dot-ship close-out: the manifest field is gone, the on-disk default
/// `<space_root>/skills/` path is skill-marker-only (skills arrive via
/// `defineSkill` declare-forms in `stack.config.ts` post Wave 5), so
/// there is nothing else under the space root to watch.
fn spawn_source_root_watch(
    watcher: &mut ConfigWatcher,
    kernel: Arc<KernelLog>,
    space_root: &Path,
    _project_ship_dir: &Path,
    space: Option<SpaceId>,
    debounce: Duration,
) {
    // `stack.config.ts` — a single file at the space root. The watch is rooted at
    // the space root itself (non-recursive: siblings like Cargo.toml or
    // README.md are noise), filtered to the one file so unrelated
    // top-level churn never authors a signal.
    if space_root.exists() {
        let cap = FsCap::new_unchecked(
            space_root.to_path_buf(),
            FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
            FsMode::ReadOnly,
        );
        let opts = WatchOpts::non_recursive(Some(debounce));
        if let Ok(rx) = LocalFsWatch::new().watch(&cap, ".", opts) {
            watcher.push(spawn_filtered_file_consumer(
                kernel.clone(),
                rx,
                space_root.to_path_buf(),
                space.clone(),
                |rel| rel == "stack.config.ts",
            ));
        }
    }
}

/// Like [`spawn_domain_consumer`] but authors ONLY `config.file_changed`
/// under [`SOURCE_ROOT_DOMAIN`] (no coarse `<domain>.changed` ping) and
/// applies `filter` to each touched rel before authoring — the
/// source-root watches' roots aren't a whole domain dir, so not every
/// delta under the watched root is necessarily in scope (the `stack.config.ts`
/// watch, in particular, shares its root with unrelated top-level files).
fn spawn_filtered_file_consumer(
    kernel: Arc<KernelLog>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<FsDelta>,
    dir: PathBuf,
    space: Option<SpaceId>,
    filter: fn(&str) -> bool,
) -> tokio::task::JoinHandle<()> {
    let fs = LocalFsBroker::new();
    tokio::spawn(async move {
        while let Some(delta) = rx.recv().await {
            for rel in &delta.rel {
                if filter(rel) {
                    author_file_changed_router(&kernel, &fs, SOURCE_ROOT_DOMAIN, &dir, rel, space.clone())
                        .await;
                }
            }
        }
    })
}

fn spawn_domain_watch(
    watcher: &mut ConfigWatcher,
    kernel: Arc<KernelLog>,
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
    kernel: Arc<KernelLog>,
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

/// Spawner-side variant of [`author_change`] used by the watcher consumer task.
async fn author_change_router(
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

/// Spawner-side per-file authorship used by the watcher consumer task.
///
/// Always emits the generic, **transient** `config.file_changed` signal
/// — even for the agents domain. The watcher no longer names
/// `agent.def.updated`; that durable event is owned by the `agent-def`
/// domain, which subscribes to this signal and emits the durable row
/// (job/transient-events spike, `.artifacts/transient-events-design.md`).
/// The watcher therefore carries ZERO domain / agent knowledge — only
/// the watched directory's name and the file's rel path + content hash.
async fn author_file_changed_router(
    kernel: &Arc<KernelLog>,
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
    if let Err(e) = kernel.emit(envelope, &ctx).await {
        tracing::warn!("config: failed to emit config.file_changed: {e}");
    }
}
