//! Connect-RPC contract tests — Rust client side.
//!
//! Asserts that the generated types parse the wire-format fixtures in
//! `proto/fixtures/auth_service.json` correctly. The gateway-side test
//! (`apps/gateway/test/connect-contract.test.ts`) asserts the inverse —
//! that the TS server produces those same fixtures.
//!
//! Both sides reading the same fixture file catches drift in field naming,
//! int64/string encoding (proto3 JSON quirk), default-value handling,
//! and any new method added to one side but not the other.
//!
//! What this is NOT: a real HTTP integration test. That would need a live
//! gateway running under tokio + CI orchestration. Fixture lockstep catches
//! the drift class without the orchestration.

use ship_gateway_client::{
    DaemonAttestResponse, GetMeResponse, RefreshResponse, RevokeKeyResponse,
    TokenExchangeResponse,
};

const FIXTURES_JSON: &str = include_str!(concat!(env!("PROTO_ROOT"), "/fixtures/auth_service.json"));

fn fixture_response_ok(method: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(FIXTURES_JSON).expect("fixtures parse");
    serde_json::to_string(&v[method]["response_ok"]).expect("response_ok serialize")
}

#[test]
fn token_exchange_response_parses_from_fixture() {
    let json = fixture_response_ok("TokenExchange");
    let resp: TokenExchangeResponse =
        serde_json::from_str(&json).expect("parse TokenExchangeResponse");
    assert_eq!(resp.key, "ship_test_key_abc123");
    assert_eq!(resp.user_id, "user-test-1");
    assert_eq!(resp.expires_at, 1_924_992_000);
}

#[test]
fn refresh_response_parses_from_fixture() {
    let json = fixture_response_ok("Refresh");
    let resp: RefreshResponse = serde_json::from_str(&json).expect("parse RefreshResponse");
    assert!(resp.ok);
    assert_eq!(resp.user_id, "user-test-1");
    assert_eq!(
        resp.note,
        "API keys do not refresh in place; run `ship login` to get a fresh key."
    );
}

#[test]
fn daemon_attest_response_parses_from_fixture() {
    let json = fixture_response_ok("DaemonAttest");
    let resp: DaemonAttestResponse =
        serde_json::from_str(&json).expect("parse DaemonAttestResponse");
    assert!(resp.jwt.starts_with("eyJ"));
    assert_eq!(resp.issuer, "https://getship.dev");
    assert_eq!(resp.expires_at, 1_924_995_600);
}

#[test]
fn get_me_response_parses_from_fixture() {
    let json = fixture_response_ok("GetMe");
    let resp: GetMeResponse = serde_json::from_str(&json).expect("parse GetMeResponse");
    // user is a proto3 message field — buffa::MessageField rather than
    // Option. `as_option` returns Option<&User>.
    let user = resp.user.as_option().expect("user field present");
    assert_eq!(user.id, "user-test-1");
    assert_eq!(user.name, "Test User");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.image, "https://example.com/avatar.png");
}

#[test]
fn revoke_key_response_parses_from_fixture() {
    let json = fixture_response_ok("RevokeKey");
    let resp: RevokeKeyResponse =
        serde_json::from_str(&json).expect("parse RevokeKeyResponse");
    assert!(resp.revoked);
}

#[test]
fn every_proto_method_has_a_fixture() {
    let v: serde_json::Value = serde_json::from_str(FIXTURES_JSON).expect("fixtures parse");
    for method in &["TokenExchange", "Refresh", "DaemonAttest", "GetMe", "RevokeKey"] {
        let fixture = &v[method];
        assert!(
            fixture.is_object(),
            "fixture for {method} missing or not an object"
        );
        let path = fixture["method_path"]
            .as_str()
            .unwrap_or_else(|| panic!("{method}.method_path missing"));
        assert_eq!(path, format!("/ship.gateway.v1.AuthService/{method}"));
        assert!(fixture["request"].is_object(), "{method}.request missing");
        assert!(
            fixture["response_ok"].is_object(),
            "{method}.response_ok missing"
        );
    }
}
