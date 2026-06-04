//! Declarative-tier config watcher — registers the six built-in `.ship/`
//! domains (agents, skills, views, pipelines, rules, collections) against
//! the generic [`crate::services::watcher::WatcherService`].
//!
//! Both user-scope (`~/.ship/library/`) and project-scope
//! (`<workspace>/.ship/`) dirs are watched when present. Each domain has its
//! own debounce so a burst of file writes emits exactly one event per
//! domain. Watchers are RAII via the returned [`WatcherService`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use notify::RecursiveMode;
use tokio::sync::Mutex;

use crate::config::resolve_watcher_debounce;
use crate::services::watcher::{PathChangedSignal, WatcherService, WatcherSpec};
use ship_substrate::ShipHome;
use ship_substrate::events::EmitContext;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::ports::{FsBroker, FsCap, FsMode, FsRoot, LocalFsBroker};
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{AgentDefUpdated, ConfigChanged, ConfigFileChanged};

/// Re-export so existing call sites that imported `config::ConfigWatcher`
/// continue to work — the type is the generic [`WatcherService`].
pub type ConfigWatcher = WatcherService;

struct DomainCfg {
    dir_name: &'static str,
    event_type: &'static str,
    recursive: RecursiveMode,
}

const DOMAINS: &[DomainCfg] = &[
    DomainCfg {
        dir_name: "agents",
        event_type: event_types::AGENT_CHANGED,
        recursive: RecursiveMode::NonRecursive,
    },
    DomainCfg {
        dir_name: "skills",
        event_type: event_types::SKILL_CHANGED,
        recursive: RecursiveMode::Recursive,
    },
    DomainCfg {
        dir_name: "views",
        event_type: event_types::VIEW_CHANGED,
        recursive: RecursiveMode::Recursive,
    },
    DomainCfg {
        dir_name: "pipelines",
        event_type: event_types::PIPELINE_CHANGED,
        recursive: RecursiveMode::NonRecursive,
    },
    DomainCfg {
        dir_name: "rules",
        event_type: event_types::RULE_CHANGED,
        recursive: RecursiveMode::NonRecursive,
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
        recursive: RecursiveMode::NonRecursive,
    },
];

/// Spawn watchers for the six declarative config domains, both at
/// user-scope and (when present) project-scope.
pub fn spawn_config_watcher(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    workspace_dir: Option<PathBuf>,
) -> Result<WatcherService> {
    let library_dir = ShipHome::at(ship_dir).library_dir();

    // Project-scope config events are Space-scoped (registry: config.changed /
    // *.changed → EventScope::Space). The space key is the project root's
    // path-hash identity — derived from the watched path. There is no daemon
    // home-space global to read it from (#3: the global pseudo-space is gone).
    let project_space = workspace_dir
        .as_deref()
        .map(|root| SpaceId::new(ship_substrate::home::path_hash(root)));
    let project_ship_dir = workspace_dir.map(ShipHome::project_ship_dir);

    let mut service = WatcherService::new();
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
        let rx = service.register(WatcherSpec {
            dir: user_dir,
            recursive: domain.recursive,
            debounce,
        })?;
        spawn_domain_consumer(&mut service, kernel.clone(), rx, domain, None);

        // Project scope (only when a workspace is attached).
        if let Some(ref pd) = project_ship_dir {
            let proj_dir = pd.join(domain.dir_name);
            match service.register(WatcherSpec {
                dir: proj_dir.clone(),
                recursive: domain.recursive,
                debounce,
            }) {
                Ok(rx) => spawn_domain_consumer(
                    &mut service,
                    kernel.clone(),
                    rx,
                    domain,
                    project_space.clone(),
                ),
                Err(e) => tracing::warn!(
                    "config watcher: could not watch {}: {e}",
                    proj_dir.display()
                ),
            }
        }
    }

    Ok(service)
}

/// Spawn the consumer loop that authors config events from a watcher's
/// debounced [`PathChangedSignal`] stream, and adopt it into the service's RAII
/// lifetime. The config service — not the watcher — owns event authorship: it
/// names the domain `event_type`, sets the `subject`/`entity_id`, and mints the
/// scope binding from the resolved `space`.
fn spawn_domain_consumer(
    service: &mut WatcherService,
    kernel: Arc<Mutex<KernelRouter>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<PathChangedSignal>,
    domain: &'static DomainCfg,
    space: Option<SpaceId>,
) {
    let event_type = domain.event_type;
    let subject = domain.dir_name.to_string();
    // S1 (agents-arc): the per-file authorship now branches by domain inside
    // `author_file_changed` — the agents domain emits the fine-grained,
    // space-stamped, content-hashed `agent.def.updated` (the Format-2 def
    // notification the read-only index re-reads on); other domains keep the
    // generic `config.file_changed`. The hash read goes through the FsBroker door
    // (no raw `std::fs`).
    let fs = LocalFsBroker::new();
    let task = tokio::spawn(async move {
        while let Some(signal) = rx.recv().await {
            // Coarse domain signal — existing consumers (compile invalidation).
            author_change(&kernel, event_type, &subject, space.clone()).await;
            // Fine-grained per-file events — Format-2 reactivity / sync.
            for path in &signal.paths {
                author_file_changed(&kernel, &fs, &subject, &signal.dir, path, space.clone()).await;
            }
        }
    });
    service.push_task(task);
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
/// marker.
async fn author_file_changed(
    kernel: &Arc<Mutex<KernelRouter>>,
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
    // Empty hash on read failure = a "deleted" marker (file gone).
    let content_hash = fs.read(&cap, &rel).map(|b| ship_types::sha256_hex(&b)).unwrap_or_default();

    let (event_type, payload_json) = if domain == "agents" {
        let p = AgentDefUpdated { path: rel.clone(), content_hash };
        (event_types::AGENT_DEF_UPDATED, serde_json::to_value(&p))
    } else {
        let p = ConfigFileChanged { domain: domain.to_string(), path: rel.clone(), content_hash };
        (event_types::CONFIG_FILE_CHANGED, serde_json::to_value(&p))
    };
    let envelope = match payload_json
        .map_err(|e| e.to_string())
        .and_then(|p| EventEnvelope::new(event_type, &rel, &p).map_err(|e| e.to_string()))
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
    use std::time::Duration;
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
        // config.file_changed. (The non-agents `config.file_changed` path is
        // proved in `ship-config/tests/watcher.rs`.) The hash read goes through
        // the FsBroker.
        let dir = ship_dir.join("agents");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("x.jsonc");
        std::fs::write(&file, "{}").unwrap();
        author_file_changed(&kernel, &fs, "agents", &dir, &file, Some(SpaceId::new("sp-1"))).await;
        assert_eq!(
            count_events(&ship_substrate::db::space_db_path(&ship_dir, "sp-1"), event_types::AGENT_DEF_UPDATED),
            1,
            "agents domain authors agent.def.updated into the space db"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "flaky on WSL2/CI — notify inotify watch fails with ENFILE under parallel test load"]
    async fn agent_file_write_emits_agent_changed() {
        let (_tmp, ship_dir, kernel) = setup().await;
        let _watcher =
            spawn_config_watcher(kernel.clone(), ship_dir.clone(), None).unwrap();

        let agents_dir = ShipHome::at(ship_dir.clone()).library_dir().join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::write(agents_dir.join("test.jsonc"), r#"{"id":"test"}"#).unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            count_events(&ship_dir, event_types::AGENT_CHANGED) > 0,
            "expected agent.changed event after writing agent config"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "flaky on WSL2/CI — notify inotify watch fails with ENFILE under parallel test load"]
    async fn skill_dir_write_emits_skill_changed() {
        let (_tmp, ship_dir, kernel) = setup().await;
        let _watcher =
            spawn_config_watcher(kernel.clone(), ship_dir.clone(), None).unwrap();

        let skill_dir =
            ShipHome::at(ship_dir.clone()).library_dir().join("skills/my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), "# my-skill\n").unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            count_events(&ship_dir, event_types::SKILL_CHANGED) > 0,
            "expected skill.changed event"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "flaky on WSL2/CI — notify inotify watch fails with ENFILE under parallel test load"]
    async fn rapid_writes_coalesce_to_one_event() {
        let (_tmp, ship_dir, kernel) = setup().await;
        let _watcher =
            spawn_config_watcher(kernel.clone(), ship_dir.clone(), None).unwrap();

        let agents_dir = ShipHome::at(ship_dir.clone()).library_dir().join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        for i in 0..5u8 {
            std::fs::write(agents_dir.join(format!("agent{i}.jsonc")), "{}").unwrap();
        }

        tokio::time::sleep(Duration::from_millis(600)).await;

        assert_eq!(
            count_events(&ship_dir, event_types::AGENT_CHANGED),
            1,
            "5 rapid writes should produce exactly 1 debounced event"
        );
    }
}
