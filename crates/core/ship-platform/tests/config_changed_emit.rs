//! Verify `set_setting_and_emit` writes a `daemon_settings` row AND
//! publishes a `config.changed` kernel event with the expected payload.

use ship_platform::config::{get_setting, set_setting_and_emit};
 use ship_substrate::events::{EventFilter, new_isolated_for_test};
 use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
 use ship_substrate::project::get_global_dir;
 use ship_types::event_types::CONFIG_CHANGED;
 
 #[tokio::test(flavor = "multi_thread")]
 async fn set_setting_and_emit_publishes_config_changed() {
     ship_platform::install_defaults();
     let ship_dir = get_global_dir().expect("auto-isolated test ship dir");
     let kernel = new_isolated_for_test(ship_dir.clone()).expect("isolated kernel");

    let key = "test.setting.emit_path";
    let value = serde_json::json!("changed");

    set_setting_and_emit(&kernel, key, value.clone())
        .await
        .expect("set_setting_and_emit succeeds");

    // Setting persisted.
    let stored = get_setting(key).expect("get_setting").expect("value present");
    assert_eq!(stored, value);

    // config.changed is Runtime-scoped → the kernel sink lands it in runtime.db.
    let store = SqliteEventStore::new_at_db_path(
        &ship_substrate::db::runtime_db_path(&ship_dir),
    )
    .expect("event store");
    let events = store
        .query(&EventFilter {
            event_type: Some(CONFIG_CHANGED.to_string()),
            limit: Some(50),
            ..Default::default()
        })
        .expect("query events");

    let hit = events
        .iter()
        .find(|e| e.entity_id == key)
        .expect("config.changed event for our key");

    let payload: serde_json::Value =
        serde_json::from_str(&hit.payload_json).expect("payload is JSON");
    assert_eq!(payload["subject"], serde_json::json!(key));
    assert_eq!(payload["detail"], serde_json::json!(value.to_string()));
}
