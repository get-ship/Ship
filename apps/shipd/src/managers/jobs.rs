//! JobManager — owns the job dispatch subscription.

use crate::supervisor::job_dispatch::DispatchHandle;

/// Manages the job dispatch lifecycle.
#[derive(Default)]
pub struct JobManager {
    pub dispatch_handle: Option<DispatchHandle>,
}

impl JobManager {
    pub fn new() -> Self {
        Self::default()
    }
}
