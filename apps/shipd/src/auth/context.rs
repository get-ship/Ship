//! `AuthContext` — the authenticated caller identity injected into request
//! extensions by the auth middleware, and the SOLE session→binding minter.

use ship_substrate::events::CallerKind;
use agent_schema::Authority;
use ship_types::{ScopeBinding, SpaceId, local_device_id};

/// Authenticated caller context injected into request extensions by the
/// auth middleware. Handlers extract this to build `EmitContext`.
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub caller_kind: CallerKind,
    pub agent_id: Option<String>,
    /// The space this request belongs to (the pointer content-id), carried
    /// from the authenticated session. A space-window session carries its OWN
    /// space here; operator/agent requests leave it `None`, which
    /// the binding keys as `None` (Phase 2 fail-soft — no home-space fallback on
    /// the emit path). This is the per-connection space source. See spec
    /// `.artifacts/specs/space/128-multi-space-emit.md`.
    pub space_id: Option<String>,
    pub session_id: Option<String>,
    /// Capability grants (the gate's grant axis), minted from the session's
    /// provisioned set. Empty for the operator (cleared by is_operator) and for
    /// agents with no restricted domains. The sole capability axis.
    pub grants: Vec<String>,
    /// True iff this connection presented a **verified operator cred** — the
    /// SOLE source of operator authority (S6, #124). Set ONLY when a
    /// pid-attested operator session (`DaemonSession::is_operator_cred`) cleared
    /// `pid_authorizes` in the bearer middleware. The CLI transport, the daemon
    /// token, the browser cookie, and the local peer-uid grant NO elevation —
    /// they leave this `false`. `binding()` derives `Authority::Operator` from
    /// THIS flag, never from `caller_kind`. A cloud-issued operator cred can
    /// later set it through its own verified minter without touching the gate.
    pub operator_cred: bool,
}

impl AuthContext {
    /// Mint the authoritative [`ScopeBinding`] for this verified session — the
    /// SOLE session→binding constructor (auth-is-the-sole-minter, #196). The
    /// home-space fallback dies here: a session with no resolvable space keys
    /// `None` (the intended P2 fail-soft; P3 rejects).
    ///
    /// Authority derivation (S6 elevation-source kill, #124): `Operator` is
    /// granted SOLELY when the connection carries a verified operator cred
    /// (`operator_cred`, set only by a pid-attested operator session that
    /// cleared `pid_authorizes`). The CLI transport, the daemon token, the
    /// browser cookie, and the local peer-uid no longer confer operator. An SDK
    /// token is capped at `Sdk` and is NEVER `Operator` (#207).
    fn binding(&self) -> ScopeBinding {
        // NO home-space fallback — the global pseudo-space is gone (S2).
        let space = self.space_id.clone().map(SpaceId::new);
        let authority = if self.operator_cred {
            // THE one elevation source — a verified operator cred.
            Authority::Operator
        } else if self.caller_kind == CallerKind::Sdk {
            // Capped: the on-disk SDK token is never operator-class.
            Authority::Sdk
        } else if let Some(sid) = self.space_id.clone() {
            Authority::Space { space_id: sid }
        } else if let Some(id) = self.agent_id.clone() {
            Authority::Agent { id }
        } else {
            Authority::Sdk
        };
        // FLAG: server-auth — device_id is a per-install placeholder, account
        // is anonymous until the server-auth arc lands.
        ScopeBinding::from_session(local_device_id(), None, space, authority)
    }

    /// Convert to the runtime's `EmitContext`, carrying the session's minted
    /// binding. `kind` is the emitting `CallerKind` — session-originated emits
    /// pass the caller's own kind; daemon-mediated emits pass
    /// `CallerKind::Runtime` (the daemon is a trusted intermediary that may
    /// reach reserved namespaces while still carrying the caller's binding).
    pub fn to_emit_context(&self, kind: CallerKind) -> ship_substrate::events::EmitContext {
        ship_substrate::events::EmitContext {
            caller_kind: kind,
            skill_id: None,
            binding: self.binding(),
            session_id: self.session_id.clone(),
        }
    }

    /// Whether this caller is operator-class — the gate's operator verdict.
    /// Reads SOLELY the verified operator cred (S6): true iff `operator_cred`
    /// flipped `Authority::Operator` in [`binding`](Self::binding).
    pub fn is_operator(&self) -> bool {
        *self.binding().authority() == Authority::Operator
    }

    /// The caller's resolved [`Authority`]. shipd projects this into Connect
    /// request extensions so crate-owned wires (e.g. `ship-account`) can gate on
    /// the shared `agent_schema::Authority` without coupling to shipd's
    /// `AuthContext`. The SOLE authority derivation is [`binding`](Self::binding).
    pub fn authority(&self) -> Authority {
        self.binding().authority().clone()
    }
}
