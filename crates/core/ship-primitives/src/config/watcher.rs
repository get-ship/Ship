//! Declarative-tier config watcher — emits domain-specific `<domain>.changed`
//! events whenever config files under `.ship/` change on disk.
//!
//! Covers agents, skills, views, pipelines, and rules. Each domain has its own
//! 200 ms debounce so a burst of file writes (editor save, agent scaffold) emits
//! exactly one event per domain. Both the user-scope (`~/.ship/library/`) and the
//! project-scope (`<workspace>/.ship/`) dirs are watched when present.
//!
//! Watchers are RAII: dropping [`ConfigWatcher`] stops all filesystem watches and
//! cancels all debounce tasks.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use ship_substrate::ShipHome;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_types::{CallerKind, EmitContext};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::ConfigChanged;

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
];

/// RAII handle — dropping it stops all watchers and debounce tasks.
pub struct ConfigWatcher {
    _watchers: Vec<notify::RecommendedWatcher>,
    _tasks: Vec<tokio::task::JoinHandle<()>>,
}

/// Spawn per-domain filesystem watchers for declarative config dirs.
///
/// - `ship_dir`: the global `~/.ship` root (user-scope dirs live under
///   `ship_dir/library/<domain>/`)
/// - `workspace_dir`: optional project root; project-scope dirs live under
///   `workspace_dir/.ship/<domain>/` — watched only when the dir exists
pub fn spawn_config_watcher(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    workspace_dir: Option<PathBuf>,
) -> Result<ConfigWatcher> {
    let library_dir = ShipHome::at(ship_dir).library_dir();
    let project_ship_dir = workspace_dir.map(ShipHome::project_ship_dir);

    let mut watchers = Vec::new();
    let mut tasks = Vec::new();

    for domain in DOMAINS {
        let user_dir = library_dir.join(domain.dir_name);
        let proj_dir = project_ship_dir.as_ref().map(|p| p.join(domain.dir_name));

        // Ensure user-scope dir exists so the watcher has somewhere to watch.
        if !user_dir.exists() {
            let _ = std::fs::create_dir_all(&user_dir);
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<()>();
        let tx_cb = tx.clone();

        let mut w = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res
                && matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                )
            {
                let _ = tx_cb.send(());
            }
        })?;

        if user_dir.exists() {
            w.watch(&user_dir, domain.recursive)?;
        }

        // Watch project scope if the dir exists (don't create it — project
        // dirs are created by the project init flow, not by the watcher).
        if let Some(ref pd) = proj_dir {
            if pd.exists() {
                if let Err(e) = w.watch(pd, domain.recursive) {
                    tracing::warn!(
                        "config watcher: could not watch {}: {e}",
                        pd.display()
                    );
                }
            }
        }

        watchers.push(w);

        let kernel = kernel.clone();
        let event_type = domain.event_type;
        let domain_name = domain.dir_name;

        let task = tokio::spawn(async move {
            while rx.recv().await.is_some() {
                tokio::time::sleep(Duration::from_millis(200)).await;
                while rx.try_recv().is_ok() {}
                emit_domain_event(&kernel, event_type, domain_name).await;
            }
        });
        tasks.push(task);
    }

    Ok(ConfigWatcher { _watchers: watchers, _tasks: tasks })
}

async fn emit_domain_event(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    domain: &str,
) {
    let payload = ConfigChanged {
        subject: domain.to_string(),
        detail: None,
    };
    let envelope = match EventEnvelope::new(event_type, domain, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("config watcher: failed to build {event_type} envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("config watcher: failed to emit {event_type}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::db::ensure_db_at;
    use ship_substrate::events::new_isolated_for_test;
    use ship_substrate::events::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    use tempfile::tempdir;
    use crate::testutil::init_project;

    async fn setup() -> (tempfile::TempDir, PathBuf, Arc<Mutex<KernelRouter>>) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let kernel = new_isolated_for_test(ship_dir.clone()).unwrap();
        (tmp, ship_dir, kernel)
    }

    fn count_events(ship_dir: &PathBuf, event_type: &str) -> usize {
        let store = SqliteEventStore::new_at(ship_dir).unwrap();
        store
            .query(&EventFilter {
                event_type: Some(event_type.to_string()),
                limit: Some(50),
                ..Default::default()
            })
            .unwrap()
            .len()
    }

    #[tokio::test(flavor = "multi_thread")]
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
