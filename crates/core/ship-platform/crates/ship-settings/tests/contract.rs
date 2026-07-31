//! Contract proofs for settings.proto: message encode/decode roundtrips + the
//! RPC policy descriptor (capability + blast radius), read out of the embedded
//! FileDescriptorSet (proto-as-SSOT). The unified scope-discriminated
//! `SettingsService` replaces the retired `ConfigService`.

use buffa::Message;
use ship_ops_rpc::policy::{BlastRadius, MethodPolicy, read_policies};
use ship_proto::ship::ops::v1::{SetSettingReq, Setting, SettingList, SettingScope};

// ── Message roundtrips ──────────────────────────────────────────────────────

#[test]
fn set_setting_req_roundtrips_with_scope() {
    let req = SetSettingReq {
        key: "git.log_limit".into(),
        value_json: serde_json::to_vec(&serde_json::json!(200)).unwrap(),
        scope: buffa::EnumValue::from(SettingScope::SETTING_SCOPE_SPACE),
        ..Default::default()
    };
    let bytes = req.encode_to_vec();
    let back = SetSettingReq::decode_from_slice(&bytes).unwrap();
    assert_eq!(back.key, "git.log_limit");
    assert_eq!(back.scope, SettingScope::SETTING_SCOPE_SPACE);
    let v: serde_json::Value = serde_json::from_slice(&back.value_json).unwrap();
    assert_eq!(v, serde_json::json!(200));
}

#[test]
fn setting_roundtrips_with_scope_and_is_default() {
    let s = Setting {
        key: "terminal.program".into(),
        value_json: serde_json::to_vec(&serde_json::json!("tmux")).unwrap(),
        scope: buffa::EnumValue::from(SettingScope::SETTING_SCOPE_RUNTIME),
        is_default: true,
        ..Default::default()
    };
    let bytes = s.encode_to_vec();
    let back = Setting::decode_from_slice(&bytes).unwrap();
    assert_eq!(back.key, "terminal.program");
    assert_eq!(back.scope, SettingScope::SETTING_SCOPE_RUNTIME);
    assert!(back.is_default);
}

#[test]
fn setting_list_roundtrips() {
    let list = SettingList {
        settings: vec![
            Setting {
                key: "a".into(),
                value_json: b"1".to_vec(),
                is_default: false,
                ..Default::default()
            },
            Setting {
                key: "b".into(),
                value_json: b"2".to_vec(),
                is_default: true,
                ..Default::default()
            },
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

fn settings_policy(method: &str) -> MethodPolicy {
    read_policies()
        .into_iter()
        .find(|p| p.service == "SettingsService" && p.method == method)
        .unwrap_or_else(|| panic!("SettingsService.{method} not in descriptor set"))
}

#[test]
fn settings_reads_are_c0_settings_read() {
    for m in ["List", "Get", "Watch"] {
        let p = settings_policy(m);
        assert_eq!(p.capability, "ship.settings.read", "{m} capability");
        assert_eq!(p.blast, BlastRadius::C0Read, "{m} must be C0");
    }
}

#[test]
fn settings_set_is_c1_settings_write() {
    let p = settings_policy("Set");
    assert_eq!(p.capability, "ship.settings.write");
    assert_eq!(p.blast, BlastRadius::C1Space);
}

#[test]
fn settings_service_has_exactly_four_rpcs() {
    let methods: Vec<String> = read_policies()
        .into_iter()
        .filter(|p| p.service == "SettingsService")
        .map(|p| p.method)
        .collect();
    assert_eq!(methods.len(), 4, "Get/List/Set/Watch — got {methods:?}");
    for m in ["List", "Get", "Set", "Watch"] {
        assert!(methods.iter().any(|x| x == m), "missing {m}");
    }
}

// ── Contract-completeness (fixtures axis), settings.* slice ───────────────────
//
// The repo-wide gate is `xtask::gen_fixtures::ops::every_typed_method_has_a_
// validator_and_fixture`, which cannot run in this worktree (xtask → deck →
// deck-core needs bootstrapped SUI bundle assets — the "deck wall"). This is the
// SAME check, scoped to SettingsService and runnable without deck: every typed
// `settings.*` method's per-service fixture value must DESERIALIZE against its
// generated proto response type. Proves the settings_service.json fixture
// S3 added is contract-valid. ship-contracts CI gates codegen drift but does
// NOT gate this decode check; no other Rust test covers SettingsService
// contract deserialization (ship-ops-rpc's contract.rs reads agents/analytics/
// rules only).

#[test]
fn settings_fixtures_deserialize_against_their_proto_types() {
    use ship_proto::ship::ops::v1::WatchEvent;

    // PROTO_ROOT is baked by this crate's build.rs. Default =
    // DEP_SHIP_PROTO_PROTO_DIR (the proto SSOT path published by ship-proto's
    // own build.rs via `links = "ship-proto"`), so a normal build needs no
    // env var. SHIP_PROTO_DIR is the explicit override for alternate layouts.
    let settings_path =
        std::path::Path::new(env!("PROTO_ROOT")).join("fixtures/settings_service.json");
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&settings_path).expect("read settings_service.json"))
            .unwrap();

    // Each typed SettingsService method ⇒ its `response_ok` body parses as
    // the response type the descriptor declares (the gate's per-method check).
    let get = &settings["Get"]["response_ok"];
    serde_json::from_value::<Setting>(get.clone()).expect("settings.get matches Setting");

    let list = &settings["List"]["response_ok"];
    serde_json::from_value::<SettingList>(list.clone()).expect("settings.list matches SettingList");

    let set = &settings["Set"]["response_ok"];
    serde_json::from_value::<Setting>(set.clone()).expect("settings.set matches Setting");

    let watch = &settings["Watch"]["response_ok"];
    serde_json::from_value::<WatchEvent>(watch.clone()).expect("settings.watch matches WatchEvent");
}
