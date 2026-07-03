use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub label: String,
    pub mounts: Vec<FsMount>,
    pub network: NetworkPolicy,
    pub env: HashMap<String, String>,
    pub limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsMount {
    pub host_path: PathBuf,
    pub sandbox_path: PathBuf,
    pub mode: MountMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MountMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum NetworkPolicy {
    #[default]
    DenyAll,
    Allowlist { hosts: Vec<String> },
    Proxy { listener: String, allow: Vec<String> },
    Open,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_bytes: Option<u64>,
    pub cpu_quota_micros: Option<u64>,
    pub pids_max: Option<u32>,
}

impl SandboxPolicy {
    pub fn workspace_default(workspace: PathBuf) -> Self {
        let label = format!("workspace:{}", workspace.display());
        Self {
            label,
            mounts: vec![FsMount {
                host_path: workspace.clone(),
                sandbox_path: workspace,
                mode: MountMode::ReadWrite,
            }],
            network: NetworkPolicy::DenyAll,
            env: HashMap::new(),
            limits: ResourceLimits::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_default_mounts_passthrough_at_identical_path() {
        let p = PathBuf::from("/home/alice/proj");
        let pol = SandboxPolicy::workspace_default(p.clone());
        let m = &pol.mounts[0];
        assert_eq!(m.host_path, p);
        assert_eq!(m.sandbox_path, p);
        assert_eq!(m.mode, MountMode::ReadWrite);
    }

    #[test]
    fn default_network_is_deny_all() {
        let pol = SandboxPolicy::default();
        assert!(matches!(pol.network, NetworkPolicy::DenyAll));
    }

    #[test]
    fn policy_roundtrips_through_json() {
        let pol = SandboxPolicy::workspace_default(PathBuf::from("/tmp/x"));
        let s = serde_json::to_string(&pol).unwrap();
        let back: SandboxPolicy = serde_json::from_str(&s).unwrap();
        assert_eq!(back.mounts.len(), 1);
    }
}
