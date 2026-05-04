//! Global KernelRouter singleton — initialized once at server startup.
//!
//! Shared across all connections (MCP agents, Studio) so cross-actor routing
//! works within a single process. CLI contexts that never call
//! `init_kernel_router` get `None` from `kernel_router()` — routing is skipped
//! and only platform.db persistence happens.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

use crate::events::kernel_router::KernelRouter;
use crate::events::payload_schema::PayloadSchemaValidator;
use crate::events::validator::{ElevatedValidator, NamespaceValidator, ReservedNamespaceValidator};

static KERNEL_ROUTER: OnceLock<Arc<Mutex<KernelRouter>>> = OnceLock::new();

/// Initialize the global KernelRouter rooted at `base_dir`.
///
/// Idempotent — returns the existing instance if already initialized.
/// `base_dir` is the directory under which `kernel/` and `actors/` are
/// created (typically the project's `.ship/` directory).
pub fn init_kernel_router(base_dir: PathBuf) -> anyhow::Result<Arc<Mutex<KernelRouter>>> {
    if let Some(kr) = KERNEL_ROUTER.get() {
        return Ok(kr.clone());
    }
    let router = KernelRouter::new(base_dir)?
        .with_validator(Box::new(NamespaceValidator))
        .with_validator(Box::new(ReservedNamespaceValidator))
        .with_validator(Box::new(ElevatedValidator))
        .with_validator(Box::new(PayloadSchemaValidator));
    let arc = Arc::new(Mutex::new(router));
    // Ignore failure — another thread may have won the race; take its value.
    let _ = KERNEL_ROUTER.set(arc);
    Ok(KERNEL_ROUTER.get().expect("just set").clone())
}

/// Get the global KernelRouter, or `None` if not yet initialized.
///
/// Returns `None` in CLI contexts where no server has started. Callers must
/// handle this gracefully (skip routing, persist-only).
pub fn kernel_router() -> Option<Arc<Mutex<KernelRouter>>> {
    KERNEL_ROUTER.get().cloned()
}

/// Build an isolated KernelRouter for tests — does NOT populate the global
/// OnceLock, so each test gets its own instance rooted at its own tempdir.
///
/// Without this, tests sharing a process (cargo test default) race on the
/// OnceLock: whichever test runs first wins and subsequent tests get a
/// kernel pointing at the first test's now-dropped tempdir, producing
/// `unable to open database file` when they try to write.
#[doc(hidden)]
pub fn new_isolated_for_test(base_dir: PathBuf) -> anyhow::Result<Arc<Mutex<KernelRouter>>> {
    let router = KernelRouter::new(base_dir)?
        .with_validator(Box::new(NamespaceValidator))
        .with_validator(Box::new(ReservedNamespaceValidator))
        .with_validator(Box::new(ElevatedValidator))
        .with_validator(Box::new(PayloadSchemaValidator));
    Ok(Arc::new(Mutex::new(router)))
}
