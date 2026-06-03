//! Contract proofs for settings.proto: message encode/decode roundtrips + the
//! RPC policy descriptor (capability + blast radius), read out of the embedded
//! FileDescriptorSet (proto-as-SSOT). The unified scope-discriminated
//! `SettingsService` replaces the retired `ConfigService`.

use buffa::Message;
use ship_ops_rpc::policy::{read_policies, BlastRadius, MethodPolicy};
use ship_ops_rpc::{Setting, SettingList, SettingScope, SetSettingReq};

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
// `settings.*` method's theme.json corpus value must DESERIALIZE against its
// generated proto response type. Proves the fixtures S3 added are contract-valid.

#[test]
fn settings_fixtures_deserialize_against_their_proto_types() {
    use ship_ops_rpc::WatchEvent;

    let theme_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../../proto/fixtures/theme.json");
    let theme: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&theme_path).expect("read theme.json")).unwrap();
    let responses = theme.get("responses").and_then(|v| v.as_object()).expect("responses object");

    // Each typed SettingsService method ⇒ its corpus value parses as the response
    // type the descriptor declares (the gate's per-method check).
    let get: &serde_json::Value = responses.get("settings.get").expect("settings.get corpus");
    serde_json::from_value::<Setting>(get.clone()).expect("settings.get matches Setting");

    let list = responses.get("settings.list").expect("settings.list corpus");
    serde_json::from_value::<SettingList>(list.clone()).expect("settings.list matches SettingList");

    let set = responses.get("settings.set").expect("settings.set corpus");
    serde_json::from_value::<Setting>(set.clone()).expect("settings.set matches Setting");

    let watch = responses.get("settings.watch").expect("settings.watch corpus");
    serde_json::from_value::<WatchEvent>(watch.clone()).expect("settings.watch matches WatchEvent");
}
