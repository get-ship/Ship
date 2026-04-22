//! Cron trigger source adapter.
//!
//! Maintains a set of cron schedules and fires `external.cron.<hook_name>` events
//! on schedule. Persists last-fire timestamps via the KV store so restarts don't
//! double-fire within the same scheduled interval.

use std::sync::Mutex;

use chrono::{DateTime, Utc};
use cron::Schedule;
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use crate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use super::super::def::{HookDef, TriggerSourceDef};
use super::TriggerSource;

const CRON_KV_NS: &str = "cron.last_fire";

struct CronEntry {
    def: HookDef,
    schedule: Schedule,
    next_fire: Option<DateTime<Utc>>,
}

impl CronEntry {
    fn new(def: HookDef, schedule: Schedule) -> Self {
        let next_fire = schedule.upcoming(Utc).next();
        Self { def, schedule, next_fire }
    }

    fn advance(&mut self) {
        // Advance to the next scheduled time after the current next_fire.
        self.next_fire = self.schedule.upcoming(Utc).next();
    }
}

pub struct CronSource {
    reload_tx: Mutex<Option<UnboundedSender<Vec<HookDef>>>>,
}

impl CronSource {
    pub fn new() -> Self {
        Self { reload_tx: Mutex::new(None) }
    }
}

impl TriggerSource for CronSource {
    fn start(
        &self,
        kernel: std::sync::Arc<tokio::sync::Mutex<KernelRouter>>,
    ) -> tokio::task::JoinHandle<()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        *self.reload_tx.lock().unwrap() = Some(tx);
        tokio::spawn(cron_source_task(rx, kernel))
    }

    fn reload(&self, new_defs: &[HookDef]) {
        if let Some(tx) = self.reload_tx.lock().unwrap().as_ref() {
            let _ = tx.send(new_defs.to_vec());
        }
    }
}

async fn cron_source_task(
    mut reload_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<HookDef>>,
    kernel: std::sync::Arc<tokio::sync::Mutex<KernelRouter>>,
) {
    let mut entries: Vec<CronEntry> = Vec::new();

    loop {
        // Determine how long to sleep before the next trigger fires.
        let sleep_duration = next_sleep(&entries);

        tokio::select! {
            biased;

            new_defs = reload_rx.recv() => {
                let Some(defs) = new_defs else { break };
                entries = build_entries(defs);
            }

            _ = tokio::time::sleep(sleep_duration) => {
                let now = Utc::now();
                for entry in entries.iter_mut() {
                    let Some(fire_at) = entry.next_fire else { continue };
                    if fire_at > now {
                        continue;
                    }
                    // Check last-fire from KV to prevent double-fire on restart.
                    let hook_name = entry.def.name.clone();
                    if let Some(last) = load_last_fire(&hook_name) {
                        if last >= fire_at {
                            // Already fired for this scheduled interval.
                            entry.advance();
                            continue;
                        }
                    }
                    emit_cron_event(&hook_name, fire_at, now, &kernel).await;
                    save_last_fire(&hook_name, now);
                    entry.advance();
                }
            }
        }
    }
}

fn next_sleep(entries: &[CronEntry]) -> std::time::Duration {
    let now = Utc::now();
    let min_fire = entries
        .iter()
        .filter_map(|e| e.next_fire)
        .filter(|t| *t > now)
        .min();

    match min_fire {
        Some(t) => {
            let delta = t - now;
            delta
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(1))
        }
        // No scheduled triggers; check again in 60 seconds.
        None => std::time::Duration::from_secs(60),
    }
}

fn build_entries(defs: Vec<HookDef>) -> Vec<CronEntry> {
    let mut entries = Vec::new();
    for def in defs {
        if let Some(TriggerSourceDef::Cron(cdef)) = &def.trigger.source {
            match super::super::loader::normalize_and_parse_cron(&cdef.expression) {
                Ok(schedule) => entries.push(CronEntry::new(def, schedule)),
                Err(e) => tracing::warn!(
                    hook = def.name,
                    expr = cdef.expression,
                    "cron-source: invalid expression at runtime: {e}"
                ),
            }
        }
    }
    entries
}

async fn emit_cron_event(
    hook_name: &str,
    scheduled_for: DateTime<Utc>,
    fired_at: DateTime<Utc>,
    kernel: &std::sync::Arc<tokio::sync::Mutex<KernelRouter>>,
) {
    let event_type = format!("external.cron.{}", hook_name);
    let payload = json!({
        "scheduled_for": scheduled_for.to_rfc3339(),
        "fired_at": fired_at.to_rfc3339(),
    });
    match EventEnvelope::new(&event_type, hook_name, &payload) {
        Ok(envelope) => {
            let ctx = EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                workspace_id: None,
                session_id: None,
            };
            if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
                tracing::warn!(hook = hook_name, "cron-source: emit failed: {e}");
            }
        }
        Err(e) => tracing::warn!(hook = hook_name, "cron-source: build envelope failed: {e}"),
    }
}

fn load_last_fire(hook_name: &str) -> Option<DateTime<Utc>> {
    let val = crate::db::kv::get(CRON_KV_NS, hook_name).ok()??;
    let ts = val.as_str()?;
    ts.parse::<DateTime<Utc>>().ok()
}

fn save_last_fire(hook_name: &str, fired_at: DateTime<Utc>) {
    let val = serde_json::Value::String(fired_at.to_rfc3339());
    if let Err(e) = crate::db::kv::set(CRON_KV_NS, hook_name, &val) {
        tracing::warn!(hook = hook_name, "cron-source: persist last_fire failed: {e}");
    }
}
