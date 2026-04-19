//! Pidfile-based [`SupervisionAdapter`]. Wraps slice 1's `pidfile` module.

use anyhow::Result;
use std::path::Path;

use super::pidfile;
use super::supervision::{AdapterKind, StartContext, SupervisionAdapter, SupervisionStatus};

pub struct PidfileAdapter;

impl SupervisionAdapter for PidfileAdapter {
    fn name(&self) -> AdapterKind {
        AdapterKind::Pidfile
    }

    fn status(&self, global_dir: &Path) -> Result<SupervisionStatus> {
        let Some((record, lock_held)) = pidfile::read(global_dir)? else {
            return Ok(SupervisionStatus {
                adapter: AdapterKind::Pidfile,
                running: false,
                pid: None,
                version: None,
                uptime_secs: None,
                extras: Vec::new(),
            });
        };

        let uptime_secs = chrono::DateTime::parse_from_rfc3339(&record.started_at)
            .ok()
            .map(|dt| {
                (chrono::Utc::now() - dt.with_timezone(&chrono::Utc))
                    .num_seconds()
                    .max(0) as u64
            });

        let mut extras = Vec::new();
        extras.push(("started_at".to_string(), record.started_at.clone()));
        extras.push(("role".to_string(), format!("{:?}", record.role).to_lowercase()));
        extras.push(("commit_sha".to_string(), record.commit_sha.clone()));
        if !lock_held {
            extras.push(("lock".to_string(), "released (stale pidfile)".to_string()));
        }

        Ok(SupervisionStatus {
            adapter: AdapterKind::Pidfile,
            running: lock_held,
            pid: Some(record.pid),
            version: Some(record.version),
            uptime_secs,
            extras,
        })
    }

    fn start(&self, ctx: &StartContext) -> Result<()> {
        super::daemon_start_inner(&ctx.host, ctx.dev)
    }

    fn stop(&self) -> Result<()> {
        super::daemon_stop_inner()
    }
}
