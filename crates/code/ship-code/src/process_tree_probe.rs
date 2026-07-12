//! Concrete `ProcessTreeProbe` backed by the cross-platform `sysinfo` crate.
//! Moved from ship-substrate (L0) to ship-code (L1).

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::Mutex;

use crate::dispatch_ports::{ProcessTreeProbe, TreeSample};

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
