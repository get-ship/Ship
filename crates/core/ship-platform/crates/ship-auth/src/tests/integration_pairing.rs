/// Integration tests — PairingRegistry flow.
///
/// Covers:
/// - init/confirm/claim happy path
/// - code one-shot (second confirm rejected)
/// - claim one-shot (second claim returns Unknown)
/// - per-source rate limiting
/// - global MAX_PENDING ceiling
/// - attempt-count lockout (code grinding prevention)
/// - dev-mode flag set/clear
/// - `paired_session_scopes` returns Full
use crate::pairing::{
    ClaimResult, ConfirmError, InitError, PairingRegistry, INIT_RATE_PER_MINUTE, MAX_CONFIRM_ATTEMPTS,
    MAX_PENDING, generate_claim_token, generate_code, paired_session_scopes, set_dev_mode, is_dev_mode,
};
use crate::scopes::Scope;

// ── Happy path ────────────────────────────────────────────────────────────────

#[test]
fn auth_pairing_flow_init_confirm_claim_roundtrip() {
    let mut reg = PairingRegistry::new();
    let pairing = reg.init(Some("studio".to_string()), "").unwrap();

    // Newly initialized — pending.
    assert_eq!(reg.claim(&pairing.claim_token), ClaimResult::Pending);

    // Confirm via the code.
    let returned_claim = reg.confirm(&pairing.code, "ship_sess_ok".to_string()).unwrap();
    assert_eq!(returned_claim, pairing.claim_token);

    // Claim retrieves the session token.
    let result = reg.claim(&pairing.claim_token);
    assert_eq!(result, ClaimResult::Approved { session_token: "ship_sess_ok".to_string() });

    // Second claim: one-shot — Unknown.
    assert_eq!(reg.claim(&pairing.claim_token), ClaimResult::Unknown);
}

#[test]
fn auth_pairing_flow_unknown_claim_returns_unknown() {
    let mut reg = PairingRegistry::new();
    assert_eq!(reg.claim("no-such-claim"), ClaimResult::Unknown);
}

#[test]
fn auth_pairing_flow_two_init_produce_distinct_codes_and_claims() {
    let mut reg = PairingRegistry::new();
    let a = reg.init(None, "").unwrap();
    let b = reg.init(None, "").unwrap();
    assert_ne!(a.code, b.code);
    assert_ne!(a.claim_token, b.claim_token);
}

// ── Code one-shot ─────────────────────────────────────────────────────────────

#[test]
fn auth_pairing_flow_code_is_one_shot_second_confirm_rejected() {
    let mut reg = PairingRegistry::new();
    let p = reg.init(None, "").unwrap();
    reg.confirm(&p.code, "tok1".to_string()).unwrap();
    // The code was consumed — second confirm with the same code → UnknownCode.
    let err = reg.confirm(&p.code, "tok2".to_string()).unwrap_err();
    assert_eq!(err, ConfirmError::UnknownCode);
}

// ── Unknown / bad code ────────────────────────────────────────────────────────

#[test]
fn auth_pairing_flow_confirm_unknown_code_fails() {
    let mut reg = PairingRegistry::new();
    let err = reg.confirm("ZZZ-ZZZ", "ship_sess_x".to_string()).unwrap_err();
    assert_eq!(err, ConfirmError::UnknownCode);
}

// ── Attempt-count lockout (code-grinding prevention) ─────────────────────────

#[test]
fn auth_pairing_flow_lockout_after_max_confirm_attempts() {
    let mut reg = PairingRegistry::new();
    let p = reg.init(None, "").unwrap();
    // Exhaust the per-pairing budget with wrong-code guesses.
    for _ in 0..MAX_CONFIRM_ATTEMPTS {
        let _ = reg.confirm("XXX-XXX", "tok".to_string());
    }
    // Even the correct code is now locked out.
    let err = reg.confirm(&p.code, "tok".to_string()).unwrap_err();
    assert_eq!(err, ConfirmError::TooManyAttempts);
}

// ── Rate limit ────────────────────────────────────────────────────────────────

#[test]
fn auth_pairing_flow_per_source_rate_limit_enforced() {
    let mut reg = PairingRegistry::new();
    for _ in 0..INIT_RATE_PER_MINUTE {
        reg.init(None, "10.0.0.1").unwrap();
    }
    let err = reg.init(None, "10.0.0.1").unwrap_err();
    assert_eq!(err, InitError::RateLimited);
    // A different source still has its own independent budget.
    reg.init(None, "10.0.0.2").unwrap();
}

#[test]
fn auth_pairing_flow_empty_source_bypasses_rate_limit() {
    let mut reg = PairingRegistry::new();
    // Empty source string = rate limit disabled (test/internal use).
    for _ in 0..(INIT_RATE_PER_MINUTE + 5) {
        reg.init(None, "").unwrap();
    }
    // No error — only the global MAX_PENDING would stop it.
}

// ── Global MAX_PENDING ceiling ────────────────────────────────────────────────

#[test]
fn auth_pairing_flow_max_pending_ceiling_enforced() {
    let mut reg = PairingRegistry::new();
    for _ in 0..MAX_PENDING {
        reg.init(None, "").unwrap();
    }
    let err = reg.init(None, "").unwrap_err();
    assert_eq!(err, InitError::TooManyPending);
}

// ── paired_session_scopes ─────────────────────────────────────────────────────

#[test]
fn auth_pairing_paired_session_scopes_returns_full() {
    // A paired client (Studio) that confirmed via CLI gets Full — same level
    // as the daemon token. The user physically confirmed machine ownership.
    let scopes = paired_session_scopes();
    assert_eq!(scopes, vec![Scope::Full]);
    assert!(Scope::AdminDaemon.is_granted_by(&scopes));
}

// ── dev_mode flag ─────────────────────────────────────────────────────────────

#[test]
fn auth_pairing_dev_mode_toggle() {
    // Store original state and restore to avoid test pollution.
    let orig = is_dev_mode();
    set_dev_mode(true);
    assert!(is_dev_mode());
    set_dev_mode(false);
    assert!(!is_dev_mode());
    // Restore original.
    set_dev_mode(orig);
}

// ── generate_code / generate_claim_token shapes ───────────────────────────────

#[test]
fn auth_pairing_generate_code_excludes_confusing_chars() {
    // The alphabet must avoid 0/O, 1/I/L so a human reading a code aloud
    // doesn't get the digits and letters mixed up.
    let code = generate_code().unwrap();
    assert!(!code.contains('0'));
    assert!(!code.contains('O'));
    assert!(!code.contains('1'));
    assert!(!code.contains('I'));
    assert!(!code.contains('L'));
    assert_eq!(code.len(), 7); // XXX-XXX
}

#[test]
fn auth_pairing_generate_claim_token_is_long() {
    // 256 bits of entropy is unguessable; the `claim_` prefix identifies
    // these tokens in logs and protects against them being mistaken for
    // other token families.
    let token = generate_claim_token().unwrap();
    assert!(token.starts_with("claim_"));
    assert_eq!(token.len(), 6 + 64); // prefix + 32 bytes hex
}
