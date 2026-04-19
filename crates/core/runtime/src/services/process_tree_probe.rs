//! Concrete `ProcessTreeProbe` backed by the cross-platform `sysinfo` crate.
//!
//! The monitor asks for a sample of the dispatched root PID and all its
//! descendants. We walk the parent-PID map with a depth-capped BFS and sum
//! each process's `accumulated_cpu_time()`. That counter is monotonic per
//! process; if a child exits between samples, its contribution is dropped,
//! but the tree-level sum still climbs whenever *anything* in the tree
//! burns CPU — which is the signal the step monitor needs.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::Mutex;

use crate::services::dispatch_ports::{ProcessTreeProbe, TreeSample};

/// Guardrail against pathological parent-PID cycles. Real agent trees are
/// shallow (tmux → shell → provider CLI → tool-call subprocess → rustc).
const MAX_TREE_DEPTH: u32 = 16;

pub struct SysinfoProcessTreeProbe {
    system: Arc<Mutex<System>>,
}

impl SysinfoProcessTreeProbe {
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new())),
        }
    }
}

impl Default for SysinfoProcessTreeProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessTreeProbe for SysinfoProcessTreeProbe {
    async fn sample(&self, root_pid: u32) -> Option<TreeSample> {
        let mut sys = self.system.lock().await;
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );

        let root = Pid::from_u32(root_pid);
        sys.process(root)?;

        let pids = collect_tree_pids(&sys, root);
        let mut total_ticks: u64 = 0;
        for pid in &pids {
            if let Some(p) = sys.process(*pid) {
                total_ticks = total_ticks.saturating_add(p.accumulated_cpu_time());
            }
        }

        Some(TreeSample {
            cpu_ticks: total_ticks,
            process_count: pids.len() as u32,
        })
    }
}

/// BFS over parent→child relationships, bounded by `MAX_TREE_DEPTH`.
/// Returns every PID in the tree, root first. sysinfo doesn't expose a
/// children-of helper, so we invert the parent index ourselves on each
/// call — cheap relative to the refresh itself.
fn collect_tree_pids(sys: &System, root: Pid) -> Vec<Pid> {
    let mut children: HashMap<Pid, Vec<Pid>> = HashMap::new();
    for (pid, proc) in sys.processes() {
        if let Some(parent) = proc.parent() {
            children.entry(parent).or_default().push(*pid);
        }
    }

    let mut out = vec![root];
    let mut queue: VecDeque<(Pid, u32)> = VecDeque::new();
    queue.push_back((root, 0));

    while let Some((pid, depth)) = queue.pop_front() {
        if depth >= MAX_TREE_DEPTH {
            continue;
        }
        if let Some(kids) = children.get(&pid) {
            for k in kids {
                if *k == pid {
                    continue;
                }
                out.push(*k);
                queue.push_back((*k, depth + 1));
            }
        }
    }

    out
}

#[cfg(test)]
mod smoke {
    use super::*;

    /// Exercise the real sysinfo path against the test process's own PID.
    /// Guarantees the probe builds and returns a sample without panics on
    /// the target platform. Asserts only that we got a non-empty tree —
    /// CPU ticks are platform-timing-dependent and not portable.
    #[tokio::test]
    async fn sample_self_returns_nonempty_tree() {
        let probe = SysinfoProcessTreeProbe::new();
        let pid = std::process::id();
        let sample = probe
            .sample(pid)
            .await
            .expect("probe should see the running test process");
        assert!(sample.process_count >= 1, "tree should contain at least the root");
    }
}
