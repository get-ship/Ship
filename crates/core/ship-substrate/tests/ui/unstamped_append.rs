// compile_fail: SqliteEventStore::append requires Stamped, not EventEnvelope.
extern crate ship_substrate;
extern crate ship_types;
extern crate serde_json;

use ship_substrate::persistence::{EventStore, SqliteEventStore};
use ship_types::EventEnvelope;

fn try_append(store: &SqliteEventStore, env: &EventEnvelope) {
    // Must not compile: expected &Stamped, found &EventEnvelope.
    let _ = store.append(env);
}

fn main() {}
