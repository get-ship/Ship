//! External trigger source adapters — webhook, file, and cron.
//!
//! Each adapter implements [`TriggerSource`]: a `start` that spawns a background
//! task and a `reload` that updates the adapter's registered trigger set.
//! [`SourceManager`] owns all three and coordinates reload on hook hot-reload.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::def::HookDef;
use crate::events::KernelRouter;

pub mod cron;
pub mod file;
pub mod webhook;

#[cfg(test)]
mod tests;

/// Common interface for all external trigger source adapters.
pub trait TriggerSource: Send + Sync {
    /// Start the adapter's background task and return its join handle.
    fn start(&self, kernel: Arc<Mutex<KernelRouter>>) -> tokio::task::JoinHandle<()>;
    /// Update the adapter's registered trigger set (called on hot-reload).
    fn reload(&self, new_defs: &[HookDef]);
}

/// Owns all source adapters and forwards reload calls to each.
pub struct SourceManager {
    pub webhook: Arc<webhook::WebhookSource>,
    pub file: Arc<file::FileSource>,
    pub cron: Arc<cron::CronSource>,
}

impl SourceManager {
    pub fn new(ship_dir: PathBuf) -> Self {
        Self {
            webhook: Arc::new(webhook::WebhookSource::new()),
            file: Arc::new(file::FileSource::new(ship_dir)),
            cron: Arc::new(cron::CronSource::new()),
        }
    }

    /// Start all adapters' background tasks.
    pub fn start_all(&self, kernel: Arc<Mutex<KernelRouter>>) {
        self.webhook.start(kernel.clone());
        self.file.start(kernel.clone());
        self.cron.start(kernel.clone());
    }

    /// Reload all adapters with the updated hook set.
    pub fn reload(&self, defs: &[HookDef]) {
        self.webhook.reload(defs);
        self.file.reload(defs);
        self.cron.reload(defs);
    }
}
