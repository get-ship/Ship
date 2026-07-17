//! Behavioral proofs for `ship_space::rules::list_space_rules`.
//!
//! Storage: the per-space `settings_overlay` partition — the read-model
//! surfaced by `ship_config::overlay`. Writes here go through the same
//! substrate path the canonical SettingsService writers use, so the proof
//! exercises the full overlay round-trip (ensure_table → insert → read →
//! parse), not a private shortcut.
//!
//! Test isolation: under a Rust test binary, `ShipHome::from_env()`
//! returns the auto-allocated thread-local test home
//! (`<tmpdir>/ship-test-ship-home-<pid>-<threadid>/.ship`). Every test
//! calls `ship_substrate::project::get_global_dir()` to reach that home
//! and uses a unique per-test `space_id` so concurrent test cases don't
//! overwrite each other's `settings_overlay` rows. The substrate's test
//! isolation gate then naturally prevents tests from touching the
//! operator's real ship home.
//!
//! The overlay writer uses the same substrate accessors the
//! SettingsService ships with, so the resulting row shape matches
//! `StoredRule` and the parser exercises every field (body, order,
//! enabled, attribution).

use std::sync::Arc;

use ship_config::overlay;
use ship_space::rules::{RULES_KEY_PREFIX, SpaceRule, list_space_rules};
use ship_substrate::data::DataService;
use ship_substrate::ports::SingleBackendDataStore;
use ship_substrate::testutil::EnvGuard;

/// The auto-isolated test ship home (no env mutation needed; the
/// substrate's `auto_test_ship_home` provisions a per-thread tmp dir).
fn ship_dir() -> std::path::PathBuf {
    ship_substrate::project::get_global_dir().unwrap()
}

/// A substrate `DataStore` over the test-isolated per-space `space.db`.
async fn overlay_store_for(space_id: &str) -> Arc<dyn ship_substrate::ports::DataStore> {
    let db_path = ship_substrate::db::space_db_path(&ship_dir(), space_id);
    let svc = DataService::open_at(&db_path).unwrap();
    Arc::new(SingleBackendDataStore::new(svc, space_id))
}

async fn write_rule(
    store: Arc<dyn ship_substrate::ports::DataStore>,
    name: &str,
    body: &str,
    order: i64,
    enabled: bool,
    attribution: Option<&str>,
) {
    let key = format!("{RULES_KEY_PREFIX}{name}");
    let value = serde_json::json!({
        "body": body,
        "order": order,
        "enabled": enabled,
        "attribution": attribution,
    });
    overlay::set(store.as_ref(), &key, &value).await.unwrap();
}

/// Proves the basic contract: rows are returned in `(order ASC, name ASC)`
/// order, regardless of the order writers persist them. Three rows in
/// reverse write order → still ascending by `order`, ties by `name`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_space_rules_orders_by_order_then_name() {
    let store = overlay_store_for("rules-ord").await;

    // Write in reverse of the expected order so the test proves the read
    // sorts on read, not on insert.
    write_rule(store.clone(), "zebra", "zebra body", 30, true, Some("alice")).await;
    write_rule(store.clone(), "apple", "apple body", 10, true, Some("bob")).await;
    write_rule(store.clone(), "mango", "mango body", 20, true, Some("carol")).await;

    // Two rows tied on order — alphabetical tie-break.
    write_rule(store.clone(), "kite", "kite body", 10, true, Some("dan")).await;

    let rules = list_space_rules("rules-ord").await.unwrap();
    let names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["apple", "kite", "mango", "zebra"],
        "ordering = (order asc, name asc); got {names:?}"
    );

    // Field plumbing survives the round-trip.
    let apple = rules.iter().find(|r| r.name == "apple").unwrap();
    assert_eq!(apple.body, "apple body");
    assert_eq!(apple.order, 10);
    assert_eq!(apple.attribution.as_deref(), Some("bob"));
}

/// Proves `enabled: false` rows are EXCLUDED from the returned list. The
/// spec fence is explicit: "excludes `enabled: false`". Disabled rows
/// stay in storage (the soft-delete + recall contract).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_space_rules_excludes_disabled_rows() {
    let store = overlay_store_for("rules-enabled").await;

    write_rule(store.clone(), "live", "live body", 0, true, None).await;
    write_rule(store.clone(), "off", "off body", 5, false, Some("hold")).await;
    write_rule(store.clone(), "later", "later body", 10, true, None).await;

    let rules = list_space_rules("rules-enabled").await.unwrap();
    let names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["live", "later"],
        "disabled 'off' is filtered; got {names:?}"
    );
}

/// Proves the reader ignores overlay rows outside the `rules.` namespace
/// — the SPACE scope also persists SettingsService rows the daemon cares
/// about, and those must not be smuggled into the rule list.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_space_rules_ignores_non_rule_overlay_rows() {
    let store = overlay_store_for("rules-namespace").await;

    // The rule we want.
    write_rule(store.clone(), "review", "review body", 0, true, Some("qa")).await;
    // Adjacent overlay rows that share the partition but are NOT rules.
    overlay::set(
        store.as_ref(),
        "editor.font",
        &serde_json::json!("mono"),
    )
    .await
    .unwrap();
    overlay::set(
        store.as_ref(),
        "hooks.max_depth",
        &serde_json::json!(3),
    )
    .await
    .unwrap();

    let rules = list_space_rules("rules-namespace").await.unwrap();
    assert_eq!(rules.len(), 1, "only the rules.* row survives; got {rules:?}");
    assert_eq!(rules[0].name, "review");
}

/// Proves the JSON coercion is permissive: a row with `order` as a
/// stringified number (a setting-writer legacy shape) still parses, and
/// `attribution: null` round-trips as `None`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_space_rules_handles_stringified_order_and_null_attribution() {
    let store = overlay_store_for("rules-coerce").await;
    let key = format!("{RULES_KEY_PREFIX}legacy");
    // Stringified `order` and `attribution: null` (the spec's wire shape
    // for "no attribution").
    let value = serde_json::json!({
        "body": "legacy",
        "order": "7",
        "enabled": true,
        "attribution": null,
    });
    overlay::set(store.as_ref(), &key, &value).await.unwrap();

    let rules = list_space_rules("rules-coerce").await.unwrap();
    assert_eq!(rules.len(), 1);
    let r = &rules[0];
    assert_eq!(r.name, "legacy");
    assert_eq!(r.body, "legacy");
    assert_eq!(r.order, 7, "stringified \"7\" coerces to i64(7)");
    assert_eq!(r.attribution, None, "json null → None");

    // Missing fields fall back to defaults: no body → "", no order → 0,
    // no enabled → true (enabled default = row visible).
    let key2 = format!("{RULES_KEY_PREFIX}bare");
    overlay::set(store.as_ref(), &key2, &serde_json::json!({})).await.unwrap();
    let rules = list_space_rules("rules-coerce").await.unwrap();
    let bare = rules.iter().find(|r| r.name == "bare").unwrap();
    assert_eq!(bare.body, "");
    assert_eq!(bare.order, 0);
    assert!(bare.enabled, "missing enabled defaults to true (visible)");
    assert_eq!(bare.attribution, None);
}

/// Proves a never-opened space (no `space.db` on disk) reads as an empty
/// list — the same graceful-empty behaviour the overlay reader exhibits
/// for a fresh project, repeated here end-to-end via `list_space_rules`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_space_rules_empty_when_never_opened() {
    // No overlay writes — the per-space db is lazily provisioned on
    // first open, so even opening it leaves an empty settings_overlay.
    let rules = list_space_rules("rules-fresh").await.unwrap();
    assert!(rules.is_empty(), "never-opened space → empty list");
}

/// Sanity-check: the public shape conforms to the contract the spec
/// stipulates. (The schema lives in `rules::StoredRule`, parsed into a
/// `SpaceRule`; this test pins the public field set so a future rename
/// forces a deliberate update here.)
#[test]
fn space_rule_field_set_is_stable() {
    let r = SpaceRule {
        name: "x".into(),
        body: "y".into(),
        order: 1,
        enabled: true,
        attribution: Some("z".into()),
    };
    assert_eq!(r.name, "x");
    assert_eq!(r.body, "y");
    assert_eq!(r.order, 1);
    assert!(r.enabled);
    assert_eq!(r.attribution.as_deref(), Some("z"));
}

/// The `EnvGuard` import is kept named so the unused-imports lint doesn't
/// drift if a test later gains its own SHIP_HOME redirection (mirrors the
/// shape other platform integration tests use).
#[allow(dead_code)]
fn _env_guard_marker(_g: EnvGuard) {}
