//! Event payload types — immutable once persisted.
//!
//! New payload fields must be `Option<T>` (or `#[serde(default)]`) so existing
//! v1 events deserialize correctly. The `version` field on `EventEnvelope` is
//! always 1 — reserved for future schema evolution where projections would
//! match on version to handle incompatible changes.
//!
//! See test `event_versioning_v1_compiled_without_new_fields` for enforcement.

pub use ship_types::payload::*;

pub mod event_types {
    pub use ship_types::event_types::*;
}

#[cfg(test)]
mod tests {
    use super::event_types::*;

    #[test]
    fn event_type_constants_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for &t in ALL {
            assert!(seen.insert(t), "duplicate event type constant: {t}");
        }
    }

    #[test]
    fn all_constants_have_expected_count() {
        assert_eq!(ALL.len(), 43, "exactly 43 event type constants required");
    }
}
