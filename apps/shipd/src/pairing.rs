//! Pairing flow — PKCE-style authentication for browser clients.
//!
//! A browser client (Studio) can't read `daemon.token` from disk. To prove
//! control of the machine, the user must confirm a short-lived pairing code
//! from a terminal — which requires filesystem access to `daemon.token`.
//!
//! Flow:
//!   1. Client calls `POST /auth/pair/init` → daemon returns `{ code, claim_token }`.
//!      Code is short (human-readable), claim_token is long (unguessable).
//!   2. Client displays `code` to the user and polls `GET /auth/pair/status?claim_token=...`.
//!   3. User runs `ship auth pair <code>` in a terminal. The CLI reads
//!      `daemon.token` and posts to `POST /api/auth/pair/confirm` with the code.
//!   4. Daemon marks the code approved, issues a session token with `Scope::Full`
//!      and binds it to `claim_token`.
//!   5. Client polls, receives the session token, uses it for all further requests.
//!
//! Security properties:
//!   - Agents without `daemon.token` access can't complete step 3.
//!   - Agents with `daemon.token` can't escalate beyond what they already have.
//!   - Pairing codes are one-shot and expire in `CODE_TTL`.
//!   - Claim tokens are long-random so attackers can't guess one to poll.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::scopes::Scope;

/// Global dev-mode flag. Set once at startup via `run_network(dev=true)`.
/// When on, browser clients that call /auth/pair/init get an auto-approved
/// session token without requiring CLI confirmation. Never set in production.
static DEV_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_dev_mode(on: bool) {
    DEV_MODE.store(on, Ordering::SeqCst);
}

pub fn is_dev_mode() -> bool {
    DEV_MODE.load(Ordering::SeqCst)
}

/// How long a pairing code stays valid before a confirmation is required.
pub const CODE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// How long the client has to poll for the session token after approval.
pub const CLAIM_TTL: Duration = Duration::from_secs(1800); // 30 minutes

/// TTL for paired session tokens (server issues, client stores).
pub const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60); // 30 days

#[derive(Debug, Clone)]
pub struct PendingPairing {
    pub code: String,
    pub claim_token: String,
    pub client_label: Option<String>,
    pub created_at: Instant,
    /// Set to Some(session_token) once the user confirms via CLI.
    pub approved_session: Option<String>,
}

impl PendingPairing {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > CLAIM_TTL
    }

    pub fn code_expired(&self) -> bool {
        self.created_at.elapsed() > CODE_TTL
    }
}

/// Registry of active pairing requests. Keyed by `claim_token` for status lookup;
/// a secondary index on `code` lets confirm find the pending pairing.
#[derive(Default)]
pub struct PairingRegistry {
    by_claim: HashMap<String, PendingPairing>,
    /// code → claim_token (for confirm lookup)
    code_to_claim: HashMap<String, String>,
}

impl PairingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove any expired pending pairings.
    pub fn gc(&mut self) {
        let expired_claims: Vec<String> = self
            .by_claim
            .iter()
            .filter(|(_, p)| p.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        for claim in expired_claims {
            if let Some(p) = self.by_claim.remove(&claim) {
                self.code_to_claim.remove(&p.code);
            }
        }
    }

    /// Create a new pending pairing and return it.
    pub fn init(&mut self, label: Option<String>) -> Result<PendingPairing, &'static str> {
        self.gc();
        let code = generate_code().map_err(|_| "entropy error generating code")?;
        let claim_token = generate_claim_token().map_err(|_| "entropy error generating claim token")?;
        let pairing = PendingPairing {
            code: code.clone(),
            claim_token: claim_token.clone(),
            client_label: label,
            created_at: Instant::now(),
            approved_session: None,
        };
        self.code_to_claim.insert(code, claim_token.clone());
        self.by_claim.insert(claim_token, pairing.clone());
        Ok(pairing)
    }

    /// Confirm a pairing by its code. Attaches the issued session token.
    /// Returns the claim_token so the issuer can also record the session.
    pub fn confirm(&mut self, code: &str, session_token: String) -> Result<String, ConfirmError> {
        self.gc();
        let claim_token = self
            .code_to_claim
            .get(code)
            .ok_or(ConfirmError::UnknownCode)?
            .clone();
        let pairing = self
            .by_claim
            .get_mut(&claim_token)
            .ok_or(ConfirmError::UnknownCode)?;
        if pairing.code_expired() {
            self.code_to_claim.remove(code);
            self.by_claim.remove(&claim_token);
            return Err(ConfirmError::CodeExpired);
        }
        if pairing.approved_session.is_some() {
            return Err(ConfirmError::AlreadyApproved);
        }
        pairing.approved_session = Some(session_token);
        // Remove code from secondary index — single-use.
        self.code_to_claim.remove(code);
        Ok(claim_token)
    }

    /// Poll for the session token. Removes the pairing once claimed.
    pub fn claim(&mut self, claim_token: &str) -> ClaimResult {
        self.gc();
        let pairing = match self.by_claim.get(claim_token) {
            Some(p) => p.clone(),
            None => return ClaimResult::Unknown,
        };
        if pairing.is_expired() {
            self.by_claim.remove(claim_token);
            return ClaimResult::Expired;
        }
        match pairing.approved_session {
            Some(session_token) => {
                // One-shot claim — remove the pairing on success.
                self.by_claim.remove(claim_token);
                ClaimResult::Approved { session_token }
            }
            None => ClaimResult::Pending,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfirmError {
    UnknownCode,
    CodeExpired,
    AlreadyApproved,
}

#[derive(Debug, PartialEq)]
pub enum ClaimResult {
    Pending,
    Approved { session_token: String },
    Expired,
    Unknown,
}

/// Shared pairing registry used by the daemon.
pub type SharedPairingRegistry = Arc<Mutex<PairingRegistry>>;

pub fn new_shared() -> SharedPairingRegistry {
    Arc::new(Mutex::new(PairingRegistry::new()))
}

/// Generate a human-readable pairing code. 6 characters from an
/// unambiguous alphabet (no 0/O, 1/I/L), formatted as XXX-XXX.
fn generate_code() -> Result<String, getrandom::Error> {
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 6];
    getrandom::getrandom(&mut bytes)?;
    let chars: String = bytes
        .iter()
        .map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char)
        .collect();
    Ok(format!("{}-{}", &chars[..3], &chars[3..]))
}

/// Generate a long-random claim token. 256 bits of entropy, hex-encoded.
fn generate_claim_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)?;
    Ok(format!("claim_{}", crate::auth::hex_encode(&bytes)))
}

/// Default scopes for a paired session. Paired clients (Studio, etc.) are
/// trusted at the same level as the daemon token — the user has physically
/// confirmed via CLI that they own the machine.
pub fn paired_session_scopes() -> Vec<Scope> {
    vec![Scope::Full]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_produces_unique_codes_and_claims() {
        let mut reg = PairingRegistry::new();
        let a = reg.init(None).unwrap();
        let b = reg.init(None).unwrap();
        assert_ne!(a.code, b.code);
        assert_ne!(a.claim_token, b.claim_token);
    }

    #[test]
    fn confirm_unknown_code_fails() {
        let mut reg = PairingRegistry::new();
        let err = reg.confirm("BAD-COD", "ship_sess_x".to_string()).unwrap_err();
        assert_eq!(err, ConfirmError::UnknownCode);
    }

    #[test]
    fn confirm_then_claim_roundtrip() {
        let mut reg = PairingRegistry::new();
        let pairing = reg.init(Some("studio".to_string())).unwrap();

        assert_eq!(reg.claim(&pairing.claim_token), ClaimResult::Pending);

        let claim = reg.confirm(&pairing.code, "ship_sess_abc".to_string()).unwrap();
        assert_eq!(claim, pairing.claim_token);

        let result = reg.claim(&pairing.claim_token);
        assert_eq!(
            result,
            ClaimResult::Approved { session_token: "ship_sess_abc".to_string() }
        );

        // Second claim is Unknown — one-shot.
        assert_eq!(reg.claim(&pairing.claim_token), ClaimResult::Unknown);
    }

    #[test]
    fn confirm_twice_rejects_second() {
        let mut reg = PairingRegistry::new();
        let p = reg.init(None).unwrap();
        reg.confirm(&p.code, "tok1".to_string()).unwrap();
        // Code was consumed; second confirm gets UnknownCode.
        let err = reg.confirm(&p.code, "tok2".to_string()).unwrap_err();
        assert_eq!(err, ConfirmError::UnknownCode);
    }

    #[test]
    fn unknown_claim_token_returns_unknown() {
        let mut reg = PairingRegistry::new();
        assert_eq!(reg.claim("no-such-claim"), ClaimResult::Unknown);
    }

    #[test]
    fn code_alphabet_excludes_confusing_chars() {
        let code = generate_code().unwrap();
        assert!(!code.contains('0'));
        assert!(!code.contains('O'));
        assert!(!code.contains('1'));
        assert!(!code.contains('I'));
        assert!(!code.contains('L'));
        assert_eq!(code.len(), 7); // XXX-XXX
    }

    #[test]
    fn claim_token_is_long() {
        let token = generate_claim_token().unwrap();
        assert!(token.starts_with("claim_"));
        assert_eq!(token.len(), 6 + 64); // prefix + 32 bytes hex
    }
}
