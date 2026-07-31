use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::time::SystemTime;

use crate::event_registry::{self, EventScope};

thread_local! {
    static ULID_GEN: RefCell<ulid::Generator> = const { RefCell::new(ulid::Generator::new()) };
}

/// Monotonic ULID generator. `Generator::generate_from_datetime` fails only on
/// 64-bit randomness-counter exhaustion within the same millisecond (≈10^19
/// IDs/ms) or clock-rollback past 1970 — neither is recoverable here, so we
/// panic instead of poisoning every event-envelope caller with a Result.
#[allow(clippy::expect_used)]
fn monotonic_ulid() -> String {
    ULID_GEN.with(|g| {
        g.borrow_mut()
            .generate_from_datetime(SystemTime::now())
            .expect("ULID generation failed")
            .to_string()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EventEnvelope {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor: String,
    pub payload_json: String,
    pub version: u32,
    pub causation_id: Option<String>,
    /// Partition axis this event belongs to. Stamped at `new()` from the
    /// event-type registry (fail-closed `Runtime` for unregistered types).
    /// `#[serde(default)]` so envelopes serialized before the tuple existed
    /// still decode (→ Runtime).
    #[serde(default)]
    pub scope: EventScope,
    /// The id within `scope` — Space→space_id, Account→account_id,
    /// Runtime→machine/device id. Together `(scope, scope_key)`
    /// is the unified replacement for the loose `space_id: Option<String>`:
    /// scope and its key always travel together.
    pub scope_key: Option<String>,
    pub session_id: Option<String>,
    pub actor_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub target_actor_id: Option<String>,
    pub elevated: bool,
    pub created_at: DateTime<Utc>,
    /// Author account identity (the signing principal's account), when present.
    /// Additive Phase-2 groundwork — populated by `stamp_author` where the
    /// binding carries an account; `None` otherwise. `#[serde(default)]` so
    /// pre-existing envelopes (no author keys) decode to `None`.
    #[serde(default)]
    pub author_account_id: Option<String>,
    /// Author device identity. Populated by `stamp_author` from the binding's
    /// always-present device id. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub author_device_id: Option<String>,
    /// Signature over [`EventEnvelope::signing_bytes`]. None in Phase 2
    /// (groundwork). LATER POLICY (sign-iff-syncs): Space + Account events get
    /// signed; Runtime-scope (high-volume per-machine execution state) is NEVER
    /// signed — it never leaves the machine. Runtime-unsigned keeps signing off
    /// the hot path. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub signature: Option<String>,
    /// Reserved: cryptographic signature payload for self-certifying logs.
    /// Never required by validators; ignored until signing is wired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
    /// Reserved: writer identity for per-writer-chain CRDT sync.
    /// Never required by validators; ignored until multi-writer sync is wired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer: Option<String>,
    /// Reserved: causal ordering metadata (self-certifying / CRDT sync).
    /// Never required by validators; ignored until causal merge is wired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causality: Option<String>,
}

impl EventEnvelope {
    pub fn new<P: Serialize>(
        event_type: &str,
        entity_id: &str,
        payload: &P,
    ) -> Result<Self, serde_json::Error> {
        let payload_json = serde_json::to_string(payload)?;
        Ok(Self {
            id: monotonic_ulid(),
            event_type: event_type.to_string(),
            entity_id: entity_id.to_string(),
            actor: "ship".to_string(),
            payload_json,
            version: 1,
            causation_id: None,
            scope: event_registry::scope_for(event_type).unwrap_or(EventScope::Runtime),
            scope_key: None,
            session_id: None,
            actor_id: None,
            parent_actor_id: None,
            target_actor_id: None,
            elevated: false,
            created_at: Utc::now(),
            author_account_id: None,
            author_device_id: None,
            signature: None,
            sig: None,
            writer: None,
            causality: None,
        })
    }

    /// Canonical, deterministic byte form signed (in a later phase) to attest
    /// authorship + integrity. **FROZEN CONTRACT:** once any signature is ever
    /// produced over this form, the field set + framing cannot change.
    ///
    /// IN (immutable authored facts, fixed order): `id`, `event_type`,
    /// `entity_id`, `version` (decimal), `scope` (`as_str`), `scope_key`
    /// (`""` when None), `causation_id` (`""` when None), `author_account_id`
    /// (`""` when None), `author_device_id` (`""` when None), `created_at`
    /// (RFC3339 UTC), `sha256_hex(payload_json)`.
    ///
    /// OUT (mutable / delivery-routing / local — excluded so re-delivery or a
    /// local re-stamp never invalidates a signature): `actor`, `actor_id`,
    /// `parent_actor_id`, `target_actor_id`, `session_id`, `elevated`, and
    /// `signature` itself.
    ///
    /// Framing: each field is length-prefixed (`(len as u32).to_be_bytes()`
    /// then the bytes), in the fixed order above — no boundary ambiguity. The
    /// payload is carried as its `sha256_hex` digest, NOT raw; a verifier needs
    /// the raw `payload_json` to recompute the digest. NOT `serde_json` of the
    /// struct (key-order/whitespace are unstable across versions and would
    /// couple the signed form to the wire form).
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut field = |bytes: &[u8]| {
            out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            out.extend_from_slice(bytes);
        };
        field(self.id.as_bytes());
        field(self.event_type.as_bytes());
        field(self.entity_id.as_bytes());
        field(self.version.to_string().as_bytes());
        field(self.scope.as_str().as_bytes());
        field(self.scope_key.as_deref().unwrap_or("").as_bytes());
        field(self.causation_id.as_deref().unwrap_or("").as_bytes());
        field(self.author_account_id.as_deref().unwrap_or("").as_bytes());
        field(self.author_device_id.as_deref().unwrap_or("").as_bytes());
        field(self.created_at.to_rfc3339().as_bytes());
        field(crate::hash::sha256_hex(self.payload_json.as_bytes()).as_bytes());
        out
    }

    pub fn with_causation(mut self, causation_id: &str) -> Self {
        self.causation_id = Some(causation_id.to_string());
        self
    }

    /// Set the full `(scope, scope_key)` tuple directly. The explicit form used
    /// by call sites that know the event's partition (Account/System/Device),
    /// not only Space.
    pub fn with_scope(mut self, scope: EventScope, scope_key: Option<&str>) -> Self {
        self.scope = scope;
        self.scope_key = scope_key.map(str::to_string);
        self
    }

    /// The space id this event belongs to, or `None` when it is not space-scoped.
    /// The legacy `space_id` accessor expressed over the new tuple — read sites
    /// that genuinely mean "which space" use this.
    pub fn space_id(&self) -> Option<&str> {
        match self.scope {
            EventScope::Space => self.scope_key.as_deref(),
            _ => None,
        }
    }

    pub fn with_actor_id(mut self, actor_id: &str) -> Self {
        self.actor_id = Some(actor_id.to_string());
        self
    }

    pub fn with_parent_actor_id(mut self, parent_actor_id: &str) -> Self {
        self.parent_actor_id = Some(parent_actor_id.to_string());
        self
    }

    pub fn with_target(mut self, target_actor_id: &str) -> Self {
        self.target_actor_id = Some(target_actor_id.to_string());
        self
    }

    pub fn elevate(mut self) -> Self {
        self.elevated = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_new_sets_required_fields() {
        let env = EventEnvelope::new("actor.created", "ws-1", &serde_json::json!({}))
            .expect("new must succeed");
        assert_eq!(env.event_type, "actor.created");
        assert_eq!(env.entity_id, "ws-1");
        assert_eq!(env.id.len(), 26, "id must be a 26-char ULID");
        assert_eq!(env.version, 1);
        assert!(!env.elevated);
    }

    #[test]
    fn event_envelope_builder_chain() {
        let env = EventEnvelope::new("actor.created", "ws-1", &serde_json::json!({}))
            .unwrap()
            .with_causation("cause-1")
            .with_actor_id("agent.x")
            .with_target("agent.y")
            .elevate();
        assert_eq!(env.causation_id.as_deref(), Some("cause-1"));
        assert_eq!(env.actor_id.as_deref(), Some("agent.x"));
        assert_eq!(env.target_actor_id.as_deref(), Some("agent.y"));
        assert!(env.elevated);
    }

    #[test]
    fn event_envelope_roundtrips_serde() {
        let env = EventEnvelope::new("actor.created", "ws-1", &serde_json::json!({})).unwrap();
        let json = serde_json::to_string(&env).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.event_type, env.event_type);
    }

    #[test]
    fn event_envelope_roundtrips_legacy_without_author_or_signature() {
        let json = serde_json::json!({
            "id": "01J2X",
            "event_type": "actor.created",
            "entity_id": "ws-1",
            "actor": "ship",
            "payload_json": "{}",
            "version": 1,
            "scope": "runtime",
            "scope_key": null,
            "session_id": null,
            "actor_id": null,
            "parent_actor_id": null,
            "target_actor_id": null,
            "elevated": false,
            "created_at": "2024-01-01T00:00:00Z"
        });
        let decoded: EventEnvelope = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.author_account_id, None);
        assert_eq!(decoded.author_device_id, None);
        assert_eq!(decoded.signature, None);
        assert_eq!(decoded.sig, None);
        assert_eq!(decoded.writer, None);
        assert_eq!(decoded.causality, None);
    }

    #[test]
    fn reserved_fields_absent_from_serialization_when_none() {
        let env = EventEnvelope::new("actor.created", "ws-1", &serde_json::json!({})).unwrap();
        let json = serde_json::to_value(&env).unwrap();
        let obj = json.as_object().unwrap();
        assert!(!obj.contains_key("sig"), "sig must skip when None");
        assert!(!obj.contains_key("writer"), "writer must skip when None");
        assert!(!obj.contains_key("causality"), "causality must skip when None");
    }
}
