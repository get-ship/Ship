//! Contract proofs for config.proto: message encode/decode roundtrips + the
//! RPC policy descriptor (capability + blast radius), read out of the embedded
//! FileDescriptorSet (proto-as-SSOT).

use buffa::Message;
use ship_ops_rpc::policy::{read_policies, BlastRadius, MethodPolicy};
use ship_ops_rpc::{Setting, SettingList, SetSettingReq};

// ── Message roundtrips ──────────────────────────────────────────────────────

#[test]
fn set_setting_req_roundtrips() {
    let req = SetSettingReq {
        key: "git.log_limit".into(),
        value_json: serde_json::to_vec(&serde_json::json!(200)).unwrap(),
        ..Default::default()
    };
    let bytes = req.encode_to_vec();
    let back = SetSettingReq::decode_from_slice(&bytes).unwrap();
    assert_eq!(back.key, "git.log_limit");
    let v: serde_json::Value = serde_json::from_slice(&back.value_json).unwrap();
    assert_eq!(v, serde_json::json!(200));
}

#[test]
fn setting_roundtrips_with_is_default() {
    let s = Setting {
        key: "terminal.program".into(),
        value_json: serde_json::to_vec(&serde_json::json!("tmux")).unwrap(),
        is_default: true,
        ..Default::default()
    };
    let bytes = s.encode_to_vec();
    let back = Setting::decode_from_slice(&bytes).unwrap();
    assert_eq!(back.key, "terminal.program");
    assert!(back.is_default);
}

#[test]
fn setting_list_roundtrips() {
    let list = SettingList {
        settings: vec![
            Setting { key: "a".into(), value_json: b"1".to_vec(), is_default: false, ..Default::default() },
            Setting { key: "b".into(), value_json: b"2".to_vec(), is_default: true, ..Default::default() },
        ],
        ..Default::default()
    };
    let bytes = list.encode_to_vec();
    let back = SettingList::decode_from_slice(&bytes).unwrap();
    assert_eq!(back.settings.len(), 2);
    assert_eq!(back.settings[0].key, "a");
    assert_eq!(back.settings[1].key, "b");
    assert!(back.settings[1].is_default);
}

// ── RPC policy descriptor ───────────────────────────────────────────────────

fn config_policy(method: &str) -> MethodPolicy {
    read_policies()
        .into_iter()
        .find(|p| p.service == "ConfigService" && p.method == method)
        .unwrap_or_else(|| panic!("ConfigService.{method} not in descriptor set"))
}

#[test]
fn config_reads_are_c0_config_read() {
    for m in ["List", "Get", "Watch"] {
        let p = config_policy(m);
        assert_eq!(p.capability, "ship.config.read", "{m} capability");
        assert_eq!(p.blast, BlastRadius::C0Read, "{m} must be C0");
    }
}

#[test]
fn config_set_is_c1_config_write() {
    let p = config_policy("Set");
    assert_eq!(p.capability, "ship.config.write");
    assert_eq!(p.blast, BlastRadius::C1Space);
}

#[test]
fn config_service_has_exactly_four_rpcs() {
    let methods: Vec<String> = read_policies()
        .into_iter()
        .filter(|p| p.service == "ConfigService")
        .map(|p| p.method)
        .collect();
    assert_eq!(methods.len(), 4, "List/Get/Set/Watch — got {methods:?}");
    for m in ["List", "Get", "Set", "Watch"] {
        assert!(methods.iter().any(|x| x == m), "missing {m}");
    }
}
