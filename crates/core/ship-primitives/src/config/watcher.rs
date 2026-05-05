//! Declarative-tier config watcher — registers the five built-in `.ship/`
//! domains (agents, skills, views, pipelines, rules) against the generic
//! [`crate::services::watcher::WatcherService`].
//!
//! Both user-scope (`~/.ship/library/`) and project-scope
//! (`<workspace>/.ship/`) dirs are watched when present. Each domain has its
//! own debounce so a burst of file writes emits exactly one event per
//! domain. Watchers are RAII via the returned [`WatcherService`].

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use notify::RecursiveMode;
use tokio::sync::Mutex;

use crate::services::watcher::{DEFAULT_DEBOUNCE, WatcherService, WatcherSpec};
use ship_substrate::ShipHome;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_types::event_types;

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
];

/// Spawn watchers for the five declarative config domains, both at
/// user-scope and (when present) project-scope.
pub fn spawn_config_watcher(
    kernel: Arc<Mutex<KernelRouter>>,
    ship_dir: PathBuf,
    workspace_dir: Option<PathBuf>,
) -> Result<WatcherService> {
    let library_dir = ShipHome::at(ship_dir).library_dir();
    let project_ship_dir = workspace_dir.map(ShipHome::project_ship_dir);

    let mut service = WatcherService::new();

    for domain in DOMAINS {
        // User scope — ensure the dir exists so we always have somewhere to
        // attach. (Project scope is created by `ship init`, not the watcher.)
        let user_dir = library_dir.join(domain.dir_name);
        if !user_dir.exists() {
            let _ = std::fs::create_dir_all(&user_dir);
        }
        service.register(
            kernel.clone(),
            WatcherSpec {
                dir: user_dir,
                event_type: domain.event_type,
                subject: domain.dir_name.to_string(),
                recursive: domain.recursive,
                debounce: DEFAULT_DEBOUNCE,
            },
        )?;

        // Project scope (only when a workspace is attached).
        if let Some(ref pd) = project_ship_dir {
            let proj_dir = pd.join(domain.dir_name);
            if let Err(e) = service.register(
                kernel.clone(),
                WatcherSpec {
                    dir: proj_dir.clone(),
                    event_type: domain.event_type,
                    subject: domain.dir_name.to_string(),
                    recursive: domain.recursive,
                    debounce: DEFAULT_DEBOUNCE,
                },
            ) {
                tracing::warn!(
                    "config watcher: could not watch {}: {e}",
                    proj_dir.display()
                );
            }
        }
    }

    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::db::ensure_db_at;
    use ship_substrate::events::new_isolated_for_test;
    use ship_substrate::events::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    use std::time::Duration;
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
