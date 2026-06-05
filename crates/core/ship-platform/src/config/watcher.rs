//! Declarative-tier config watcher — registers the six built-in `.ship/`
//! domains (agents, skills, views, pipelines, rules, collections) against the
//! substrate filesystem-watch port ([`FsWatch`]).
//!
//! Both user-scope (`~/.ship/library/`) and project-scope (`<workspace>/.ship/`)
//! dirs are watched when present. Each domain has its own debounce so a burst of
//! file writes emits exactly one event per domain. The port owns the boring
//! mechanism (notify wiring + kind filtering + debounce coalescing); this module
//! owns what is config's: scope resolution and kernel-event AUTHORSHIP.
//!
//! Watchers are RAII via the returned [`ConfigWatcher`]: dropping it aborts every
//! consumer task, which drops its [`FsDelta`] receiver and tears down the
//! underlying substrate watch.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::config::resolve_watcher_debounce;
use ship_substrate::ShipHome;
use ship_substrate::events::EmitContext;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::ports::{
    FsBroker, FsCap, FsDelta, FsMode, FsRoot, FsWatch, LocalFsBroker, LocalFsWatch, WatchOpts,
};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{AgentDefUpdated, ConfigChanged, ConfigFileChanged};

/// RAII handle holding every config-domain consumer task. Dropping it aborts the
/// tasks, dropping each [`FsDelta`] receiver and tearing down the substrate watch
/// behind it.
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
    DomainCfg {
        dir_name: "agents",
        event_type: event_types::AGENT_CHANGED,
        recursive: false,
    },
    DomainCfg {
        dir_name: "skills",
        event_type: event_types::SKILL_CHANGED,
        recursive: true,
    },
    DomainCfg {
        dir_name: "views",
        event_type: event_types::VIEW_CHANGED,
        recursive: true,
    },
    DomainCfg {
        dir_name: "pipelines",
        event_type: event_types::PIPELINE_CHANGED,
        recursive: false,
    },
    DomainCfg {
        dir_name: "rules",
        event_type: event_types::RULE_CHANGED,
        recursive: false,
    },
    // Collections — `.ship/collections/<id>.jsonc` defines a queryable
    // bucket schema. The collection schema is lazy-loaded on every
    // read/write (collections.create_item calls ensure_collection_table
    // before insert), so a freshly-dropped jsonc file ALREADY works at
    // the data layer. What the watcher buys is UI invalidation: Studio's
    // collections list / detail can subscribe to collection.changed and
    // refresh without an operator-driven reload. Surfaced by #184 when
    // adding sui-eval-runs.jsonc didn't repaint the Studio view.
    DomainCfg {
        dir_name: "collections",
        event_type: event_types::COLLECTION_CHANGED,
        recursive: false,
    },
];

/// Spawn watchers for the six declarative config domains, both at
/// user-scope and (when present) project-scope.
pub fn spawn_config_watcher(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    workspace_dir: Option<PathBuf>,
) -> Result<ConfigWatcher> {
    let library_dir = ShipHome::at(ship_dir).library_dir();

    // Project-scope config events are Space-scoped (registry: config.changed /
    // *.changed → EventScope::Space). The space key is the project root's
    // path-hash identity — derived from the watched path. There is no daemon
    // home-space global to read it from (#3: the global pseudo-space is gone).
    let project_space = workspace_dir
        .as_deref()
        .map(|root| SpaceId::new(ship_substrate::home::path_hash(root)));
    let project_ship_dir = workspace_dir.map(ShipHome::project_ship_dir);

    let mut watcher = ConfigWatcher::new();
    let debounce = resolve_watcher_debounce();

    for domain in DOMAINS {
        // User scope — ensure the dir exists so we always have somewhere to
        // attach. (Project scope is created by `ship init`, not the watcher.)
        // FLAG: user-scope config classification → P3 registry decision. These
        // `~/.ship/library/` watchers have no project space; they author with a
        // keyless daemon-internal binding pending the P3 user/account scoping.
        let user_dir = library_dir.join(domain.dir_name);
        if !user_dir.exists() {
            let _ = std::fs::create_dir_all(&user_dir);
        }
        spawn_domain_watch(&mut watcher, kernel.clone(), user_dir, domain, None, debounce);

        // Project scope (only when a workspace is attached).
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

/// Open an [`FsWatch`] subscription on `dir` and spawn the consumer that authors
/// config events from the raw [`FsDelta`] stream. No-op if `dir` doesn't exist or
/// the watch fails to attach (logged). The cap is rooted AT the domain dir so the
/// delta `rel` paths are already relative to the domain — exactly what the
/// authorship needs.
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

/// Spawn the consumer loop that authors config events from a domain's raw
/// [`FsDelta`] stream. The config service — not the watcher — owns event
/// authorship: it names the domain `event_type`, sets the `subject`/`entity_id`,
/// and mints the scope binding from the resolved `space`.
fn spawn_domain_consumer(
    kernel: Arc<Mutex<KernelRouter>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<FsDelta>,
    domain: &'static DomainCfg,
    dir: PathBuf,
    space: Option<SpaceId>,
) -> tokio::task::JoinHandle<()> {
    let event_type = domain.event_type;
    let subject = domain.dir_name.to_string();
    // S1 (agents-arc): the per-file authorship branches by domain inside
    // `author_file_changed` — the agents domain emits the fine-grained,
    // space-stamped, content-hashed `agent.def.updated`; other domains keep the
    // generic `config.file_changed`. The hash read goes through the FsBroker
    // door (no raw `std::fs`).
    let fs = LocalFsBroker::new();
    tokio::spawn(async move {
        while let Some(delta) = rx.recv().await {
            // Coarse domain signal — existing consumers (compile invalidation).
            author_change(&kernel, event_type, &subject, space.clone()).await;
            // Fine-grained per-file events — Format-2 reactivity / sync.
            for rel in &delta.rel {
                author_file_changed(&kernel, &fs, &subject, &dir, rel, space.clone()).await;
            }
        }
    })
}

/// Author the coarse `<domain>.changed` config event with the SAME
/// `event_type` / `subject` / `entity_id` the watcher used to emit, so config
/// reactivity (compile invalidation, Studio) is unbroken.
async fn author_change(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    subject: &str,
    space: Option<SpaceId>,
) {
    let payload = ConfigChanged {
        subject: subject.to_string(),
        detail: None,
    };
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

/// Author the fine-grained per-file change event for one changed file: the path
/// relative to the watched domain dir + its content hash (empty when removed).
///
/// S1 (agents-arc): the agents domain emits `agent.def.updated` (Space-scoped,
/// the Format-2 def notification the read-only index re-reads on); every other
/// domain emits the generic `config.file_changed` (Runtime-scoped). Both carry
/// `{path, content_hash}` and NEVER file content. The hash read goes through the
/// FsBroker door (no raw `std::fs`); a read failure = empty hash = "deleted"
/// marker. `rel` is already relative to the domain dir (the substrate watch is
/// rooted there).
async fn author_file_changed(
    kernel: &Arc<Mutex<KernelRouter>>,
    fs: &dyn FsBroker,
    domain: &str,
    dir: &std::path::Path,
    rel: &str,
    space: Option<SpaceId>,
) {
    let cap = FsCap::new_unchecked(
        dir.to_path_buf(),
        FsRoot::Space(ship_substrate::ports::SpaceId::new("config")),
        FsMode::ReadOnly,
    );
    // Empty hash on read failure = a "deleted" marker (file gone).
    let content_hash = fs.read(&cap, rel).map(|b| ship_types::sha256_hex(&b)).unwrap_or_default();

    let (event_type, payload_json) = if domain == "agents" {
        let p = AgentDefUpdated { path: rel.to_string(), content_hash };
        (event_types::AGENT_DEF_UPDATED, serde_json::to_value(&p))
    } else {
        let p = ConfigFileChanged { domain: domain.to_string(), path: rel.to_string(), content_hash };
        (event_types::CONFIG_FILE_CHANGED, serde_json::to_value(&p))
    };
    let envelope = match payload_json
        .map_err(|e| e.to_string())
        .and_then(|p| EventEnvelope::new(event_type, rel, &p).map_err(|e| e.to_string()))
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::db::ensure_db_at;
    use ship_substrate::events::new_isolated_for_test;
    use ship_substrate::events::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    use std::path::Path;
    use tempfile::tempdir;
    use crate::testutil::init_project;

    async fn setup() -> (tempfile::TempDir, PathBuf, Arc<Mutex<KernelRouter>>) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let kernel = new_isolated_for_test(ship_dir.clone()).unwrap();
        (tmp, ship_dir, kernel)
    }

    // Count events of `event_type` in a specific scope db. The kernel sink
    // routes by scope, so config.* (Runtime) lands in runtime.db while
    // agent/skill.changed (Space) land in the bound space.db — the caller
    // names the db to read.
    fn count_events(db_path: &Path, event_type: &str) -> usize {
        let store = SqliteEventStore::new_at_db_path(db_path).unwrap();
        store
            .query(&EventFilter {
                event_type: Some(event_type.to_string()),
                limit: Some(50),
                ..Default::default()
            })
            .unwrap()
            .len()
    }

    // T6 — the config service (not the watcher) authors config events with the
    // IDENTICAL event_type / subject / entity_id the watcher used to emit. Not
    // flaky: drives the authoring path directly with a synthetic signal instead
    // of relying on inotify.
    #[tokio::test(flavor = "multi_thread")]
    async fn config_service_authors_domain_event_with_identical_shape() {
        let (_tmp, ship_dir, kernel) = setup().await;
        // Coarse domain event — same event_type (AGENT_CHANGED) + entity_id
        // (subject "agents") the watcher produced before the relocation.
        author_change(&kernel, event_types::AGENT_CHANGED, "agents", Some(SpaceId::new("sp-1")))
            .await;
        // agent.changed is Space-scoped → the kernel sink routes it to the bound
        // space's db (sp-1), not platform.db.
        let store = SqliteEventStore::new_at_db_path(
            &ship_substrate::db::space_db_path(&ship_dir, "sp-1"),
        )
        .unwrap();
        let evs = store
            .query(&EventFilter {
                event_type: Some(event_types::AGENT_CHANGED.to_string()),
                limit: Some(10),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(evs.len(), 1, "config service must author exactly one agent.changed");
        assert_eq!(evs[0].entity_id, "agents", "entity_id (subject) preserved");

        let fs = ship_substrate::ports::LocalFsBroker::new();

        // Fine-grained per-file event for the AGENTS domain — S1 evolves this to
        // `agent.def.updated` (Space-scoped → bound space's db), not the generic
        // config.file_changed. The hash read goes through the FsBroker. `rel` is
        // the file path relative to the domain dir (cap root).
        let dir = ship_dir.join("agents");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.jsonc"), "{}").unwrap();
        author_file_changed(&kernel, &fs, "agents", &dir, "x.jsonc", Some(SpaceId::new("sp-1"))).await;
        assert_eq!(
            count_events(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"), event_types::AGENT_DEF_UPDATED),
            1,
            "agents domain authors agent.def.updated into the space db"
        );
    }
}
